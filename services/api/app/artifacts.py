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
from app.config import get_settings
from app.db import get_db
from app.models import AuditEvent, CollectionRun, RawArtifact, Source

router = APIRouter(prefix="/artifacts", tags=["artifacts"])


class RawArtifactCreate(BaseModel):
    source_id: str | None = None
    collection_run_id: str | None = None
    content_base64: str
    mime_type: str | None = None
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    requested_by_actor_id: str = "local-owner"


class RawArtifactRead(BaseModel):
    id: str
    source_id: str | None
    collection_run_id: str | None
    content_hash: str
    storage_path: str
    mime_type: str | None
    size_bytes: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _decode_content_base64(content_base64: str) -> bytes:
    try:
        return base64.b64decode(content_base64, validate=True)
    except (binascii.Error, ValueError) as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Invalid base64 content") from exc


def _audit_artifact_created(
    db: Session,
    *,
    artifact: RawArtifact,
    actor_id: str,
    existed: bool,
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
                "content_already_existed": existed,
            },
        )
    )


@router.get("", response_model=list[RawArtifactRead])
def list_raw_artifacts(db: Session = Depends(get_db)) -> list[RawArtifact]:
    statement = select(RawArtifact).order_by(RawArtifact.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=RawArtifactRead, status_code=status.HTTP_201_CREATED)
def create_raw_artifact(payload: RawArtifactCreate, db: Session = Depends(get_db)) -> RawArtifact:
    if payload.source_id is not None and db.get(Source, payload.source_id) is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")

    if payload.collection_run_id is not None:
        collection_run = db.get(CollectionRun, payload.collection_run_id)
        if collection_run is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Collection run not found")
        if payload.source_id is not None and collection_run.source_id != payload.source_id:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Collection run does not belong to the source",
            )

    content = _decode_content_base64(payload.content_base64)
    try:
        stored = store_artifact_bytes(content, get_settings().artifact_store_path)
    except ArtifactStoreError as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    artifact = RawArtifact(
        id=str(uuid4()),
        source_id=payload.source_id,
        collection_run_id=payload.collection_run_id,
        content_hash=stored.content_hash,
        storage_path=stored.storage_path,
        mime_type=payload.mime_type,
        size_bytes=stored.size_bytes,
        metadata_json=payload.metadata_json,
    )
    db.add(artifact)
    _audit_artifact_created(
        db,
        artifact=artifact,
        actor_id=payload.requested_by_actor_id,
        existed=stored.existed,
    )
    db.commit()
    db.refresh(artifact)
    return artifact


@router.get("/{artifact_id}", response_model=RawArtifactRead)
def get_raw_artifact(artifact_id: str, db: Session = Depends(get_db)) -> RawArtifact:
    artifact = db.get(RawArtifact, artifact_id)
    if artifact is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Artifact not found")
    return artifact
