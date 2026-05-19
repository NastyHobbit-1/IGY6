# DIFF-123: Runtime Smoke Test

Status: Locked

## Type

Change-bearing

## Objective

Add practical runtime smoke-test instructions and a safe Bash/WSL-compatible
script for checking the local Docker stack without starting, stopping, deleting,
or modifying runtime data by default.

## Baseline Facts

- DIFF-122 added UI smoke interaction coverage.
- The runtime remains Rust-primary with FastAPI fallback required.
- Docker Compose defines the local stack in `infra/docker-compose.yml`.
- Normal stop commands must not use `down -v`.

## Allowed Scope

- Add `scripts/runtime-smoke.sh`.
- Update README and user guide with runtime smoke-test usage and practical
  troubleshooting.
- Add completion notes and verification results to this DIFF.

## Prohibited Scope

- No backend behavior changes.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No Docker volume deletion.
- No `down -v` in normal docs or scripts.
- No runtime/private data commits.
- No secrets.
- No arbitrary shell/user-provided argv execution.
- No unrelated cleanup or refactor.
- No locked DIFF edits.

## Required Script Behavior

- Bash/WSL compatible.
- Uses `set -Eeuo pipefail`.
- Default mode checks an already-running stack only.
- Does not start or stop unless explicit flags are passed.
- Checks Docker Compose config validity.
- Checks expected services when the stack is up.
- Checks `http://127.0.0.1:8000/health/live`.
- Checks `http://127.0.0.1:8000/health/ready` when available.
- Checks `http://127.0.0.1:3000`.
- Prints clear PASS/FAIL lines.
- On failure, prints next diagnostic commands for web/API logs.
- Supports `--check`, `--start`, `--stop`, `--detached`, and `--help`.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/runtime-smoke.sh`
- `scripts/runtime-smoke.sh --help`
- `scripts/runtime-smoke.sh --check` may fail clearly if the stack is not
  running and must not create side effects
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `npm --prefix apps/web test`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Notes

- Added `scripts/runtime-smoke.sh`.
- Default `--check` mode validates an already-running stack only.
- Added explicit `--start`, `--stop`, `--detached`, and `--help` flags.
- `--stop` uses `docker compose down` and the script never uses `down -v`.
- The script checks Docker Compose config, expected running services, API live,
  API ready, and web UI response, then prints PASS/FAIL lines with diagnostic
  log commands on failure.
- Updated README and user guide with runtime smoke instructions, long Docker
  commands, WSL alias references, empty `ps` meaning, `127.0.0.1:3000` refused
  meaning, and Phoenix `GET / 200 OK` log context.

## Verification Results

- Passed: `git status --short`
- Passed: `git diff --check`
- Passed: `bash -n scripts/runtime-smoke.sh`
- Passed after executable mode fix: `scripts/runtime-smoke.sh --help`
- Expected clear failure, no side effects: `scripts/runtime-smoke.sh --check`
  - Docker and curl were available.
  - Docker Compose config was valid.
  - No compose services were running.
  - API live, API ready, and web UI did not respond.
  - Script printed next diagnostic commands for API and web logs.
- Passed: `npm --prefix apps/web run build`
- Passed: `npm --prefix apps/web run test:ui-smoke`
- Passed: `npm --prefix apps/web test`
- Passed: `python3 scripts/rust-route-parity.py --check`
  - `fastapi=91`
  - `rust_native=64`
  - `web_used=45`
  - `missing_from_rust=30`
  - `web_requires_fallback=0`
- Passed: `scripts/rust-cutover.sh --check`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
