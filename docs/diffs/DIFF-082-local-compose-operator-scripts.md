# DIFF-082: Local Compose Operator Scripts

Status: Locked

## Type

Change-bearing

## Objective

Add safe Bash operator scripts for starting, stopping, and recovering the local
IGY6 Docker Compose stack without requiring the user to remember full Compose
commands.

## Baseline Facts

- `DIFF-081` already exists and is locked for manual text ingest/vector
  population.
- The worktree was clean before this DIFF started.
- The Compose file is `infra/docker-compose.yml`.
- The normal run command is
  `docker compose -f infra/docker-compose.yml --env-file .env up --build`.
- The normal stop command is
  `docker compose -f infra/docker-compose.yml --env-file .env down`.
- Published service ports are bound to `127.0.0.1` in Compose.
- Runtime/private/persistent data belongs under `IGY6_DATA_ROOT`, defaulted in
  `.env.example` as `../IGY6_Data`.
- No `scripts/` directory existed before this DIFF.

## Allowed Scope

- `docs/diffs/DIFF-082-local-compose-operator-scripts.md`
- New scripts under `scripts/`
- `README.md` updates documenting the new scripts
- `.gitignore` update only if needed to prevent operator runtime state from
  being committed
- Narrow shell syntax or smoke checks

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose service rewrites.
- No port changes.
- No `.env` changes.
- No database migrations.
- No ingestion changes.
- No API behavior changes.
- No UI changes.
- No broad refactor.
- No dependency additions.
- No destructive Docker cleanup.
- No volume deletion.
- No automatic git checkout, reset, or stash.

## Required Tags

- Commit message must include `DIFF-082`.
- Final response must identify `DIFF-082`.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/run.sh scripts/stop.sh scripts/run-last-healthy-config.sh`
- `scripts/run.sh --help`
- `scripts/stop.sh --help`
- `scripts/run-last-healthy-config.sh --help`
- If safe, run detached start, direct health checks, run-last-healthy-config,
  and stop.

## Completion Criteria

- `scripts/run.sh` starts the stack with the existing Compose file and `.env`.
- `scripts/run.sh --detached` starts detached, performs health checks, and
  writes a safe last-healthy snapshot only if health checks pass.
- `scripts/stop.sh` stops the stack without deleting volumes, images, or
  persistent data.
- `scripts/run-last-healthy-config.sh` reads the last healthy snapshot, displays
  safe metadata, warns on commit/worktree drift, and starts from saved safe
  Compose/env metadata only if paths are valid.
- Scripts work from outside the repo by resolving repo root from script
  location.
- Scripts do not store secrets or raw `.env` contents.
- README documents the operator scripts while preserving existing manual Docker
  commands.
- Verification results are recorded below before locking.

## Verification Result

- `git status --short` was run before edits and showed a clean worktree.
- `git status --short` after edits showed only DIFF-082 scoped files.
- `git diff --check` passed.
- `bash -n scripts/run.sh scripts/stop.sh scripts/run-last-healthy-config.sh`
  passed.
- `bash -n scripts/lib/igy6-ops.sh` passed.
- `scripts/run.sh --help` passed.
- `scripts/stop.sh --help` passed.
- `scripts/run-last-healthy-config.sh --help` passed.
- `/home/nasty/projects/IGY6/scripts/run.sh --help` from `/tmp` passed.
- `/home/nasty/projects/IGY6/scripts/stop.sh --help` from `/tmp` passed.
- `scripts/run-last-healthy-config.sh` refused cleanly when no snapshot existed
  at `/home/nasty/projects/IGY6_Data/ops/last-healthy.json`.
- `scripts/run.sh --detached` initially failed inside the sandbox because Docker
  socket access was denied; rerunning with approved Docker access succeeded.
- `scripts/run.sh --detached` passed, health checks passed, and wrote
  `/home/nasty/projects/IGY6_Data/ops/last-healthy.json`.
- `curl http://127.0.0.1:8000/health/ready` returned healthy JSON while the
  stack was running.
- Snapshot inspection confirmed safe metadata only; grep for secret-like keys
  and placeholder secret values returned no matches.
- `scripts/run-last-healthy-config.sh` passed with the snapshot, warned about
  the dirty working tree, and started the stack detached without changing git
  state.
- `scripts/stop.sh` passed and stopped the stack without deleting volumes,
  images, or runtime data.
- `/home/nasty/projects/IGY6/scripts/run.sh --detached` from `/tmp` passed and
  wrote the snapshot again.
- `/home/nasty/projects/IGY6/scripts/stop.sh` from `/tmp` passed and stopped the
  stack again without deleting volumes, images, or runtime data.
