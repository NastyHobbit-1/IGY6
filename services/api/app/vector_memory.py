from typing import Any

import httpx
from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel

from app.config import Settings, get_settings

router = APIRouter(prefix="/memory/vector", tags=["memory-vector"])


class VectorCollectionStatus(BaseModel):
    collection_name: str
    exists: bool
    detail: dict[str, Any] | None = None


def qdrant_collection_payload(vector_size: int) -> dict[str, Any]:
    return {
        "vectors": {
            "size": vector_size,
            "distance": "Cosine",
        }
    }


def _collection_url(settings: Settings) -> str:
    base_url = settings.qdrant_url.rstrip("/")
    return f"{base_url}/collections/{settings.qdrant_chunk_collection}"


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
