# DIFF-077: Settings Page Env Dry Run

Status: Locked

## Type

Change-bearing

## Objective

Add a local-only Settings page/section that can edit all allowed `.env` keys
through a backend-enforced verify-dry-run-before-save workflow, with backups,
audit events, secret masking, atomic writes, and clear restart warnings.

## Baseline Facts

- The worktree was clean before this DIFF started.
- No active or in-progress DIFF existed before this DIFF.
- `.env.example` defines the current local Docker Compose configuration keys.
- `.env` is ignored by git and is intended for local runtime configuration.
- The API container currently receives environment values but does not have a
  controlled project `.env` file mount.
- Existing `audit_events` can record settings changes without a migration.
- The current UI already uses a dark AI-console shell and same-origin API proxy
  route for chat retrieval preview.

## Allowed Scope

- `docs/diffs/DIFF-077-settings-page-env-dry-run.md`
- `README.md`
- `.env.example`
- `infra/docker-compose.yml`
- `services/api/app/main.py`
- `services/api/app/config.py`
- `services/api/app/settings_env.py`
- `services/api/app/audit.py` only if needed
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/api/settings/*`
- Small UI-only component files under `apps/web/src/app/components/` only if
  needed
- Tests only if an existing test structure exists or minimal local unit tests
  can be added without new dependencies

## Prohibited Scope

- No automatic Docker restart.
- No automatic container recreate.
- No background daemon.
- No external services.
- No cloud sync.
- No auth system.
- No new dependencies unless absolutely unavoidable and justified here.
- No database migration unless absolutely required; prefer existing
  `audit_events`.
- No secret values committed to git.
- No arbitrary file editing outside the configured `.env` path.
- No editing `.env.example` with real secrets.
- No removing existing UI behavior.
- No changing unrelated backend behavior.
- No broad refactor.
- No unrelated cleanup.
- No ComfyUI, model manager, image generation, or AI-stack features.

## Required Tags

- Commit message must include `DIFF-077`.
- Final response must identify `DIFF-077`.

## Verification

- `git diff --check`
- `python3 -m compileall services/api services/worker`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- If tests are added, run only relevant tests.

## Completion Criteria

- Settings UI can load sanitized `.env`.
- User can edit every allowlisted `.env` key.
- Secret values are masked by default.
- Verify Dry Run validates proposed config without writing.
- Save is disabled until dry-run passes.
- Save requires the matching dry-run token/hash.
- Backend creates backup before writing.
- Backend writes `.env` atomically.
- Backend records audit event without secret values.
- UI clearly warns restart/recreate may be required.
- README documents settings usage and safety limitations.
- No automatic restart was added.
- No unrelated behavior changed.
- Prohibited scope was avoided.
- Verification results are recorded below.

## Verification Result

- `git diff --check` passed.
- `python3 -m compileall services/api services/worker` passed.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- No full Docker stack start was run because the DIFF did not require it and the
  requested static/build/config checks passed.
