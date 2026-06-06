# DIFF-220 - Missing Evidence Prompting

Status: Complete

## Purpose

When evidence is weak or absent, guide the user on what to add next without
fabricating conclusions or implying that missing local evidence means the
real-world information does not exist.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `948e47800108e73d27e8eb3614edfcdeca3a343a`
- DIFF-219 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-219-evidence-grounded-answer-generation-mvp.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `apps/web/src/app/api/evidence-answers/route.ts`
- `crates/igy6-evidence-answer/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`

## Product Changes Made

- Added a normal-user `Missing Evidence Prompts` panel in Results.
- The panel summarizes:
  - evidence status;
  - processed evidence item count;
  - processed chunk count;
  - latest retrieved hit count from saved answers/task evidence summaries;
  - missing-information note count;
  - missing-information notes where available;
  - why more evidence may be needed;
  - the next safe action.
- The panel classifies:
  - `insufficient-evidence` when processed local evidence/chunks are absent or
    the latest evidence answer has no hits;
  - `weak-evidence` when some evidence exists but missing-information notes or a
    low hit count remain;
  - `evidence-available` when local evidence is available while still reminding
    users to inspect citations.
- Suggested next source types are scoped to supported local paths:
  - manual text upload;
  - conversation_history;
  - user_observation;
  - local_project only when an enabled scoped local_project source already
    exists.
- Added Results guidance that missing local evidence is a coverage gap, not
  proof of real-world absence.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

The existing Results data already exposes evidence items, chunks, sources,
saved evidence answer records, and task-plan evidence summaries.

## Unsupported States Handled

- The prompt does not automatically collect data.
- The prompt does not add browser scraping, account scraping, connector import,
  or external service collection.
- The prompt does not imply that absent local evidence proves real-world
  absence.
- The prompt does not fabricate conclusions when evidence is absent.
- The prompt does not bypass approvals or source onboarding.

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
- `docs/diffs/DIFF-220-missing-evidence-prompting.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-220; DIFF-220 is `Status: Complete`.

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
