"""Collector contracts for future source connectors."""

from app.local_project import LocalProjectConnector
from app.manual_upload import ManualUploadConnector
from app.registry import get_connector, list_source_types
from app.normalization import build_normalized_document_ref, classify_sensitivity_label
from app.runner import run_dry_run

__all__ = [
    "LocalProjectConnector",
    "ManualUploadConnector",
    "build_normalized_document_ref",
    "classify_sensitivity_label",
    "get_connector",
    "list_source_types",
    "run_dry_run",
]
