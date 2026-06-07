#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPO_ROOT="${REPO_ROOT}" python3 - "$@" <<'PY'
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(os.environ["REPO_ROOT"]).resolve()
DEFAULT_EXPORT_DIR = REPO_ROOT / ".igy6-local" / "exports"
SUPPORTED_SCHEMA_VERSION = "igy6.backup_export.v1"
EXPECTED_TOP_LEVEL_FIELDS = ("schema_version", "created_at_utc", "repo", "export", "classes", "warnings")
SUPPORTED_CLASSES = {
    "sources",
    "source_permissions",
    "approvals",
    "audit_events",
    "collection_runs",
    "artifact_metadata",
    "documents",
    "chunks",
    "evidence_items",
    "claims",
    "evidence_answers",
    "feedback",
    "outcomes",
    "work_items",
    "task_plans",
    "reports",
    "patterns",
    "hypotheses",
    "predictions",
    "recommendations",
    "improvements",
    "experiments",
}
SECRET_KEY_RE = re.compile(r"(secret|token|password|cookie|credential|private_key|api_key)", re.IGNORECASE)
CONTENT_KEY_RE = re.compile(r"(content|body|text)", re.IGNORECASE)
PRIVATE_PATH_RE = re.compile(r"(^|[\s\"'=])((/home|/Users|/mnt|/var|/tmp)/[A-Za-z0-9_.@+/-]+|[A-Za-z]:\\[^\\]+)")


def latest_bundle() -> Path:
    if not DEFAULT_EXPORT_DIR.is_dir():
        raise SystemExit("FAIL .igy6-local/exports does not exist; pass --bundle PATH")
    candidates = sorted(DEFAULT_EXPORT_DIR.glob("igy6-backup-export-*.json"))
    if not candidates:
        raise SystemExit("FAIL no igy6-backup-export-*.json files found; pass --bundle PATH")
    return candidates[-1]


def load_bundle(path: Path) -> dict[str, Any]:
    try:
        with path.open(encoding="utf-8") as handle:
            payload = json.load(handle)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"FAIL malformed bundle JSON: {exc}") from exc
    except OSError as exc:
        raise SystemExit(f"FAIL could not read bundle: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit("FAIL bundle root must be a JSON object")
    return payload


def scalar(value: Any) -> str:
    if value is None or value == "":
        return "(missing)"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float, str)):
        text = str(value).replace("\n", " ").replace("\r", " ").strip()
        return text[:240] if text else "(missing)"
    return "(not scalar)"


def records_for_class(class_payload: Any) -> list[Any]:
    if isinstance(class_payload, dict):
        records = class_payload.get("records")
        if isinstance(records, list):
            return records
    return []


def walk(value: Any, path: str = "$") -> list[tuple[str, str, Any]]:
    findings: list[tuple[str, str, Any]] = []
    if isinstance(value, dict):
        for key, item in value.items():
            key_text = str(key)
            item_path = f"{path}.{key_text}"
            if SECRET_KEY_RE.search(key_text) and item != "[excluded]":
                findings.append(("secret_key", item_path, item))
            if CONTENT_KEY_RE.search(key_text) and item != "[excluded]":
                findings.append(("raw_content_key", item_path, item))
            findings.extend(walk(item, item_path))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            findings.extend(walk(item, f"{path}[{index}]"))
    elif isinstance(value, str):
        if SECRET_KEY_RE.search(value) and value != "[excluded]":
            findings.append(("secret_value_hint", path, value))
        if PRIVATE_PATH_RE.search(value):
            findings.append(("raw_path_hint", path, value))
    return findings


def validate(bundle: dict[str, Any]) -> tuple[list[str], list[str], list[str], dict[str, int], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    missing = [field for field in EXPECTED_TOP_LEVEL_FIELDS if field not in bundle]
    if missing:
        errors.append(f"missing top-level fields: {', '.join(missing)}")

    schema_version = bundle.get("schema_version")
    if schema_version != SUPPORTED_SCHEMA_VERSION:
        errors.append(f"unsupported schema_version: {scalar(schema_version)}")

    export = bundle.get("export")
    if not isinstance(export, dict):
        errors.append("export must be an object")
        export = {}
    classes = bundle.get("classes")
    if not isinstance(classes, dict):
        errors.append("classes must be an object")
        classes = {}

    record_counts: dict[str, int] = {}
    declared_counts = export.get("record_counts") if isinstance(export, dict) else None
    if not isinstance(declared_counts, dict):
        warnings.append("export.record_counts is missing or not an object")
        declared_counts = {}

    unsupported_classes = sorted(set(classes) - SUPPORTED_CLASSES)
    for class_name in unsupported_classes:
        warnings.append(f"unsupported class present: {class_name}")

    missing_supported = sorted(SUPPORTED_CLASSES - set(classes))
    for class_name in missing_supported:
        warnings.append(f"supported class absent from bundle: {class_name}")

    for class_name, class_payload in sorted(classes.items()):
        records = records_for_class(class_payload)
        record_counts[class_name] = len(records)
        declared = declared_counts.get(class_name)
        if isinstance(declared, int) and declared != len(records):
            warnings.append(f"{class_name} declared count {declared} differs from records length {len(records)}")

    findings = walk(bundle)
    unsafe_findings: list[str] = []
    for finding_type, path, value in findings:
        preview = scalar(value)
        unsafe_findings.append(f"{finding_type} at {path}: {preview}")
    if unsafe_findings:
        warnings.append("unsafe bundle content hints found; pass --strict-safety to fail closed")

    return errors, warnings, unsafe_findings, record_counts, unsupported_classes


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate an IGY6 backup export bundle without restoring it.")
    parser.add_argument("--bundle", type=Path, help="Path to a DIFF-229 backup export bundle.")
    parser.add_argument("--latest", action="store_true", help="Use the newest bundle in .igy6-local/exports.")
    parser.add_argument(
        "--strict-safety",
        action="store_true",
        help="Exit nonzero when secret/content/private-path hints are present.",
    )
    args = parser.parse_args()

    if args.bundle and args.latest:
        raise SystemExit("FAIL choose either --bundle or --latest, not both")
    bundle_path = args.bundle if args.bundle else latest_bundle()
    bundle_path = bundle_path.expanduser().resolve()

    bundle = load_bundle(bundle_path)
    errors, warnings, unsafe_findings, counts, unsupported_classes = validate(bundle)

    try:
        display_path = bundle_path.relative_to(REPO_ROOT)
    except ValueError:
        display_path = bundle_path

    print("IGY6 Restore Dry-Run MVP")
    print(f"bundle: {display_path}")
    print(f"schema_version: {scalar(bundle.get('schema_version'))}")
    print(f"created_at_utc: {scalar(bundle.get('created_at_utc'))}")
    repo = bundle.get("repo") if isinstance(bundle.get("repo"), dict) else {}
    print(f"repo_branch: {scalar(repo.get('branch'))}")
    print(f"repo_head: {scalar(repo.get('head'))}")
    print("restore_mode: dry_run_only")
    print("would_restore:")
    for class_name in sorted(counts):
        print(f"  {class_name}: {counts[class_name]}")
    print("writes: none")
    print("runtime_targets: PostgreSQL=none artifacts=none Qdrant=none Neo4j=none Redis=none MLflow=none Phoenix=none")
    if unsupported_classes:
        print("unsupported_classes:")
        for class_name in unsupported_classes:
            print(f"  {class_name}")
    if warnings:
        print("warnings:")
        for warning in warnings:
            print(f"  {warning}")
    if unsafe_findings:
        print("unsafe_findings:")
        for finding in unsafe_findings:
            print(f"  {finding}")
    print(f"strict_safety: {'true' if args.strict_safety else 'false'}")
    if errors:
        print("errors:")
        for error in errors:
            print(f"  {error}")
        return 1
    if args.strict_safety and unsafe_findings:
        print("errors:")
        print("  strict safety failed because unsafe bundle content hints were found")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
PY
