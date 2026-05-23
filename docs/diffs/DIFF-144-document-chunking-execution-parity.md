# DIFF-144: Document Chunking Execution Parity

Status: Locked

## Type

Change-bearing Rust worker document-chunking parity update.

## Objective

Implement the second Rust worker execution parity layer for `document_chunking`
while keeping Python/Celery `worker` and `beat` active.

Decision:

- Add Rust-side `document_chunking` execution planning and SQL/audit/status
  executor contracts.
- Preserve Python/Celery chunk and evidence-item generation semantics.
- Do not migrate `chunk_vector_upsert`.
- Do not perform Qdrant work.
- Do not remove `services/worker/`, the `worker` service, or the `beat`
  service.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-142 is complete and locked.
- DIFF-143 is complete and locked.
- DIFF-143 added `collection_normalization` planning/executor contracts.
- Python/Celery still owns live end-to-end worker process execution.

## Allowed Scope

- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-144-document-chunking-execution-parity.md`
- Live runtime/migration docs whose worker wording was stale.

## Prohibited Scope

- No DIFF-145 work.
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

DIFF-144 adds `document_chunking` execution parity contracts in
`crates/igy6-worker`.

Covered behavior:

- Validate the work item is a `document_chunking` item.
- Validate payload `document_ids` or legacy single `document_id` matches the
  task request.
- Validate chunk size bounds from 100 through 5000.
- Require every requested normalized document to exist.
- Reject empty document text deterministically.
- Generate deterministic chunk plans using the existing Rust chunking crate.
- Plan `chunks` inserts with Python/Celery field semantics:
  `embedding_status=not_started`, location character offsets, and metadata
  containing `generated_by=DIFF-052`, chunk size, and parent work item ID.
- Plan `evidence_items` inserts for each generated chunk with
  `evidence_type=document_chunk`, no observed timestamp, no confidence, and
  DIFF-052 metadata.
- Skip documents that already have chunks.
- Plan originating `work_items` completed/failed status updates.
- Plan `document_chunks.generated` and `document_chunks.failed` audit events.
- Plan chained `chunk_vector_upsert` work item creation when new chunks are
  created, including the existing DIFF-066/DIFF-074 governance payload.
- Provide SQL contract strings for status updates, chunk inserts, evidence-item
  inserts, chained work-item insert, and audit insert.

Not implemented:

- No chunk vector upsert.
- No Qdrant operation.
- No Neo4j operation.
- No Python/Celery worker or beat removal.
- No full Rust-only runtime claim.

## Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery `worker` and `beat` services. Rust-only is not claimed.

DIFF-144 narrows the worker migration gap by covering `document_chunking`
planning/executor contracts. Python/Celery remains required because
`chunk_vector_upsert`, live process ownership, and beat/scheduled-work posture
are not yet replaced.

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

- Rust worker has scoped `document_chunking` parity contracts.
- Tests cover success, missing document, invalid payload, empty text, invalid
  chunk size, generated ID coverage, duplicate skip behavior, audit shape,
  chained work-item creation, status transitions, and SQL plan shape.
- Manifest records document chunking parity status.
- Docs state Python/Celery worker and beat remain active.
- DIFF-145 remains out of scope.

## Completion Notes

DIFF-144 adds Rust-side document-chunking execution planning and executor
contracts in `crates/igy6-worker`.

The migration covers the second worker job family only:

- `evidence.generate_document_chunks`
- `document_chunking`

The Rust worker contract now models the DB writes, status transitions, and
audit events required to execute document chunking safely. It does not execute
the next pipeline stage. Chained `chunk_vector_upsert` work items are planned
only because Python/Celery chunking already creates them after successful chunk
creation.

Next recommended DIFF:

- DIFF-145 Rust `chunk_vector_upsert` execution parity.

## Verification Results

- `git status --short` inspected scoped DIFF-144 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed after formatting the Rust worker changes.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed with 27 tests.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-144 changed no
  UI source or UI-consumed status text.
