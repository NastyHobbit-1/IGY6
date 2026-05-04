from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class DryRunResult:
    source_id: str
    connector_name: str
    allowed: bool
    summary: str
    estimated_items: int | None = None
    warnings: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)


_SCAFFOLD_CONNECTORS = {
    "local_project": "local_project",
    "manual_upload": "manual_upload",
}


def run_connector_dry_run(
    *,
    source_id: str,
    source_type: str,
    source_name: str,
    source_location: str | None,
    source_metadata: dict[str, Any],
    permission_id: str,
    permission_source_id: str,
    permission_scope: dict[str, Any],
    allowed_operations: list[str],
    external_model_policy: str,
    approval_required: bool,
) -> DryRunResult:
    if permission_source_id != source_id:
        raise ValueError("Source permission does not belong to the source")

    connector_name = _SCAFFOLD_CONNECTORS.get(source_type)
    if connector_name is None:
        raise ValueError(f"No connector registered for source type: {source_type}")

    if allowed_operations and not {"dry_run", "read"}.intersection(allowed_operations):
        raise ValueError(f"{source_type} sources must allow dry_run or read operations")

    return DryRunResult(
        source_id=source_id,
        connector_name=connector_name,
        allowed=True,
        summary=f"{source_name} dry-run validated source and permission metadata only.",
        estimated_items=None,
        warnings=[],
        metadata={
            "source_type": source_type,
            "source_location": source_location,
            "source_metadata": source_metadata,
            "permission_id": permission_id,
            "permission_scope": permission_scope,
            "allowed_operations": allowed_operations,
            "external_model_policy": external_model_policy,
            "approval_required": approval_required,
            "preview_only": True,
        },
    )
