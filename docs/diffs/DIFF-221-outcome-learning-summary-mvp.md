# DIFF-221 - Outcome Learning Summary MVP

Status: Complete

## Purpose

Start turning outcomes and feedback into user-visible learning summaries so the
owner can see repeated failed advice, repeated successful methods, and possible
improvement candidates without claiming autonomous method changes.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `07ae806c0ff1865b8006a21099742f463e93aa44`
- DIFF-220 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-199-feedback-outcome-to-improvement-review-ux.md`
- `docs/diffs/DIFF-201-improvement-experiment-proposal-review-ux.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`

## Product Changes Made

- Added an `Outcome Learning Summary` surface in Results before the existing
  feedback/outcome capture and improvement review controls.
- The summary groups existing records into:
  - recent negative/unresolved feedback and outcomes;
  - recent positive/successful feedback and outcomes;
  - repeated failed labels;
  - repeated failed targets;
  - repeated successful labels;
  - repeated successful targets.
- Linked visible signals to answer, report, task-plan, or work-item context
  where loaded records provide the target ID.
- Added a candidate improvement prompt that directs users to the existing
  Improvement review form when weak feedback or unresolved outcomes are visible.
- Existing improvement item creation remains explicit and user-triggered through
  the current Results improvement review workflow.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

The existing `GET /feedback`, `GET /outcomes`, `GET /improvements`,
`GET /evidence-answers`, `GET /reports`, `GET /agent/task-plans`, and
`GET /work-items` data already supports the summary.

## Unsupported States Handled

- The summary does not change future reasoning behavior automatically.
- The summary does not claim self-improvement is complete.
- The summary does not auto-promote methods.
- The summary does not run experiments.
- The summary does not claim calibration, forecasting, graph reasoning, or
  autonomous reasoning.
- Empty states are honest when no repeated signal is detectable.

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
- `docs/diffs/DIFF-221-outcome-learning-summary-mvp.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-221; DIFF-221 is `Status: Complete`.

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
  graph-reasoning, or forecasting claim was added.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
