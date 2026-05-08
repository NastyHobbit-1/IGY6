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


@router.post("/retrieval-preview", response_model=ChatRetrievalPreviewResponse)
def create_chat_retrieval_preview(
    payload: ChatRetrievalPreviewRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> ChatRetrievalPreviewResponse:
    return build_retrieval_preview(db, settings, payload)
