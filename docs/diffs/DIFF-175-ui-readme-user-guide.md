# DIFF-175: UI README User Guide

Status: Locked

## Type

Documentation only

## Objective

Create a complete user-facing guide for the current tabbed IGY6 web UI.

## Baseline Facts

- DIFF-173 changed the main UI to a tabbed normal-user dashboard.
- DIFF-174 added simple run, stop, restart, and status scripts.
- Rust-only application API/worker runtime remains claimed.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.

## Allowed Scope

- Add `docs/ui/README.md`.
- Update root `README.md` to link to the UI guide.
- Add this DIFF record.
- Inspect UI source to keep documentation aligned with visible UI.

## Prohibited Scope

- Do not change UI code.
- Do not change Rust backend code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-176.
- Do not claim unsupported UI capabilities.
- Do not describe fake features as working.

## Verification

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`

## Completion Criteria

- `docs/ui/README.md` exists.
- The guide documents Home, Add Data, Work, Results, Settings, and Advanced.
- The guide explains visible interface concepts and common empty/error states.
- The guide includes start/status/open/stop commands.
- The guide includes workflows for startup readiness, adding data, checking
  processing, viewing results, and using Advanced only when needed.
- The guide documents safety rules and current limitations without claiming
  unsupported capabilities.

## Result

- Added `docs/ui/README.md` as the user-facing tabbed UI guide.
- Documented Home, Add Data, Work, Results, Settings, and Advanced.
- Added interface-item explanations for readiness, primary actions, activity,
  uploads/sources, processing, evidence/results/reports, settings/safety, and
  diagnostics.
- Added workflows for startup readiness, adding data, checking processing,
  viewing results, and using Advanced only when needed.
- Documented troubleshooting, safety/data rules, and current limitations.
- Linked the guide from the root README.
- Runtime ownership did not change.

## Verification Result

- `git status --short` showed only DIFF-175 scoped changes.
- `git diff --check` passed.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `python3 scripts/post-cutover-runtime-audit.py` passed.
- `scripts/post-cutover-smoke.sh --check` passed; live API probes were skipped
  because the local API was not running, as designed for non-live check mode.
- `scripts/fresh-clone-startup-check.sh --check` passed.
- `scripts/runtime-lifecycle-check.sh --check` passed.
