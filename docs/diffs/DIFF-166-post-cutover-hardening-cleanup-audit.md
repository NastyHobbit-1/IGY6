# DIFF-166: Post-Cutover Hardening Cleanup Audit

Status: Locked

## Type

Change-bearing

## Objective

Audit and harden the repository after DIFF-165 locked the Rust-only application
API and worker runtime. Clean stale current-runtime claims, improve
post-cutover documentation, and keep rollback/smoke-test guidance accurate
without changing runtime ownership.

## Baseline Facts

- DIFF-165 archived `services/worker/` to
  `archive/legacy-python/services-worker/`.
- Python/Celery worker is no longer active in base Docker Compose.
- Celery `beat` is no longer active in base Docker Compose.
- Rust-only runtime is claimed for the application API and worker path.
- Remaining non-Rust components are expected supporting components: Next.js web,
  PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix.

## Allowed Scope

- Update `README.md` if stale.
- Update `docs/runtime/PROCESSING_STATUS.md` if stale.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` if stale.
- Update `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md` if stale.
- Update `configs/rust-cutover-manifest.json` if stale.
- Clean stale references to active Python/Celery worker only if clearly wrong.
- Add non-destructive verification scripts/checks if useful.
- Improve post-cutover docs, run commands, rollback notes, and smoke-test
  instructions.

## Prohibited Scope

- Do not edit locked DIFFs.
- Do not mutate `.env`.
- Do not touch runtime/private data or anything under `IGY6_DATA_ROOT`.
- Do not remove `archive/legacy-python/`.
- Do not remove DIFF governance docs.
- Do not rewrite unrelated UI/product behavior.
- Do not start broad feature work.
- Do not claim non-Rust infrastructure is rewritten in Rust.
- Do not reopen Rust migration unless a real blocker is found.

## Required Tags

Use `DIFF-166` in the final change summary and any commit or review note.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
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
- Repository search for stale active Python/Celery runtime claims.

## Completion Criteria

- Current-runtime docs and manifest agree that the application API and worker
  runtime are Rust-only.
- Archived Python references remain clearly historical or rollback-only.
- No active Compose/runtime reference to `services/worker/` or Python/Celery
  worker remains outside archive/history.
- Required verification is run or any blocker is recorded.

## Result

DIFF-166 completed. Runtime ownership did not change. The audit added
`scripts/post-cutover-runtime-audit.py`, updated post-cutover docs and manifest
status, and confirmed stale active Python/Celery runtime claims are absent from
the current active runtime surface. Archived Python references remain only for
history, route parity, and rollback review.

## Out Of Scope Follow-Up

No DIFF-167 scope is opened here. Future work should be limited to specific
post-cutover hardening tasks found by this audit.
