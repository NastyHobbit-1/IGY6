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


def schema_sql() -> str:
    return "\n".join(
        [
            "CREATE TABLE IF NOT EXISTS sources (",
            "  id varchar(36) PRIMARY KEY,",
            "  name varchar(255) NOT NULL,",
            "  source_type varchar(64) NOT NULL,",
            "  location text,",
            "  owner_actor_id varchar(128) NOT NULL DEFAULT 'local-owner',",
            "  sensitivity varchar(64) NOT NULL DEFAULT 'internal',",
            "  trust_level varchar(64) NOT NULL DEFAULT 'unreviewed',",
            "  enabled boolean NOT NULL DEFAULT true,",
            "  metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  updated_at timestamptz NOT NULL DEFAULT now()",
            ");",
            "CREATE TABLE IF NOT EXISTS collection_runs (",
            "  id varchar(36) PRIMARY KEY,",
            "  source_id varchar(36) REFERENCES sources(id),",
            "  status varchar(64) NOT NULL DEFAULT 'created',",
            "  dry_run boolean NOT NULL DEFAULT true,",
            "  requested_by_actor_id varchar(128) NOT NULL DEFAULT 'local-owner',",
            "  summary_json jsonb NOT NULL DEFAULT '{}'::jsonb,",
            "  error_message text,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  updated_at timestamptz NOT NULL DEFAULT now()",
            ");",
            "CREATE TABLE IF NOT EXISTS raw_artifacts (",
            "  id varchar(36) PRIMARY KEY,",
            "  source_id varchar(36) REFERENCES sources(id),",
            "  collection_run_id varchar(36) REFERENCES collection_runs(id),",
            "  content_hash varchar(128) NOT NULL,",
            "  storage_path text NOT NULL,",
            "  mime_type varchar(255),",
            "  size_bytes integer,",
            "  metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  updated_at timestamptz NOT NULL DEFAULT now()",
            ");",
            "CREATE TABLE IF NOT EXISTS normalized_documents (",
            "  id varchar(36) PRIMARY KEY,",
            "  raw_artifact_id varchar(36) REFERENCES raw_artifacts(id),",
            "  source_id varchar(36) REFERENCES sources(id),",
            "  title varchar(255),",
            "  document_type varchar(64) NOT NULL DEFAULT 'unknown',",
            "  language varchar(32),",
            "  text_content text NOT NULL,",
            "  sensitivity varchar(64) NOT NULL DEFAULT 'internal',",
            "  metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  updated_at timestamptz NOT NULL DEFAULT now()",
            ");",
            "CREATE TABLE IF NOT EXISTS work_items (",
            "  id varchar(36) PRIMARY KEY,",
            "  work_type varchar(64) NOT NULL,",
            "  status varchar(64) NOT NULL DEFAULT 'queued',",
            "  requested_by_actor_id varchar(128) NOT NULL DEFAULT 'local-owner',",
            "  payload_json jsonb NOT NULL DEFAULT '{}'::jsonb,",
            "  error_message text,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  updated_at timestamptz NOT NULL DEFAULT now()",
            ");",
            "CREATE TABLE IF NOT EXISTS audit_events (",
            "  id integer GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,",
            "  created_at timestamptz NOT NULL DEFAULT now(),",
            "  actor_id varchar(128) NOT NULL DEFAULT 'system',",
            "  event_type varchar(128) NOT NULL,",
            "  decision varchar(64),",
            "  resource_type varchar(64),",
            "  resource_id varchar(128),",
            "  correlation_id varchar(128),",
            "  details_json jsonb NOT NULL DEFAULT '{}'::jsonb",
            ");",
        ]
    )


def observation_sql() -> str:
    return "\n".join(
        [
            "\\pset format unaligned",
            "\\pset fieldsep '|'",
            "\\pset tuples_only off",
            "SELECT 'work_item' AS section, id, work_type, status, COALESCE(error_message, '') AS error_message FROM work_items WHERE id = 'diff-152-canary-work-item' ORDER BY id;",
            "SELECT 'chained_work_item' AS section, id, work_type, status, payload_json->>'parent_work_item_id' AS parent_work_item_id FROM work_items WHERE payload_json->>'parent_work_item_id' = 'diff-152-canary-work-item' ORDER BY id;",
            "SELECT 'audit_event' AS section, event_type, decision, resource_type, resource_id, correlation_id FROM audit_events WHERE correlation_id = 'diff-152-canary-work-item' OR resource_id = 'diff-152-canary-work-item' ORDER BY id;",
            "SELECT 'normalized_document' AS section, id, raw_artifact_id, source_id, title, document_type, sensitivity, length(text_content)::text AS text_length FROM normalized_documents WHERE raw_artifact_id = 'diff-152-canary-raw' ORDER BY id;",
            "SELECT 'raw_artifact' AS section, id, storage_path, content_hash, size_bytes::text FROM raw_artifacts WHERE id = 'diff-152-canary-raw' ORDER BY id;",
        ]
    )


def write_sql(path: Path, include_schema: bool, include_fixture: bool, include_observation: bool) -> None:
    sections: list[str] = []
    if include_schema:
        sections.append(schema_sql())
    if include_fixture:
        sections.append(fixture_sql())
    if include_observation:
        sections.append(observation_sql())
    if not sections:
        raise SystemExit("--write-sql requires at least one SQL-emitting flag")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n\n".join(sections) + "\n", encoding="utf-8")


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
    parser.add_argument("--emit-schema-sql", action="store_true", help="Print minimal canary schema SQL.")
    parser.add_argument("--emit-sql", action="store_true", help="Print SQL seed statements.")
    parser.add_argument(
        "--emit-observation-sql",
        action="store_true",
        help="Print read-only SQL queries for observing the selected canary.",
    )
    parser.add_argument(
        "--write-sql",
        help="Write selected SQL sections to a file for an isolated canary database.",
    )
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

    if args.emit_schema_sql:
        print()
        print("-- DIFF-153 minimal canary schema SQL")
        print(schema_sql())

    if args.emit_sql:
        print()
        print("-- DIFF-152 deterministic canary seed SQL")
        print(fixture_sql())

    if args.emit_observation_sql:
        print()
        print("-- DIFF-153 read-only canary observation SQL")
        print(observation_sql())

    if args.write_sql:
        write_sql(
            Path(args.write_sql),
            include_schema=args.emit_schema_sql,
            include_fixture=args.emit_sql,
            include_observation=args.emit_observation_sql,
        )
        print()
        print(f"Wrote SQL: {args.write_sql}")

    if args.write_artifact:
        target = write_artifact(Path(args.data_root))
        print()
        print(f"Wrote synthetic artifact: {target}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
