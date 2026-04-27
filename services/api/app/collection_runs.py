from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, CollectionRun, Source

router = APIRouter(prefix="/collection-runs", tags=["collection-runs"])


class CollectionRunCreate(BaseModel):
    source_id: str | None = None
    requested_by_actor_id: str = "local-owner"
    summary_json: dict[str, Any] = Field(default_factory=dict)
    dry_run: bool = True


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


@router.get("/{collection_run_id}", response_model=CollectionRunRead)
def get_collection_run(collection_run_id: str, db: Session = Depends(get_db)) -> CollectionRun:
    collection_run = db.get(CollectionRun, collection_run_id)
    if collection_run is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Collection run not found")
    return collection_run
