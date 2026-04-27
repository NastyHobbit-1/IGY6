from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, Report

router = APIRouter(prefix="/reports", tags=["reports"])

REPORT_TYPES = {
    "summary",
    "evidence_review",
    "decision_note",
    "handoff",
    "experiment_summary",
}

REPORT_STATUSES = {
    "placeholder",
    "requested",
    "draft",
    "ready",
    "archived",
}


class ReportCreate(BaseModel):
    title: str = Field(min_length=1, max_length=255)
    report_type: str = Field(min_length=1, max_length=64)
    status: str = "requested"
    requested_by_actor_id: str = "local-owner"
    artifact_path: str | None = None
    metadata_json: dict[str, Any] = Field(default_factory=dict)

    @field_validator("report_type")
    @classmethod
    def validate_report_type(cls, value: str) -> str:
        if value not in REPORT_TYPES:
            raise ValueError(f"Unknown report type: {value}")
        return value

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in REPORT_STATUSES:
            raise ValueError(f"Unknown report status: {value}")
        return value


class ReportRead(BaseModel):
    id: str
    title: str
    report_type: str
    status: str
    requested_by_actor_id: str
    artifact_path: str | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_report_created(db: Session, report: Report) -> None:
    db.add(
        AuditEvent(
            actor_id=report.requested_by_actor_id,
            event_type="report.created",
            decision="recorded",
            resource_type="report",
            resource_id=report.id,
            correlation_id=None,
            details_json={
                "report_type": report.report_type,
                "status": report.status,
            },
        )
    )


@router.get("", response_model=list[ReportRead])
def list_reports(db: Session = Depends(get_db)) -> list[Report]:
    statement = select(Report).order_by(Report.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=ReportRead, status_code=status.HTTP_201_CREATED)
def create_report(payload: ReportCreate, db: Session = Depends(get_db)) -> Report:
    report = Report(
        id=str(uuid4()),
        title=payload.title,
        report_type=payload.report_type,
        status=payload.status,
        requested_by_actor_id=payload.requested_by_actor_id,
        artifact_path=payload.artifact_path,
        metadata_json=payload.metadata_json,
    )
    db.add(report)
    _audit_report_created(db, report)
    db.commit()
    db.refresh(report)
    return report


@router.get("/{report_id}", response_model=ReportRead)
def get_report(report_id: str, db: Session = Depends(get_db)) -> Report:
    report = db.get(Report, report_id)
    if report is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Report not found")
    return report
