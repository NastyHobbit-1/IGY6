# DIFF-222 - Prediction / Recommendation Record Creation MVP

Status: Complete

## Purpose

Add controlled prediction/recommendation record creation from evidence.
Predictions and recommendations must be evidence-linked, reviewable, and
outcome-trackable. They must not be automatically executed or presented as
guaranteed truth.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `247722fdae538ed742d24c99e9cd96353aff45fd`
- DIFF-221 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`

## Existing Capability Found

- `POST /analysis/predictions` already exists.
- `POST /analysis/recommendations` already exists.
- Both routes require existing evidence IDs and validate input through the Rust
  gateway.
- Predictions and recommendations are already listed in Results.
- Outcome routes already support prediction and recommendation target types for
  later review.

## Product Changes Made

- Added a normal-user `Prediction / Recommendation Creation` form in Results.
- Users can create:
  - prediction records;
  - recommendation records.
- The form stores or passes:
  - title/summary;
  - required evidence IDs;
  - optional answer/report/task context metadata;
  - confidence;
  - uncertainty;
  - expected result;
  - disproof criteria;
  - review status;
  - timeframe if known;
  - recommendation risk level;
  - recommendation approval-required flag.
- The form clearly states that records are reviewable owner-created records, not
  automatic execution, guaranteed truth, forecasting engine output, or
  autonomous reasoning.
- Added frontend typing for prediction/recommendation evidence and metadata
  fields already returned by the gateway.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

Existing Rust gateway routes already support creation and evidence validation.

## Unsupported States Handled

- The form is disabled until at least one candidate evidence ID is available.
- The user can edit evidence IDs, but the gateway validates that referenced
  evidence exists.
- Prediction creation requires an expected result.
- Recommendations are not executed by this form.
- The UI does not claim forecasting engine output, calibration, or guaranteed
  correctness.

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-222-prediction-recommendation-record-creation-mvp.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-222; DIFF-222 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No hidden data transfer was added.
- No arbitrary command execution was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No prediction or recommendation auto-execution was added.
- No autonomous reasoning, autonomous self-improvement, full chat-memory,
  graph-reasoning, forecasting, or calibration claim was added.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
