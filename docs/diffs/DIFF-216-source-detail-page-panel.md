# DIFF-216 - Source Detail Page / Panel

Status: Complete

## Purpose

Add a normal-user source detail panel so users can inspect one source's state,
lineage, review metadata, and safe next action without using Advanced raw JSON.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `31809926e7decb8c6ce86b577b762b53132de753`
- DIFF-215 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-210-source-trust-sensitivity-management-ux.md`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-213-conversation-history-import-mvp.md`
- `docs/diffs/DIFF-214-user-observation-ingestion-mvp.md`
- `docs/diffs/DIFF-215-guided-source-onboarding-completion.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`

## Current Source Detail Capability Found

- Add Data already listed sources with source type, sensitivity, permission
  count, and enabled state.
- DIFF-210 added source trust/sensitivity review.
- Results already had source/evidence history grouped by recent collection
  runs.
- Advanced exposed raw source JSON and raw IDs.

## Product Workflow Gap Found

Normal users could see source rows and review source trust, but there was no
single source detail panel showing one source's permissions, collection runs,
artifact metadata, documents, chunks, evidence, direct feedback/outcome links,
correction/supersession indicators, and next safe action.

## Product Changes Made

- Added a normal Add Data source detail panel.
- Each source detail card shows:
  - source id, label, type, trust level, sensitivity, and enabled/disabled
    state;
  - source permissions and approval-required state;
  - linked collection runs;
  - linked raw artifact metadata;
  - linked documents and chunk counts;
  - linked evidence previews;
  - direct source feedback count;
  - direct source outcome count where present;
  - evidence correction/supersession review indicators;
  - a safe next action based on source enabled state and lineage.
- The panel uses existing loaded data from current list routes; no new backend
  route was needed.
- The panel keeps raw IDs available in details for audit/troubleshooting while
  avoiding raw artifact contents.
- Updated the UI guide with the Source Detail behavior and limitations.

## Backend/API Changes

No Rust backend or Next.js proxy changes were required.

The panel uses existing read data already loaded by the page:

- `/sources`
- `/collection-runs`
- `/artifacts`
- `/evidence/documents`
- `/evidence/chunks`
- `/evidence/items`
- `/feedback`
- `/outcomes`

## Unsupported States Handled

- Raw artifact contents are not displayed.
- Secrets are not exposed.
- Source detail does not claim complete policy enforcement.
- Disabled sources are shown honestly and are not deleted.
- Superseded or corrected evidence remains visible with review indicators.
- Direct source outcomes are shown only where linked; missing outcome links are
  shown honestly.
- Empty source lineage remains an honest empty state.

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
- Live source detail inspection against PostgreSQL was not run because it
  requires the owner's normal WSL stack/database. The UI behavior was covered
  by the Next.js build and uses existing read routes.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-216-source-detail-page-panel.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-216; DIFF-216 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No raw artifact content dump was added.
- No secret exposure was added.
- No source deletion behavior was added.
- No fake controls were added.
- No unsupported source type was added.
- No browser scraping was added.
- No account scraping was added.
- No connector import was added.
- No external service call was added.
- No hosted AI call was added.
- No arbitrary command execution was added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
