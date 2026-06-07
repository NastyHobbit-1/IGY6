from app.contracts import DryRunResult, SourcePermissionRef, SourceRef
from app.registry import get_connector


def run_dry_run(source: SourceRef, permission: SourcePermissionRef) -> DryRunResult:
    connector = get_connector(source.source_type)
    if connector.source_type != source.source_type:
        raise ValueError(
            f"Connector {connector.name} handles {connector.source_type}, not {source.source_type}"
        )
    if permission.source_id != source.id:
        raise ValueError("Source permission does not belong to the source")
    return connector.dry_run(source, permission)
