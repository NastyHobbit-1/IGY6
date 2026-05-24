# DIFF-165: Archive Python Worker Final Rust Runtime Audit

Status: Locked

## Type

Change-bearing

## Objective

Archive the legacy Python/Celery worker source if no active production runtime
path depends on it, update the Rust cutover manifest and runtime docs, and lock
the final Rust-only runtime audit.

## Baseline Facts

- DIFF-164 moved the base Docker Compose `worker` service to the Rust worker
  daemon built from `crates/igy6-worker/Dockerfile`.
- Base Docker Compose no longer defines a Python/Celery `worker` service.
- Base Docker Compose no longer defines a Celery `beat` service.
- `services/worker/` remains in the repository only for rollback/archive review
  pending this DIFF.
- The legacy FastAPI API source was already archived under
  `archive/legacy-python/services-api`.

## Decision

Decision A: legacy Python worker can be archived.

Base Docker Compose no longer references `services/worker/`, no Python/Celery
worker service remains active, and Celery `beat` has been retired because no
repo-defined beat schedule or periodic task registration was found. The archived
worker source remains available at `archive/legacy-python/services-worker/` and
through git history for rollback analysis.

## Allowed Scope

- Move `services/worker/` to `archive/legacy-python/services-worker/` with
  history-preserving `git mv`.
- Update `configs/rust-cutover-manifest.json`.
- Update `configs/legacy-fastapi-route-classification.json` only for the
  final Rust-only claim gate.
- Update `README.md`.
- Update `docs/runtime/PROCESSING_STATUS.md`.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Update `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`.
- Update existing final audit/archive docs when needed to keep the runtime
  record accurate.
- Update `crates/igy6-worker` status/help text only where required to stop
  DIFF-164's deferred Rust-only claim from contradicting DIFF-165.
- Update `scripts/rust-cutover.sh` only where required to validate the final
  `rust-only-application-runtime` manifest target.
- Run repository searches proving no active Compose/runtime reference to
  `services/worker/` remains.

## Prohibited Scope

- Do not delete the legacy worker source outright.
- Do not archive or edit DIFF governance entrypoints except this DIFF file.
- Do not mutate `.env`.
- Do not touch production/private runtime data or anything under
  `IGY6_DATA_ROOT`.
- Do not start DIFF-166.
- Do not edit locked DIFFs.
- Do not claim Rust-only runtime if any Python/Celery worker or beat service is
  still active in the production Compose/runtime path.

## Required Tags

Use `DIFF-165` in the final change summary and any commit or review note.

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
- `cargo run -p igy6-worker -- --dry-run --once`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `docker compose -f infra/docker-compose.yml -f infra/docker-compose.rust-worker-canary.yml --env-file .env.example --profile rust-worker-canary config`
- `npm --prefix apps/web run build`
- Repository search proving no active Compose/runtime reference to
  `services/worker/` remains.

## Completion Criteria

- Decision A or B is recorded.
- If Decision A, `services/worker/` is archived under the established legacy
  Python archive path and rollback instructions point to the archive and git
  history.
- If Decision B, no archive move occurs and the exact runtime dependency/blocker
  plus next DIFF is documented.
- Runtime docs and manifest distinguish active Rust-only runtime from archived
  legacy Python history.
- Required verification is run or any blocker is recorded.

## Result

Decision A completed. `services/worker/` was archived to
`archive/legacy-python/services-worker/`. Base Docker Compose and the Rust
worker canary override use the Rust worker image, no active Python/Celery worker
or Celery `beat` service remains, and Rust-only application runtime is claimed
for the API and worker path.

## Out Of Scope Follow-Up

No DIFF-166 scope is opened here. Future work may harden non-Rust supporting
services, but this DIFF is limited to final worker archive and runtime audit.
