# DIFF-201 Improvement Experiment Proposal / Review UX

Status: Complete

## Purpose

Expose improvement and experiment metadata as a controlled review surface. Users
can see proposed improvements and experiment records, and can record planned
experiment metadata without starting autonomous execution or changing
production behavior.

## Branch And Baseline

- Branch before work: `dev`.
- HEAD before work:
  `c5d0c0e Complete DIFF-200 safe task queue dispatch visibility`.
- Working tree before work: clean.
- `dev` ahead/behind `origin/dev` before work: ahead by 5 from DIFF-196 through
  DIFF-200.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `docs/diffs/DIFF-200-safe-task-queue-dispatch-visibility.md`
- `services/self_improvement/README.md`
- `services/reports/README.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- targeted improvement, experiment, MLflow, Phoenix, and Optuna grep output

## Current Improvement/Experiment Behavior

- `GET /improvements` lists improvement item metadata.
- `POST /improvements` creates proposed improvement items.
- `GET /experiments` lists experiment metadata.
- `POST /experiments` creates experiment records with status such as
  `planned`, plus optional metrics/artifact/metadata JSON.
- `POST /experiments/:id/status` can update experiment record status.
- `services/self_improvement/README.md` states the self-improvement service is
  Phase 0 placeholder material and does not run experiments, Optuna studies,
  DSPy optimization, or method promotion.
- MLflow and Phoenix exist as supporting infrastructure, but this UI does not
  start runs or traces.

## UX/API Changes Made

- Added `ExperimentRecord` typing and loaded `/experiments` in the main page.
- Added `ImprovementExperimentReview` in Results.
- The review surface lists recent improvement items with target area, objective,
  proposer, priority, and status.
- The review surface lists recent experiment records with status, linked
  improvement item, MLflow ID, Optuna study name, and created timestamp.
- Added a minimal form to record planned experiment metadata for an existing
  improvement item.
- Planned experiment records include proposal scope, proposed success metric,
  `execution_model: not_started`, and
  `autonomous_method_change: false`.

## Review/Proposal Verification Result

- Proposal creation uses existing `POST /experiments` persistence.
- Creation is disabled when no improvement item exists.
- The UI states clearly that planned experiment records are review metadata
  only.
- No experiment runner, MLflow run, Optuna study, Phoenix trace workflow, or
  method promotion was implemented.

## Safety Notes

- No autonomous self-improvement was added.
- No production method change was added.
- No experiment execution was added.
- No claim was made that self-improvement is complete.
- Missing or incomplete integration is displayed honestly as review-only
  metadata.

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
- `git diff --name-status`: showed only DIFF-201 scoped files.
- `npm --prefix apps/web run build`: passed.
- Private/dev files remained tracked.
- Stale status scan continued to report older out-of-scope draft/status strings
  and command transcripts already known from prior DIFFs.
- Full operator smoke was skipped because this DIFF changes UI review/proposal
  controls around existing routes, not runtime/API/operator smoke script
  behavior.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-201-improvement-experiment-proposal-review-ux.md`
