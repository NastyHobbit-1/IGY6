from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import and_, or_, select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import Chunk, EvidenceItem, NormalizedDocument, RawArtifact, Source

router = APIRouter(prefix="/retrieval", tags=["retrieval"])


class RetrievalChunkRead(BaseModel):
    id: str
    document_id: str
    chunk_index: int
    text_content: str
    location_json: dict[str, Any]
    embedding_status: str
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class RetrievalDocumentRead(BaseModel):
    id: str
    raw_artifact_id: str | None
    source_id: str | None
    title: str | None
    document_type: str
    language: str | None
    sensitivity: str
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class RetrievalSourceRead(BaseModel):
    id: str
    name: str
    source_type: str
    location: str | None
    owner_actor_id: str
    sensitivity: str
    trust_level: str
    enabled: bool
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class RetrievalRawArtifactRead(BaseModel):
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


class RetrievalEvidenceItemRead(BaseModel):
    id: str
    source_id: str | None
    document_id: str | None
    chunk_id: str | None
    evidence_type: str
    statement: str
    observed_at: datetime | None
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class RetrievalTrailRead(BaseModel):
    chunk: RetrievalChunkRead
    document: RetrievalDocumentRead
    source: RetrievalSourceRead | None = None
    raw_artifact: RetrievalRawArtifactRead | None = None
    evidence_items: list[RetrievalEvidenceItemRead] = Field(default_factory=list)


def get_retrieval_chunk_trail(db: Session, chunk_id: str) -> RetrievalTrailRead:
    chunk = db.get(Chunk, chunk_id)
    if chunk is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Chunk not found")

    document = db.get(NormalizedDocument, chunk.document_id)
    if document is None:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Chunk document not found")

    raw_artifact: RawArtifact | None = None
    if document.raw_artifact_id is not None:
        raw_artifact = db.get(RawArtifact, document.raw_artifact_id)
        if raw_artifact is None:
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Document raw artifact not found")

    source_id = document.source_id
    if source_id is None and raw_artifact is not None:
        source_id = raw_artifact.source_id

    source: Source | None = None
    if source_id is not None:
        source = db.get(Source, source_id)
        if source is None:
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Trail source not found")

    evidence_statement = (
        select(EvidenceItem)
        .where(
            or_(
                EvidenceItem.chunk_id == chunk.id,
                and_(EvidenceItem.chunk_id.is_(None), EvidenceItem.document_id == document.id),
            )
        )
        .order_by(EvidenceItem.created_at.desc(), EvidenceItem.id.asc())
    )
    evidence_items = list(db.scalars(evidence_statement).all())

    return RetrievalTrailRead(
        chunk=RetrievalChunkRead.model_validate(chunk),
        document=RetrievalDocumentRead.model_validate(document),
        source=RetrievalSourceRead.model_validate(source) if source is not None else None,
        raw_artifact=RetrievalRawArtifactRead.model_validate(raw_artifact) if raw_artifact is not None else None,
        evidence_items=[RetrievalEvidenceItemRead.model_validate(item) for item in evidence_items],
    )


@router.get("/chunks/{chunk_id}/trail", response_model=RetrievalTrailRead)
def get_chunk_retrieval_trail(chunk_id: str, db: Session = Depends(get_db)) -> RetrievalTrailRead:
    return get_retrieval_chunk_trail(db, chunk_id)
