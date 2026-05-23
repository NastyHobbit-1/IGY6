# DIFF-143: Collection Normalization Execution Parity

Status: Locked

## Type

Change-bearing Rust worker collection-normalization parity update.

## Objective

Implement the first Rust worker execution parity layer for
`collection_normalization` while keeping Python/Celery `worker` and `beat`
active.

Decision:

- Add Rust-side `collection_normalization` execution planning and
  SQL/audit/status executor contracts.
- Preserve Python/Celery normalization semantics for raw UTF-8 artifacts.
- Do not migrate `document_chunking`.
- Do not migrate `chunk_vector_upsert`.
- Do not remove `services/worker/`, the `worker` service, or the `beat`
  service.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-142 is complete and locked.
- DIFF-142 added queue-claim contract planning only.
- DIFF-141 recommends migrating worker execution one job family at a time.
- Python/Celery still owns live end-to-end worker process execution.

## Allowed Scope

- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-143-collection-normalization-execution-parity.md`
- Live runtime/migration docs whose worker wording was stale.

## Prohibited Scope

- No DIFF-144 work.
- No `document_chunking` execution migration.
- No `chunk_vector_upsert` execution migration.
- No Qdrant calls from the Rust worker.
- No Neo4j calls from the Rust worker.
- No removal of `services/worker/`.
- No removal of `worker` or `beat` from Docker Compose.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.

## Implementation Notes

DIFF-143 adds `collection_normalization` execution parity contracts in
`crates/igy6-worker`.

Covered behavior:

- Validate the work item is a `collection_normalization` item.
- Validate payload `collection_run_id` and ordered `raw_artifact_ids` match the
  task request.
- Require the collection run to exist.
- Require every requested raw artifact to exist.
- Reject raw artifacts that do not belong to the collection run.
- Decode artifact bytes as UTF-8 and reject non-UTF-8 content with the same
  deterministic failure posture.
- Plan `normalized_documents` inserts with Python/Celery field semantics:
  `document_type=text`, `language=null`, `sensitivity=internal`, title from
  artifact metadata, and metadata containing `generated_by=DIFF-051`,
  raw content hash, raw storage path, and parent work item ID.
- Skip raw artifacts that already have normalized documents.
- Plan originating `work_items` completed/failed status updates.
- Plan `collection_normalization.completed` and
  `collection_normalization.failed` audit events.
- Plan chained `document_chunking` work item creation when new normalized
  documents are created, including the existing DIFF-066/DIFF-074 governance
  payload.
- Provide SQL contract strings for status updates, document inserts, chained
  work-item insert, and audit insert.

Not implemented:

- No document chunk generation.
- No evidence item generation.
- No chunk vector upsert.
- No Qdrant operation.
- No Neo4j operation.
- No Python/Celery worker or beat removal.
- No full Rust-only runtime claim.

## Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery `worker` and `beat` services. Rust-only is not claimed.

DIFF-143 narrows the worker migration gap by covering
`collection_normalization` planning/executor contracts. Python/Celery remains
required because `document_chunking`, `chunk_vector_upsert`, live process
ownership, and beat/scheduled-work posture are not yet replaced.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-worker`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

`npm --prefix apps/web run build` is not required unless UI/status source text
changes.

## Completion Criteria

- Rust worker has scoped `collection_normalization` parity contracts.
- Tests cover success, missing artifact, invalid payload, non-UTF-8 input,
  duplicate skip behavior, audit shape, chained work-item creation, status
  transitions, and SQL plan shape.
- Manifest records collection normalization parity status.
- Docs state Python/Celery worker and beat remain active.
- DIFF-144 remains out of scope.

## Completion Notes

DIFF-143 adds Rust-side collection-normalization execution planning and
executor contracts in `crates/igy6-worker`.

The migration covers the first worker job family only:

- `collection.normalize_collection_run`
- `collection_normalization`

The Rust worker contract now models the DB writes, status transitions, and
audit events required to execute normalization safely. It does not execute the
next pipeline stage. Chained `document_chunking` work items are planned only
because Python/Celery normalization already creates them after successful
document creation.

Next recommended DIFF:

- DIFF-144 Rust `document_chunking` execution parity.

## Verification Results

- `git status --short` inspected scoped DIFF-143 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed after formatting the Rust worker changes.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed with 19 tests.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-143 changed no
  UI source or UI-consumed status text.
