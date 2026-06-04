# DIFF-200 Safe Task Queue Dispatch Visibility

Status: Complete

## Purpose

Improve visibility into task/work queue dispatch without changing worker
semantics. Users can see whether work is supported, waiting, running,
completed, failed, unsupported, or only represented by safe dispatch metadata.

## Branch And Baseline

- Branch before work: `dev`.
- HEAD before work:
  `7dafa8e Complete DIFF-199 feedback outcome to improvement review UX`.
- Working tree before work: clean.
- `dev` ahead/behind `origin/dev` before work: ahead by 4 from DIFF-196 through
  DIFF-199.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `docs/diffs/DIFF-199-feedback-outcome-to-improvement-review-ux.md`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-work-queue-reports/src/lib.rs`
- `apps/web/src/app/page.tsx`
- targeted work queue and dispatch grep output

## Current Work/Dispatch Behavior

- Work items are listed from `/work-items`.
- Supported bounded dispatch work types are:
  - `collection_normalization`;
  - `document_chunking`;
  - `chunk_vector_upsert`.
- The worker daemon processes the supported ingestion/evidence pipeline.
- The Rust gateway dispatch route validates queued work and intent
  verification, then records safe dispatch metadata. It does not execute
  arbitrary user commands.
- Unsupported work dispatch returns an unsupported validation error.

## UX/API Changes Made

- Added `workItemDispatchVisibility` UI helper.
- Each recent Work item now shows:
  - supported vs unsupported bounded dispatch type;
  - current state;
  - whether intent verification is visible;
  - whether dispatch is safe metadata only or worker-managed/not dispatched
    from the UI.
- Existing safe next-step guidance remains visible.
- No retry, recovery, or force-dispatch button was added.
- No worker queue semantics changed.
- No backend/API route changed.

## Verification

Commands run:

```bash
git status --short
git diff --check
git diff --name-status
npm --prefix apps/web run build
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

Results:

- `git diff --check`: passed.
- `git diff --name-status`: showed only DIFF-200 scoped files.
- `npm --prefix apps/web run build`: passed.
- Private/dev files remained tracked.
- Stale status scan continued to report older out-of-scope draft/status strings
  and command transcripts already known from prior DIFFs.
- Full operator smoke was skipped because this DIFF only changes Work tab
  visibility and does not alter runtime/API/operator smoke script behavior.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-200-safe-task-queue-dispatch-visibility.md`
