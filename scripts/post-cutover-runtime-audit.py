#!/usr/bin/env python3
"""DIFF-166 post-cutover runtime audit.

This check is intentionally non-destructive. It reads repository text files only
and verifies that the active Compose/runtime surface does not point at the
archived Python/Celery worker.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

ACTIVE_RUNTIME_FILES = [
    "infra/docker-compose.yml",
    "infra/docker-compose.rust-worker-canary.yml",
    "README.md",
    "docs/runtime/PROCESSING_STATUS.md",
    "docs/rust-migration/RUST_CUTOVER_ROLLBACK.md",
    "archive/legacy-python/README.md",
]


def read_text(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def load_json(path: str) -> dict:
    with (REPO_ROOT / path).open(encoding="utf-8") as handle:
        return json.load(handle)


def assert_absent(errors: list[str], path: str, patterns: list[str]) -> None:
    text = read_text(path)
    for pattern in patterns:
        if re.search(pattern, text, re.IGNORECASE | re.MULTILINE):
            errors.append(f"{path} contains stale active-runtime pattern: {pattern}")


def main() -> int:
    errors: list[str] = []

    manifest = load_json("configs/rust-cutover-manifest.json")
    final_audit = manifest.get("final_rust_only_runtime_audit", {})
    route_classification = load_json("configs/legacy-fastapi-route-classification.json")

    if manifest.get("target_architecture") != "rust-only-application-runtime":
        errors.append("manifest target_architecture is not rust-only-application-runtime")
    if manifest.get("fastapi_fallback_required") is not False:
        errors.append("manifest fastapi_fallback_required must be false")
    if final_audit.get("python_celery_worker_active") is not False:
        errors.append("final audit must mark Python/Celery worker inactive")
    if final_audit.get("python_celery_beat_active") is not False:
        errors.append("final audit must mark Celery beat inactive")
    if final_audit.get("services_worker_archived") is not True:
        errors.append("final audit must mark services_worker_archived true")
    if final_audit.get("rust_only_application_runtime_claimed") is not True:
        errors.append("final audit must claim Rust-only application runtime")
    if route_classification.get("rust_only_claim_allowed") is not True:
        errors.append("route classification must allow Rust-only claim")

    if (REPO_ROOT / "services/worker").exists():
        errors.append("services/worker still exists outside the archive")
    if not (REPO_ROOT / "archive/legacy-python/services-worker").is_dir():
        errors.append("archive/legacy-python/services-worker is missing")

    base_compose = read_text("infra/docker-compose.yml")
    canary_compose = read_text("infra/docker-compose.rust-worker-canary.yml")
    if "dockerfile: crates/igy6-worker/Dockerfile" not in base_compose:
        errors.append("base Compose worker does not use crates/igy6-worker/Dockerfile")
    if "dockerfile: crates/igy6-worker/Dockerfile" not in canary_compose:
        errors.append("canary Compose worker does not use crates/igy6-worker/Dockerfile")

    active_patterns = [
        r"services/worker",
        r"celery\s+-A\s+app\.celery_app",
        r"beat\s+--loglevel",
        r"Docker Compose still runs Python/Celery",
        r"Python/Celery [`']?worker[`']? and [`']?beat[`']? remain active",
        r"Rust-only runtime is not claimed",
        r"full Rust-only runtime is not claimed",
    ]
    for path in ACTIVE_RUNTIME_FILES:
        assert_absent(errors, path, active_patterns)

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print(
        "Post-cutover runtime audit passed: active API/worker runtime is Rust-only; "
        "legacy Python source is archive/rollback-only."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
