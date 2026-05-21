from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import and_, or_, select
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.models import Chunk, EvidenceItem, NormalizedDocument, RawArtifact, Source
from app.vector_memory import (
    CHUNK_VECTOR_SEARCH_MAX_LIMIT,
    ChunkVectorSearchRequest,
    search_chunk_vectors,
)

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


class HydratedChunkSearchRequest(BaseModel):
    query: str
    limit: int = Field(default=10, ge=1, le=CHUNK_VECTOR_SEARCH_MAX_LIMIT)


class HydratedChunkSearchHit(BaseModel):
    score: float
    qdrant_payload: dict[str, Any]
    chunk: RetrievalChunkRead
    document: RetrievalDocumentRead
    source: RetrievalSourceRead | None = None
    raw_artifact: RetrievalRawArtifactRead | None = None
    evidence_items: list[RetrievalEvidenceItemRead] = Field(default_factory=list)


class HydratedChunkSearchResult(BaseModel):
    query: str
    collection_name: str
    collection_exists: bool
    hits: list[HydratedChunkSearchHit]


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


def is_source_allowed_for_retrieval(source: RetrievalSourceRead | None) -> bool:
    # DIFF-074 keeps the current policy minimal: disabled sources are hidden.
    # Future sensitivity, trust, and external-model filters should extend this
    # single policy hook instead of scattering retrieval checks.
    if source is None:
        return True
    return source.enabled


def search_hydrated_chunks(
    db: Session,
    settings: Settings,
    payload: HydratedChunkSearchRequest,
) -> HydratedChunkSearchResult:
    vector_results = search_chunk_vectors(
        settings,
        ChunkVectorSearchRequest(query=payload.query, limit=payload.limit),
    )

    hits: list[HydratedChunkSearchHit] = []
    for vector_hit in vector_results.hits[: payload.limit]:
        if vector_hit.chunk_id is None:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Vector search hit missing chunk_id",
            )

        trail = get_retrieval_chunk_trail(db, vector_hit.chunk_id)
        if not is_source_allowed_for_retrieval(trail.source):
            continue
        hits.append(
            HydratedChunkSearchHit(
                score=vector_hit.score,
                qdrant_payload=vector_hit.payload,
                chunk=trail.chunk,
                document=trail.document,
                source=trail.source,
                raw_artifact=trail.raw_artifact,
                evidence_items=trail.evidence_items,
            )
        )

    return HydratedChunkSearchResult(
        query=vector_results.query,
        collection_name=vector_results.collection_name,
        collection_exists=vector_results.collection_exists,
        hits=hits,
    )


@router.get("/chunks/{chunk_id}/trail", response_model=RetrievalTrailRead)
def get_chunk_retrieval_trail(chunk_id: str, db: Session = Depends(get_db)) -> RetrievalTrailRead:
    return get_retrieval_chunk_trail(db, chunk_id)


@router.post("/chunks/search", response_model=HydratedChunkSearchResult)
def search_retrieval_chunks(
    payload: HydratedChunkSearchRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> HydratedChunkSearchResult:
    return search_hydrated_chunks(db, settings, payload)
