# DIFF-219 - Evidence-Grounded Answer Generation MVP

Status: Complete

## Purpose

Improve the Results answer experience so users can ask over evidence and receive
a clear evidence-grounded answer packet, not just raw retrieval hits.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `5e43f5b5c7e2b1b8a70f20e42b4b89f084a7b4a7`
- DIFF-218 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-217-evidence-detail-page-panel.md`
- `docs/diffs/DIFF-218-local-llm-provider-status-routing-ux.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `apps/web/src/app/api/evidence-answers/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-evidence-answer/src/lib.rs`

## Product Changes Made

- Added a normal-user evidence-grounded answer packet to Results `Ask Over
  Evidence`.
- The packet now separates:
  - answer status;
  - answer summary text;
  - facts extracted from retrieved evidence/chunk previews;
  - assumptions;
  - inferences;
  - uncertainty;
  - missing information;
  - citation labels and evidence/document/chunk/source IDs;
  - source/document/chunk trails;
  - retrieval hit count;
  - deterministic fallback and local model/provider status.
- Raw retrieved evidence hits remain visible below the packet.
- Saving an answer record now stores the packet fields in the existing
  persisted answer-record columns instead of saving only retrieval metadata.
- The UI distinguishes retrieved evidence, deterministic answer packet output,
  local model not-called state, and insufficient-evidence state.
- Updated the UI guide for the new answer packet behavior.

## Backend/API Changes

No backend or proxy changes were required.

The existing retrieval-preview route provides enough local evidence context for
the normal UI, and the existing evidence-answer record schema already supports
the packet fields used by this DIFF.

## Unsupported States Handled

- If no hits are returned, the packet shows `insufficient_evidence` and does
  not fabricate a conclusion.
- Retrieval scores are presented as similarity signals, not proof.
- The packet states that relevant sources not yet ingested, chunked, or
  embedded are missing from the answer.
- The retrieval-preview path does not claim local LLM contribution.
- The UI does not claim complete long-term memory, full chat memory, full
  reasoning, autonomous reasoning, autonomous self-improvement, graph reasoning,
  or forecasting.

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
- Live local LLM generation was not run because this DIFF uses the existing
  retrieval-preview path and does not contact a provider.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-219-evidence-grounded-answer-generation-mvp.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan did not identify DIFF-219 as active, in-progress, or draft.

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
