# DIFF-147: Rust Worker Runtime Harness And Cutover Readiness

Status: Locked

## Type

Change-bearing Rust worker runtime harness and cutover readiness gate.

## Objective

Create the safe Rust worker runtime harness needed before Python/Celery worker
process ownership can be replaced.

Decision:

- Add a Rust worker binary.
- Add safe, non-mutating runtime modes.
- Add bounded queue/claim planning and configuration validation.
- Keep Python/Celery `worker` active.
- Keep Python/Celery `beat` active.
- Do not remove `services/worker/`.
- Do not remove `worker` or `beat` from Docker Compose.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-146 decided Rust worker process cutover was not ready.
- Rust API path is complete.
- FastAPI fallback is removed.
- `services/api/` is archived.
- Rust worker parity contracts cover:
  - `collection_normalization`
  - `document_chunking`
  - `chunk_vector_upsert`
- Python/Celery `worker` and `beat` remain active.

## Implementation Notes

DIFF-147 adds `crates/igy6-worker/src/main.rs` and runtime harness helpers in
`crates/igy6-worker/src/lib.rs`.

Runtime modes:

- `--help`
- `--check`
- `--dry-run`
- `--once`

Default behavior:

- No arguments defaults to safe `--check`.
- No mode mutates runtime data.
- No live execution mode is enabled in DIFF-147.

Validated configuration:

- `DATABASE_URL` must be PostgreSQL-shaped.
- `QDRANT_URL` must be `http(s)` and must not contain credentials.
- `IGY6_DATA_ROOT` must be a non-root local path.
- `QDRANT_CHUNK_COLLECTION` is restricted to safe ASCII name characters.
- `QDRANT_CHUNK_VECTOR_SIZE` must be at least 1.
- Claim limit is bounded from 1 through 16.
- Modeled poll interval is bounded from 100 through 60000 milliseconds.

Planned behavior:

- Build bounded queued-work SELECT planning.
- Build bounded claim UPDATE planning.
- Report supported work types.
- Render structured non-secret status output.

Blocked side effects:

- No PostgreSQL connection.
- No runtime queue mutation.
- No artifact-store reads.
- No audit writes.
- No Qdrant HTTP calls.
- No Celery or beat control.
- No arbitrary shell command execution.

## Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery `worker` and `beat` services. Rust-only is not claimed.

DIFF-147 is a readiness gate, not a cutover. Python/Celery remains required for
live worker execution because Rust live DB/audit writes, artifact reads,
Qdrant side effects, worker container wiring, and beat/scheduler posture are
not replaced.

## Allowed Scope Completed

- Added Rust worker binary.
- Added safe CLI/runtime modes and tests.
- Updated `configs/rust-cutover-manifest.json`.
- Updated README and runtime migration docs.
- Added this DIFF document.

No Docker Compose service was removed. No Python worker source was removed. No
`.env` or runtime/private data was touched.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-worker`
- `cargo run -p igy6-worker -- --help`
- `cargo run -p igy6-worker -- --check`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

`npm --prefix apps/web run build` is not required unless UI/status source text
changes.

## Completion Criteria

- Rust worker binary exists.
- Default command is safe and non-mutating.
- Check/dry-run/once modes exist.
- Runtime config validation is covered by tests.
- Queue/claim planning is covered by tests.
- Unsupported arguments and unsafe settings fail closed.
- Manifest and docs state Python/Celery worker and beat remain active.
- Full Rust-only runtime is not claimed.
- DIFF-148 remains out of scope.

## Completion Notes

DIFF-147 adds a safe Rust worker runtime harness but does not enable live Rust
worker execution.

Next recommended DIFF:

- DIFF-148 Rust worker live executor canary and side-effect verification.

## Verification Results

- `git status --short` inspected scoped DIFF-147 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed after formatting the worker harness.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed with 38 tests.
- `cargo run -p igy6-worker -- --help` passed and rendered safe harness
  usage.
- `cargo run -p igy6-worker -- --check` passed and reported
  `live_execution_enabled=false`, `mutates_runtime_data=false`,
  `python_celery_worker_required=true`, and
  `python_celery_beat_required=true`.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-147 changed no
  UI source or UI-consumed status text.
