# DIFF-241 - Prediction / Recommendation Generation And Calibration MVP

Status: Complete

## Branch And Baseline

- Active branch before work: `dev`
- HEAD before work: `8808989 Complete DIFF-240 pattern conflict drift anomaly expansion`
- `dev` ahead/behind `origin/dev` before work: ahead by one local DIFF commit, not behind
- Controlling plan: `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Scope

This DIFF adds a real read-only calibration summary for existing persisted
prediction/recommendation records and explicit owner-recorded outcomes.

It does not implement a forecasting engine, automatic recommendation execution,
hosted AI generation, external model calls, advanced calibration statistics, or
automatic future-behavior changes.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-222-prediction-recommendation-record-creation-mvp.md`
- `docs/diffs/DIFF-223-prediction-recommendation-outcome-review.md`
- `README.md`
- `docs/ui/README.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`

## Changes

- Added Rust-native `GET /analysis/calibration/summary`.
- The route reads persisted prediction, recommendation, and outcome records and
  returns:
  - prediction/recommendation counts;
  - evidence-linked record count;
  - records with explicit outcomes;
  - outcome status counts;
  - prediction vs recommendation outcome totals;
  - descriptive confidence bands.
- Added deterministic Rust unit coverage for the calibration summary helper.
- Added route coverage and no-database behavior coverage through the existing
  gateway tests.
- Added a Results-tab summary display that consumes the new endpoint and keeps
  the limitations visible to normal users.
- Updated `docs/ui/README.md`.

## Verification

- `git status --short` - showed only DIFF-241 scoped changes before commit.
- `git diff --check` - passed.
- `git diff --name-status` - showed:
  - `M apps/web/src/app/page.tsx`
  - `M crates/igy6-gateway/src/lib.rs`
  - `M docs/ui/README.md`
- `npm --prefix apps/web run build` - passed.
- `cargo fmt --all --check` - passed after applying `cargo fmt --all`.
- `cargo test --workspace` - passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort` - private/dev instruction files remained tracked.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true` - still reports older out-of-scope draft/template/status-command strings; no new active/in-progress DIFF was left by this DIFF.

## Files Changed

- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-241-prediction-recommendation-generation-calibration-mvp.md`

## Notes

- No full Docker smoke was run from Codex per environment rule.
- No runtime/private data was dumped.
- No recommendation was executed.
- No hosted AI call, hidden external transfer, browser/account scraping,
  credential/cookie/token collection, arbitrary command execution behavior,
  destructive delete, destructive restore, unsafe backup archive, `.env` edit,
  main work, merge, cherry-pick, push, promotion, fake control, or private/dev
  file removal was performed.
