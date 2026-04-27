from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session, selectinload

from app.db import get_db
from app.models import AuditEvent, Source, SourcePermission

router = APIRouter(prefix="/sources", tags=["sources"])


class SourcePermissionCreate(BaseModel):
    scope_json: dict[str, Any] = Field(default_factory=dict)
    allowed_operations: list[str] = Field(default_factory=list)
    external_model_policy: str = "blocked"
    approval_required: bool = True
    created_by_actor_id: str = "local-owner"


class SourcePermissionRead(SourcePermissionCreate):
    id: str
    source_id: str
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class SourceCreate(BaseModel):
    name: str = Field(min_length=1, max_length=255)
    source_type: str = Field(min_length=1, max_length=64)
    location: str | None = None
    owner_actor_id: str = "local-owner"
    sensitivity: str = "internal"
    trust_level: str = "unreviewed"
    enabled: bool = True
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    permission: SourcePermissionCreate | None = None


class SourceRead(BaseModel):
    id: str
    name: str
    source_type: str
    location: str | None
    owner_actor_id: str
    sensitivity: str
    trust_level: str
    enabled: bool
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime
    permissions: list[SourcePermissionRead] = Field(default_factory=list)

    model_config = {"from_attributes": True}


def _audit(
    db: Session,
    *,
    actor_id: str,
    event_type: str,
    resource_type: str,
    resource_id: str,
    details: dict[str, Any],
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type=event_type,
            decision="recorded",
            resource_type=resource_type,
            resource_id=resource_id,
            correlation_id=None,
            details_json=details,
        )
    )


@router.get("", response_model=list[SourceRead])
def list_sources(db: Session = Depends(get_db)) -> list[Source]:
    statement = select(Source).options(selectinload(Source.permissions)).order_by(Source.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=SourceRead, status_code=status.HTTP_201_CREATED)
def create_source(payload: SourceCreate, db: Session = Depends(get_db)) -> Source:
    source = Source(
        id=str(uuid4()),
        name=payload.name,
        source_type=payload.source_type,
        location=payload.location,
        owner_actor_id=payload.owner_actor_id,
        sensitivity=payload.sensitivity,
        trust_level=payload.trust_level,
        enabled=payload.enabled,
        metadata_json=payload.metadata_json,
    )
    db.add(source)

    if payload.permission is not None:
        db.add(
            SourcePermission(
                id=str(uuid4()),
                source=source,
                scope_json=payload.permission.scope_json,
                allowed_operations=payload.permission.allowed_operations,
                external_model_policy=payload.permission.external_model_policy,
                approval_required=payload.permission.approval_required,
                created_by_actor_id=payload.permission.created_by_actor_id,
            )
        )

    _audit(
        db,
        actor_id=payload.owner_actor_id,
        event_type="source.created",
        resource_type="source",
        resource_id=source.id,
        details={
            "source_type": payload.source_type,
            "sensitivity": payload.sensitivity,
            "permission_included": payload.permission is not None,
        },
    )
    db.commit()
    db.refresh(source)
    return db.scalar(
        select(Source).options(selectinload(Source.permissions)).where(Source.id == source.id)
    ) or source


@router.get("/{source_id}", response_model=SourceRead)
def get_source(source_id: str, db: Session = Depends(get_db)) -> Source:
    source = db.scalar(
        select(Source).options(selectinload(Source.permissions)).where(Source.id == source_id)
    )
    if source is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    return source


@router.get("/{source_id}/permissions", response_model=list[SourcePermissionRead])
def list_source_permissions(source_id: str, db: Session = Depends(get_db)) -> list[SourcePermission]:
    if db.get(Source, source_id) is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")
    statement = select(SourcePermission).where(SourcePermission.source_id == source_id)
    return list(db.scalars(statement).all())


@router.post(
    "/{source_id}/permissions",
    response_model=SourcePermissionRead,
    status_code=status.HTTP_201_CREATED,
)
def create_source_permission(
    source_id: str,
    payload: SourcePermissionCreate,
    db: Session = Depends(get_db),
) -> SourcePermission:
    if db.get(Source, source_id) is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")

    permission = SourcePermission(
        id=str(uuid4()),
        source_id=source_id,
        scope_json=payload.scope_json,
        allowed_operations=payload.allowed_operations,
        external_model_policy=payload.external_model_policy,
        approval_required=payload.approval_required,
        created_by_actor_id=payload.created_by_actor_id,
    )
    db.add(permission)
    _audit(
        db,
        actor_id=payload.created_by_actor_id,
        event_type="source_permission.created",
        resource_type="source",
        resource_id=source_id,
        details={
            "permission_id": permission.id,
            "allowed_operations": payload.allowed_operations,
            "approval_required": payload.approval_required,
            "external_model_policy": payload.external_model_policy,
        },
    )
    db.commit()
    db.refresh(permission)
    return permission
