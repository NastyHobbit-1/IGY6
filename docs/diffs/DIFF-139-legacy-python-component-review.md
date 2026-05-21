# DIFF-139: Legacy Python Component Review

Status: Locked

## Type

Change-bearing legacy Python archive and runtime documentation update.

## Objective

Review legacy Python components after DIFF-138 removed FastAPI fallback wiring.

Decision:

- Archive `services/api/` to `archive/legacy-python/services-api/`.
- Retain `services/worker/` because Python/Celery worker and beat remain active
  runtime components.
- Do not claim full Rust-only repository or runtime operation.

## Baseline Facts

- DIFF-138 is complete and locked.
- `infra/docker-compose.yml` no longer defines or wires `legacy-api`.
- Rust route parity records 91 archived FastAPI routes, 94 Rust-native routes,
  45 web-used routes, 0 FastAPI routes missing from Rust, and 0 web-used routes
  requiring fallback.
- `services/worker/` remains referenced by Docker Compose for both `worker` and
  `beat` services.
- `crates/igy6-worker` is not a verified replacement for live Python/Celery
  worker execution.

## Allowed Scope

- `services/api/` move to `archive/legacy-python/services-api/`
- `archive/legacy-python/README.md`
- `configs/rust-cutover-manifest.json`
- `configs/legacy-fastapi-route-classification.json`
- `scripts/rust-route-parity.py`
- `scripts/runtime-smoke.sh`
- `README.md`
- `docs/rust-migration/`
- `docs/architecture.md`
- `docs/operations.md`
- `docs/user-guide.md`
- `infra/migrations/README.md`
- This DIFF document.

## Prohibited Scope

- No DIFF-140 or later work until DIFF-139 is locked.
- No deletion of archived Python code.
- No archive or deletion of `services/worker/`.
- No Python/Celery worker parity claims.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, redesign, data model changes,
  migration changes, or dependency changes.

## Required Tags

Use `DIFF-139` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious archive or parity behavior.

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
- `npm --prefix apps/web run build`

## Completion Criteria

- `services/api/` is archived only after confirming no active runtime path needs
  it.
- `services/worker/` is retained if Python/Celery worker and beat execution
  remains active.
- Manifest, README, Docker Compose-facing docs, rollback docs, and migration
  docs state the runtime honestly.
- Verification passes before DIFF-139 is locked.

## Completion Notes

DIFF-139 archives the tracked legacy FastAPI API source and retains active
Python/Celery worker execution.

Archive decision:

- `services/api/` moved to `archive/legacy-python/services-api/`.
- Route parity still inventories the archived FastAPI source for audit and
  regression checks.
- `legacy-api` remains absent from Docker Compose after DIFF-138.

Retain decision:

- `services/worker/` remains active because Docker Compose still builds it for
  both `worker` and `beat`.
- `worker` runs `celery -A app.celery_app:celery_app worker --loglevel=INFO`.
- `beat` runs `celery -A app.celery_app:celery_app beat --loglevel=INFO`.
- Rust worker code is planning-only and does not replace live Python/Celery
  execution, database writes, audit writes, or Qdrant work.

Full Rust-only repository or runtime operation is not claimed.

## Verification Results

- `git status --short` inspected before DIFF-139 and before lock.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- `python3 -m py_compile scripts/rust-route-parity.py` passed.
- `bash -n scripts/runtime-smoke.sh` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed active `worker` and `beat` services built from
  `services/worker`, with no `legacy-api` service.
- `npm --prefix apps/web run build` passed.
