import base64
import binascii
from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.artifact_store import ArtifactStoreError, store_artifact_bytes
from app.collector_dry_run import DryRunResult, run_connector_dry_run
from app.config import Settings, get_settings
from app.db import get_db
from app.local_project_collection import LocalProjectCollectionError, collect_local_project_files
from app.models import (
    Approval,
    AuditEvent,
    Chunk,
    CollectionRun,
    EvidenceItem,
    NormalizedDocument,
    RawArtifact,
    Source,
    SourcePermission,
    WorkItem,
)
from app.vector_memory import ChunkVectorUpsertResult, upsert_chunk_vectors_by_ids

router = APIRouter(prefix="/collection-runs", tags=["collection-runs"])

COLLECTION_APPROVAL_REQUEST_TYPES = {
    "source_collection",
    "manual_upload_collection",
    "local_project_collection",
}


class CollectionRunCreate(BaseModel):
    source_id: str | None = None
    requested_by_actor_id: str = "local-owner"
    summary_json: dict[str, Any] = Field(default_factory=dict)
    dry_run: bool = True


class CollectionDryRunPreviewCreate(BaseModel):
    source_id: str
    source_permission_id: str
    requested_by_actor_id: str = "local-owner"
    notes: dict[str, Any] = Field(default_factory=dict)


class ManualUploadCollectionCreate(BaseModel):
    source_id: str
    source_permission_id: str
    approval_id: str | None = None
    content_base64: str
    filename: str | None = None
    mime_type: str | None = None
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    requested_by_actor_id: str = "local-owner"


class ManualUploadIngestCreate(ManualUploadCollectionCreate):
    chunk_size: int = Field(default=1000, ge=100, le=5000)


class LocalProjectCollectionCreate(BaseModel):
    source_id: str
    source_permission_id: str
    approval_id: str | None = None
    requested_by_actor_id: str = "local-owner"


class CollectionRunRead(BaseModel):
    id: str
    source_id: str | None
    status: str
    dry_run: bool
    requested_by_actor_id: str
    summary_json: dict[str, Any]
    error_message: str | None
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class ManualUploadIngestResult(BaseModel):
    collection_run: CollectionRunRead
    raw_artifact_id: str
    raw_artifact_reused: bool
    document_id: str
    document_reused: bool
    chunk_ids: list[str]
    chunks_reused: bool
    evidence_item_ids: list[str]
    vector_upsert: ChunkVectorUpsertResult


def _audit_collection_run_created(db: Session, collection_run: CollectionRun) -> None:
    db.add(
        AuditEvent(
            actor_id=collection_run.requested_by_actor_id,
            event_type="collection_run.created",
            decision="recorded",
            resource_type="collection_run",
            resource_id=collection_run.id,
            correlation_id=None,
            details_json={
                "source_id": collection_run.source_id,
                "dry_run": collection_run.dry_run,
                "status": collection_run.status,
            },
        )
    )


def _audit_collection_run_dry_run(
    db: Session,
    *,
    actor_id: str,
    collection_run: CollectionRun,
    source_permission_id: str,
    decision: str,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="collection_run.dry_run_preview",
            decision=decision,
            resource_type="collection_run",
            resource_id=collection_run.id,
            correlation_id=None,
            details_json={
                "source_id": collection_run.source_id,
                "source_permission_id": source_permission_id,
                "status": collection_run.status,
                "error_message": collection_run.error_message,
            },
        )
    )


def _audit_raw_artifact_created(
    db: Session,
    *,
    actor_id: str,
    artifact: RawArtifact,
    content_already_existed: bool,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="raw_artifact.created",
            decision="recorded",
            resource_type="raw_artifact",
            resource_id=artifact.id,
            correlation_id=None,
            details_json={
                "source_id": artifact.source_id,
                "collection_run_id": artifact.collection_run_id,
                "content_hash": artifact.content_hash,
                "storage_path": artifact.storage_path,
                "size_bytes": artifact.size_bytes,
                "content_already_existed": content_already_existed,
            },
        )
    )


def _queue_normalization_work_item(
    db: Session,
    *,
    actor_id: str,
    collection_run: CollectionRun,
    source_permission_id: str,
    raw_artifact_ids: list[str],
    collection_mode: str,
) -> WorkItem:
    work_item = WorkItem(
        id=str(uuid4()),
        work_type="collection_normalization",
        status="queued",
        requested_by_actor_id=actor_id,
        payload_json={
            "collection_run_id": collection_run.id,
            "source_id": collection_run.source_id,
            "source_permission_id": source_permission_id,
            "raw_artifact_ids": raw_artifact_ids,
            "artifact_count": len(raw_artifact_ids),
            "collection_mode": collection_mode,
            "scaffold_only": False,
            "executes_normalization": True,
            "worker_task_name": "collection.normalize_collection_run",
            "normalization_input_type": "utf_8_text",
            "intent_verification_recorded": True,
            "intent_verification": {
                "original_request": f"Queue normalization for {collection_mode}",
                "interpretation": "Create a queued worker item to normalize collected UTF-8 text artifacts.",
                "proposed_work_type": "collection_normalization",
                "sources_likely_used": [collection_run.source_id] if collection_run.source_id else [],
                "expected_output": "Normalized document records for the collected raw artifacts.",
                "safety_requirements": [
                    "Use only stored local artifacts linked to this collection run.",
                    "Do not perform external model calls or system-changing actions.",
                ],
                "assumptions": ["Collected artifacts are UTF-8 text artifacts supported by the current worker."],
                "missing_information": [],
                "recorded_by": "DIFF-074 collection enqueue governance",
            },
        },
        error_message=None,
    )
    db.add(work_item)
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="work_item.created",
            decision="queued",
            resource_type="work_item",
            resource_id=work_item.id,
            correlation_id=collection_run.id,
            details_json={
                "work_type": work_item.work_type,
                "collection_run_id": collection_run.id,
                "raw_artifact_ids": raw_artifact_ids,
                "scaffold_only": False,
                "executes_normalization": True,
            },
        )
    )
    return work_item


def _decode_content_base64(content_base64: str) -> bytes:
    try:
        return base64.b64decode(content_base64, validate=True)
    except (binascii.Error, ValueError) as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Invalid base64 content") from exc


def _require_utf8_text_artifact_content(content: bytes) -> None:
    try:
        text = content.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Manual upload normalization currently supports UTF-8 text artifacts only",
        ) from exc
    if not text.strip():
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Manual upload content is empty",
        )


def _require_supported_text_mime_type(mime_type: str | None) -> None:
    if mime_type is None or mime_type == "":
        return
    normalized = mime_type.split(";", 1)[0].strip().lower()
    if normalized.startswith("text/") or normalized in {"application/json"}:
        return
    raise HTTPException(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        detail="Unsupported manual upload file type; this ingestion path supports UTF-8 text only",
    )


def _split_text_chunks(text: str, chunk_size: int) -> list[str]:
    return [text[index : index + chunk_size] for index in range(0, len(text), chunk_size) if text[index : index + chunk_size]]


def _require_permission_operation(permission: SourcePermission, allowed: set[str], action_label: str) -> None:
    if not set(permission.allowed_operations).intersection(allowed):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"Source permission does not allow {action_label}",
        )


def _require_collection_approval(
    db: Session,
    *,
    approval_id: str | None,
    source: Source,
    permission: SourcePermission,
    operation: str,
) -> Approval | None:
    if not permission.approval_required:
        return None
    if approval_id is None:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Collection requires an approved approval record",
        )

    approval = db.get(Approval, approval_id)
    if approval is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Approval not found")
    if approval.status != "approved":
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Approval is not approved")
    if approval.request_type not in COLLECTION_APPROVAL_REQUEST_TYPES:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Approval is not for source collection")

    payload = approval.request_payload_json or {}
    expected_values = {
        "source_id": source.id,
        "source_permission_id": permission.id,
        "operation": operation,
    }
    for key, expected_value in expected_values.items():
        actual_value = payload.get(key)
        if actual_value is None:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail=f"Approval is missing required {key} for requested collection",
            )
        if actual_value != expected_value:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail=f"Approval {key} does not match requested collection",
            )
    return approval


def _build_dry_run_summary(
    source: Source,
    permission: SourcePermission,
    result: DryRunResult | None,
    notes: dict[str, Any],
) -> dict[str, Any]:
    return {
        "source": {
            "id": source.id,
            "name": source.name,
            "source_type": source.source_type,
            "sensitivity": source.sensitivity,
            "enabled": source.enabled,
        },
        "permission": {
            "id": permission.id,
            "allowed_operations": permission.allowed_operations,
            "scope": permission.scope_json,
            "external_model_policy": permission.external_model_policy,
            "approval_required": permission.approval_required,
        },
        "preview": {
            "mode": "connector_dry_run_preview",
            "would_collect": False,
            "would_create_artifacts": False,
            "would_normalize": False,
            "would_enqueue_worker": False,
        },
        "connector_result": None
        if result is None
        else {
            "connector_name": result.connector_name,
            "allowed": result.allowed,
            "summary": result.summary,
            "estimated_items": result.estimated_items,
            "warnings": result.warnings,
            "metadata": result.metadata,
        },
        "notes": notes,
    }


def _persist_dry_run(
    db: Session,
    *,
    source: Source,
    permission: SourcePermission,
    requested_by_actor_id: str,
    summary_json: dict[str, Any],
    status_value: str,
    error_message: str | None,
) -> CollectionRun:
    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=source.id,
        status=status_value,
        dry_run=True,
        requested_by_actor_id=requested_by_actor_id,
        summary_json=summary_json,
        error_message=error_message,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)
    _audit_collection_run_dry_run(
        db,
        actor_id=requested_by_actor_id,
        collection_run=collection_run,
        source_permission_id=permission.id,
        decision="rejected" if error_message else "recorded",
    )
    db.commit()
    db.refresh(collection_run)
    return collection_run


def _get_or_create_raw_artifact(
    db: Session,
    *,
    source: Source,
    collection_run: CollectionRun,
    stored_content_hash: str,
    stored_storage_path: str,
    stored_size_bytes: int,
    mime_type: str | None,
    metadata_json: dict[str, Any],
    requested_by_actor_id: str,
    content_already_existed: bool,
) -> tuple[RawArtifact, bool]:
    existing = db.scalar(
        select(RawArtifact)
        .where(RawArtifact.source_id == source.id, RawArtifact.content_hash == stored_content_hash)
        .order_by(RawArtifact.created_at.asc())
        .limit(1)
    )
    if existing is not None:
        return existing, True

    artifact = RawArtifact(
        id=str(uuid4()),
        source_id=source.id,
        collection_run_id=collection_run.id,
        content_hash=stored_content_hash,
        storage_path=stored_storage_path,
        mime_type=mime_type,
        size_bytes=stored_size_bytes,
        metadata_json=metadata_json,
    )
    db.add(artifact)
    _audit_raw_artifact_created(
        db,
        actor_id=requested_by_actor_id,
        artifact=artifact,
        content_already_existed=content_already_existed,
    )
    return artifact, False


def _get_or_create_normalized_document(
    db: Session,
    *,
    artifact: RawArtifact,
    text_content: str,
    title: str | None,
    sensitivity: str,
    requested_by_actor_id: str,
) -> tuple[NormalizedDocument, bool]:
    existing = db.scalar(
        select(NormalizedDocument)
        .where(NormalizedDocument.raw_artifact_id == artifact.id)
        .order_by(NormalizedDocument.created_at.asc())
        .limit(1)
    )
    if existing is not None:
        return existing, True

    document = NormalizedDocument(
        id=str(uuid4()),
        raw_artifact_id=artifact.id,
        source_id=artifact.source_id,
        title=title,
        document_type="text",
        language=None,
        text_content=text_content,
        sensitivity=sensitivity,
        metadata_json={
            "generated_by": "DIFF-081",
            "raw_content_hash": artifact.content_hash,
            "raw_storage_path": artifact.storage_path,
        },
    )
    db.add(document)
    db.add(
        AuditEvent(
            actor_id=requested_by_actor_id,
            event_type="normalized_document.created",
            decision="recorded",
            resource_type="normalized_document",
            resource_id=document.id,
            correlation_id=artifact.collection_run_id,
            details_json={
                "source_id": artifact.source_id,
                "raw_artifact_id": artifact.id,
                "generated_by": "DIFF-081",
            },
        )
    )
    return document, False


def _get_or_create_chunks_and_evidence(
    db: Session,
    *,
    document: NormalizedDocument,
    chunk_size: int,
    requested_by_actor_id: str,
) -> tuple[list[Chunk], list[EvidenceItem], bool]:
    existing_chunks = list(
        db.scalars(
            select(Chunk).where(Chunk.document_id == document.id).order_by(Chunk.chunk_index.asc())
        ).all()
    )
    if existing_chunks:
        evidence_items = list(
            db.scalars(
                select(EvidenceItem)
                .where(EvidenceItem.chunk_id.in_([chunk.id for chunk in existing_chunks]))
                .order_by(EvidenceItem.created_at.asc())
            ).all()
        )
        return existing_chunks, evidence_items, True

    text_chunks = _split_text_chunks(document.text_content, chunk_size)
    if not text_chunks:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Document text is empty")

    chunks: list[Chunk] = []
    evidence_items: list[EvidenceItem] = []
    for index, text in enumerate(text_chunks):
        chunk = Chunk(
            id=str(uuid4()),
            document_id=document.id,
            chunk_index=index,
            text_content=text,
            location_json={
                "char_start": index * chunk_size,
                "char_end": index * chunk_size + len(text),
            },
            embedding_status="not_started",
            metadata_json={
                "generated_by": "DIFF-081",
                "chunk_size": chunk_size,
            },
        )
        evidence_item = EvidenceItem(
            id=str(uuid4()),
            source_id=document.source_id,
            document_id=document.id,
            chunk_id=chunk.id,
            evidence_type="document_chunk",
            statement=text,
            observed_at=None,
            confidence=None,
            metadata_json={
                "generated_by": "DIFF-081",
                "chunk_index": index,
            },
        )
        db.add(chunk)
        db.add(evidence_item)
        chunks.append(chunk)
        evidence_items.append(evidence_item)

    db.add(
        AuditEvent(
            actor_id=requested_by_actor_id,
            event_type="document_chunks.generated",
            decision="recorded",
            resource_type="normalized_document",
            resource_id=document.id,
            correlation_id=document.raw_artifact_id,
            details_json={
                "source_id": document.source_id,
                "chunk_count": len(chunks),
                "evidence_count": len(evidence_items),
                "generated_by": "DIFF-081",
            },
        )
    )
    return chunks, evidence_items, False


@router.get("", response_model=list[CollectionRunRead])
def list_collection_runs(db: Session = Depends(get_db)) -> list[CollectionRun]:
    statement = select(CollectionRun).order_by(CollectionRun.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=CollectionRunRead, status_code=status.HTTP_201_CREATED)
def create_collection_run(payload: CollectionRunCreate, db: Session = Depends(get_db)) -> CollectionRun:
    if payload.source_id is not None and db.get(Source, payload.source_id) is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")

    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=payload.source_id,
        status="dry_run_requested" if payload.dry_run else "created",
        dry_run=payload.dry_run,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json=payload.summary_json,
        error_message=None,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)
    db.commit()
    db.refresh(collection_run)
    return collection_run


@router.post("/dry-run", response_model=CollectionRunRead, status_code=status.HTTP_201_CREATED)
def create_collection_dry_run_preview(
    payload: CollectionDryRunPreviewCreate,
    db: Session = Depends(get_db),
) -> CollectionRun:
    source = db.get(Source, payload.source_id)
    if source is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    if not source.enabled:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is disabled")

    permission = db.get(SourcePermission, payload.source_permission_id)
    if permission is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source permission not found")
    if permission.source_id != source.id:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Source permission does not belong to the source",
        )
    _require_permission_operation(permission, {"dry_run", "read"}, "dry-run preview")

    try:
        result = run_connector_dry_run(
            source_id=source.id,
            source_type=source.source_type,
            source_name=source.name,
            source_location=source.location,
            source_metadata=source.metadata_json,
            permission_id=permission.id,
            permission_source_id=permission.source_id,
            permission_scope=permission.scope_json,
            allowed_operations=permission.allowed_operations,
            external_model_policy=permission.external_model_policy,
            approval_required=permission.approval_required,
        )
    except ValueError as exc:
        return _persist_dry_run(
            db,
            source=source,
            permission=permission,
            requested_by_actor_id=payload.requested_by_actor_id,
            summary_json=_build_dry_run_summary(source, permission, None, payload.notes),
            status_value="dry_run_failed",
            error_message=str(exc),
        )

    return _persist_dry_run(
        db,
        source=source,
        permission=permission,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json=_build_dry_run_summary(source, permission, result, payload.notes),
        status_value="dry_run_previewed",
        error_message=None,
    )


@router.post("/manual-upload", response_model=CollectionRunRead, status_code=status.HTTP_201_CREATED)
def create_manual_upload_collection(
    payload: ManualUploadCollectionCreate,
    db: Session = Depends(get_db),
) -> CollectionRun:
    source = db.get(Source, payload.source_id)
    if source is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    if not source.enabled:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is disabled")
    if source.source_type != "manual_upload":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is not a manual_upload source")

    permission = db.get(SourcePermission, payload.source_permission_id)
    if permission is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source permission not found")
    if permission.source_id != source.id:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Source permission does not belong to the source",
        )
    _require_permission_operation(permission, {"collect", "read"}, "manual upload collection")
    approval = _require_collection_approval(
        db,
        approval_id=payload.approval_id,
        source=source,
        permission=permission,
        operation="manual_upload_collection",
    )

    _require_supported_text_mime_type(payload.mime_type)
    content = _decode_content_base64(payload.content_base64)
    _require_utf8_text_artifact_content(content)
    try:
        stored = store_artifact_bytes(content, get_settings().artifact_store_path)
    except ArtifactStoreError as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=source.id,
        status="completed",
        dry_run=False,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json={
            "mode": "manual_upload_collection",
            "source_permission_id": permission.id,
            "filename": payload.filename,
            "content_hash": stored.content_hash,
            "storage_path": stored.storage_path,
            "size_bytes": stored.size_bytes,
            "content_already_existed": stored.existed,
            "would_normalize": True,
            "normalization_input_type": "utf_8_text",
            "approval_id": approval.id if approval else None,
        },
        error_message=None,
    )
    artifact = RawArtifact(
        id=str(uuid4()),
        source_id=source.id,
        collection_run_id=collection_run.id,
        content_hash=stored.content_hash,
        storage_path=stored.storage_path,
        mime_type=payload.mime_type,
        size_bytes=stored.size_bytes,
        metadata_json={
            **payload.metadata_json,
            "filename": payload.filename,
            "source_permission_id": permission.id,
            "approval_id": approval.id if approval else None,
        },
    )

    db.add(collection_run)
    db.add(artifact)
    _audit_collection_run_created(db, collection_run)
    _audit_raw_artifact_created(
        db,
        actor_id=payload.requested_by_actor_id,
        artifact=artifact,
        content_already_existed=stored.existed,
    )
    work_item = _queue_normalization_work_item(
        db,
        actor_id=payload.requested_by_actor_id,
        collection_run=collection_run,
        source_permission_id=permission.id,
        raw_artifact_ids=[artifact.id],
        collection_mode="manual_upload_collection",
    )
    collection_run.summary_json = {
        **collection_run.summary_json,
        "normalization_work_item_created": True,
        "normalization_work_item_id": work_item.id,
        "raw_artifact_ids": [artifact.id],
    }
    db.commit()
    db.refresh(collection_run)
    return collection_run


@router.post("/manual-upload/ingest", response_model=ManualUploadIngestResult, status_code=status.HTTP_201_CREATED)
def ingest_manual_upload_collection(
    payload: ManualUploadIngestCreate,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> ManualUploadIngestResult:
    source = db.get(Source, payload.source_id)
    if source is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    if not source.enabled:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is disabled")
    if source.source_type != "manual_upload":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is not a manual_upload source")

    permission = db.get(SourcePermission, payload.source_permission_id)
    if permission is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source permission not found")
    if permission.source_id != source.id:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Source permission does not belong to the source",
        )
    _require_permission_operation(permission, {"collect", "read"}, "manual upload ingestion")
    approval = _require_collection_approval(
        db,
        approval_id=payload.approval_id,
        source=source,
        permission=permission,
        operation="manual_upload_collection",
    )

    _require_supported_text_mime_type(payload.mime_type)
    content = _decode_content_base64(payload.content_base64)
    _require_utf8_text_artifact_content(content)
    text_content = content.decode("utf-8")

    try:
        stored = store_artifact_bytes(content, settings.artifact_store_path)
    except ArtifactStoreError as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=source.id,
        status="ingesting",
        dry_run=False,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json={
            "mode": "manual_upload_ingest",
            "source_permission_id": permission.id,
            "filename": payload.filename,
            "content_hash": stored.content_hash,
            "storage_path": stored.storage_path,
            "size_bytes": stored.size_bytes,
            "content_already_existed": stored.existed,
            "normalization_input_type": "utf_8_text",
            "chunk_size": payload.chunk_size,
            "approval_id": approval.id if approval else None,
            "external_model_calls": False,
            "embedding_method": "local_hash_v1",
        },
        error_message=None,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)

    artifact, raw_artifact_reused = _get_or_create_raw_artifact(
        db,
        source=source,
        collection_run=collection_run,
        stored_content_hash=stored.content_hash,
        stored_storage_path=stored.storage_path,
        stored_size_bytes=stored.size_bytes,
        mime_type=payload.mime_type,
        metadata_json={
            **payload.metadata_json,
            "filename": payload.filename,
            "source_permission_id": permission.id,
            "approval_id": approval.id if approval else None,
            "ingested_by": "DIFF-081",
        },
        requested_by_actor_id=payload.requested_by_actor_id,
        content_already_existed=stored.existed,
    )
    document, document_reused = _get_or_create_normalized_document(
        db,
        artifact=artifact,
        text_content=text_content,
        title=payload.filename or artifact.metadata_json.get("filename") or artifact.id,
        sensitivity=source.sensitivity,
        requested_by_actor_id=payload.requested_by_actor_id,
    )
    chunks, evidence_items, chunks_reused = _get_or_create_chunks_and_evidence(
        db,
        document=document,
        chunk_size=payload.chunk_size,
        requested_by_actor_id=payload.requested_by_actor_id,
    )
    db.flush()
    chunk_ids = [chunk.id for chunk in chunks]
    evidence_item_ids = [item.id for item in evidence_items]
    collection_run.summary_json = {
        **collection_run.summary_json,
        "raw_artifact_id": artifact.id,
        "raw_artifact_reused": raw_artifact_reused,
        "document_id": document.id,
        "document_reused": document_reused,
        "chunk_ids": chunk_ids,
        "chunks_reused": chunks_reused,
        "evidence_item_ids": evidence_item_ids,
    }
    db.commit()
    db.refresh(collection_run)

    try:
        vector_upsert = upsert_chunk_vectors_by_ids(db, settings, chunk_ids)
    except HTTPException as exc:
        collection_run.status = "vector_upsert_failed"
        collection_run.error_message = str(exc.detail)
        collection_run.summary_json = {
            **collection_run.summary_json,
            "vector_collection": settings.qdrant_chunk_collection,
            "vector_upsert_completed": False,
            "vector_error": str(exc.detail),
        }
        db.add(
            AuditEvent(
                actor_id=payload.requested_by_actor_id,
                event_type="manual_upload_ingest.vector_failed",
                decision="failed",
                resource_type="collection_run",
                resource_id=collection_run.id,
                correlation_id=collection_run.id,
                details_json={
                    "vector_collection": settings.qdrant_chunk_collection,
                    "error_message": str(exc.detail),
                },
            )
        )
        db.commit()
        raise

    collection_run.status = "completed"
    collection_run.summary_json = {
        **collection_run.summary_json,
        "vector_collection": vector_upsert.collection_name,
        "vector_collection_exists": vector_upsert.collection_exists,
        "vector_upsert_completed": True,
        "chunks_upserted": vector_upsert.chunks_upserted,
    }
    db.add(
        AuditEvent(
            actor_id=payload.requested_by_actor_id,
            event_type="manual_upload_ingest.completed",
            decision="completed",
            resource_type="collection_run",
            resource_id=collection_run.id,
            correlation_id=collection_run.id,
            details_json={
                "source_id": source.id,
                "raw_artifact_id": artifact.id,
                "document_id": document.id,
                "chunk_count": len(chunk_ids),
                "evidence_count": len(evidence_item_ids),
                "vector_collection": vector_upsert.collection_name,
                "chunks_upserted": vector_upsert.chunks_upserted,
                "generated_by": "DIFF-081",
            },
        )
    )
    db.commit()
    db.refresh(collection_run)

    return ManualUploadIngestResult(
        collection_run=CollectionRunRead.model_validate(collection_run),
        raw_artifact_id=artifact.id,
        raw_artifact_reused=raw_artifact_reused,
        document_id=document.id,
        document_reused=document_reused,
        chunk_ids=chunk_ids,
        chunks_reused=chunks_reused,
        evidence_item_ids=evidence_item_ids,
        vector_upsert=vector_upsert,
    )


@router.post("/local-project", response_model=CollectionRunRead, status_code=status.HTTP_201_CREATED)
def create_local_project_collection(
    payload: LocalProjectCollectionCreate,
    db: Session = Depends(get_db),
) -> CollectionRun:
    source = db.get(Source, payload.source_id)
    if source is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    if not source.enabled:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is disabled")
    if source.source_type != "local_project":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Source is not a local_project source")

    permission = db.get(SourcePermission, payload.source_permission_id)
    if permission is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source permission not found")
    if permission.source_id != source.id:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Source permission does not belong to the source",
        )
    _require_permission_operation(permission, {"collect", "read"}, "local project collection")
    approval = _require_collection_approval(
        db,
        approval_id=payload.approval_id,
        source=source,
        permission=permission,
        operation="local_project_collection",
    )

    try:
        result = collect_local_project_files(
            source_location=source.location,
            permission_scope=permission.scope_json,
            artifact_store_path=get_settings().artifact_store_path,
        )
    except (ArtifactStoreError, LocalProjectCollectionError) as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc)) from exc

    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=source.id,
        status="completed",
        dry_run=False,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json={
            "mode": "local_project_collection",
            "source_permission_id": permission.id,
            "total_files": result.total_files,
            "collected_files": result.collected_files,
            "skipped_files": result.skipped_files,
            "would_normalize": True,
            "normalization_input_type": "utf_8_text",
            "normalization_note": "Worker normalization currently supports UTF-8 text artifacts only.",
            "approval_id": approval.id if approval else None,
        },
        error_message=None,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)

    raw_artifact_ids: list[str] = []
    for collected_file in result.files:
        artifact = RawArtifact(
            id=str(uuid4()),
            source_id=source.id,
            collection_run_id=collection_run.id,
            content_hash=collected_file.artifact.content_hash,
            storage_path=collected_file.artifact.storage_path,
            mime_type=None,
            size_bytes=collected_file.artifact.size_bytes,
            metadata_json={
                "source_permission_id": permission.id,
                "approval_id": approval.id if approval else None,
                "source_path": collected_file.source_path,
                "relative_path": collected_file.relative_path,
                "content_already_existed": collected_file.artifact.existed,
            },
        )
        raw_artifact_ids.append(artifact.id)
        db.add(artifact)
        _audit_raw_artifact_created(
            db,
            actor_id=payload.requested_by_actor_id,
            artifact=artifact,
            content_already_existed=collected_file.artifact.existed,
        )

    work_item = _queue_normalization_work_item(
        db,
        actor_id=payload.requested_by_actor_id,
        collection_run=collection_run,
        source_permission_id=permission.id,
        raw_artifact_ids=raw_artifact_ids,
        collection_mode="local_project_collection",
    )
    collection_run.summary_json = {
        **collection_run.summary_json,
        "normalization_work_item_created": True,
        "normalization_work_item_id": work_item.id,
        "raw_artifact_ids": raw_artifact_ids,
    }
    db.commit()
    db.refresh(collection_run)
    return collection_run


@router.get("/{collection_run_id}", response_model=CollectionRunRead)
def get_collection_run(collection_run_id: str, db: Session = Depends(get_db)) -> CollectionRun:
    collection_run = db.get(CollectionRun, collection_run_id)
    if collection_run is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Collection run not found")
    return collection_run
