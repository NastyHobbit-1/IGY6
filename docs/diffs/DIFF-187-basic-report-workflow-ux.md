# DIFF-187 Basic Report Workflow UX

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- Public/runtime-safe changes can be promoted to `main` later by explicit instruction.
- This DIFF does not merge, cherry-pick, remove private/dev files, touch `main`,
  push, or start Rust migration work.

## Purpose

Add the smallest honest user-facing report workflow surface after evidence
retrieval is verified. Users should be able to create and render a basic local
metadata report when the backend supports it, without implying unsupported
evidence synthesis or export behavior.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-work-queue-reports/src/lib.rs`
- `services/reports/README.md`

## Current Report Capability Found

- The Rust gateway exposes report routes:
  - `GET /reports`
  - `GET /reports/{report_id}`
  - `POST /reports`
  - `POST /reports/{report_id}/render`
  - `POST /reports/{report_id}/status`
  - `POST /reports/{report_id}/work-item`
- The report creation route creates report metadata records with type/status.
- The report render route writes a local markdown artifact and updates the
  report to `ready`.
- The current renderer is metadata/inventory oriented. It records counts and
  report boundaries; it does not read raw artifact contents, call external
  models, or synthesize a full evidence narrative.
- The normal Results tab lists reports, but report create/render controls were
  only available in Advanced route controls.

## UX/API Changes Made

- Added a normal-user Basic report workflow section to the Results tab.
- The workflow creates report metadata through the existing `POST /reports`
  route and can immediately render the report through the existing
  `POST /reports/{report_id}/render` route.
- The UI shows the report id, type, status, and rendered artifact availability
  returned by the API.
- The UI is intentionally honest about current capability: reports are local
  markdown metadata summaries and do not claim evidence synthesis, raw artifact
  reading, or external model generation.
- The Create report action is disabled until the page has at least one evidence,
  document, or chunk record available.
- Recent report cards now expose report id and whether the report is still
  metadata-only or has a markdown artifact.
- No API or response contract changes were made.

## Report Workflow Verification Result

- Runtime verification used synthetic manual-upload text with token
  `diff187-report-token-1780311777`.
- The synthetic upload produced completed work item `work-18b4f021269fb192`.
- A basic report was created and rendered through the existing API:
  `report-18b4f02145b3978f`.
- The rendered report returned status `ready` and an artifact path.
- The web UI root included the `data-basic-report-workflow` and
  `data-basic-report-status` markers, and the created report id was visible in
  the Results page HTML.
- Browser automation was not available in this run, so verification used the web
  build, live stack probes, API calls, and curl/grep page marker checks.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-187-basic-report-workflow-ux.md`

## Verification Summary

- `git status --short` before work: clean.
- Current branch before work: `dev`.
- HEAD before work: `a05011f Complete DIFF-186 work status recovery UX polish`.
- `dev` was even with `origin/dev` before the DIFF-187 commit.
- `git diff --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env config --quiet`:
  passed.
- Non-secret `IGY6_DATA_ROOT` key presence check: passed.
- `../IGY6_Data` directory existence check: passed.
- `npm --prefix apps/web run build`: passed.
- Rust formatting/tests were not run because no Rust files changed.
- Port conflict check before stack start: no listeners found on 3000, 8000, or
  8765.
- `scripts/run.sh`: started the local stack.
- API live probe: HTTP 200.
- API ready probe: HTTP 200.
- Web root probe: HTTP 200.
- Synthetic manual upload/report workflow check: passed.
- `scripts/stop.sh`: stopped the stack.
- Final port check: no listeners found on 3000, 8000, or 8765.
- Private/dev files remained tracked.
- No merge, cherry-pick, push, `main` work, `.env` edit, Docker Compose edit, or
  broad refactor was performed.

## Final Status

DIFF-187 is complete.
