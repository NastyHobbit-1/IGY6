from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import RawArtifact

router = APIRouter(prefix="/artifacts", tags=["artifacts"])


class RawArtifactRead(BaseModel):
    id: str
    source_id: str | None
    collection_run_id: str | None
    content_hash: str
    storage_path: str
    mime_type: str | None
    size_bytes: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


@router.get("", response_model=list[RawArtifactRead])
def list_raw_artifacts(db: Session = Depends(get_db)) -> list[RawArtifact]:
    statement = select(RawArtifact).order_by(RawArtifact.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/{artifact_id}", response_model=RawArtifactRead)
def get_raw_artifact(artifact_id: str, db: Session = Depends(get_db)) -> RawArtifact:
    artifact = db.get(RawArtifact, artifact_id)
    if artifact is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Artifact not found")
    return artifact
