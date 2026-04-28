"""Collector contracts for future source connectors."""

from app.local_project import LocalProjectConnector
from app.manual_upload import ManualUploadConnector
from app.registry import get_connector, list_source_types

__all__ = [
    "LocalProjectConnector",
    "ManualUploadConnector",
    "get_connector",
    "list_source_types",
]
