# DIFF-142: Worker Execution Contract Queue Claim

Status: Locked

## Type

Change-bearing Rust worker queue-claim foundation.

## Objective

Add a Rust worker execution contract and queue-claim foundation without
executing any worker job family.

Decision:

- Define Rust-side queue-claim contracts for the existing worker work types.
- Keep claim planning bounded, local, auditable, and execution-free.
- Do not start DIFF-143 collection normalization execution parity.
- Keep Python/Celery `worker` and `beat` active.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-141 is complete and locked.
- Python/Celery `worker` and `beat` remain active runtime components.
- Existing queued worker work types are `collection_normalization`,
  `document_chunking`, and `chunk_vector_upsert`.
- Rust gateway dispatch currently records non-executing dispatch metadata and
  does not invoke Celery.
- `crates/igy6-worker` already contains deterministic planning for a UTF-8
  normalization/chunk/vector pipeline, but it does not perform live queue
  claiming or execution.

## Allowed Scope

- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-142-worker-execution-contract-queue-claim.md`
- `docs/runtime/PROCESSING_STATUS.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/agents/DIFF-142_WORKER_EXECUTION_CONTRACT_QUEUE_CLAIM.md` if needed to
  persist the missing agent prompt for future runs.

## Prohibited Scope

- No DIFF-143 work.
- No `collection_normalization` execution implementation.
- No `document_chunking` execution implementation.
- No `chunk_vector_upsert` execution implementation.
- No removal of `services/worker/`.
- No removal of `worker` or `beat` from Docker Compose.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, redesign, data model changes,
  migration changes, or dependency changes.

## Implementation Notes

DIFF-142 adds Rust worker queue-claim planning in `crates/igy6-worker`.

The contract covers:

- Allowed work types:
  - `collection_normalization`
  - `document_chunking`
  - `chunk_vector_upsert`
- Legacy Celery task-name mapping for compatibility:
  - `collection.normalize_collection_run`
  - `evidence.generate_document_chunks`
  - `memory.vector.upsert_chunks`
- Claim eligibility:
  - status must be `queued`
  - intent verification must be present
  - claim actor must be non-empty
  - work-type payload must satisfy the existing dispatch contract
- Bounded query planning:
  - claim limit must be between 1 and 16
  - query uses `FOR UPDATE SKIP LOCKED`
  - update moves a still-queued item to `running`
- Audit contract:
  - event type `work_item.claimed`
  - decision `running`
  - execution status `claimed_without_execution`

The implementation does not connect to PostgreSQL, mutate work items, execute
task handlers, read artifacts, write audit events, call Qdrant, call Neo4j, or
replace Celery.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

`npm --prefix apps/web run build` is not required unless web-facing status text
changes.

## Completion Criteria

- Rust queue-claim contract exists and is tested.
- Contract supports only the three existing worker work types.
- Contract rejects unsupported work types, unqueued work items, missing intent
  verification, invalid payloads, empty claim actors, and invalid claim limits.
- Contract records execution-free claim posture.
- Python/Celery `worker` and `beat` remain active.
- DIFF-143 execution work remains out of scope.
- Required verification passes or blocked checks are recorded precisely.

## Completion Notes

DIFF-142 adds a Rust worker queue-claim contract and bounded claim planning in
`crates/igy6-worker`.

Implemented:

- `WorkerTaskKind` for the three existing worker work types.
- Legacy Celery task-name mapping for compatibility.
- `QueueClaimCandidate`, `QueueClaimPlan`, and `QueueClaimQueryPlan`.
- Bounded claim query planning with `FOR UPDATE SKIP LOCKED`.
- Claim validation for queued status, intent verification, non-empty actor,
  supported work type, payload shape, and claim limit.
- Execution-free audit posture: `work_item.claimed`, decision `running`,
  execution status `claimed_without_execution`.
- Tests covering valid claims and rejection paths.

Not implemented:

- No `collection_normalization` execution.
- No `document_chunking` execution.
- No `chunk_vector_upsert` execution.
- No live PostgreSQL queue claiming.
- No worker DB writes or audit writes.
- No artifact reads.
- No Qdrant or Neo4j calls.
- No Python/Celery worker or beat removal.

The requested prompt file
`docs/agents/DIFF-142_WORKER_EXECUTION_CONTRACT_QUEUE_CLAIM.md` was missing at
the start of this DIFF. DIFF-142 adds it for future runs.

Next recommended DIFF:

- DIFF-143 Rust `collection_normalization` execution parity.

## Verification Results

- `git status --short` inspected scoped DIFF-142 changes and pre-existing
  uncommitted DIFF-140 documentation changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed after formatting the new worker code.
- `cargo test -p igy6-worker` passed with 11 tests.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-142 changed no
  web-facing UI/status source text.
