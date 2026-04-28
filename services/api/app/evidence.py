from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, model_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, Chunk, Claim, EvidenceItem, NormalizedDocument, Source

router = APIRouter(prefix="/evidence", tags=["evidence"])


class NormalizedDocumentRead(BaseModel):
    id: str
    raw_artifact_id: str | None
    source_id: str | None
    title: str | None
    document_type: str
    language: str | None
    text_content: str
    sensitivity: str
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class EvidenceItemRead(BaseModel):
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


class EvidenceItemCreate(BaseModel):
    source_id: str | None = None
    document_id: str | None = None
    chunk_id: str | None = None
    evidence_type: str = Field(min_length=1, max_length=64)
    statement: str = Field(min_length=1)
    observed_at: datetime | None = None
    confidence: int | None = Field(default=None, ge=0, le=100)
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    created_by_actor_id: str = "local-owner"

    @model_validator(mode="after")
    def validate_link_presence(self) -> "EvidenceItemCreate":
        if self.source_id is None and self.document_id is None and self.chunk_id is None:
            raise ValueError("At least one evidence link must be provided")
        return self


class ChunkRead(BaseModel):
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


class ClaimRead(BaseModel):
    id: str
    claim_text: str
    claim_type: str
    status: str
    evidence_ids: list[str] = Field(default_factory=list)
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


def _audit_evidence_item_created(db: Session, evidence_item: EvidenceItem, actor_id: str) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="evidence_item.created",
            decision="recorded",
            resource_type="evidence_item",
            resource_id=evidence_item.id,
            correlation_id=None,
            details_json={
                "source_id": evidence_item.source_id,
                "document_id": evidence_item.document_id,
                "chunk_id": evidence_item.chunk_id,
                "evidence_type": evidence_item.evidence_type,
            },
        )
    )


def _validate_evidence_links(
    db: Session,
    payload: EvidenceItemCreate,
) -> None:
    source: Source | None = None
    document: NormalizedDocument | None = None
    chunk: Chunk | None = None

    if payload.source_id is not None:
        source = db.get(Source, payload.source_id)
        if source is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Source not found")

    if payload.document_id is not None:
        document = db.get(NormalizedDocument, payload.document_id)
        if document is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Document not found")
        if payload.source_id is not None and document.source_id != payload.source_id:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Document does not belong to the source",
            )

    if payload.chunk_id is not None:
        chunk = db.get(Chunk, payload.chunk_id)
        if chunk is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Chunk not found")
        chunk_document = db.get(NormalizedDocument, chunk.document_id)
        if chunk_document is None:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Chunk document not found",
            )
        if document is not None and chunk.document_id != document.id:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Chunk does not belong to the document",
            )
        if payload.source_id is not None and chunk_document.source_id != payload.source_id:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Chunk does not belong to the source",
            )


@router.get("/documents", response_model=list[NormalizedDocumentRead])
def list_documents(db: Session = Depends(get_db)) -> list[NormalizedDocument]:
    statement = select(NormalizedDocument).order_by(NormalizedDocument.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/documents/{document_id}", response_model=NormalizedDocumentRead)
def get_document(document_id: str, db: Session = Depends(get_db)) -> NormalizedDocument:
    document = db.get(NormalizedDocument, document_id)
    if document is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Document not found")
    return document


@router.get("/items", response_model=list[EvidenceItemRead])
def list_evidence_items(db: Session = Depends(get_db)) -> list[EvidenceItem]:
    statement = select(EvidenceItem).order_by(EvidenceItem.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/items/{evidence_item_id}", response_model=EvidenceItemRead)
def get_evidence_item(evidence_item_id: str, db: Session = Depends(get_db)) -> EvidenceItem:
    evidence_item = db.get(EvidenceItem, evidence_item_id)
    if evidence_item is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Evidence item not found")
    return evidence_item


@router.get("/chunks", response_model=list[ChunkRead])
def list_chunks(db: Session = Depends(get_db)) -> list[Chunk]:
    statement = select(Chunk).order_by(Chunk.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/chunks/{chunk_id}", response_model=ChunkRead)
def get_chunk(chunk_id: str, db: Session = Depends(get_db)) -> Chunk:
    chunk = db.get(Chunk, chunk_id)
    if chunk is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Chunk not found")
    return chunk


@router.get("/claims", response_model=list[ClaimRead])
def list_claims(db: Session = Depends(get_db)) -> list[Claim]:
    statement = select(Claim).order_by(Claim.created_at.desc())
    return list(db.scalars(statement).all())


@router.get("/claims/{claim_id}", response_model=ClaimRead)
def get_claim(claim_id: str, db: Session = Depends(get_db)) -> Claim:
    claim = db.get(Claim, claim_id)
    if claim is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Claim not found")
    return claim


@router.post("/items", response_model=EvidenceItemRead, status_code=status.HTTP_201_CREATED)
def create_evidence_item(payload: EvidenceItemCreate, db: Session = Depends(get_db)) -> EvidenceItem:
    _validate_evidence_links(db, payload)

    evidence_item = EvidenceItem(
        id=str(uuid4()),
        source_id=payload.source_id,
        document_id=payload.document_id,
        chunk_id=payload.chunk_id,
        evidence_type=payload.evidence_type,
        statement=payload.statement,
        observed_at=payload.observed_at,
        confidence=payload.confidence,
        metadata_json=payload.metadata_json,
    )
    db.add(evidence_item)
    _audit_evidence_item_created(db, evidence_item, payload.created_by_actor_id)
    db.commit()
    db.refresh(evidence_item)
    return evidence_item
