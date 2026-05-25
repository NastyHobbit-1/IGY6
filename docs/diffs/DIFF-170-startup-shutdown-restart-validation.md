# DIFF-170: Startup Shutdown Restart Validation

Status: Locked

## Type

Change-bearing

## Objective

Add a non-destructive startup, shutdown, and restart validation process for the
post-cutover Docker Compose runtime.

## Baseline Facts

- DIFF-168 added `scripts/post-cutover-smoke.sh --check`.
- DIFF-169 added `scripts/fresh-clone-startup-check.sh --check`.
- The active application API runtime is the Rust gateway.
- The active application worker runtime is the Rust worker daemon.
- Python/FastAPI fallback, Python/Celery worker, and Celery beat are inactive.
- Runtime/private data remains outside the repo under `IGY6_DATA_ROOT`.

## Allowed Scope

- Add this DIFF record.
- Add a non-destructive runtime lifecycle check script.
- Update current runtime docs, the project completion plan, and manifest if
  needed.
- Validate Compose config, planned startup command, planned shutdown command,
  restart command shape, service names, Rust API/worker ownership, and rollback
  posture.

## Prohibited Scope

- Do not mutate `.env`.
- Do not touch runtime/private data by default.
- Do not start or stop Docker Compose by default.
- Do not run broad worker queues.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-171.
- Do not change runtime ownership.
- Do not perform UI feature work.

## Required Tags

Use `DIFF-170` in the final change summary and any commit or review note.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/post-cutover-smoke.sh`
- `bash -n scripts/fresh-clone-startup-check.sh`
- `bash -n scripts/runtime-lifecycle-check.sh`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
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

- A lifecycle validation command is documented.
- The command is non-destructive by default.
- The command validates startup, shutdown, and restart command shapes without
  starting or stopping services.
- The command validates Rust-only application API/worker ownership in Compose.
- Documentation distinguishes the lifecycle check from a live restart test.

## Result

- Added `scripts/runtime-lifecycle-check.sh --check` as the DIFF-170 lifecycle
  validator.
- The validator checks Compose config, required service names, absence of active
  legacy API/beat services, Rust gateway and Rust worker daemon ownership,
  documented start/shutdown/restart command shapes, non-volume-removing
  shutdown posture, rollback posture, the post-cutover runtime audit, and the
  post-cutover smoke suite.
- The validator does not start, stop, restart, or mutate Docker Compose services
  by default.
- Rust-only application API/worker runtime ownership remains claimed. Non-Rust
  components remain the expected web and infrastructure services.
