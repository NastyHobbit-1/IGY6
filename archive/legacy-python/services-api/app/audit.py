from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent

router = APIRouter(prefix="/audit-events", tags=["audit"])


class AuditEventRead(BaseModel):
    id: int
    created_at: datetime
    actor_id: str
    event_type: str
    decision: str | None
    resource_type: str | None
    resource_id: str | None
    correlation_id: str | None
    details_json: dict[str, Any]

    model_config = {"from_attributes": True}


@router.get("", response_model=list[AuditEventRead])
def list_audit_events(db: Session = Depends(get_db)) -> list[AuditEvent]:
    statement = select(AuditEvent).order_by(AuditEvent.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/{audit_event_id}", response_model=AuditEventRead)
def get_audit_event(audit_event_id: int, db: Session = Depends(get_db)) -> AuditEvent:
    audit_event = db.get(AuditEvent, audit_event_id)
    if audit_event is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Audit event not found")
    return audit_event
