# DIFF-079: Portable External Data Root

Status: Locked

## Type

Change-bearing

## Objective

Replace Docker named volumes for persistent IGY6 runtime data with configurable
bind mounts rooted at a dedicated external data folder, and update all
documentation, settings metadata, validation, and ignore rules so the project
still runs correctly and the storage model is clear.

## Baseline Facts

- The worktree was clean before this DIFF started.
- No active or in-progress DIFF existed before this DIFF.
- `infra/docker-compose.yml` currently uses named volumes for PostgreSQL,
  Qdrant, Neo4j data/logs, MLflow, and Phoenix.
- API and worker currently mount `../storage` at `/workspace/storage`.
- The API settings editor expects the project repo mounted at
  `/workspace/project` and `.env` at `/workspace/project/.env`.
- Runtime paths inside containers currently use stable paths such as
  `/workspace/storage/artifacts`, `/workspace/storage/exports`, and
  `/workspace/storage/env_backups`.

## Allowed Scope

- `docs/diffs/DIFF-079-portable-external-data-root.md`
- `README.md`
- `.env.example`
- `.gitignore`
- `infra/docker-compose.yml`
- `services/api/app/config.py`
- `services/api/app/settings_env.py`
- `apps/web/src/app/page.tsx` only if Settings UI metadata must display the new
  setting correctly
- `apps/web/src/app/globals.css` only if Settings UI needs small visual support

## Prohibited Scope

- No backend feature behavior changes unrelated to storage paths.
- No database schema changes.
- No migrations.
- No API route changes except settings metadata/validation if required.
- No new dependencies.
- No auth system.
- No automatic Docker restart.
- No automatic data migration from old named volumes.
- No automatic deletion of Docker named volumes.
- No ComfyUI or AI-stack features.
- No broad refactor.
- No unrelated cleanup.

## Required Tags

- Commit message must include `DIFF-079`.
- Final response must identify `DIFF-079`.

## Verification

- `git diff --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `python3 -m compileall services/api services/worker`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run lint` is optional if a lint script exists.

## Completion Criteria

- Persistent service data no longer uses Docker named volumes.
- Persistent service data is bind-mounted under `IGY6_DATA_ROOT`.
- `.env.example` includes `IGY6_DATA_ROOT`.
- Settings backend recognizes and validates `IGY6_DATA_ROOT`.
- Settings UI can display/edit `IGY6_DATA_ROOT` through the existing dry-run and
  apply flow.
- README documents the new external data-folder storage model and migration
  limitation.
- `.gitignore` protects accidental in-repo runtime data and `.env` files.
- Existing container paths for artifacts, exports, env backups, and env file
  remain correct.
- Docker Compose config validation passes.
- No automatic migration or deletion of old Docker named volumes was added.
- No unrelated behavior changed.
- Prohibited scope was avoided.
- Verification results are recorded below.

## Verification Result

- `git diff --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `python3 -m compileall services/api services/worker` passed.
- `npm --prefix apps/web run build` passed.
- `npm --prefix apps/web run lint` was not run because the web package does not
  define a lint script.
- Docker was not started; only Compose configuration validation was required.
