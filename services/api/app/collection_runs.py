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
from app.config import get_settings
from app.db import get_db
from app.local_project_collection import LocalProjectCollectionError, collect_local_project_files
from app.models import AuditEvent, CollectionRun, RawArtifact, Source, SourcePermission

router = APIRouter(prefix="/collection-runs", tags=["collection-runs"])


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
    content_base64: str
    filename: str | None = None
    mime_type: str | None = None
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    requested_by_actor_id: str = "local-owner"


class LocalProjectCollectionCreate(BaseModel):
    source_id: str
    source_permission_id: str
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


def _decode_content_base64(content_base64: str) -> bytes:
    try:
        return base64.b64decode(content_base64, validate=True)
    except (binascii.Error, ValueError) as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Invalid base64 content") from exc


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
    if permission.allowed_operations and not {"dry_run", "read"}.intersection(permission.allowed_operations):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Source permission does not allow dry-run preview",
        )

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
    if permission.allowed_operations and not {"collect", "read"}.intersection(permission.allowed_operations):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Source permission does not allow manual upload collection",
        )

    content = _decode_content_base64(payload.content_base64)
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
            "would_normalize": False,
            "would_enqueue_worker": False,
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
    db.commit()
    db.refresh(collection_run)
    return collection_run


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
    if permission.allowed_operations and not {"collect", "read"}.intersection(permission.allowed_operations):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Source permission does not allow local project collection",
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
            "would_normalize": False,
            "would_enqueue_worker": False,
        },
        error_message=None,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)

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
                "source_path": collected_file.source_path,
                "relative_path": collected_file.relative_path,
                "content_already_existed": collected_file.artifact.existed,
            },
        )
        db.add(artifact)
        _audit_raw_artifact_created(
            db,
            actor_id=payload.requested_by_actor_id,
            artifact=artifact,
            content_already_existed=collected_file.artifact.existed,
        )

    db.commit()
    db.refresh(collection_run)
    return collection_run


@router.get("/{collection_run_id}", response_model=CollectionRunRead)
def get_collection_run(collection_run_id: str, db: Session = Depends(get_db)) -> CollectionRun:
    collection_run = db.get(CollectionRun, collection_run_id)
    if collection_run is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Collection run not found")
    return collection_run
