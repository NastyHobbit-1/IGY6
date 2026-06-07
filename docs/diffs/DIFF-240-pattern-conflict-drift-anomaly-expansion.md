# DIFF-240 - Pattern / Conflict / Drift / Anomaly Expansion

Status: Complete

## Branch And Baseline

- Active branch before work: `dev`
- HEAD before work: `dd6d5eb Complete DIFF-239 graph extraction relationship reasoning foundation`
- `dev` ahead/behind `origin/dev` before work: even with `origin/dev`
- Controlling plan: `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Scope

This DIFF expands the existing Rust baseline pattern detector beyond UI-only
review prompts. It adds bounded candidate creation from already persisted local
evidence and outcome metadata.

This is baseline review logic only. It does not implement advanced statistical
validation, forecasting, causality discovery, automatic behavior changes, hosted
AI, external collection, browser/account scraping, or destructive lifecycle
behavior.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`
- `docs/diffs/DIFF-236-browser-web-router-collector-mvp.md`
- `docs/diffs/DIFF-237-pdf-image-audio-video-import-mvp.md`
- `docs/diffs/DIFF-238-local-project-pc-diagnostics-collector-hardening.md`
- `docs/diffs/DIFF-239-graph-extraction-relationship-reasoning-foundation.md`
- `README.md`
- `docs/ui/README.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`

## Changes

- Expanded the Rust `detect_baseline_patterns` path to load existing outcome
  records in addition to evidence records.
- Added candidate support for:
  - recurrence
  - missing-information gap
  - cross-source agreement
  - cross-source conflict
  - configuration drift
  - anomaly signal
  - failed-advice recurrence
  - successful-method recurrence
- Added candidate metadata for detector version, detector key, review status,
  support count, evidence count, linked source IDs, linked outcome IDs where
  applicable, and unverified-review notes.
- Updated the UI pattern panel to reflect the expanded detector categories and
  to keep the statistical/anomaly limitations explicit.
- Updated `docs/ui/README.md` to document the real persisted detector behavior
  and its limits.

## Verification

- `git status --short` - showed only DIFF-240 scoped changes before commit.
- `git diff --check` - passed.
- `git diff --name-status` - showed:
  - `M apps/web/src/app/page.tsx`
  - `M crates/igy6-gateway/src/lib.rs`
  - `M docs/ui/README.md`
- `npm --prefix apps/web run build` - passed.
- `cargo fmt --all --check` - passed.
- `cargo test --workspace` - passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort` - private/dev instruction files remained tracked.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true` - still reports older out-of-scope draft/template/status-command strings; no new active/in-progress DIFF was left by this DIFF.

## Files Changed

- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-240-pattern-conflict-drift-anomaly-expansion.md`

## Notes

- No full Docker smoke was run from Codex per environment rule.
- No runtime/private data was dumped.
- No hosted AI call, hidden external transfer, browser/account scraping,
  credential/cookie/token collection, destructive delete, destructive restore,
  unsafe backup archive, `.env` edit, main work, merge, cherry-pick, push,
  promotion, fake control, or private/dev file removal was performed.
