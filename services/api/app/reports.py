from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.artifact_store import ArtifactStoreError, store_artifact_bytes
from app.config import get_settings
from app.db import get_db
from app.models import (
    Approval,
    AuditEvent,
    CollectionRun,
    EvidenceItem,
    ExperimentRun,
    FeedbackEvent,
    ImprovementItem,
    Outcome,
    Pattern,
    RawArtifact,
    Recommendation,
    Report,
    Source,
    WorkItem,
)
from app.work_items import WorkItemRead

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


class ReportStatusUpdate(BaseModel):
    status: str = Field(min_length=1, max_length=64)
    actor_id: str = "local-owner"
    artifact_path: str | None = None

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in REPORT_STATUSES:
            raise ValueError(f"Unknown report status: {value}")
        return value


class ReportWorkItemCreate(BaseModel):
    requested_by_actor_id: str = "local-owner"
    notes: str | None = None


class ReportRenderRequest(BaseModel):
    actor_id: str = "local-owner"
    notes: str | None = None


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


def _audit_report_status_updated(
    db: Session,
    *,
    actor_id: str,
    report: Report,
    previous_status: str,
    previous_artifact_path: str | None,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="report.status_updated",
            decision=report.status,
            resource_type="report",
            resource_id=report.id,
            correlation_id=None,
            details_json={
                "previous_status": previous_status,
                "new_status": report.status,
                "previous_artifact_path": previous_artifact_path,
                "new_artifact_path": report.artifact_path,
            },
        )
    )


def _audit_report_work_item_created(db: Session, work_item: WorkItem, report: Report) -> None:
    db.add(
        AuditEvent(
            actor_id=work_item.requested_by_actor_id,
            event_type="work_item.created",
            decision="queued",
            resource_type="work_item",
            resource_id=work_item.id,
            correlation_id=report.id,
            details_json={
                "work_type": work_item.work_type,
                "status": work_item.status,
                "report_id": report.id,
                "report_type": report.report_type,
                "scaffold_only": True,
            },
        )
    )


def _audit_report_rendered(
    db: Session,
    *,
    actor_id: str,
    report: Report,
    artifact: RawArtifact,
    content_already_existed: bool,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="report.rendered",
            decision="ready",
            resource_type="report",
            resource_id=report.id,
            correlation_id=artifact.id,
            details_json={
                "artifact_id": artifact.id,
                "artifact_path": artifact.storage_path,
                "content_hash": artifact.content_hash,
                "content_already_existed": content_already_existed,
            },
        )
    )


def _latest_titles(records: list[Any], label: str, field_name: str) -> list[str]:
    lines: list[str] = []
    for record in records[:5]:
        value = getattr(record, field_name)
        status_value = getattr(record, "status", None)
        suffix = f" [{status_value}]" if status_value is not None else ""
        lines.append(f"- {label}: {value}{suffix}")
    return lines


def build_report_markdown(report: Report, counts: dict[str, int], sections: list[str], notes: str | None) -> str:
    lines = [
        f"# {report.title}",
        "",
        f"- Report ID: `{report.id}`",
        f"- Report type: `{report.report_type}`",
        f"- Requested by: `{report.requested_by_actor_id}`",
        f"- Status before render: `{report.status}`",
        "",
        "## Inventory Counts",
        "",
    ]
    for key in sorted(counts):
        lines.append(f"- {key}: {counts[key]}")
    lines.extend(["", "## Recent Records", ""])
    lines.extend(sections or ["- No recent records available."])
    lines.extend(
        [
            "",
            "## Boundaries",
            "",
            "- This report is generated from local metadata records only.",
            "- It does not read raw artifact contents.",
            "- It does not call external models or execute actions.",
        ]
    )
    if notes:
        lines.extend(["", "## Notes", "", notes])
    lines.append("")
    return "\n".join(lines)


def _build_report_content(db: Session, report: Report, notes: str | None) -> str:
    sources = list(db.scalars(select(Source).order_by(Source.created_at.desc()).limit(5)).all())
    evidence = list(db.scalars(select(EvidenceItem).order_by(EvidenceItem.created_at.desc()).limit(5)).all())
    patterns = list(db.scalars(select(Pattern).order_by(Pattern.created_at.desc()).limit(5)).all())
    recommendations = list(db.scalars(select(Recommendation).order_by(Recommendation.created_at.desc()).limit(5)).all())

    counts = {
        "approvals": len(list(db.scalars(select(Approval.id)).all())),
        "artifacts": len(list(db.scalars(select(RawArtifact.id)).all())),
        "collection_runs": len(list(db.scalars(select(CollectionRun.id)).all())),
        "evidence_items": len(list(db.scalars(select(EvidenceItem.id)).all())),
        "experiments": len(list(db.scalars(select(ExperimentRun.id)).all())),
        "feedback_events": len(list(db.scalars(select(FeedbackEvent.id)).all())),
        "improvement_items": len(list(db.scalars(select(ImprovementItem.id)).all())),
        "outcomes": len(list(db.scalars(select(Outcome.id)).all())),
        "patterns": len(list(db.scalars(select(Pattern.id)).all())),
        "recommendations": len(list(db.scalars(select(Recommendation.id)).all())),
        "sources": len(list(db.scalars(select(Source.id)).all())),
        "work_items": len(list(db.scalars(select(WorkItem.id)).all())),
    }
    sections = [
        *_latest_titles(sources, "source", "name"),
        *_latest_titles(evidence, "evidence", "statement"),
        *_latest_titles(patterns, "pattern", "summary"),
        *_latest_titles(recommendations, "recommendation", "recommendation_text"),
    ]
    return build_report_markdown(report, counts, sections, notes)


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


@router.post("/{report_id}/status", response_model=ReportRead)
def update_report_status(
    report_id: str,
    payload: ReportStatusUpdate,
    db: Session = Depends(get_db),
) -> Report:
    report = db.get(Report, report_id)
    if report is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Report not found")

    previous_status = report.status
    previous_artifact_path = report.artifact_path
    report.status = payload.status
    if payload.artifact_path is not None:
        report.artifact_path = payload.artifact_path
    _audit_report_status_updated(
        db,
        actor_id=payload.actor_id,
        report=report,
        previous_status=previous_status,
        previous_artifact_path=previous_artifact_path,
    )
    db.commit()
    db.refresh(report)
    return report


@router.post("/{report_id}/work-item", response_model=WorkItemRead, status_code=status.HTTP_201_CREATED)
def create_report_work_item(
    report_id: str,
    payload: ReportWorkItemCreate,
    db: Session = Depends(get_db),
) -> WorkItem:
    report = db.get(Report, report_id)
    if report is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Report not found")

    work_item = WorkItem(
        id=str(uuid4()),
        work_type="report_generation",
        status="queued",
        requested_by_actor_id=payload.requested_by_actor_id,
        payload_json={
            "report_id": report.id,
            "report_type": report.report_type,
            "report_status": report.status,
            "scaffold_only": True,
            "executes_report_generation": False,
            "notes": payload.notes,
        },
        error_message=None,
    )
    db.add(work_item)
    _audit_report_work_item_created(db, work_item, report)
    db.commit()
    db.refresh(work_item)
    return work_item


@router.post("/{report_id}/render", response_model=ReportRead)
def render_report(
    report_id: str,
    payload: ReportRenderRequest,
    db: Session = Depends(get_db),
) -> Report:
    report = db.get(Report, report_id)
    if report is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Report not found")

    content = _build_report_content(db, report, payload.notes).encode("utf-8")
    try:
        stored = store_artifact_bytes(content, get_settings().artifact_store_path)
    except ArtifactStoreError as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    artifact = RawArtifact(
        id=str(uuid4()),
        source_id=None,
        collection_run_id=None,
        content_hash=stored.content_hash,
        storage_path=stored.storage_path,
        mime_type="text/markdown",
        size_bytes=stored.size_bytes,
        metadata_json={
            "generated_by": "DIFF-067",
            "artifact_kind": "report",
            "report_id": report.id,
            "report_type": report.report_type,
            "filename": f"{report.id}.md",
        },
    )
    db.add(artifact)
    report.status = "ready"
    report.artifact_path = stored.storage_path
    report.metadata_json = {
        **(report.metadata_json or {}),
        "rendered_artifact_id": artifact.id,
        "rendered_mime_type": "text/markdown",
    }
    _audit_report_rendered(
        db,
        actor_id=payload.actor_id,
        report=report,
        artifact=artifact,
        content_already_existed=stored.existed,
    )
    db.commit()
    db.refresh(report)
    return report


@router.get("/{report_id}", response_model=ReportRead)
def get_report(report_id: str, db: Session = Depends(get_db)) -> Report:
    report = db.get(Report, report_id)
    if report is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Report not found")
    return report
