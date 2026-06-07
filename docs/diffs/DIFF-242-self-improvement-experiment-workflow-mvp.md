# DIFF-242 - Self-Improvement Experiment Workflow MVP

Status: Complete

## Branch And Baseline

- Active branch before work: `dev`
- HEAD before work: `705115c Complete DIFF-241 prediction recommendation generation calibration MVP`
- `dev` ahead/behind `origin/dev` before work: ahead by two local DIFF commits, not behind
- Controlling plan: `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Scope

This DIFF adds a controlled self-improvement experiment workflow from persisted
improvement item to experiment proposal/dry-run metadata. It does not execute
experiments, self-edit code, change runtime methods, call hosted services, call
MLflow/Optuna/Phoenix, or claim autonomous self-improvement.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-199-feedback-outcome-to-improvement-review-ux.md`
- `docs/diffs/DIFF-201-improvement-experiment-proposal-review-ux.md`
- `README.md`
- `docs/ui/README.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`

## Changes

- Added Rust-native `POST /experiments/propose-from-improvement`.
- The route validates:
  - linked improvement item ID;
  - bounded proposal scope;
  - at least one success criterion;
  - bounded dry-run summary;
  - bounded result comparison plan.
- The route creates a planned experiment record with:
  - success criteria;
  - result comparison plan and `not_run` comparison status;
  - dry-run metadata;
  - review status;
  - accepted-method metadata with approval required;
  - execution/autonomous method-change fields set false.
- The route patches the linked improvement item metadata with the latest
  experiment proposal ID and approval-required posture.
- Added an approval gate for later `accepted` experiment status updates:
  accepted status requires `accepted_method.approval_id` and the approval must
  be an approved `experiment_acceptance` approval record.
- Added `accepted`, `rejected`, and `deferred` experiment review statuses.
- Updated the Results experiment review UI to use the new proposal route and
  collect success criteria, dry-run summary, and result comparison plan.
- Updated `docs/ui/README.md`.

## Verification

- `git status --short` - showed only DIFF-242 scoped changes before commit.
- `git diff --check` - passed.
- `git diff --name-status` - showed:
  - `M apps/web/src/app/page.tsx`
  - `M crates/igy6-gateway/src/lib.rs`
  - `M docs/ui/README.md`
- `npm --prefix apps/web run build` - passed.
- `cargo fmt --all --check` - passed after applying `cargo fmt --all`.
- `cargo test --workspace` - passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort` - private/dev instruction files remained tracked.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true` - still reports older out-of-scope draft/template/status-command strings; no new active/in-progress DIFF was left by this DIFF.

## Files Changed

- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-242-self-improvement-experiment-workflow-mvp.md`

## Notes

- No full Docker smoke was run from Codex per environment rule.
- No runtime/private data was dumped.
- No experiment execution, self-editing, autonomous self-improvement, method
  promotion, MLflow run, Optuna study, Phoenix workflow, hosted AI call, hidden
  external transfer, browser/account scraping, credential/cookie/token
  collection, arbitrary command execution behavior, destructive delete,
  destructive restore, unsafe backup archive, `.env` edit, main work, merge,
  cherry-pick, push, promotion, fake control, or private/dev file removal was
  performed.
