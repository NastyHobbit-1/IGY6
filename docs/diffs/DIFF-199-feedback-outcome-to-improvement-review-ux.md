# DIFF-199 Feedback/Outcome To Improvement Review UX

Status: Complete

## Purpose

Start closing the self-improvement loop by connecting feedback/outcome records
to improvement review. The UI helps the user see when feedback or outcomes may
indicate an improvement area, and can create a proposed improvement item through
existing local persistence.

## Branch And Baseline

- Branch before work: `dev`.
- HEAD before work:
  `6d9340b Complete DIFF-198 approval to action execution UX`.
- Working tree before work: clean.
- `dev` ahead/behind `origin/dev` before work: ahead by 3 from DIFF-196 through
  DIFF-198.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `docs/diffs/DIFF-198-approval-to-action-execution-ux.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- feedback, outcome, and improvement route definitions found by targeted grep

## Current Feedback/Outcome/Improvement Behavior

- `GET /feedback` returns persisted feedback events with metadata.
- `POST /feedback` persists feedback. Weak non-source feedback labels can
  automatically create a proposed improvement item.
- `GET /outcomes` returns persisted outcomes with metadata.
- `POST /outcomes` persists outcomes and updates supported target status.
- `GET /improvements` returns improvement item metadata.
- `POST /improvements` can create a proposed improvement item with target area,
  objective, priority, proposer, and metadata.
- There is no autonomous method change or experiment execution in these routes.

## UX/API Changes Made

- Added `ImprovementRecord` typing and loaded `/improvements` into the page.
- Extended the Results feedback/outcome workflow with an Improvement Review
  section.
- The section shows weak feedback signals and unresolved outcome signals.
- Signals show whether a linked improvement already exists by metadata.
- Added a minimal "Propose improvement item" form backed by existing
  `/improvements` persistence.
- Created improvement metadata records include signal kind, signal label, target
  type, target ID, and either feedback ID or outcome ID.

## Improvement-Item Persistence Result

- Improvement item creation is available through existing `POST /improvements`.
- The UI uses only supported target areas:
  `parsing`, `retrieval`, `prediction`, `reporting`, `reasoning`, and `safety`.
- The created record is a proposal only. It does not run experiments, tune
  methods, update production behavior, or claim learning.

## Safety Notes

- No autonomous self-improvement was added.
- No experiment execution was added.
- No method changes or production promotion were added.
- The UI is honest when no weak feedback or unresolved outcome signal exists.
- Synthetic/local review metadata is used; raw uploaded text is not displayed.

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
- `git diff --name-status`: showed only DIFF-199 scoped files.
- `npm --prefix apps/web run build`: passed.
- Private/dev files remained tracked.
- Stale status scan continued to report older out-of-scope draft/status strings
  and command transcripts already known from prior DIFFs.
- Full operator smoke was skipped because this DIFF changes UI review and
  proposal controls around existing routes, not runtime/API/operator smoke
  script behavior.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-199-feedback-outcome-to-improvement-review-ux.md`
