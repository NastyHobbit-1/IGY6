# DIFF-234 - Defer Promotion And Consolidate Next Build Phase

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `95977e9 Complete DIFF-233 product claims audit`
- `dev` tracking state before work: aligned with `origin/dev`
- Working tree before work: clean

## Owner Instruction

The owner instructed:

- Do not promote to `main` yet.
- Do not start promotion DIFFs.
- Do not perform DIFF-234 Public Promotion Candidate Audit.
- Do not perform DIFF-235 Selective Public Promotion Dry Run.
- Do not perform DIFF-236 Main Promotion.
- Update the build plan so promotion is deferred.
- Compress the next 20 product-building areas into 10 larger build DIFFs.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/` current tail listing through DIFF-233
- `README.md`
- `docs/ui/README.md`
- private/dev tracked file list from `git ls-files`

## Plan Changes Made

- Removed the old active promotion sequence from the ordered plan:
  - DIFF-234 — Public Promotion Candidate Audit
  - DIFF-235 — Selective Public Promotion Dry Run
  - DIFF-236 — Main Promotion
- Added an explicit promotion deferral section.
- Recorded that future promotion audit, dry-run, and main promotion work must
  be rescheduled later only by explicit owner instruction.
- Reaffirmed:
  - do not merge `dev` into `main`;
  - do not cherry-pick broad dev commits into `main`;
  - do not touch `main` during product-build DIFFs;
  - do not create promotion branches without explicit owner instruction;
  - selectively promote only necessary public/runtime-safe files later, when
    the owner requests it;
  - keep private/dev/build instruction files on `dev`.

## New DIFF-235 Through DIFF-244 Build Sequence

1. DIFF-235 — Source Expansion And Connector Contract Foundation
2. DIFF-236 — Browser / Web / Router Collector MVP
3. DIFF-237 — PDF / Image / Audio / Video Import MVP
4. DIFF-238 — Local Project And PC Diagnostics Collector Hardening
5. DIFF-239 — Graph Extraction And Relationship Reasoning Foundation
6. DIFF-240 — Pattern / Conflict / Drift / Anomaly Expansion
7. DIFF-241 — Prediction / Recommendation Generation And Calibration MVP
8. DIFF-242 — Self-Improvement Experiment Workflow MVP
9. DIFF-243 — Guardrails / Tool-Use / External-Model Policy Hardening
10. DIFF-244 — Data Lifecycle Hardening And Release Readiness

Because DIFF-234 is needed to update the plan and defer promotion, the 10
product-build DIFFs are DIFF-235 through DIFF-244.

## Scope Confirmation

This DIFF only updates planning docs.

No runtime product feature was implemented. No runtime app code, Rust code,
`.env`, branch policy file, runtime/private data, promotion branch, `main`
branch, merge, cherry-pick, push, public-file promotion, fake control,
arbitrary command execution, or private/dev file removal was performed.

## Files Changed

- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-234-defer-promotion-consolidate-next-build-phase.md`

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- `npm --prefix apps/web run build` was not required because this DIFF changed
  planning docs only and no UI/runtime code.
- Rust checks were not required because no Rust files changed.
- Full Docker smoke was not run from Codex because this DIFF is planning-only
  and Codex local Docker access is not reliable by design.

## Verification Summary

- Planning-only edits were applied.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope `Status: Draft` strings
  and command examples in historical DIFF records; this DIFF is
  `Status: Complete`.
