# DIFF-189 Source and Evidence History Detail UX

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- Public/runtime-safe changes can be promoted to `main` later by explicit
  instruction.
- This DIFF does not merge, cherry-pick, remove private/dev files, touch
  `main`, push, edit `.env`, or start Rust migration work.

## Purpose

Improve the normal user view for inspecting what was uploaded and what evidence
came out of it after manual upload, work status, and retrieval are verified.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `scripts/e2e-manual-upload-smoke.py`

## Current Source/Evidence History Behavior

- The UI already lists recent collection runs, raw artifacts, documents, chunks,
  and evidence items.
- Manual upload success shows the created source, collection run, work item, and
  artifact identifiers.
- The Work tab shows status and related identifiers where work item payload data
  is available.
- Results shows the individual record categories but does not correlate recent
  source/run/artifact/document/chunk/evidence lineage in one normal-user view.
- API read routes already expose enough metadata for a UI-only history detail
  improvement.

## UX/API Changes Made

- Added a Results-tab `Source and evidence history` panel.
- The panel correlates recent collection runs with source, raw artifact,
  normalized document, chunk, and evidence identifiers.
- The history panel shows counts and first linked identifiers without displaying
  raw uploaded text or artifact file contents.
- Empty state remains honest when no lineage records are available.
- No API, Rust gateway, database schema, or response contract changes were made.

## Source/Evidence Detail Verification Result

- Live web root returned HTTP 200.
- The Results page contained the `data-source-evidence-history` marker.
- At least one `data-source-history-item` was present, confirming visible
  source/evidence history detail from existing runtime records.
- Retrieval regression returned HTTP 200 with `items=5` and `status=retrieved`.
- Browser automation was not available in this run, so verification used the web
  build, live stack probes, API calls, and curl/grep page marker checks.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`

## Verification Summary

- `git status --short` before DIFF-189: clean after DIFF-188 commit.
- `docker compose -f infra/docker-compose.yml --env-file .env config --quiet`:
  passed.
- Non-secret `IGY6_DATA_ROOT` key presence check: passed.
- `../IGY6_Data` directory existence check: passed.
- `npm --prefix apps/web run build`: passed.
- Rust formatting/tests were not run because no Rust files changed in DIFF-189.
- Port conflict check before stack start: no listeners found on 3000, 8000, or
  8765.
- `scripts/run.sh`: started the local stack.
- API ready probe: HTTP 200.
- Web root probe: HTTP 200.
- Source/evidence history UI marker check: passed.
- Results/retrieval regression: passed.
- `scripts/stop.sh`: stopped the stack.
- Final port check: no listeners found on 3000, 8000, or 8765.
- Private/dev files remained tracked.
- No merge, cherry-pick, push, `main` work, `.env` edit, Docker Compose edit,
  API contract change, runtime data dump, or broad refactor was performed.

## Final Status

DIFF-189 is complete.
