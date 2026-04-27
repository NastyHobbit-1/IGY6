from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import Claim, EvidenceItem, NormalizedDocument

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
