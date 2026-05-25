# DIFF-173: Tabbed Normal User Main UI

Status: Locked

## Type

Change-bearing UI-only

## Objective

Convert the IGY6 main web interface into a clean tabbed product dashboard for
normal users while preserving the current Rust-only application API and worker
runtime ownership.

## Baseline Facts

- DIFF-168 added the post-cutover runtime smoke suite.
- DIFF-169 added fresh-clone startup validation.
- DIFF-170 added startup/shutdown/restart lifecycle validation.
- DIFF-171 polished README and active docs.
- DIFF-172 completed the prior UI polish pass.
- Rust-only application API/worker runtime remains claimed.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.

## Allowed Scope

- Update files under `apps/web`.
- Add this DIFF record.
- Add or update tab UI structure, layout, copy, status presentation, empty
  states, and accessibility basics.
- Move diagnostics/debug/runtime/service detail into a lower-priority Advanced
  area.
- Preserve existing API fetch paths and current working behavior.

## Prohibited Scope

- Do not change Rust backend code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-174.
- Do not add fake runtime data or unsupported workflows.
- Do not reintroduce active Python/FastAPI/Celery runtime wording.
- Do not expose developer/internal detail as the default main interface.

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

- Main screen uses tabs instead of a scroll-heavy section list.
- Default tab answers what can be done next, whether the system is ready, and
  whether attention is needed.
- Normal-user tabs use plain language: Home, Add Data, Work, Results, Settings,
  and Advanced.
- Developer/internal detail is hidden from the default experience and available
  only in Advanced or collapsible details.
- Runtime wording stays truthful without making legacy service names primary.
- Existing API-backed workflows remain compatible.

## Result

- Replaced the scroll-heavy main interface with a tabbed dashboard using Home,
  Add Data, Work, Results, Settings, and Advanced tabs.
- Made Home the default normal-user entry point with a readiness summary, one
  clear next-action area, and plain-language attention guidance.
- Moved technical runtime/service details and the route console into Advanced.
- Reworded main-tab status language to use normal-user wording such as System
  ready, Background worker ready, Add data, Processing, and Results.
- Preserved existing API-backed behavior and fetch paths.
- Runtime ownership did not change.

## Verification Result

- `git status --short` showed only DIFF-173 scoped changes.
- `git diff --check` passed.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `python3 scripts/post-cutover-runtime-audit.py` passed.
- `scripts/post-cutover-smoke.sh --check` passed; live API probes were skipped
  because the local API was not running, as designed for non-live check mode.
- `scripts/fresh-clone-startup-check.sh --check` passed.
- `scripts/runtime-lifecycle-check.sh --check` passed.
- Additional UI smoke coverage passed with
  `npm --prefix apps/web run test:ui-smoke`.
