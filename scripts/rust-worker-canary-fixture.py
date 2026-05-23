#!/usr/bin/env python3
"""DIFF-152 deterministic Rust worker canary fixture helper.

Default behavior is non-mutating: print the selected canary IDs, artifact
metadata, and SQL needed to seed exactly one queued collection_normalization
work item. Writing the synthetic artifact requires --write-artifact and a
canary/test data root.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


ACTOR_ID = "diff-152-canary"
SOURCE_ID = "diff-152-canary-source"
COLLECTION_RUN_ID = "diff-152-canary-run"
RAW_ARTIFACT_ID = "diff-152-canary-raw"
WORK_ITEM_ID = "diff-152-canary-work-item"
ARTIFACT_TEXT = (
    "DIFF-152 synthetic Rust worker canary artifact.\n"
    "This file contains non-sensitive local fixture text only.\n"
    "It exists to verify one selected collection_normalization canary in a later DIFF.\n"
)


def json_sql(value: Any) -> str:
    return "'" + json.dumps(value, sort_keys=True).replace("'", "''") + "'::jsonb"


def text_sql(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def artifact_plan() -> dict[str, Any]:
    artifact_bytes = ARTIFACT_TEXT.encode("utf-8")
    content_hash = hashlib.sha256(artifact_bytes).hexdigest()
    storage_path = f"sha256/{content_hash[0:2]}/{content_hash[2:4]}/{content_hash}"
    return {
        "content": ARTIFACT_TEXT,
        "content_hash": content_hash,
        "size_bytes": len(artifact_bytes),
        "storage_path": storage_path,
    }


def work_item_payload() -> dict[str, Any]:
    return {
        "collection_run_id": COLLECTION_RUN_ID,
        "raw_artifact_ids": [RAW_ARTIFACT_ID],
        "worker_task_name": "ingestion.normalize_collection",
        "generated_by": "DIFF-152",
        "intent_verification_recorded": True,
        "intent_verification": {
            "original_request": "Create one safe Rust worker canary fixture.",
            "interpretation": (
                "Seed one synthetic queued collection_normalization work item "
                "for a later explicitly gated Rust canary run."
            ),
            "proposed_work_type": "collection_normalization",
            "sources_likely_used": [SOURCE_ID],
            "expected_output": (
                "One normalized_documents row and one chained document_chunking "
                "work item if the later live canary is run."
            ),
            "safety_requirements": [
                "Use synthetic fixture data only.",
                "Run only the selected work item.",
                "Keep Python/Celery worker and beat active until cutover is proven.",
            ],
            "assumptions": [
                "The future canary DIFF will isolate this selected work item from Celery races."
            ],
            "missing_information": [],
            "recorded_by": "DIFF-152 fixture",
        },
    }


def fixture_sql() -> str:
    artifact = artifact_plan()
    source_metadata = {
        "generated_by": "DIFF-152",
        "fixture": "rust_worker_canary",
        "non_production": True,
    }
    collection_summary = {
        "generated_by": "DIFF-152",
        "fixture": "rust_worker_canary",
        "raw_artifact_ids": [RAW_ARTIFACT_ID],
        "selected_work_item_id": WORK_ITEM_ID,
    }
    artifact_metadata = {
        "filename": "diff-152-rust-worker-canary.txt",
        "generated_by": "DIFF-152",
        "fixture": "rust_worker_canary",
        "non_sensitive": True,
    }
    payload = work_item_payload()
    return "\n".join(
        [
            "BEGIN;",
            "INSERT INTO sources (id, name, source_type, location, owner_actor_id, sensitivity, trust_level, enabled, metadata_json)",
            (
                f"VALUES ({text_sql(SOURCE_ID)}, 'DIFF-152 Rust worker canary source', "
                f"'manual_upload', 'synthetic://diff-152-rust-worker-canary', "
                f"{text_sql(ACTOR_ID)}, 'internal', 'test_fixture', true, {json_sql(source_metadata)})"
            ),
            "ON CONFLICT (id) DO UPDATE SET enabled = true, metadata_json = EXCLUDED.metadata_json, updated_at = now();",
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json)",
            (
                f"VALUES ({text_sql(COLLECTION_RUN_ID)}, {text_sql(SOURCE_ID)}, "
                f"'collected', false, {text_sql(ACTOR_ID)}, {json_sql(collection_summary)})"
            ),
            "ON CONFLICT (id) DO UPDATE SET status = 'collected', summary_json = EXCLUDED.summary_json, updated_at = now();",
            "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json)",
            (
                f"VALUES ({text_sql(RAW_ARTIFACT_ID)}, {text_sql(SOURCE_ID)}, "
                f"{text_sql(COLLECTION_RUN_ID)}, {text_sql(artifact['content_hash'])}, "
                f"{text_sql(artifact['storage_path'])}, 'text/plain', {artifact['size_bytes']}, "
                f"{json_sql(artifact_metadata)})"
            ),
            "ON CONFLICT (id) DO UPDATE SET content_hash = EXCLUDED.content_hash, storage_path = EXCLUDED.storage_path, metadata_json = EXCLUDED.metadata_json, updated_at = now();",
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message)",
            (
                f"VALUES ({text_sql(WORK_ITEM_ID)}, 'collection_normalization', 'queued', "
                f"{text_sql(ACTOR_ID)}, {json_sql(payload)}, NULL)"
            ),
            "ON CONFLICT (id) DO UPDATE SET status = 'queued', payload_json = EXCLUDED.payload_json, error_message = NULL, updated_at = now();",
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json)",
            (
                f"VALUES ({text_sql(ACTOR_ID)}, 'rust_worker_canary_fixture.selected', "
                f"'selected', 'work_item', {text_sql(WORK_ITEM_ID)}, {text_sql(WORK_ITEM_ID)}, "
                f"{json_sql({'generated_by': 'DIFF-152', 'selected_work_item_id': WORK_ITEM_ID, 'work_type': 'collection_normalization'})});"
            ),
            "COMMIT;",
        ]
    )


def ensure_canary_data_root(data_root: Path) -> None:
    normalized = data_root.expanduser().resolve()
    lowered = str(normalized).lower()
    if "canary" not in lowered and "diff152" not in lowered and "diff-152" not in lowered:
        raise SystemExit(
            "Refusing to write artifact: --data-root must clearly be a canary/diff152 test path."
        )


def write_artifact(data_root: Path) -> Path:
    ensure_canary_data_root(data_root)
    artifact = artifact_plan()
    target = data_root.expanduser().resolve() / "artifacts" / artifact["storage_path"]
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists() and target.read_text(encoding="utf-8") != ARTIFACT_TEXT:
        raise SystemExit(f"Refusing to overwrite non-matching artifact: {target}")
    target.write_text(ARTIFACT_TEXT, encoding="utf-8")
    return target


def main() -> int:
    parser = argparse.ArgumentParser(description="Prepare the DIFF-152 Rust worker canary fixture.")
    parser.add_argument("--emit-sql", action="store_true", help="Print SQL seed statements.")
    parser.add_argument(
        "--write-artifact",
        action="store_true",
        help="Write the synthetic artifact under --data-root/artifacts.",
    )
    parser.add_argument(
        "--data-root",
        default="/tmp/igy6-diff152-canary",
        help="Synthetic canary IGY6_DATA_ROOT used only with --write-artifact.",
    )
    args = parser.parse_args()

    artifact = artifact_plan()
    selected = {
        "diff": "DIFF-152",
        "decision": "A",
        "fixture": "rust_worker_canary_collection_normalization",
        "selected_work_item_id": WORK_ITEM_ID,
        "source_id": SOURCE_ID,
        "collection_run_id": COLLECTION_RUN_ID,
        "raw_artifact_id": RAW_ARTIFACT_ID,
        "work_type": "collection_normalization",
        "artifact_storage_path": artifact["storage_path"],
        "artifact_content_hash": artifact["content_hash"],
        "artifact_size_bytes": artifact["size_bytes"],
        "live_canary_command": (
            "IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- "
            f"--once --canary-live --canary-work-item {WORK_ITEM_ID}"
        ),
        "live_canary_run_by_fixture": False,
    }
    print(json.dumps(selected, indent=2, sort_keys=True))

    if args.emit_sql:
        print()
        print("-- DIFF-152 deterministic canary seed SQL")
        print(fixture_sql())

    if args.write_artifact:
        target = write_artifact(Path(args.data_root))
        print()
        print(f"Wrote synthetic artifact: {target}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
