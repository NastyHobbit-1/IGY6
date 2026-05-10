from pathlib import Path
from typing import Any
from uuid import uuid4

from sqlalchemy import (
    Column,
    DateTime,
    Integer,
    MetaData,
    String,
    Table,
    Text,
    create_engine,
    func,
    insert,
    select,
    update,
)
from sqlalchemy.dialects.postgresql import JSONB

from app.celery_app import celery_app
from app.config import get_settings

metadata = MetaData()

work_items = Table(
    "work_items",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("work_type", String(64), nullable=False),
    Column("status", String(64), nullable=False),
    Column("requested_by_actor_id", String(128), nullable=False),
    Column("payload_json", JSONB, nullable=False),
    Column("error_message", Text),
    Column("updated_at", DateTime(timezone=True)),
)

collection_runs = Table(
    "collection_runs",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("source_id", String(36)),
)

raw_artifacts = Table(
    "raw_artifacts",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("source_id", String(36)),
    Column("collection_run_id", String(36)),
    Column("content_hash", String(128), nullable=False),
    Column("storage_path", Text, nullable=False),
    Column("metadata_json", JSONB, nullable=False),
)

normalized_documents = Table(
    "normalized_documents",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("raw_artifact_id", String(36)),
    Column("source_id", String(36)),
    Column("title", String(255)),
    Column("document_type", String(64), nullable=False),
    Column("language", String(32)),
    Column("text_content", Text, nullable=False),
    Column("sensitivity", String(64), nullable=False),
    Column("metadata_json", JSONB, nullable=False),
)

chunks = Table(
    "chunks",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("document_id", String(36), nullable=False),
    Column("chunk_index", Integer, nullable=False),
    Column("text_content", Text, nullable=False),
    Column("location_json", JSONB, nullable=False),
    Column("embedding_status", String(64), nullable=False),
    Column("metadata_json", JSONB, nullable=False),
)

evidence_items = Table(
    "evidence_items",
    metadata,
    Column("id", String(36), primary_key=True),
    Column("source_id", String(36)),
    Column("document_id", String(36)),
    Column("chunk_id", String(36)),
    Column("evidence_type", String(64), nullable=False),
    Column("statement", Text, nullable=False),
    Column("observed_at", DateTime(timezone=True)),
    Column("confidence", Integer),
    Column("metadata_json", JSONB, nullable=False),
)

audit_events = Table(
    "audit_events",
    metadata,
    Column("id", Integer, primary_key=True),
    Column("actor_id", String(128), nullable=False),
    Column("event_type", String(128), nullable=False),
    Column("decision", String(64)),
    Column("resource_type", String(64)),
    Column("resource_id", String(128)),
    Column("correlation_id", String(128)),
    Column("details_json", JSONB, nullable=False),
)


def _engine():
    return create_engine(get_settings().database_url, pool_pre_ping=True)


def _read_artifact_bytes(storage_path: str) -> bytes:
    relative_path = Path(storage_path)
    if relative_path.is_absolute():
        raise RuntimeError("Artifact storage path must be relative")

    root = Path(get_settings().artifact_store_path).expanduser().resolve()
    target = (root / relative_path).resolve()
    try:
        target.relative_to(root)
    except ValueError as exc:
        raise RuntimeError("Artifact storage path escapes artifact store") from exc

    if not target.is_file():
        raise RuntimeError("Artifact file not found")
    return target.read_bytes()


def _decode_utf8_artifact(content: bytes) -> str:
    try:
        return content.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise RuntimeError("Artifact is not UTF-8 text") from exc


def _document_title(raw_artifact: dict[str, Any]) -> str | None:
    metadata_json = raw_artifact.get("metadata_json") or {}
    for key in ("filename", "relative_path", "source_path"):
        value = metadata_json.get(key)
        if isinstance(value, str) and value:
            return value[:255]
    return raw_artifact["id"]


def _split_text_chunks(text: str, chunk_size: int) -> list[str]:
    return [
        text[index : index + chunk_size]
        for index in range(0, len(text), chunk_size)
        if text[index : index + chunk_size]
    ]


def _audit(
    connection,
    *,
    actor_id: str,
    event_type: str,
    decision: str,
    resource_type: str,
    resource_id: str,
    correlation_id: str,
    details: dict[str, Any],
) -> None:
    connection.execute(
        insert(audit_events).values(
            actor_id=actor_id,
            event_type=event_type,
            decision=decision,
            resource_type=resource_type,
            resource_id=resource_id,
            correlation_id=correlation_id,
            details_json=details,
        )
    )


@celery_app.task(name="phase0.health")
def health() -> dict[str, str]:
    return {"status": "ok", "service": "worker", "phase": "0"}


@celery_app.task(name="collection.normalization_scaffold")
def normalization_scaffold(
    work_item_id: str,
    collection_run_id: str,
    raw_artifact_ids: list[str],
) -> dict[str, object]:
    return {
        "status": "not_executed",
        "scaffold_only": True,
        "work_item_id": work_item_id,
        "collection_run_id": collection_run_id,
        "raw_artifact_ids": raw_artifact_ids,
        "message": "DIFF-050 scaffold only; normalization execution is out of scope.",
    }


@celery_app.task(name="collection.normalize_collection_run")
def normalize_collection_run(
    work_item_id: str,
    collection_run_id: str,
    raw_artifact_ids: list[str],
) -> dict[str, object]:
    engine = _engine()
    created_document_ids: list[str] = []
    skipped_raw_artifact_ids: list[str] = []
    actor_id = "worker"

    with engine.begin() as connection:
        work_item = connection.execute(
            select(work_items).where(work_items.c.id == work_item_id)
        ).mappings().one_or_none()
        if work_item is None:
            raise RuntimeError("Work item not found")
        actor_id = work_item["requested_by_actor_id"]
        connection.execute(
            update(work_items)
            .where(work_items.c.id == work_item_id)
            .values(status="running", error_message=None, updated_at=func.now())
        )

    try:
        with engine.begin() as connection:
            work_item = connection.execute(
                select(work_items).where(work_items.c.id == work_item_id)
            ).mappings().one()
            if work_item["work_type"] != "collection_normalization":
                raise RuntimeError("Work item is not a collection_normalization item")

            payload = work_item["payload_json"] or {}
            expected_artifact_ids = payload.get("raw_artifact_ids") or []
            if payload.get("collection_run_id") != collection_run_id:
                raise RuntimeError("Work item collection_run_id does not match task request")
            if list(expected_artifact_ids) != list(raw_artifact_ids):
                raise RuntimeError("Work item raw_artifact_ids do not match task request")

            collection_run = connection.execute(
                select(collection_runs).where(collection_runs.c.id == collection_run_id)
            ).mappings().one_or_none()
            if collection_run is None:
                raise RuntimeError("Collection run not found")

            rows = connection.execute(
                select(raw_artifacts).where(raw_artifacts.c.id.in_(raw_artifact_ids))
            ).mappings().all()
            artifacts_by_id = {row["id"]: dict(row) for row in rows}
            missing_artifact_ids = [
                artifact_id for artifact_id in raw_artifact_ids if artifact_id not in artifacts_by_id
            ]
            if missing_artifact_ids:
                raise RuntimeError(f"Raw artifacts not found: {', '.join(missing_artifact_ids)}")

            for artifact_id in raw_artifact_ids:
                artifact = artifacts_by_id[artifact_id]
                if artifact["collection_run_id"] != collection_run_id:
                    raise RuntimeError("Raw artifact does not belong to the collection run")

                existing_document_id = connection.execute(
                    select(normalized_documents.c.id).where(
                        normalized_documents.c.raw_artifact_id == artifact_id
                    )
                ).scalar_one_or_none()
                if existing_document_id is not None:
                    skipped_raw_artifact_ids.append(artifact_id)
                    continue

                text_content = _decode_utf8_artifact(_read_artifact_bytes(artifact["storage_path"]))
                document_id = str(uuid4())
                connection.execute(
                    insert(normalized_documents).values(
                        id=document_id,
                        raw_artifact_id=artifact_id,
                        source_id=artifact["source_id"],
                        title=_document_title(artifact),
                        document_type="text",
                        language=None,
                        text_content=text_content,
                        sensitivity="internal",
                        metadata_json={
                            "generated_by": "DIFF-051",
                            "raw_content_hash": artifact["content_hash"],
                            "raw_storage_path": artifact["storage_path"],
                            "work_item_id": work_item_id,
                        },
                    )
                )
                created_document_ids.append(document_id)

            connection.execute(
                update(work_items)
                .where(work_items.c.id == work_item_id)
                .values(status="completed", error_message=None, updated_at=func.now())
            )
            _audit(
                connection,
                actor_id=actor_id,
                event_type="collection_normalization.completed",
                decision="completed",
                resource_type="work_item",
                resource_id=work_item_id,
                correlation_id=collection_run_id,
                details={
                    "collection_run_id": collection_run_id,
                    "created_document_ids": created_document_ids,
                    "skipped_raw_artifact_ids": skipped_raw_artifact_ids,
                },
            )
    except Exception as exc:
        with engine.begin() as connection:
            connection.execute(
                update(work_items)
                .where(work_items.c.id == work_item_id)
                .values(status="failed", error_message=str(exc), updated_at=func.now())
            )
            _audit(
                connection,
                actor_id=actor_id,
                event_type="collection_normalization.failed",
                decision="failed",
                resource_type="work_item",
                resource_id=work_item_id,
                correlation_id=collection_run_id,
                details={
                    "collection_run_id": collection_run_id,
                    "raw_artifact_ids": raw_artifact_ids,
                    "error_message": str(exc),
                },
            )
        return {
            "status": "failed",
            "work_item_id": work_item_id,
            "collection_run_id": collection_run_id,
            "error_message": str(exc),
        }

    return {
        "status": "completed",
        "work_item_id": work_item_id,
        "collection_run_id": collection_run_id,
        "created_document_ids": created_document_ids,
        "skipped_raw_artifact_ids": skipped_raw_artifact_ids,
    }


@celery_app.task(name="evidence.generate_document_chunks")
def generate_document_chunks(
    document_ids: list[str],
    chunk_size: int = 1000,
    work_item_id: str | None = None,
) -> dict[str, object]:
    if chunk_size < 100 or chunk_size > 5000:
        raise RuntimeError("Chunk size must be between 100 and 5000")

    engine = _engine()
    actor_id = "worker"
    created_chunk_ids: list[str] = []
    created_evidence_ids: list[str] = []
    skipped_document_ids: list[str] = []

    if work_item_id is not None:
        with engine.begin() as connection:
            work_item = connection.execute(
                select(work_items).where(work_items.c.id == work_item_id)
            ).mappings().one_or_none()
            if work_item is None:
                raise RuntimeError("Work item not found")
            if work_item["work_type"] != "document_chunking":
                raise RuntimeError("Work item is not a document_chunking item")
            payload = work_item["payload_json"] or {}
            expected_document_ids = payload.get("document_ids")
            if expected_document_ids is None and payload.get("document_id") is not None:
                expected_document_ids = [payload["document_id"]]
            if expected_document_ids is not None and list(expected_document_ids) != list(document_ids):
                raise RuntimeError("Work item document IDs do not match task request")
            actor_id = work_item["requested_by_actor_id"]
            connection.execute(
                update(work_items)
                .where(work_items.c.id == work_item_id)
                .values(status="running", error_message=None, updated_at=func.now())
            )

    try:
        with engine.begin() as connection:
            rows = connection.execute(
                select(normalized_documents).where(normalized_documents.c.id.in_(document_ids))
            ).mappings().all()
            documents_by_id = {row["id"]: dict(row) for row in rows}
            missing_document_ids = [
                document_id for document_id in document_ids if document_id not in documents_by_id
            ]
            if missing_document_ids:
                raise RuntimeError(f"Documents not found: {', '.join(missing_document_ids)}")

            for document_id in document_ids:
                existing_chunk_id = connection.execute(
                    select(chunks.c.id).where(chunks.c.document_id == document_id).limit(1)
                ).scalar_one_or_none()
                if existing_chunk_id is not None:
                    skipped_document_ids.append(document_id)
                    continue

                document = documents_by_id[document_id]
                text_chunks = _split_text_chunks(document["text_content"], chunk_size)
                if not text_chunks:
                    raise RuntimeError("Document text is empty")

                for index, text in enumerate(text_chunks):
                    chunk_id = str(uuid4())
                    evidence_id = str(uuid4())
                    connection.execute(
                        insert(chunks).values(
                            id=chunk_id,
                            document_id=document_id,
                            chunk_index=index,
                            text_content=text,
                            location_json={
                                "char_start": index * chunk_size,
                                "char_end": index * chunk_size + len(text),
                            },
                            embedding_status="not_started",
                            metadata_json={
                                "generated_by": "DIFF-052",
                                "chunk_size": chunk_size,
                                "work_item_id": work_item_id,
                            },
                        )
                    )
                    connection.execute(
                        insert(evidence_items).values(
                            id=evidence_id,
                            source_id=document["source_id"],
                            document_id=document_id,
                            chunk_id=chunk_id,
                            evidence_type="document_chunk",
                            statement=text,
                            observed_at=None,
                            confidence=None,
                            metadata_json={
                                "generated_by": "DIFF-052",
                                "chunk_index": index,
                                "work_item_id": work_item_id,
                            },
                        )
                    )
                    created_chunk_ids.append(chunk_id)
                    created_evidence_ids.append(evidence_id)

            if work_item_id is not None:
                connection.execute(
                    update(work_items)
                    .where(work_items.c.id == work_item_id)
                    .values(status="completed", error_message=None, updated_at=func.now())
                )
            _audit(
                connection,
                actor_id=actor_id,
                event_type="document_chunks.generated",
                decision="completed",
                resource_type="work_item" if work_item_id is not None else "normalized_document",
                resource_id=work_item_id or document_ids[0],
                correlation_id=work_item_id or document_ids[0],
                details={
                    "document_ids": document_ids,
                    "chunk_count": len(created_chunk_ids),
                    "evidence_count": len(created_evidence_ids),
                    "skipped_document_ids": skipped_document_ids,
                },
            )
    except Exception as exc:
        with engine.begin() as connection:
            if work_item_id is not None:
                connection.execute(
                    update(work_items)
                    .where(work_items.c.id == work_item_id)
                    .values(status="failed", error_message=str(exc), updated_at=func.now())
                )
            _audit(
                connection,
                actor_id=actor_id,
                event_type="document_chunks.failed",
                decision="failed",
                resource_type="work_item" if work_item_id is not None else "normalized_document",
                resource_id=work_item_id or (document_ids[0] if document_ids else "none"),
                correlation_id=work_item_id or (document_ids[0] if document_ids else "none"),
                details={
                    "document_ids": document_ids,
                    "error_message": str(exc),
                },
            )
        return {
            "status": "failed",
            "work_item_id": work_item_id,
            "document_ids": document_ids,
            "error_message": str(exc),
        }

    return {
        "status": "completed",
        "work_item_id": work_item_id,
        "document_ids": document_ids,
        "created_chunk_ids": created_chunk_ids,
        "created_evidence_ids": created_evidence_ids,
        "skipped_document_ids": skipped_document_ids,
    }
