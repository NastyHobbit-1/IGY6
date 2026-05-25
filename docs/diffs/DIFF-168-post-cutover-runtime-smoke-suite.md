# DIFF-168: Post-Cutover Runtime Smoke Suite

Status: Locked

## Type

Change-bearing

## Objective

Create a non-destructive smoke test suite for the Rust-only application API and
worker runtime after the post-cutover audit was locked.

## Baseline Facts

- DIFF-165 archived `services/worker/` to
  `archive/legacy-python/services-worker/`.
- DIFF-166 locked the post-cutover hardening audit.
- The active application API runtime is the Rust gateway.
- The active application worker runtime is the Rust worker daemon.
- Python/FastAPI fallback, Python/Celery worker, and Celery beat are inactive.
- Remaining non-Rust components are expected supporting services: Next.js web,
  PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix.

## Allowed Scope

- Add this DIFF record.
- Add a non-destructive post-cutover runtime smoke script.
- Update current runtime docs and manifest if needed.
- Validate config, Docker Compose, Rust API health expectations, Rust worker
  help/check, route parity, and the post-cutover audit.

## Prohibited Scope

- Do not mutate `.env`.
- Do not touch runtime/private data or anything under `IGY6_DATA_ROOT`.
- Do not run broad worker queues.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-169.
- Do not change runtime ownership.
- Do not perform UI feature work.

## Required Tags

Use `DIFF-168` in the final change summary and any commit or review note.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/post-cutover-smoke.sh`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 scripts/post-cutover-runtime-audit.py`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo run -p igy6-worker -- --help`
- `cargo run -p igy6-worker -- --check`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`

## Completion Criteria

- A post-cutover smoke command exists and is non-destructive by default.
- The smoke command validates the Rust-only runtime posture without starting or
  stopping services.
- Optional live API probing is explicit and does not mutate runtime data.
- Documentation distinguishes runtime checks from archive/history contents.

## Result

DIFF-168 completed. Added `scripts/post-cutover-smoke.sh --check` as the
non-destructive post-cutover runtime smoke suite. The suite validates manifest
posture, post-cutover audit, route parity, Rust cutover checks, Docker Compose
Rust API/worker ownership, Rust worker help/check output, and optional live API
health probes when a stack is already running. Runtime ownership did not change.
