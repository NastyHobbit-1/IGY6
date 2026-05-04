from hashlib import blake2b
from math import sqrt
from typing import Any

import httpx
from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.models import Chunk

router = APIRouter(prefix="/memory/vector", tags=["memory-vector"])


class VectorCollectionStatus(BaseModel):
    collection_name: str
    exists: bool
    detail: dict[str, Any] | None = None


class ChunkVectorUpsertResult(BaseModel):
    chunks_selected: int
    chunks_upserted: int


def qdrant_collection_payload(vector_size: int) -> dict[str, Any]:
    return {
        "vectors": {
            "size": vector_size,
            "distance": "Cosine",
        }
    }


def embed_text_local(text: str, vector_size: int) -> list[float]:
    if vector_size < 1:
        raise ValueError("vector_size must be at least 1")

    vector = [0.0 for _ in range(vector_size)]
    tokens = text.lower().split()
    if not tokens:
        return vector

    for token in tokens:
        digest = blake2b(token.encode("utf-8"), digest_size=16).digest()
        index = int.from_bytes(digest[:8], "big") % vector_size
        sign = 1.0 if digest[8] % 2 == 0 else -1.0
        vector[index] += sign

    magnitude = sqrt(sum(value * value for value in vector))
    if magnitude == 0:
        return vector
    return [value / magnitude for value in vector]


def qdrant_points_payload(points: list[dict[str, Any]]) -> dict[str, Any]:
    return {"points": points}


def _collection_url(settings: Settings) -> str:
    base_url = settings.qdrant_url.rstrip("/")
    return f"{base_url}/collections/{settings.qdrant_chunk_collection}"


def _points_url(settings: Settings) -> str:
    return f"{_collection_url(settings)}/points"


def get_qdrant_collection_status(settings: Settings) -> VectorCollectionStatus:
    try:
        response = httpx.get(_collection_url(settings), timeout=5)
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc

    if response.status_code == status.HTTP_404_NOT_FOUND:
        return VectorCollectionStatus(collection_name=settings.qdrant_chunk_collection, exists=False)
    if response.status_code >= 400:
        raise HTTPException(status_code=status.HTTP_502_BAD_GATEWAY, detail=response.text)

    return VectorCollectionStatus(
        collection_name=settings.qdrant_chunk_collection,
        exists=True,
        detail=response.json(),
    )


def ensure_qdrant_chunk_collection(settings: Settings) -> VectorCollectionStatus:
    current = get_qdrant_collection_status(settings)
    if current.exists:
        return current

    try:
        response = httpx.put(
            _collection_url(settings),
            json=qdrant_collection_payload(settings.qdrant_chunk_vector_size),
            timeout=10,
        )
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc

    if response.status_code >= 400:
        raise HTTPException(status_code=status.HTTP_502_BAD_GATEWAY, detail=response.text)
    return get_qdrant_collection_status(settings)


def upsert_chunk_vectors(db: Session, settings: Settings, limit: int = 100) -> ChunkVectorUpsertResult:
    ensure_qdrant_chunk_collection(settings)
    statement = (
        select(Chunk)
        .where(Chunk.embedding_status != "completed")
        .order_by(Chunk.created_at.asc())
        .limit(limit)
    )
    chunks = list(db.scalars(statement).all())
    points: list[dict[str, Any]] = []

    for chunk in chunks:
        points.append(
            {
                "id": chunk.id,
                "vector": embed_text_local(chunk.text_content, settings.qdrant_chunk_vector_size),
                "payload": {
                    "chunk_id": chunk.id,
                    "document_id": chunk.document_id,
                    "chunk_index": chunk.chunk_index,
                    "embedding_method": "local_hash_v1",
                },
            }
        )

    if not points:
        return ChunkVectorUpsertResult(chunks_selected=0, chunks_upserted=0)

    try:
        response = httpx.put(
            _points_url(settings),
            json=qdrant_points_payload(points),
            timeout=15,
        )
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc

    if response.status_code >= 400:
        raise HTTPException(status_code=status.HTTP_502_BAD_GATEWAY, detail=response.text)

    for chunk in chunks:
        chunk.embedding_status = "completed"
        chunk.metadata_json = {
            **chunk.metadata_json,
            "embedding_method": "local_hash_v1",
            "vector_collection": settings.qdrant_chunk_collection,
        }
    db.commit()
    return ChunkVectorUpsertResult(chunks_selected=len(chunks), chunks_upserted=len(points))


@router.get("/chunks", response_model=VectorCollectionStatus)
def get_chunk_vector_collection(
    settings: Settings = Depends(get_settings),
) -> VectorCollectionStatus:
    return get_qdrant_collection_status(settings)


@router.post("/chunks/ensure", response_model=VectorCollectionStatus, status_code=status.HTTP_201_CREATED)
def ensure_chunk_vector_collection(
    settings: Settings = Depends(get_settings),
) -> VectorCollectionStatus:
    return ensure_qdrant_chunk_collection(settings)


@router.post("/chunks/upsert", response_model=ChunkVectorUpsertResult)
def upsert_chunk_vector_points(
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> ChunkVectorUpsertResult:
    return upsert_chunk_vectors(db, settings)
