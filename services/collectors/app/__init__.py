"""Collector contracts for future source connectors."""

from app.local_project import LocalProjectConnector
from app.manual_upload import ManualUploadConnector

__all__ = ["LocalProjectConnector", "ManualUploadConnector"]
