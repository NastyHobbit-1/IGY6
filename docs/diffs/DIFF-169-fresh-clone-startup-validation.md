# DIFF-169: Fresh-Clone Startup Validation

Status: Locked

## Type

Change-bearing

## Objective

Add a non-destructive fresh-clone startup validation process for the
post-cutover Rust-only application API and worker runtime.

## Baseline Facts

- DIFF-168 added `scripts/post-cutover-smoke.sh --check`.
- The active application API runtime is the Rust gateway.
- The active application worker runtime is the Rust worker daemon.
- Python/FastAPI fallback, Python/Celery worker, and Celery beat are inactive.
- Runtime/private data remains outside the repo under `IGY6_DATA_ROOT`.

## Allowed Scope

- Add this DIFF record.
- Add a non-destructive fresh-clone startup check script.
- Update current runtime docs, the project completion plan, and manifest if
  needed.
- Validate required tools, `.env.example` completeness, Docker Compose config,
  Rust worker command health, post-cutover audit, route parity, and the
  post-cutover smoke suite.

## Prohibited Scope

- Do not mutate `.env`.
- Do not touch runtime/private data or anything under `IGY6_DATA_ROOT`.
- Do not run broad worker queues.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-170.
- Do not change runtime ownership.
- Do not perform UI feature work.

## Required Tags

Use `DIFF-169` in the final change summary and any commit or review note.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/post-cutover-smoke.sh`
- `bash -n scripts/fresh-clone-startup-check.sh`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
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

- A fresh-clone startup validation command is documented.
- The command is non-destructive by default.
- The command verifies that tracked repo/config inputs are sufficient for
  post-cutover startup checks.
- Documentation distinguishes the check from a live startup or end-to-end
  product journey.

## Result

DIFF-169 completed. Added `scripts/fresh-clone-startup-check.sh --check` as a
non-destructive fresh-clone startup readiness check. The check validates
required tools, required tracked files, `.env.example` coverage for Compose and
runtime keys, Docker Compose config, manifest posture, post-cutover runtime
audit, route parity, Rust worker help/check output, and the DIFF-168
post-cutover smoke suite. Runtime ownership did not change.
