# DIFF-217 - Evidence Detail Page / Panel

Status: Complete

## Purpose

Add a normal-user evidence detail panel so users can inspect one evidence item's
preview, source/document/chunk lineage, review state, feedback/outcome links,
and related outputs without mutating evidence.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `cbd14f449692991f22039dfb1a8f83402c99391b`
- DIFF-216 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-216-source-detail-page-panel.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`

## Current Evidence Detail Capability Found

- Results already listed evidence and documents.
- DIFF-211 added evidence correction/supersession review metadata.
- DIFF-212 added persisted evidence answer records with evidence IDs.
- Advanced exposed raw JSON details.

## Product Workflow Gap Found

Normal users could mark evidence review state and see evidence lists, but there
was no single evidence detail panel showing preview, source trail,
document/chunk lineage, source trust/sensitivity context, direct
feedback/outcomes, related saved answer records, and task/report links where
metadata connected them.

## Product Changes Made

- Added a normal Results evidence detail panel.
- Each evidence detail card shows:
  - evidence id;
  - bounded evidence preview;
  - source trail;
  - document and chunk lineage;
  - source trust and sensitivity context where available;
  - correction/supersession state and note;
  - superseding evidence id where linked;
  - direct feedback links;
  - direct outcome links where present;
  - related persisted answer records that cite the evidence id;
  - related task plans when metadata references the evidence id;
  - related reports when metadata references the evidence id;
  - a safe next action.
- The panel is read-only and uses existing loaded route data.
- Updated the UI guide with Evidence Detail behavior and limitations.

## Backend/API Changes

No Rust backend or Next.js proxy changes were required.

The panel uses existing read data already loaded by the page:

- `/evidence/items`
- `/sources`
- `/evidence/documents`
- `/evidence/chunks`
- `/evidence-answers`
- `/agent/task-plans`
- `/reports`
- `/feedback`
- `/outcomes`

## Unsupported States Handled

- Evidence is not mutated.
- Evidence is not deleted.
- Superseded evidence is not silently hidden.
- Raw text is bounded to previews.
- Missing source, document, chunk, answer, task-plan, report, feedback, or
  outcome links are shown honestly.
- Task plan and report related links are metadata-based only where those records
  reference the evidence id.

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
- Live evidence detail inspection against PostgreSQL was not run because it
  requires the owner's normal WSL stack/database. The UI behavior was covered
  by the Next.js build and uses existing read routes.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-217-evidence-detail-page-panel.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-217; DIFF-217 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No evidence mutation or deletion was added.
- No silent hiding of superseded evidence was added.
- No excessive raw text dump was added.
- No fake controls were added.
- No hosted AI call was added.
- No browser/account scraping, connector import, or external service call was
  added.
- No arbitrary command execution was added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
