# DIFF-223 - Prediction / Recommendation Outcome Review

Status: Complete

## Purpose

Close the review loop for prediction/recommendation records by making outcomes
easy to record and inspect. Users can mark predictions/recommendations correct,
wrong, partial, useful, not useful, or inconclusive, with evidence links where
supported.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `5b5d1074a4b55a2a8092674907ad47ed22b796e5`
- DIFF-222 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-222-prediction-recommendation-record-creation-mvp.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`

## Existing Capability Found

- The Rust outcome API already supports `prediction` and `recommendation`
  target types.
- Outcome statuses already include `correct`, `wrong`, `partial`, `useful`,
  `not_useful`, and `inconclusive`.
- Improvement item creation already exists through `POST /improvements`.

## Product Changes Made

- Added a normal-user `Prediction / Recommendation Outcome Review` panel in
  Results.
- The panel shows:
  - prediction/recommendation details;
  - status;
  - linked evidence IDs;
  - stored answer/report/task context metadata where present;
  - existing feedback counts;
  - existing outcome counts;
  - linked improvement-candidate counts;
  - existing prediction/recommendation outcome records.
- Added outcome review controls for prediction/recommendation targets:
  - outcome status;
  - optional evidence IDs;
  - summary note;
  - optional improvement-candidate creation.
- Improvement candidate creation is offered only as a user-selected proposal for
  wrong, partial, not useful, or inconclusive outcome statuses.
- Updated the Outcome Learning Summary target labels so prediction and
  recommendation outcome records are easier to inspect.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

Existing Rust gateway outcome and improvement routes already support the scoped
review behavior.

## Unsupported States Handled

- The panel does not auto-execute recommendations.
- The panel does not auto-change future recommendations.
- The panel does not claim calibration, forecasting, or autonomous reasoning is
  complete.
- Improvement candidates are proposed metadata only and do not run experiments.
- Empty states are honest when no prediction/recommendation records exist.

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
- `docs/diffs/DIFF-223-prediction-recommendation-outcome-review.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-223; DIFF-223 is `Status: Complete`.

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
