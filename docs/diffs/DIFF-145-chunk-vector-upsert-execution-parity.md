# DIFF-145: Chunk Vector Upsert Execution Parity

Status: Locked

## Type

Change-bearing Rust worker chunk-vector-upsert parity update.

## Objective

Implement the third Rust worker execution parity layer for
`chunk_vector_upsert` while keeping Python/Celery `worker` and `beat` active.

Decision:

- Add Rust-side `chunk_vector_upsert` execution planning and
  SQL/audit/status/Qdrant request executor contracts.
- Preserve Python/Celery chunk vector upsert semantics.
- Do not replace the live Python/Celery worker process.
- Do not replace or remove `beat`.
- Do not remove `services/worker/`, the `worker` service, or the `beat`
  service.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-142 is complete and locked.
- DIFF-143 is complete and locked.
- DIFF-144 is complete and locked.
- DIFF-143 added `collection_normalization` planning/executor contracts.
- DIFF-144 added `document_chunking` planning/executor contracts.
- Python/Celery still owns live end-to-end worker process execution.

## Allowed Scope

- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-145-chunk-vector-upsert-execution-parity.md`
- Live runtime/migration docs whose worker wording was stale.

## Prohibited Scope

- No DIFF-146 work.
- No live Rust worker process cutover.
- No beat or scheduler replacement.
- No removal of `services/worker/`.
- No removal of `worker` or `beat` from Docker Compose.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.

## Implementation Notes

DIFF-145 adds `chunk_vector_upsert` execution parity contracts in
`crates/igy6-worker`.

Covered behavior:

- Validate the work item is a `chunk_vector_upsert` item.
- Validate payload shape and optional `chunk_ids`.
- Validate limit bounds from 1 through 1000.
- Select chunks whose `embedding_status` is not `completed`.
- Scope selection to requested `chunk_ids` when supplied.
- Order selected chunks by ID and bound selection by limit.
- Generate deterministic local chunk vectors with the existing Rust vector
  memory helper.
- Preserve Python/Celery metadata names for the worker path:
  `embedding_method=local_hash_v1`, `generated_by=DIFF-053`, and the configured
  vector collection.
- Plan Qdrant collection status, collection ensure, and point upsert requests.
- Plan chunk `embedding_status=completed` updates.
- Plan chunk metadata merges with `embedding_method` and `vector_collection`.
- Plan originating `work_items` completed/failed status updates.
- Plan `chunk_vectors.upserted` and `chunk_vectors.failed` audit events.
- Provide SQL contract strings for chunk selection, chunk update, work-item
  status updates, and audit insert.

Not implemented:

- No live worker process replacement.
- No beat or scheduler replacement.
- No Python/Celery worker or beat removal.
- No Neo4j operation.
- No full Rust-only runtime claim.

## Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery `worker` and `beat` services. Rust-only is not claimed.

DIFF-145 narrows the worker migration gap by covering `chunk_vector_upsert`
planning/executor contracts. Python/Celery remains required because live worker
process ownership and beat/scheduled-work posture are not yet replaced.

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

- Rust worker has scoped `chunk_vector_upsert` parity contracts.
- Tests cover success, optional requested chunk scoping, unrequested limit
  selection, empty selection, invalid payloads, wrong work item, failure audit
  shape, Qdrant request planning, chunk metadata/status planning, and SQL plan
  shape.
- Manifest records chunk vector upsert parity status.
- Docs state Python/Celery worker and beat remain active.
- DIFF-146 remains out of scope.

## Completion Notes

DIFF-145 adds Rust-side chunk-vector-upsert execution planning and executor
contracts in `crates/igy6-worker`.

The migration covers the third worker job family only:

- `memory.vector.upsert_chunks`
- `chunk_vector_upsert`

The Rust worker contract now models the chunk selection, deterministic vector
generation, Qdrant request planning, chunk status/metadata updates, work-item
status transitions, and audit events required to execute chunk vector upserts
safely. It does not cut over the live worker process and does not replace
Celery Beat.

Next recommended DIFF:

- DIFF-146 worker process cutover and scheduler/beat decision.

## Verification Results

- `git status --short` inspected scoped DIFF-145 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed after formatting the Rust worker changes.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed with 34 tests.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-145 changed no
  UI source or UI-consumed status text.
