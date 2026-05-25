# DIFF-174: Simple Runtime Command Scripts

Status: Locked

## Type

Change-bearing implementation

## Objective

Add simple safe wrapper scripts for starting, stopping, restarting, and
inspecting the IGY6 Docker Compose runtime without changing runtime ownership.

## Baseline Facts

- DIFF-173 added the tabbed normal-user main UI.
- Rust-only application API/worker runtime remains claimed.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.
- Existing validation scripts remain the verification path; this DIFF adds
  convenience runtime wrapper scripts only.

## Allowed Scope

- Add or update:
  - `scripts/run.sh`
  - `scripts/stop.sh`
  - `scripts/restart.sh`
  - `scripts/status.sh`
  - `README.md`
  - this DIFF record
- Make scripts executable.
- Preserve underlying Docker Compose command transparency.

## Prohibited Scope

- Do not change Rust backend code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-175.
- Do not perform UI feature work.
- Do not change runtime ownership.
- Do not use destructive Docker commands.

## Required Script Behavior

- `scripts/run.sh` runs:
  `docker compose -f infra/docker-compose.yml --env-file .env up --build`
- `scripts/stop.sh` runs:
  `docker compose -f infra/docker-compose.yml --env-file .env down`
- `scripts/restart.sh` runs `down` and then `up --build`.
- `scripts/status.sh` runs:
  `docker compose -f infra/docker-compose.yml --env-file .env ps`
- Scripts resolve the repository root from script location.
- Scripts use `set -Eeuo pipefail`.
- Scripts require `infra/docker-compose.yml`, `.env`, and Docker Compose.
- Scripts do not create `.env`, mutate `.env`, remove volumes, or delete
  runtime data.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/run.sh`
- `bash -n scripts/stop.sh`
- `bash -n scripts/restart.sh`
- `bash -n scripts/status.sh`
- `scripts/status.sh` or document why unavailable
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`
- `npm --prefix apps/web run build`

## Completion Criteria

- The four simple wrapper scripts exist and are executable.
- Missing `.env` produces a clear message telling the user to copy
  `.env.example` to `.env`.
- README documents the simple commands and the underlying Docker Compose
  commands.
- Verification passes or any environment limitation is documented.

## Result

- Added simple runtime wrappers:
  - `scripts/run.sh`
  - `scripts/stop.sh`
  - `scripts/restart.sh`
  - `scripts/status.sh`
- Simplified `scripts/run.sh` to remove the old detached health/snapshot path
  and run only the required foreground `up --build` command.
- Kept `scripts/stop.sh` on non-volume-removing `down`.
- Added README quick commands while preserving the underlying Docker Compose
  commands for transparency.
- Runtime ownership did not change.

## Verification Result

- `git status --short` showed only DIFF-174 scoped changes.
- `git diff --check` passed.
- `bash -n scripts/run.sh` passed.
- `bash -n scripts/stop.sh` passed.
- `bash -n scripts/restart.sh` passed.
- `bash -n scripts/status.sh` passed.
- `scripts/status.sh` passed with Docker socket access and showed no running
  services.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `python3 scripts/post-cutover-runtime-audit.py` passed.
- `scripts/post-cutover-smoke.sh --check` passed; live API probes were skipped
  because the local API was not running, as designed for non-live check mode.
- `scripts/fresh-clone-startup-check.sh --check` passed.
- `scripts/runtime-lifecycle-check.sh --check` passed.
- `npm --prefix apps/web run build` passed.
