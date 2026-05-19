# DIFF-125: Worker Processing Verification

Status: Locked

## Type

Change-bearing

## Objective

Verify and document the processing pipeline after manual upload:
raw artifact -> normalized document -> chunks -> evidence -> vector memory ->
graph memory when applicable.

## Baseline Facts

- Manual upload creates raw artifact metadata and queued normalization work.
- Python/Celery worker execution remains active for runtime processing.
- Rust worker crate currently plans deterministic UTF-8 processing behavior; it
  does not replace the live Celery worker.
- Rust gateway dispatch records a bounded non-executing dispatch marker and does
  not invoke Celery directly.

## Allowed Scope

- Add processing status diagnostics script.
- Add or update runtime processing docs.
- Update README and user guide with processing diagnostics.
- Fix narrow worker/processing bugs only if discovered and safely scoped.
- Add completion notes and verification results to this DIFF.

## Prohibited Scope

- No broad worker architecture rewrite.
- No migration of all worker behavior.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No unsafe deletion.
- No Docker volume deletion.
- No secrets or runtime/private data commits.
- No arbitrary shell/user-provided argv execution.
- No approval bypass.
- No locked DIFF edits.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m py_compile` for added Python scripts
- `npm --prefix apps/web run build` if docs/UI changed
- `npm --prefix apps/web test` if web changed
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Cargo checks only if Rust files change.

## Completion Notes

- Added `scripts/processing-status-smoke.py` as a non-mutating processing
  diagnostics check for an already-running stack. It verifies Docker Compose
  config, expected service state, Redis, Postgres, API readiness, work item
  status counts, and vector chunk visibility without dispatching or executing
  work.
- Added `docs/runtime/PROCESSING_STATUS.md` to document the current runtime
  pipeline, including the honest split between Python/Celery live processing,
  Rust deterministic worker planning, and safe-limited Rust dispatch metadata.
- Updated `README.md` and `docs/user-guide.md` with processing diagnostics,
  status interpretation, and troubleshooting.
- No Rust files changed. No worker architecture was rewritten. No backend route
  was removed. FastAPI fallback remains documented where applicable.

## Verification Results

- `git diff --check`: passed
- `python3 -m py_compile scripts/processing-status-smoke.py`: passed
- `npm --prefix apps/web run build`: passed
- `npm --prefix apps/web test`: passed
- `python3 scripts/rust-route-parity.py --check`: passed
  (`fastapi=91 rust_native=64 web_used=45 missing_from_rust=30 web_requires_fallback=0`)
- `scripts/rust-cutover.sh --check`: passed
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`: passed
- Cargo checks were not run separately for this DIFF because no Rust files
  changed; `scripts/rust-cutover.sh --check` still ran the established cutover
  Rust checks.
