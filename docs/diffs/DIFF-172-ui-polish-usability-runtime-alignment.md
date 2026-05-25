# DIFF-172: UI Polish Usability Runtime Alignment

Status: Locked

## Type

Change-bearing UI-only

## Objective

Polish the IGY6 web UI so it is cleaner, easier to scan, less overwhelming,
and aligned with the current Rust-only application API/worker runtime.

## Baseline Facts

- DIFF-168 added the post-cutover runtime smoke suite.
- DIFF-169 added fresh-clone startup validation.
- DIFF-170 added startup/shutdown/restart lifecycle validation.
- DIFF-171 polished README and active docs and is locked.
- Rust-only application API/worker runtime remains claimed.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.

## Allowed Scope

- Update files under `apps/web`.
- Add this DIFF record.
- Improve visual hierarchy, spacing, grouping, copy, badges, navigation, empty
  states, error states, responsive behavior, and accessibility basics.
- Ensure visible UI status labels match the current Rust-only runtime posture.

## Prohibited Scope

- Do not change Rust backend code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-173.
- Do not add fake demo data.
- Do not add broad new workflows.
- Do not claim unsupported capabilities.
- Do not reintroduce active Python/FastAPI/Celery runtime wording.

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

- The first screen is readable at a glance.
- Runtime status is visible in one obvious area.
- Main workflow actions are visible in one obvious area.
- Advanced/debug details remain lower priority.
- UI wording reflects Rust API active, Rust worker active, FastAPI fallback
  inactive/archived, Python/Celery worker inactive/archived, and beat inactive.
- Existing API fetch paths remain compatible with current Rust routes.

## Result

- Added a first-screen runtime posture strip showing Rust API active, Rust
  worker active, FastAPI fallback inactive/archived, Python/Celery worker
  inactive/archived, and Celery beat inactive.
- Replaced the dense first-screen quickstart cards with three clear primary
  workflow cards: add authorized data, check processing, and ask with evidence.
- Clarified Work & Processing text around Rust API dispatch and Rust worker
  ownership without changing route behavior.
- Lowered advanced route console prominence and updated UI smoke coverage for
  the current Rust-only runtime wording.
- Runtime ownership did not change.
