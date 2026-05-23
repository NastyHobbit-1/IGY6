# DIFF-146: Worker Process Cutover And Scheduler Beat Decision

Status: Locked

## Type

Decision and audit DIFF for Rust worker process cutover readiness.

## Objective

Decide whether Rust worker execution is ready to replace Python/Celery worker
process ownership after DIFF-142 through DIFF-145 added worker queue-claim and
job-family parity contracts.

Decision:

- Choose Decision B: Rust worker process cutover is not ready.
- Retain Python/Celery `worker`.
- Retain Python/Celery `beat`.
- Do not remove `services/worker/`.
- Do not remove `worker` or `beat` from Docker Compose.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- Rust API path is complete.
- FastAPI fallback is removed.
- `services/api/` is archived.
- DIFF-142 added Rust worker queue-claim foundation.
- DIFF-143 added `collection_normalization` parity contracts.
- DIFF-144 added `document_chunking` parity contracts.
- DIFF-145 added `chunk_vector_upsert` parity contracts.
- Python/Celery `worker` and `beat` remain active Docker Compose services.

## Inspection Findings

`crates/igy6-worker` currently contains:

- `Cargo.toml`
- `src/lib.rs`

It does not contain:

- a `src/main.rs` worker binary;
- a Dockerfile or Compose service for a Rust worker process;
- a long-running polling loop;
- a Redis subscriber or PostgreSQL queue polling runtime;
- live PostgreSQL executor code for planned writes;
- live artifact-store reads during job execution;
- live Qdrant HTTP execution for planned vector requests;
- runtime retry/backoff/shutdown/health behavior.

`services/worker` still contains the active Celery runtime:

- Celery app wiring.
- Python worker settings.
- `phase0.health`.
- `collection.normalization_scaffold`.
- `collection.normalize_collection_run`.
- `evidence.generate_document_chunks`.
- `memory.vector.upsert_chunks`.

`infra/docker-compose.yml` still defines active `worker` and `beat` services
built from `services/worker`.

## Decision

Decision B: Rust worker process cutover is not ready.

The Rust contracts are now broad enough to describe the three queued job
families, but contracts are not runtime ownership. Removing Python/Celery now
would remove the only active process that performs end-to-end worker side
effects.

## Exact Blockers

- No Rust worker binary or container exists.
- No Rust runtime polls or subscribes to queued work items.
- No Rust runtime atomically claims queued jobs with live PostgreSQL
  `FOR UPDATE SKIP LOCKED` execution.
- No Rust runtime reads artifact bytes from the configured artifact store
  during job execution.
- No Rust runtime applies planned `normalized_documents`, `chunks`,
  `evidence_items`, `work_items`, or `audit_events` writes to PostgreSQL.
- No Rust runtime executes planned Qdrant collection ensure or point upsert
  HTTP requests.
- No Rust runtime has retry/backoff, shutdown, logging, or health behavior
  equivalent to the current worker service.
- Beat/scheduled-work posture is not replaced or formally retired.

## Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery `worker` and `beat` services. Rust-only is not claimed.

## Allowed Scope Completed

- Added this DIFF decision document.
- Updated `configs/rust-cutover-manifest.json`.
- Updated live runtime documentation that would otherwise imply the worker
  process cutover was ready.

No Compose service was removed. No Python worker source was removed. No `.env`
or runtime/private data was touched.

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

- Decision A or B is recorded.
- Manifest states whether worker and beat remain.
- Runtime docs state the actual worker posture.
- Full Rust-only runtime is not claimed unless Python/Celery worker and beat
  are removed or no longer active.
- DIFF-147 remains out of scope.

## Completion Notes

DIFF-146 chooses Decision B.

Python/Celery `worker` and `beat` remain because Rust does not yet have a live
worker process that can safely own queue polling, job claiming, DB writes,
audit writes, artifact reads, Qdrant side effects, and scheduler posture.

Next recommended DIFF:

- DIFF-147 Rust worker runtime execution harness and cutover readiness gate.

## Verification Results

- `git status --short` inspected scoped DIFF-146 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed with 34 tests.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-146 changed no
  UI source or UI-consumed status text.
