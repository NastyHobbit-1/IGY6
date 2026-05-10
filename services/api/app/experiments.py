from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, field_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, ExperimentRun, ImprovementItem

router = APIRouter(prefix="/experiments", tags=["experiments"])

EXPERIMENT_STATUSES = {
    "planned",
    "running",
    "completed",
    "failed",
    "abandoned",
}


class ExperimentRunCreate(BaseModel):
    improvement_item_id: str | None = None
    status: str = "planned"
    mlflow_run_id: str | None = None
    optuna_study_name: str | None = None
    metrics_json: dict[str, Any] = Field(default_factory=dict)
    artifacts_json: dict[str, Any] = Field(default_factory=dict)
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    actor_id: str = "local-owner"

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in EXPERIMENT_STATUSES:
            raise ValueError(f"Unknown experiment status: {value}")
        return value


class ExperimentRunStatusUpdate(BaseModel):
    status: str
    metrics_json: dict[str, Any] | None = None
    artifacts_json: dict[str, Any] | None = None
    metadata_json: dict[str, Any] | None = None
    actor_id: str = "local-owner"

    @field_validator("status")
    @classmethod
    def validate_status(cls, value: str) -> str:
        if value not in EXPERIMENT_STATUSES:
            raise ValueError(f"Unknown experiment status: {value}")
        return value


class ExperimentRunRead(BaseModel):
    id: str
    improvement_item_id: str | None
    status: str
    mlflow_run_id: str | None
    optuna_study_name: str | None
    metrics_json: dict[str, Any]
    artifacts_json: dict[str, Any]
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_experiment_created(
    db: Session,
    experiment: ExperimentRun,
    actor_id: str,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="experiment_run.created",
            decision="created",
            resource_type="experiment_run",
            resource_id=experiment.id,
            correlation_id=None,
            details_json={
                "improvement_item_id": experiment.improvement_item_id,
                "status": experiment.status,
                "mlflow_run_id": experiment.mlflow_run_id,
                "optuna_study_name": experiment.optuna_study_name,
            },
        )
    )


def _audit_experiment_status_updated(
    db: Session,
    experiment: ExperimentRun,
    previous_status: str,
    actor_id: str,
    metrics_updated: bool,
    artifacts_updated: bool,
    metadata_updated: bool,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="experiment_run.status_updated",
            decision=experiment.status,
            resource_type="experiment_run",
            resource_id=experiment.id,
            correlation_id=None,
            details_json={
                "previous_status": previous_status,
                "new_status": experiment.status,
                "metrics_updated": metrics_updated,
                "artifacts_updated": artifacts_updated,
                "metadata_updated": metadata_updated,
            },
        )
    )


@router.get("", response_model=list[ExperimentRunRead])
def list_experiment_runs(db: Session = Depends(get_db)) -> list[ExperimentRun]:
    statement = select(ExperimentRun).order_by(ExperimentRun.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("", response_model=ExperimentRunRead, status_code=status.HTTP_201_CREATED)
def create_experiment_run(
    payload: ExperimentRunCreate,
    db: Session = Depends(get_db),
) -> ExperimentRun:
    if payload.improvement_item_id is not None and db.get(ImprovementItem, payload.improvement_item_id) is None:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Improvement item not found")

    experiment = ExperimentRun(
        id=str(uuid4()),
        improvement_item_id=payload.improvement_item_id,
        status=payload.status,
        mlflow_run_id=payload.mlflow_run_id,
        optuna_study_name=payload.optuna_study_name,
        metrics_json=payload.metrics_json,
        artifacts_json=payload.artifacts_json,
        metadata_json=payload.metadata_json,
    )
    db.add(experiment)
    _audit_experiment_created(db, experiment, payload.actor_id)
    db.commit()
    db.refresh(experiment)
    return experiment


@router.post("/{experiment_run_id}/status", response_model=ExperimentRunRead)
def update_experiment_run_status(
    experiment_run_id: str,
    payload: ExperimentRunStatusUpdate,
    db: Session = Depends(get_db),
) -> ExperimentRun:
    experiment = db.get(ExperimentRun, experiment_run_id)
    if experiment is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Experiment run not found")

    previous_status = experiment.status
    experiment.status = payload.status
    if payload.metrics_json is not None:
        experiment.metrics_json = payload.metrics_json
    if payload.artifacts_json is not None:
        experiment.artifacts_json = payload.artifacts_json
    if payload.metadata_json is not None:
        experiment.metadata_json = payload.metadata_json
    _audit_experiment_status_updated(
        db,
        experiment,
        previous_status,
        payload.actor_id,
        metrics_updated=payload.metrics_json is not None,
        artifacts_updated=payload.artifacts_json is not None,
        metadata_updated=payload.metadata_json is not None,
    )
    db.commit()
    db.refresh(experiment)
    return experiment


@router.get("/{experiment_run_id}", response_model=ExperimentRunRead)
def get_experiment_run(
    experiment_run_id: str,
    db: Session = Depends(get_db),
) -> ExperimentRun:
    experiment = db.get(ExperimentRun, experiment_run_id)
    if experiment is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Experiment run not found")
    return experiment
