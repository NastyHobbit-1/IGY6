# DIFF-190 Operator Smoke Verification Bundle

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

Consolidate the proven manual verification flow from DIFF-182 through DIFF-189
into one safe, repeatable operator command bundle.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/ui/README.md`
- `docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md`
- `scripts/e2e-manual-upload-smoke.py`
- `scripts/runtime-smoke.sh`
- `scripts/post-cutover-smoke.sh`
- `scripts/processing-status-smoke.py`
- `apps/web/scripts/ui-smoke.mjs`
- `docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`
- `docs/diffs/DIFF-187-basic-report-workflow-ux.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`

## Bundle/Docs Changes Made

- Added `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`.
- The bundle gives an operator-focused command sequence for:
  - git/branch status;
  - Docker Compose config validation;
  - non-secret `IGY6_DATA_ROOT` key and directory checks;
  - web build;
  - port conflict check;
  - stack startup;
  - API live/ready and web probes;
  - synthetic manual-upload smoke with
    `scripts/e2e-manual-upload-smoke.py --run`;
  - work/evidence/retrieval/report/feedback/source-history UI marker checks;
  - explicit retrieval-preview verification;
  - stack shutdown;
  - final port check.
- The bundle uses synthetic text only and documents that raw runtime data,
  `.env` contents, secrets, and artifact files must not be printed.
- No runtime code was changed.

## Commands Verified

- `python3 scripts/e2e-manual-upload-smoke.py --help`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env config --quiet`:
  passed.
- Non-secret `IGY6_DATA_ROOT` key presence check: passed.
- `../IGY6_Data` directory existence check: passed.
- `ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true`: no checked
  listeners.
- Marker-name cross-check with `rg`: passed for all documented UI markers.
- `npm --prefix apps/web run build`: passed.

## Runtime Verification Result

- DIFF-190 did not change runtime/UI/API code, so a new mutating manual-upload
  runtime run was not required for the docs-only bundle.
- The documented runtime flow was validated against the already-proven
  DIFF-184 through DIFF-189 sequence and the existing manual-upload smoke helper
  contract.
- The live DIFF-189 run immediately before this docs bundle verified API ready,
  web root, source/evidence history marker, and retrieval preview with
  `items=5` and `status=retrieved`.
- Browser automation was not available; the bundle documents curl/grep marker
  verification as the fallback.

## Files Changed

- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `docs/diffs/DIFF-190-operator-smoke-verification-bundle.md`

## Verification Summary

- `git status --short` before DIFF-190: clean after DIFF-189 commit.
- Existing smoke scripts and docs were inspected.
- The new command bundle was checked for accurate existing script names and UI
  markers.
- No Rust files changed; Rust formatting/tests were not run for DIFF-190.
- Private/dev files remained tracked.
- No merge, cherry-pick, push, `main` work, `.env` edit, Docker Compose edit,
  runtime code change, runtime/private data dump, or broad refactor was
  performed.

## Final Status

DIFF-190 is complete.
