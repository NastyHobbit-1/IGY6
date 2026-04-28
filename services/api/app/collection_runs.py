from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, CollectionRun, Source, SourcePermission

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


def _synthesize_dry_run_summary(source: Source, permission: SourcePermission, notes: dict[str, Any]) -> dict[str, Any]:
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
            "external_model_policy": permission.external_model_policy,
            "approval_required": permission.approval_required,
        },
        "preview": {
            "mode": "dry_run_preview",
            "would_collect": False,
            "would_create_artifacts": False,
            "would_normalize": False,
            "would_enqueue_worker": False,
        },
        "notes": notes,
    }


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

    permission = db.get(SourcePermission, payload.source_permission_id)
    if permission is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source permission not found")
    if permission.source_id != source.id:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Source permission does not belong to the source",
        )

    collection_run = CollectionRun(
        id=str(uuid4()),
        source_id=source.id,
        status="dry_run_previewed",
        dry_run=True,
        requested_by_actor_id=payload.requested_by_actor_id,
        summary_json=_synthesize_dry_run_summary(source, permission, payload.notes),
        error_message=None,
    )
    db.add(collection_run)
    _audit_collection_run_created(db, collection_run)
    db.add(
        AuditEvent(
            actor_id=payload.requested_by_actor_id,
            event_type="collection_run.dry_run_preview",
            decision="recorded",
            resource_type="collection_run",
            resource_id=collection_run.id,
            correlation_id=None,
            details_json={
                "source_id": source.id,
                "source_permission_id": permission.id,
            },
        )
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
