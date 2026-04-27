from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, FeedbackEvent

router = APIRouter(prefix="/feedback", tags=["feedback"])

TARGET_TYPES = {
    "source",
    "document",
    "evidence_item",
    "claim",
    "pattern",
    "hypothesis",
    "prediction",
    "recommendation",
    "report",
    "work_item",
}

FEEDBACK_LABELS = {
    "useful",
    "not_useful",
    "wrong",
    "verified",
    "incomplete",
    "noisy",
    "trusted",
    "rejected",
}


class FeedbackCreate(BaseModel):
    target_type: str = Field(min_length=1, max_length=64)
    target_id: str = Field(min_length=1, max_length=36)
    label: str = Field(min_length=1, max_length=64)
    actor_id: str = "local-owner"
    note: str | None = None
    metadata_json: dict[str, Any] = Field(default_factory=dict)

    @field_validator("target_type")
    @classmethod
    def validate_target_type(cls, value: str) -> str:
        if value not in TARGET_TYPES:
            raise ValueError(f"Unknown feedback target type: {value}")
        return value

    @field_validator("label")
    @classmethod
    def validate_label(cls, value: str) -> str:
        if value not in FEEDBACK_LABELS:
            raise ValueError(f"Unknown feedback label: {value}")
        return value


class FeedbackRead(BaseModel):
    id: str
    target_type: str
    target_id: str
    label: str
    actor_id: str
    note: str | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_feedback_created(db: Session, feedback: FeedbackEvent) -> None:
    db.add(
        AuditEvent(
            actor_id=feedback.actor_id,
            event_type="feedback.created",
            decision="recorded",
            resource_type=feedback.target_type,
            resource_id=feedback.target_id,
            correlation_id=None,
            details_json={
                "feedback_id": feedback.id,
                "label": feedback.label,
            },
        )
    )


@router.get("", response_model=list[FeedbackRead])
def list_feedback(db: Session = Depends(get_db)) -> list[FeedbackEvent]:
    statement = select(FeedbackEvent).order_by(FeedbackEvent.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=FeedbackRead, status_code=status.HTTP_201_CREATED)
def create_feedback(payload: FeedbackCreate, db: Session = Depends(get_db)) -> FeedbackEvent:
    feedback = FeedbackEvent(
        id=str(uuid4()),
        target_type=payload.target_type,
        target_id=payload.target_id,
        label=payload.label,
        actor_id=payload.actor_id,
        note=payload.note,
        metadata_json=payload.metadata_json,
    )
    db.add(feedback)
    _audit_feedback_created(db, feedback)
    db.commit()
    db.refresh(feedback)
    return feedback


@router.get("/{feedback_id}", response_model=FeedbackRead)
def get_feedback(feedback_id: str, db: Session = Depends(get_db)) -> FeedbackEvent:
    feedback = db.get(FeedbackEvent, feedback_id)
    if feedback is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Feedback event not found")
    return feedback
