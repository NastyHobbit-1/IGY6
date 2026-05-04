from datetime import datetime
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field, model_validator
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.artifact_store import ArtifactStoreError, read_artifact_bytes
from app.config import get_settings
from app.db import get_db
from app.models import AuditEvent, Chunk, Claim, EvidenceItem, NormalizedDocument, RawArtifact, Source

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


class NormalizedDocumentCreate(BaseModel):
    raw_artifact_id: str
    source_id: str | None = None
    title: str | None = None
    document_type: str = "text"
    language: str | None = None
    sensitivity: str = "internal"
    metadata_json: dict[str, Any] = Field(default_factory=dict)
    created_by_actor_id: str = "local-owner"


class ChunkGenerationCreate(BaseModel):
    chunk_size: int = Field(default=1000, ge=100, le=5000)
    created_by_actor_id: str = "local-owner"


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


def _audit_normalized_document_created(db: Session, document: NormalizedDocument, actor_id: str) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="normalized_document.created",
            decision="recorded",
            resource_type="normalized_document",
            resource_id=document.id,
            correlation_id=None,
            details_json={
                "source_id": document.source_id,
                "raw_artifact_id": document.raw_artifact_id,
                "document_type": document.document_type,
                "sensitivity": document.sensitivity,
            },
        )
    )


def _audit_document_chunks_generated(
    db: Session,
    *,
    document: NormalizedDocument,
    actor_id: str,
    chunk_count: int,
    evidence_count: int,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="document_chunks.generated",
            decision="recorded",
            resource_type="normalized_document",
            resource_id=document.id,
            correlation_id=None,
            details_json={
                "source_id": document.source_id,
                "chunk_count": chunk_count,
                "evidence_count": evidence_count,
            },
        )
    )


def _split_text_chunks(text: str, chunk_size: int) -> list[str]:
    return [text[index : index + chunk_size] for index in range(0, len(text), chunk_size) if text[index : index + chunk_size]]


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


@router.post("/documents", response_model=NormalizedDocumentRead, status_code=status.HTTP_201_CREATED)
def create_document(payload: NormalizedDocumentCreate, db: Session = Depends(get_db)) -> NormalizedDocument:
    raw_artifact = db.get(RawArtifact, payload.raw_artifact_id)
    if raw_artifact is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Raw artifact not found")
    if payload.source_id is not None and raw_artifact.source_id != payload.source_id:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Raw artifact does not belong to the source")
    if raw_artifact.source_id is not None and db.get(Source, raw_artifact.source_id) is None:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Raw artifact source not found")

    try:
        artifact_bytes = read_artifact_bytes(raw_artifact.storage_path, get_settings().artifact_store_path)
    except ArtifactStoreError as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    try:
        text_content = artifact_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Artifact is not UTF-8 text") from exc

    document = NormalizedDocument(
        id=str(uuid4()),
        raw_artifact_id=raw_artifact.id,
        source_id=raw_artifact.source_id,
        title=payload.title,
        document_type=payload.document_type,
        language=payload.language,
        text_content=text_content,
        sensitivity=payload.sensitivity,
        metadata_json={
            **payload.metadata_json,
            "raw_content_hash": raw_artifact.content_hash,
            "raw_storage_path": raw_artifact.storage_path,
        },
    )
    db.add(document)
    _audit_normalized_document_created(db, document, payload.created_by_actor_id)
    db.commit()
    db.refresh(document)
    return document


@router.post("/documents/{document_id}/chunks", response_model=list[ChunkRead], status_code=status.HTTP_201_CREATED)
def generate_document_chunks(
    document_id: str,
    payload: ChunkGenerationCreate,
    db: Session = Depends(get_db),
) -> list[Chunk]:
    document = db.get(NormalizedDocument, document_id)
    if document is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Document not found")

    existing_chunk = db.scalar(select(Chunk).where(Chunk.document_id == document.id).limit(1))
    if existing_chunk is not None:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Document chunks already exist")

    text_chunks = _split_text_chunks(document.text_content, payload.chunk_size)
    if not text_chunks:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Document text is empty")

    chunks: list[Chunk] = []
    evidence_items: list[EvidenceItem] = []
    for index, text in enumerate(text_chunks):
        chunk = Chunk(
            id=str(uuid4()),
            document_id=document.id,
            chunk_index=index,
            text_content=text,
            location_json={
                "char_start": index * payload.chunk_size,
                "char_end": index * payload.chunk_size + len(text),
            },
            embedding_status="not_started",
            metadata_json={
                "generated_by": "DIFF-030",
                "chunk_size": payload.chunk_size,
            },
        )
        evidence_item = EvidenceItem(
            id=str(uuid4()),
            source_id=document.source_id,
            document_id=document.id,
            chunk_id=chunk.id,
            evidence_type="document_chunk",
            statement=text,
            observed_at=None,
            confidence=None,
            metadata_json={
                "generated_by": "DIFF-030",
                "chunk_index": index,
            },
        )
        chunks.append(chunk)
        evidence_items.append(evidence_item)
        db.add(chunk)
        db.add(evidence_item)

    _audit_document_chunks_generated(
        db,
        document=document,
        actor_id=payload.created_by_actor_id,
        chunk_count=len(chunks),
        evidence_count=len(evidence_items),
    )
    db.commit()
    for chunk in chunks:
        db.refresh(chunk)
    return chunks


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
