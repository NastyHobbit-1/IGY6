from typing import Literal

from fastapi import APIRouter, Depends
from pydantic import BaseModel, Field
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.retrieval import HydratedChunkSearchRequest, HydratedChunkSearchResult, search_hydrated_chunks
from app.vector_memory import CHUNK_VECTOR_SEARCH_MAX_LIMIT

router = APIRouter(prefix="/chat", tags=["chat"])


class ChatRetrievalPreviewRequest(BaseModel):
    message: str
    limit: int = Field(default=10, ge=1, le=CHUNK_VECTOR_SEARCH_MAX_LIMIT)


class ChatRetrievalPreviewResponse(BaseModel):
    message: str
    answer_status: Literal["not_generated"]
    retrieval_context: HydratedChunkSearchResult


class AnswerCitation(BaseModel):
    citation_id: str
    citation_type: Literal["evidence_item", "chunk"]
    source_id: str | None = None
    source_name: str | None = None
    document_id: str
    document_title: str | None = None
    chunk_id: str
    score: float


class AnswerStatement(BaseModel):
    text: str
    confidence: int | None = None
    citations: list[AnswerCitation]


class AnswerSourceTrail(BaseModel):
    source_id: str | None = None
    source_name: str | None = None
    source_type: str | None = None
    trust_level: str | None = None
    document_id: str
    document_title: str | None = None
    raw_artifact_id: str | None = None
    chunk_id: str
    score: float


class ChatEvidenceAnswerRequest(BaseModel):
    message: str
    limit: int = Field(default=10, ge=1, le=CHUNK_VECTOR_SEARCH_MAX_LIMIT)


class ChatEvidenceAnswerResponse(BaseModel):
    message: str
    answer_status: Literal["evidence_summary", "insufficient_evidence"]
    facts: list[AnswerStatement]
    assumptions: list[str]
    inferences: list[AnswerStatement]
    uncertainty: list[str]
    missing_information: list[str]
    source_trails: list[AnswerSourceTrail]
    retrieval_context: HydratedChunkSearchResult


def _confidence_from_hit(score: float, evidence_confidence: int | None) -> int:
    score_confidence = max(0, min(100, round(score * 100)))
    if evidence_confidence is None:
        return score_confidence
    return max(0, min(100, round((score_confidence + evidence_confidence) / 2)))


def _excerpt(value: str, max_length: int = 220) -> str:
    normalized = " ".join(value.split())
    if len(normalized) <= max_length:
        return normalized
    return f"{normalized[: max_length - 3]}..."


def build_evidence_answer_packet(
    retrieval_context: HydratedChunkSearchResult,
) -> ChatEvidenceAnswerResponse:
    facts: list[AnswerStatement] = []
    inferences: list[AnswerStatement] = []
    source_trails: list[AnswerSourceTrail] = []
    seen_fact_keys: set[str] = set()
    seen_trail_keys: set[str] = set()

    for hit in retrieval_context.hits:
        source_trail = AnswerSourceTrail(
            source_id=hit.source.id if hit.source else None,
            source_name=hit.source.name if hit.source else None,
            source_type=hit.source.source_type if hit.source else None,
            trust_level=hit.source.trust_level if hit.source else None,
            document_id=hit.document.id,
            document_title=hit.document.title,
            raw_artifact_id=hit.raw_artifact.id if hit.raw_artifact else None,
            chunk_id=hit.chunk.id,
            score=hit.score,
        )
        trail_key = f"{source_trail.document_id}:{source_trail.chunk_id}"
        if trail_key not in seen_trail_keys:
            source_trails.append(source_trail)
            seen_trail_keys.add(trail_key)

        if hit.evidence_items:
            for evidence_item in hit.evidence_items:
                fact_key = evidence_item.id
                if fact_key in seen_fact_keys:
                    continue
                citation = AnswerCitation(
                    citation_id=evidence_item.id,
                    citation_type="evidence_item",
                    source_id=hit.source.id if hit.source else evidence_item.source_id,
                    source_name=hit.source.name if hit.source else None,
                    document_id=hit.document.id,
                    document_title=hit.document.title,
                    chunk_id=hit.chunk.id,
                    score=hit.score,
                )
                facts.append(
                    AnswerStatement(
                        text=evidence_item.statement,
                        confidence=_confidence_from_hit(hit.score, evidence_item.confidence),
                        citations=[citation],
                    )
                )
                seen_fact_keys.add(fact_key)
        else:
            chunk_key = hit.chunk.id
            if chunk_key in seen_fact_keys:
                continue
            citation = AnswerCitation(
                citation_id=hit.chunk.id,
                citation_type="chunk",
                source_id=hit.source.id if hit.source else None,
                source_name=hit.source.name if hit.source else None,
                document_id=hit.document.id,
                document_title=hit.document.title,
                chunk_id=hit.chunk.id,
                score=hit.score,
            )
            facts.append(
                AnswerStatement(
                    text=_excerpt(hit.chunk.text_content),
                    confidence=_confidence_from_hit(hit.score, None),
                    citations=[citation],
                )
            )
            seen_fact_keys.add(chunk_key)

    if facts:
        cited_ids = [statement.citations[0].citation_id for statement in facts[:3]]
        inferences.append(
            AnswerStatement(
                text=(
                    "The available answer is limited to the retrieved local evidence. "
                    f"The strongest cited records are: {', '.join(cited_ids)}."
                ),
                confidence=min(statement.confidence or 0 for statement in facts[:3]),
                citations=[citation for statement in facts[:3] for citation in statement.citations],
            )
        )

    answer_status: Literal["evidence_summary", "insufficient_evidence"] = (
        "evidence_summary" if facts else "insufficient_evidence"
    )
    missing_information = []
    uncertainty = [
        "This deterministic answer packet uses local retrieval scores and stored evidence only.",
        "No external model, hidden reasoning, or graph inference was used.",
    ]
    if not facts:
        missing_information.append("No matching chunks or evidence items were retrieved for the message.")
    else:
        missing_information.append("Any relevant source not yet ingested, chunked, and embedded is absent from this answer.")

    return ChatEvidenceAnswerResponse(
        message=retrieval_context.query,
        answer_status=answer_status,
        facts=facts,
        assumptions=[
            "Registered source metadata and stored evidence records are treated as local records of what was collected.",
            "Retrieval scores are similarity signals, not proof of correctness.",
        ],
        inferences=inferences,
        uncertainty=uncertainty,
        missing_information=missing_information,
        source_trails=source_trails,
        retrieval_context=retrieval_context,
    )


def build_retrieval_preview(
    db: Session,
    settings: Settings,
    payload: ChatRetrievalPreviewRequest,
) -> ChatRetrievalPreviewResponse:
    retrieval_context = search_hydrated_chunks(
        db,
        settings,
        HydratedChunkSearchRequest(query=payload.message, limit=payload.limit),
    )
    return ChatRetrievalPreviewResponse(
        message=payload.message,
        answer_status="not_generated",
        retrieval_context=retrieval_context,
    )


def build_evidence_answer(
    db: Session,
    settings: Settings,
    payload: ChatEvidenceAnswerRequest,
) -> ChatEvidenceAnswerResponse:
    retrieval_context = search_hydrated_chunks(
        db,
        settings,
        HydratedChunkSearchRequest(query=payload.message, limit=payload.limit),
    )
    return build_evidence_answer_packet(retrieval_context)


@router.post("/retrieval-preview", response_model=ChatRetrievalPreviewResponse)
def create_chat_retrieval_preview(
    payload: ChatRetrievalPreviewRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> ChatRetrievalPreviewResponse:
    return build_retrieval_preview(db, settings, payload)


@router.post("/evidence-answer", response_model=ChatEvidenceAnswerResponse)
def create_chat_evidence_answer(
    payload: ChatEvidenceAnswerRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> ChatEvidenceAnswerResponse:
    return build_evidence_answer(db, settings, payload)
