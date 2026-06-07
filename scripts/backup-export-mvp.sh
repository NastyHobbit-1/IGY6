#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPO_ROOT="${REPO_ROOT}" python3 - "$@" <<'PY'
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


REPO_ROOT = Path(os.environ["REPO_ROOT"]).resolve()
DEFAULT_OUTPUT_DIR = REPO_ROOT / ".igy6-local" / "exports"
SCHEMA_VERSION = "igy6.backup_export.v1"
MAX_STRING_LENGTH = 400

EXPORT_CLASSES: tuple[tuple[str, str], ...] = (
    ("sources", "/sources"),
    ("source_permissions", "/source-permissions"),
    ("approvals", "/approvals"),
    ("audit_events", "/audit-events"),
    ("collection_runs", "/collection-runs"),
    ("artifact_metadata", "/artifacts"),
    ("documents", "/evidence/documents"),
    ("chunks", "/evidence/chunks"),
    ("evidence_items", "/evidence/items"),
    ("claims", "/evidence/claims"),
    ("evidence_answers", "/evidence-answers"),
    ("feedback", "/feedback"),
    ("outcomes", "/outcomes"),
    ("work_items", "/work-items"),
    ("task_plans", "/agent/task-plans"),
    ("reports", "/reports"),
    ("patterns", "/analysis/patterns"),
    ("hypotheses", "/analysis/hypotheses"),
    ("predictions", "/analysis/predictions"),
    ("recommendations", "/analysis/recommendations"),
    ("improvements", "/improvements"),
    ("experiments", "/experiments"),
)

DROP_KEY_PARTS = (
    "content",
    "body",
    "text",
    "secret",
    "token",
    "password",
    "cookie",
    "credential",
    "private_key",
    "api_key",
)
PATH_KEY_PARTS = ("path", "location", "url", "uri")


def scalar(value: Any) -> bool:
    return value is None or isinstance(value, (bool, int, float, str))


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def git_value(*args: str) -> str | None:
    try:
        result = subprocess.run(
            ["git", *args],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError:
        return None
    if result.returncode != 0:
        return None
    value = result.stdout.strip()
    return value or None


def read_json(api_base_url: str, path: str, timeout: float) -> tuple[int, Any]:
    request = urllib.request.Request(f"{api_base_url.rstrip('/')}{path}", method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8")
            return response.status, json.loads(raw) if raw else None
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        try:
            payload: Any = json.loads(raw)
        except json.JSONDecodeError:
            payload = {"detail": raw[:MAX_STRING_LENGTH]}
        return exc.code, payload
    except OSError as exc:
        return 0, {"detail": str(exc)}


def should_drop_key(key: str) -> bool:
    lowered = key.lower()
    return any(part in lowered for part in DROP_KEY_PARTS)


def should_redact_path_key(key: str) -> bool:
    lowered = key.lower()
    return any(part in lowered for part in PATH_KEY_PARTS)


def sanitize(value: Any, key: str = "") -> Any:
    if should_drop_key(key):
        return "[excluded]"
    if should_redact_path_key(key) and isinstance(value, str):
        return "[redacted-local-reference]"
    if isinstance(value, str):
        cleaned = value.replace("\r", " ").replace("\n", " ").strip()
        if len(cleaned) > MAX_STRING_LENGTH:
            return f"{cleaned[:MAX_STRING_LENGTH]}...[truncated]"
        return cleaned
    if scalar(value):
        return value
    if isinstance(value, list):
        return [sanitize(item) for item in value]
    if isinstance(value, dict):
        return {str(item_key): sanitize(item_value, str(item_key)) for item_key, item_value in value.items()}
    return str(value)[:MAX_STRING_LENGTH]


def records_from_payload(payload: Any) -> list[Any]:
    if isinstance(payload, list):
        return payload
    if isinstance(payload, dict):
        for key in ("items", "records", "data"):
            value = payload.get(key)
            if isinstance(value, list):
                return value
    return []


def build_bundle(api_base_url: str, timeout: float) -> tuple[dict[str, Any], list[str]]:
    warnings: list[str] = []
    classes: dict[str, Any] = {}
    counts: dict[str, int] = {}
    endpoints: dict[str, str] = {}

    for class_name, path in EXPORT_CLASSES:
        status, payload = read_json(api_base_url, path, timeout)
        endpoints[class_name] = path
        if status != 200:
            classes[class_name] = {
                "status": "unavailable",
                "http_status": status,
                "detail": sanitize(payload),
            }
            counts[class_name] = 0
            warnings.append(f"{class_name} unavailable from {path} (HTTP {status})")
            continue
        records = records_from_payload(payload)
        classes[class_name] = {
            "status": "included",
            "records": sanitize(records),
        }
        counts[class_name] = len(records)

    bundle = {
        "schema_version": SCHEMA_VERSION,
        "created_at_utc": utc_now(),
        "repo": {
            "branch": git_value("branch", "--show-current"),
            "head": git_value("rev-parse", "HEAD"),
        },
        "export": {
            "mode": "metadata_only",
            "api_base_url": "[local-api-reference]",
            "included_classes": [class_name for class_name, _ in EXPORT_CLASSES],
            "record_counts": counts,
            "endpoint_paths": endpoints,
            "exclusions": [
                ".env and .env backups",
                "credentials, tokens, cookies, private keys, and secret-shaped fields",
                "raw artifact bytes",
                "raw document, chunk, evidence, answer, and report text content",
                "raw local absolute paths and runtime/private data roots",
                "Docker volumes, PostgreSQL dumps, Qdrant data, Neo4j data, MLflow data, and Phoenix data",
            ],
        },
        "classes": classes,
        "warnings": warnings,
    }
    return bundle, warnings


def write_bundle(bundle: dict[str, Any], output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    filename = f"igy6-backup-export-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}.json"
    target = output_dir / filename
    target.write_text(json.dumps(bundle, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return target


def main() -> int:
    parser = argparse.ArgumentParser(description="Create a safe IGY6 metadata-only export bundle.")
    parser.add_argument("--api-base-url", default="http://127.0.0.1:8000")
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--timeout", type=float, default=8.0)
    parser.add_argument("--dry-run", action="store_true", help="Validate route availability and print counts without writing a bundle.")
    args = parser.parse_args()

    bundle, warnings = build_bundle(args.api_base_url, args.timeout)
    counts = bundle["export"]["record_counts"]
    included = sum(1 for value in bundle["classes"].values() if value.get("status") == "included")
    unavailable = len(bundle["classes"]) - included

    print("IGY6 Backup Export MVP")
    print(f"schema_version: {bundle['schema_version']}")
    print(f"mode: {bundle['export']['mode']}")
    print(f"classes_included: {included}")
    print(f"classes_unavailable: {unavailable}")
    print("record_counts:")
    for class_name in sorted(counts):
        print(f"  {class_name}: {counts[class_name]}")

    if args.dry_run:
        print("dry_run: true")
        return 0 if not warnings else 2

    target = write_bundle(bundle, args.output_dir)
    try:
        display = target.resolve().relative_to(REPO_ROOT)
    except ValueError:
        display = target
    print(f"wrote: {display}")
    if warnings:
        print("warnings:")
        for warning in warnings:
            print(f"  {warning}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
PY
