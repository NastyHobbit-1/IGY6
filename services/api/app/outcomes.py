from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import (
    AuditEvent,
    EvidenceItem,
    Hypothesis,
    Outcome,
    Pattern,
    Prediction,
    Recommendation,
    Report,
    WorkItem,
)

router = APIRouter(prefix="/outcomes", tags=["outcomes"])

TARGET_TYPES = {
    "prediction",
    "recommendation",
    "work_item",
    "hypothesis",
    "pattern",
    "report",
}

OUTCOME_STATUSES = {
    "correct",
    "wrong",
    "useful",
    "not_useful",
    "partial",
    "inconclusive",
    "confirmed",
    "disconfirmed",
}

TARGET_MODELS = {
    "prediction": Prediction,
    "recommendation": Recommendation,
    "work_item": WorkItem,
    "hypothesis": Hypothesis,
    "pattern": Pattern,
    "report": Report,
}


class OutcomeCreate(BaseModel):
    target_type: str = Field(min_length=1, max_length=64)
    target_id: str = Field(min_length=1, max_length=36)
    outcome_status: str = Field(min_length=1, max_length=64)
    summary: str | None = None
    occurred_at: datetime | None = None
    evidence_ids: list[str] = Field(default_factory=list)
    metadata_json: dict[str, Any] = Field(default_factory=dict)

    @field_validator("target_type")
    @classmethod
    def validate_target_type(cls, value: str) -> str:
        if value not in TARGET_TYPES:
            raise ValueError(f"Unknown outcome target type: {value}")
        return value

    @field_validator("outcome_status")
    @classmethod
    def validate_outcome_status(cls, value: str) -> str:
        if value not in OUTCOME_STATUSES:
            raise ValueError(f"Unknown outcome status: {value}")
        return value


class OutcomeRead(BaseModel):
    id: str
    target_type: str
    target_id: str
    outcome_status: str
    summary: str | None
    occurred_at: datetime | None
    evidence_ids: list[str] = Field(default_factory=list)
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_outcome_created(db: Session, outcome: Outcome) -> None:
    db.add(
        AuditEvent(
            actor_id="local-owner",
            event_type="outcome.created",
            decision="recorded",
            resource_type=outcome.target_type,
            resource_id=outcome.target_id,
            correlation_id=None,
            details_json={
                "outcome_id": outcome.id,
                "outcome_status": outcome.outcome_status,
            },
        )
    )


def _validate_target_exists(db: Session, target_type: str, target_id: str) -> None:
    target_model = TARGET_MODELS[target_type]
    if db.get(target_model, target_id) is None:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={
                "message": "Outcome target record does not exist",
                "target_type": target_type,
                "target_id": target_id,
            },
        )


def _validated_evidence_ids(db: Session, evidence_ids: list[str]) -> list[str]:
    unique_ids = list(dict.fromkeys(evidence_ids))
    if not unique_ids:
        return unique_ids

    statement = select(EvidenceItem.id).where(EvidenceItem.id.in_(unique_ids))
    found_ids = set(db.scalars(statement).all())
    missing_ids = [evidence_id for evidence_id in unique_ids if evidence_id not in found_ids]
    if missing_ids:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={
                "message": "Outcome records must reference existing evidence items",
                "missing_evidence_ids": missing_ids,
            },
        )
    return unique_ids


@router.get("", response_model=list[OutcomeRead])
def list_outcomes(db: Session = Depends(get_db)) -> list[Outcome]:
    statement = select(Outcome).order_by(Outcome.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=OutcomeRead, status_code=status.HTTP_201_CREATED)
def create_outcome(payload: OutcomeCreate, db: Session = Depends(get_db)) -> Outcome:
    _validate_target_exists(db, payload.target_type, payload.target_id)
    evidence_ids = _validated_evidence_ids(db, payload.evidence_ids)
    outcome = Outcome(
        id=str(uuid4()),
        target_type=payload.target_type,
        target_id=payload.target_id,
        outcome_status=payload.outcome_status,
        summary=payload.summary,
        occurred_at=payload.occurred_at,
        evidence_ids=evidence_ids,
        metadata_json=payload.metadata_json,
    )
    db.add(outcome)
    _audit_outcome_created(db, outcome)
    db.commit()
    db.refresh(outcome)
    return outcome


@router.get("/{outcome_id}", response_model=OutcomeRead)
def get_outcome(outcome_id: str, db: Session = Depends(get_db)) -> Outcome:
    outcome = db.get(Outcome, outcome_id)
    if outcome is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Outcome not found")
    return outcome
