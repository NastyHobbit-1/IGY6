from typing import Any

import httpx
import redis
from fastapi import APIRouter
from neo4j import GraphDatabase

from app.config import get_settings
from app.db import check_database

router = APIRouter(prefix="/health", tags=["health"])


def _ok() -> dict[str, str]:
    return {"status": "ok"}


def _fail(exc: Exception) -> dict[str, str]:
    return {"status": "error", "detail": str(exc)}


@router.get("/live")
def live() -> dict[str, str]:
    return _ok()


@router.get("/ready")
def ready() -> dict[str, Any]:
    settings = get_settings()
    checks: dict[str, dict[str, str]] = {}

    try:
        check_database()
        checks["postgres"] = _ok()
    except Exception as exc:
        checks["postgres"] = _fail(exc)

    try:
        redis.from_url(settings.redis_url).ping()
        checks["redis"] = _ok()
    except Exception as exc:
        checks["redis"] = _fail(exc)

    for name, url in {
        "qdrant": f"{settings.qdrant_url}/",
        "mlflow": f"{settings.mlflow_tracking_uri}/",
        "phoenix": f"{settings.phoenix_collector_endpoint}/",
    }.items():
        try:
            response = httpx.get(url, timeout=3)
            response.raise_for_status()
            checks[name] = _ok()
        except Exception as exc:
            checks[name] = _fail(exc)

    try:
        driver = GraphDatabase.driver(
            settings.neo4j_uri,
            auth=(settings.neo4j_user, settings.neo4j_password),
        )
        with driver:
            driver.verify_connectivity()
        checks["neo4j"] = _ok()
    except Exception as exc:
        checks["neo4j"] = _fail(exc)

    overall = "ok" if all(check["status"] == "ok" for check in checks.values()) else "degraded"
    return {"status": overall, "checks": checks}
