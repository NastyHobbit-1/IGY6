from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, ImprovementItem

router = APIRouter(prefix="/improvements", tags=["improvements"])

TARGET_AREAS = {
    "parsing",
    "retrieval",
    "scoring",
    "prediction",
    "reporting",
    "reasoning",
    "safety",
}

PRIORITIES = {
    "low",
    "normal",
    "high",
    "urgent",
}


class ImprovementItemCreate(BaseModel):
    target_area: str = Field(min_length=1, max_length=64)
    objective: str = Field(min_length=1)
    proposed_by_actor_id: str = "local-owner"
    priority: str = "normal"
    metadata_json: dict[str, Any] = Field(default_factory=dict)

    @field_validator("target_area")
    @classmethod
    def validate_target_area(cls, value: str) -> str:
        if value not in TARGET_AREAS:
            raise ValueError(f"Unknown improvement target area: {value}")
        return value

    @field_validator("priority")
    @classmethod
    def validate_priority(cls, value: str) -> str:
        if value not in PRIORITIES:
            raise ValueError(f"Unknown improvement priority: {value}")
        return value


class ImprovementItemRead(BaseModel):
    id: str
    target_area: str
    status: str
    objective: str
    proposed_by_actor_id: str
    priority: str
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_improvement_created(db: Session, item: ImprovementItem) -> None:
    db.add(
        AuditEvent(
            actor_id=item.proposed_by_actor_id,
            event_type="improvement_item.created",
            decision="proposed",
            resource_type="improvement_item",
            resource_id=item.id,
            correlation_id=None,
            details_json={
                "target_area": item.target_area,
                "priority": item.priority,
                "status": item.status,
            },
        )
    )


@router.get("", response_model=list[ImprovementItemRead])
def list_improvement_items(db: Session = Depends(get_db)) -> list[ImprovementItem]:
    statement = select(ImprovementItem).order_by(ImprovementItem.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=ImprovementItemRead, status_code=status.HTTP_201_CREATED)
def create_improvement_item(
    payload: ImprovementItemCreate,
    db: Session = Depends(get_db),
) -> ImprovementItem:
    item = ImprovementItem(
        id=str(uuid4()),
        target_area=payload.target_area,
        status="proposed",
        objective=payload.objective,
        proposed_by_actor_id=payload.proposed_by_actor_id,
        priority=payload.priority,
        metadata_json=payload.metadata_json,
    )
    db.add(item)
    _audit_improvement_created(db, item)
    db.commit()
    db.refresh(item)
    return item


@router.get("/{improvement_item_id}", response_model=ImprovementItemRead)
def get_improvement_item(
    improvement_item_id: str,
    db: Session = Depends(get_db),
) -> ImprovementItem:
    item = db.get(ImprovementItem, improvement_item_id)
    if item is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Improvement item not found")
    return item
