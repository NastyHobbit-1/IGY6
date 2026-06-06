# DIFF-224 - Baseline Pattern Expansion MVP

Status: Complete

## Purpose

Expand baseline pattern detection/review beyond minimal pattern records. Add
useful, honest pattern categories that help users identify repeated signals
without claiming advanced statistical validation, forecasting, or full anomaly
detection.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `ee6d8cc321bd0481b419a3f3cf73a69c76966de1`
- DIFF-223 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-219-evidence-grounded-answer-generation-mvp.md`
- `docs/diffs/DIFF-220-missing-evidence-prompting.md`
- `docs/diffs/DIFF-221-outcome-learning-summary-mvp.md`
- `docs/diffs/DIFF-222-prediction-recommendation-record-creation-mvp.md`
- `docs/diffs/DIFF-223-prediction-recommendation-outcome-review.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-write-api/src/lib.rs`

## Existing Capability Found

- `GET /analysis/patterns` already lists persisted pattern records with
  `pattern_type`, `status`, `summary`, `evidence_ids`, `confidence`, and
  metadata.
- `POST /analysis/patterns` already creates evidence-linked pattern records.
- `POST /analysis/patterns/detect-baseline` already supports baseline detector
  behavior for recurrence, missing-information gap, and cross-source statement
  review.

## Product Changes Made

- Added a normal-user `Baseline Pattern Expansion` panel in Results.
- The panel shows saved patterns with:
  - pattern type;
  - linked evidence count;
  - resolved source names where loaded evidence/source records allow it;
  - confidence or support state;
  - review status;
  - unverified note;
  - created time.
- Added local review candidates for:
  - recurrence;
  - missing-information gap;
  - cross-source agreement;
  - cross-source conflict;
  - failed-advice recurrence;
  - successful-method recurrence.
- Candidate cards show support count, confidence when available, source of the
  signal, what is still unverified, and safe next action.
- Added a save flow backed by existing `POST /analysis/patterns` for candidates
  with linked evidence IDs.
- Added a normal-user button for the existing baseline detector route.
- Review-only candidates without evidence IDs remain visible but are not
  persisted by the UI.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

Existing Rust gateway pattern routes already support the persistence needed for
evidence-linked pattern candidates.

## Unsupported States Handled

- The UI does not claim advanced statistical validation.
- The UI does not claim forecasting or full anomaly detection.
- The UI does not automatically modify future behavior.
- Weak evidence and review-only metadata signals remain visible.
- Missing local evidence is treated as a coverage gap, not proof of real-world
  absence.

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
- `docs/diffs/DIFF-224-baseline-pattern-expansion-mvp.md`
- `docs/ui/README.md`

## Classification

UI plus existing API wiring.

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-224; DIFF-224 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No hidden data transfer was added.
- No arbitrary command execution was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete, restore, or backup archive creation was performed.
- No autonomous self-improvement, full graph-reasoning, full forecasting, or
  full anomaly-detection claim was added.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
