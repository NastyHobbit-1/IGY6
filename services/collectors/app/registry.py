from app.contracts import Connector
from app.local_project import LocalProjectConnector
from app.manual_upload import ManualUploadConnector

_CONNECTORS: dict[str, Connector] = {
    "local_project": LocalProjectConnector(),
    "manual_upload": ManualUploadConnector(),
}


def list_source_types() -> list[str]:
    return sorted(_CONNECTORS)


def get_connector(source_type: str) -> Connector:
    try:
        return _CONNECTORS[source_type]
    except KeyError as exc:
        raise ValueError(f"No connector registered for source type: {source_type}") from exc
