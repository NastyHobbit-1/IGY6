from dataclasses import dataclass, field
from typing import Any, Protocol


@dataclass(frozen=True)
class SourceRef:
    id: str
    source_type: str
    name: str
    location: str | None
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class SourcePermissionRef:
    id: str
    source_id: str
    scope: dict[str, Any] = field(default_factory=dict)
    allowed_operations: list[str] = field(default_factory=list)
    external_model_policy: str = "blocked"
    approval_required: bool = True


@dataclass(frozen=True)
class CollectionRunRef:
    id: str
    source_id: str
    dry_run: bool
    requested_by_actor_id: str


@dataclass(frozen=True)
class DryRunResult:
    source_id: str
    connector_name: str
    allowed: bool
    summary: str
    estimated_items: int | None = None
    warnings: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class RawArtifactRef:
    id: str
    source_id: str
    content_hash: str
    storage_path: str
    mime_type: str | None = None
    size_bytes: int | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class NormalizedDocumentRef:
    id: str
    raw_artifact_id: str
    source_id: str
    text_content: str
    title: str | None = None
    document_type: str = "unknown"
    language: str | None = None
    sensitivity: str = "internal"
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class MetadataResult:
    values: dict[str, Any] = field(default_factory=dict)


class Connector(Protocol):
    name: str
    version: str
    source_type: str

    def validate_scope(self, permission: SourcePermissionRef) -> None:
        """Validate that the permission scope is acceptable for this connector."""

    def dry_run(self, source: SourceRef, permission: SourcePermissionRef) -> DryRunResult:
        """Describe what would be collected without collecting or writing artifacts."""

    def collect(
        self,
        source: SourceRef,
        permission: SourcePermissionRef,
        run_context: CollectionRunRef,
    ) -> list[RawArtifactRef]:
        """Collect raw artifacts after policy and approval checks have passed."""

    def normalize(self, raw: RawArtifactRef) -> NormalizedDocumentRef:
        """Convert a raw artifact reference into a normalized document reference."""

    def classify_sensitivity(self, doc: NormalizedDocumentRef) -> str:
        """Assign a sensitivity label to a normalized document."""

    def extract_metadata(self, raw: RawArtifactRef, doc: NormalizedDocumentRef) -> MetadataResult:
        """Extract metadata from a raw artifact and normalized document."""

    def cleanup(self, run_context: CollectionRunRef) -> None:
        """Clean up connector runtime state after a collection run."""
