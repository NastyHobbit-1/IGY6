from typing import Any

from app.contracts import NormalizedDocumentRef, RawArtifactRef

_KNOWN_SENSITIVITY_LABELS = {"public", "internal", "sensitive", "secret"}


def classify_sensitivity_label(value: str | None, fallback: str = "internal") -> str:
    if value in _KNOWN_SENSITIVITY_LABELS:
        return value
    return fallback


def build_normalized_document_ref(
    raw: RawArtifactRef,
    *,
    text_content: str = "",
    title: str | None = None,
    document_type: str = "unknown",
    language: str | None = None,
    sensitivity: str | None = None,
    metadata: dict[str, Any] | None = None,
) -> NormalizedDocumentRef:
    merged_metadata = {
        "normalized_from_raw_artifact_id": raw.id,
        "raw_content_hash": raw.content_hash,
        "raw_storage_path": raw.storage_path,
    }
    if metadata:
        merged_metadata.update(metadata)
    return NormalizedDocumentRef(
        id=f"normalized-{raw.id}",
        raw_artifact_id=raw.id,
        source_id=raw.source_id,
        text_content=text_content,
        title=title,
        document_type=document_type,
        language=language,
        sensitivity=classify_sensitivity_label(sensitivity),
        metadata=merged_metadata,
    )
