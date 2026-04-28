from dataclasses import dataclass
from typing import Any

from app.contracts import (
    CollectionRunRef,
    Connector,
    DryRunResult,
    MetadataResult,
    NormalizedDocumentRef,
    RawArtifactRef,
    SourcePermissionRef,
    SourceRef,
)


@dataclass(frozen=True)
class LocalProjectConnector:
    name: str = "local_project"
    version: str = "0.1.0"
    source_type: str = "local_project"

    def validate_scope(self, permission: SourcePermissionRef) -> None:
        if permission.allowed_operations and "read" not in permission.allowed_operations:
            raise ValueError("local_project sources must allow read operations")

    def dry_run(self, source: SourceRef, permission: SourcePermissionRef) -> DryRunResult:
        self.validate_scope(permission)
        return DryRunResult(
            source_id=source.id,
            connector_name=self.name,
            allowed=True,
            summary="Local project dry-run validated source and permission metadata only.",
            estimated_items=None,
            warnings=[],
            metadata={
                "source_type": source.source_type,
                "allowed_operations": permission.allowed_operations,
                "preview_only": True,
            },
        )

    def collect(
        self,
        source: SourceRef,
        permission: SourcePermissionRef,
        run_context: CollectionRunRef,
    ) -> list[RawArtifactRef]:
        raise NotImplementedError("local_project collection is not implemented in this scaffold")

    def normalize(self, raw: RawArtifactRef) -> NormalizedDocumentRef:
        raise NotImplementedError("local_project normalization is not implemented in this scaffold")

    def classify_sensitivity(self, doc: NormalizedDocumentRef) -> str:
        return doc.sensitivity

    def extract_metadata(self, raw: RawArtifactRef, doc: NormalizedDocumentRef) -> MetadataResult:
        return MetadataResult(
            values={
                "content_hash": raw.content_hash,
                "storage_path": raw.storage_path,
                "document_id": doc.id,
            }
        )

    def cleanup(self, run_context: CollectionRunRef) -> None:
        return None
