from datetime import datetime
from typing import Any
from uuid import uuid4

from celery import Celery
from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.models import AuditEvent, WorkItem

router = APIRouter(prefix="/work-items", tags=["work-items"])

WORK_ITEM_STATUSES = {
    "pending_intent_verification",
    "queued",
    "running",
    "completed",
    "failed",
    "canceled",
}


class IntentVerificationContext(BaseModel):
    original_request: str = Field(min_length=1)
    interpretation: str = Field(min_length=1)
    proposed_work_type: str = Field(min_length=1)
    expected_output: str = Field(min_length=1)
    safety_requirements: list[str] = Field(default_factory=list)
    assumptions: list[str] = Field(default_factory=list)
    missing_information: list[str] = Field(default_factory=list)
    sources_likely_used: list[str] = Field(default_factory=list)


class WorkItemCreate(BaseModel):
    work_type: str = Field(min_length=1, max_length=64)
    requested_by_actor_id: str = "local-owner"
    intent: IntentVerificationContext
    payload_json: dict[str, Any] = Field(default_factory=dict)


class WorkItemRead(BaseModel):
    id: str
    work_type: str
    status: str
    requested_by_actor_id: str
    payload_json: dict[str, Any]
    error_message: str | None
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class WorkItemStatusUpdate(BaseModel):
    status: str = Field(min_length=1, max_length=64)
    actor_id: str = "local-owner"
    error_message: str | None = None


class WorkItemDispatchRequest(BaseModel):
    actor_id: str = "local-owner"


class WorkItemDispatchResult(BaseModel):
    work_item_id: str
    work_type: str
    task_name: str
    task_id: str
    status: str


def _celery_app(settings: Settings) -> Celery:
    return Celery("igy6_api_dispatch", broker=settings.celery_broker_url, backend=settings.celery_result_backend)


def build_dispatch_plan(work_item: WorkItem) -> tuple[str, list[Any], dict[str, Any]]:
    payload = work_item.payload_json or {}
    if work_item.work_type == "collection_normalization":
        collection_run_id = payload.get("collection_run_id")
        raw_artifact_ids = payload.get("raw_artifact_ids")
        if not isinstance(collection_run_id, str) or not isinstance(raw_artifact_ids, list):
            raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Invalid normalization payload")
        return "collection.normalize_collection_run", [work_item.id, collection_run_id, raw_artifact_ids], {}

    if work_item.work_type == "document_chunking":
        document_ids = payload.get("document_ids")
        if document_ids is None and isinstance(payload.get("document_id"), str):
            document_ids = [payload["document_id"]]
        if not isinstance(document_ids, list):
            raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Invalid document chunking payload")
        chunk_size = payload.get("chunk_size", 1000)
        return "evidence.generate_document_chunks", [document_ids], {"chunk_size": chunk_size, "work_item_id": work_item.id}

    if work_item.work_type == "chunk_vector_upsert":
        limit = payload.get("limit", 100)
        return "memory.vector.upsert_chunks", [], {"limit": limit, "work_item_id": work_item.id}

    raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Unsupported work item dispatch type")


def _audit_work_item_created(db: Session, work_item: WorkItem) -> None:
    db.add(
        AuditEvent(
            actor_id=work_item.requested_by_actor_id,
            event_type="work_item.created",
            decision="intent_verification_required",
            resource_type="work_item",
            resource_id=work_item.id,
            correlation_id=None,
            details_json={
                "work_type": work_item.work_type,
                "status": work_item.status,
            },
        )
    )


def _audit_work_item_status_updated(
    db: Session,
    *,
    actor_id: str,
    work_item: WorkItem,
    previous_status: str,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="work_item.status_updated",
            decision=work_item.status,
            resource_type="work_item",
            resource_id=work_item.id,
            correlation_id=None,
            details_json={
                "previous_status": previous_status,
                "new_status": work_item.status,
                "error_message": work_item.error_message,
            },
        )
    )


def _audit_work_item_dispatched(
    db: Session,
    *,
    actor_id: str,
    work_item: WorkItem,
    task_name: str,
    task_id: str,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="work_item.dispatched",
            decision="dispatched",
            resource_type="work_item",
            resource_id=work_item.id,
            correlation_id=task_id,
            details_json={
                "work_type": work_item.work_type,
                "task_name": task_name,
                "task_id": task_id,
            },
        )
    )


@router.get("", response_model=list[WorkItemRead])
def list_work_items(db: Session = Depends(get_db)) -> list[WorkItem]:
    statement = select(WorkItem).order_by(WorkItem.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=WorkItemRead, status_code=status.HTTP_201_CREATED)
def create_work_item(payload: WorkItemCreate, db: Session = Depends(get_db)) -> WorkItem:
    work_item = WorkItem(
        id=str(uuid4()),
        work_type=payload.work_type,
        status="pending_intent_verification",
        requested_by_actor_id=payload.requested_by_actor_id,
        payload_json={
            **payload.payload_json,
            "intent_verification": payload.intent.model_dump(),
        },
        error_message=None,
    )
    db.add(work_item)
    _audit_work_item_created(db, work_item)
    db.commit()
    db.refresh(work_item)
    return work_item


@router.post("/{work_item_id}/dispatch", response_model=WorkItemDispatchResult)
def dispatch_work_item(
    work_item_id: str,
    payload: WorkItemDispatchRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> WorkItemDispatchResult:
    work_item = db.get(WorkItem, work_item_id)
    if work_item is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Work item not found")
    if work_item.status != "queued":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Only queued work items can be dispatched")

    task_name, args, kwargs = build_dispatch_plan(work_item)
    async_result = _celery_app(settings).send_task(task_name, args=args, kwargs=kwargs)
    task_id = str(async_result.id)
    work_item.payload_json = {
        **(work_item.payload_json or {}),
        "dispatch": {
            "task_name": task_name,
            "task_id": task_id,
            "dispatched_by_actor_id": payload.actor_id,
        },
    }
    _audit_work_item_dispatched(
        db,
        actor_id=payload.actor_id,
        work_item=work_item,
        task_name=task_name,
        task_id=task_id,
    )
    db.commit()
    db.refresh(work_item)
    return WorkItemDispatchResult(
        work_item_id=work_item.id,
        work_type=work_item.work_type,
        task_name=task_name,
        task_id=task_id,
        status=work_item.status,
    )


@router.post("/{work_item_id}/status", response_model=WorkItemRead)
def update_work_item_status(
    work_item_id: str,
    payload: WorkItemStatusUpdate,
    db: Session = Depends(get_db),
) -> WorkItem:
    if payload.status not in WORK_ITEM_STATUSES:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Unknown work item status")
    work_item = db.get(WorkItem, work_item_id)
    if work_item is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Work item not found")

    previous_status = work_item.status
    work_item.status = payload.status
    work_item.error_message = payload.error_message
    _audit_work_item_status_updated(
        db,
        actor_id=payload.actor_id,
        work_item=work_item,
        previous_status=previous_status,
    )
    db.commit()
    db.refresh(work_item)
    return work_item


@router.get("/{work_item_id}", response_model=WorkItemRead)
def get_work_item(work_item_id: str, db: Session = Depends(get_db)) -> WorkItem:
    work_item = db.get(WorkItem, work_item_id)
    if work_item is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Work item not found")
    return work_item
