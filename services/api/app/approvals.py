from datetime import UTC, datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import Approval, AuditEvent

router = APIRouter(prefix="/approvals", tags=["approvals"])

APPROVAL_DECISIONS = {"approved", "denied"}


class ApprovalCreate(BaseModel):
    request_type: str = Field(min_length=1, max_length=64)
    requested_by_actor_id: str = "local-owner"
    request_payload_json: dict[str, Any] = Field(default_factory=dict)


class ApprovalDecision(BaseModel):
    status: str
    decided_by_actor_id: str = "local-owner"
    decision_reason: str | None = None

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in APPROVAL_DECISIONS:
            raise ValueError("Approval decision must be approved or denied")
        return value


class ApprovalRead(BaseModel):
    id: str
    request_type: str
    status: str
    requested_by_actor_id: str
    decided_by_actor_id: str | None
    decision_reason: str | None
    request_payload_json: dict[str, Any]
    decided_at: datetime | None
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit(
    db: Session,
    *,
    actor_id: str,
    event_type: str,
    decision: str,
    approval: Approval,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type=event_type,
            decision=decision,
            resource_type="approval",
            resource_id=approval.id,
            correlation_id=None,
            details_json={
                "request_type": approval.request_type,
                "status": approval.status,
            },
        )
    )


@router.get("", response_model=list[ApprovalRead])
def list_approvals(db: Session = Depends(get_db)) -> list[Approval]:
    statement = select(Approval).order_by(Approval.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=ApprovalRead, status_code=status.HTTP_201_CREATED)
def create_approval(payload: ApprovalCreate, db: Session = Depends(get_db)) -> Approval:
    approval = Approval(
        id=str(uuid4()),
        request_type=payload.request_type,
        status="pending",
        requested_by_actor_id=payload.requested_by_actor_id,
        decided_by_actor_id=None,
        decision_reason=None,
        request_payload_json=payload.request_payload_json,
        decided_at=None,
    )
    db.add(approval)
    _audit(
        db,
        actor_id=payload.requested_by_actor_id,
        event_type="approval.requested",
        decision="pending",
        approval=approval,
    )
    db.commit()
    db.refresh(approval)
    return approval


@router.get("/{approval_id}", response_model=ApprovalRead)
def get_approval(approval_id: str, db: Session = Depends(get_db)) -> Approval:
    approval = db.get(Approval, approval_id)
    if approval is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Approval not found")
    return approval


@router.post("/{approval_id}/decision", response_model=ApprovalRead)
def decide_approval(
    approval_id: str,
    payload: ApprovalDecision,
    db: Session = Depends(get_db),
) -> Approval:
    approval = db.get(Approval, approval_id)
    if approval is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Approval not found")
    if approval.status != "pending":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Approval already decided")

    approval.status = payload.status
    approval.decided_by_actor_id = payload.decided_by_actor_id
    approval.decision_reason = payload.decision_reason
    approval.decided_at = datetime.now(UTC)
    _audit(
        db,
        actor_id=payload.decided_by_actor_id,
        event_type="approval.decided",
        decision=payload.status,
        approval=approval,
    )
    db.commit()
    db.refresh(approval)
    return approval
