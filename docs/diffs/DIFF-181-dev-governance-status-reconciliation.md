# DIFF-181: Dev Governance Status Reconciliation

Status: Complete

## Type

Documentation / governance metadata reconciliation

## Objective

Reconcile stale DIFF status metadata on `dev` and record the current branch
policy without removing any private/dev/build instruction files.

## Branch Policy Recorded

- Future IGY6 work happens on `dev`.
- Private/dev/build instruction files stay on `dev`.
- `main` remains the public/runtime-clean branch.
- Later, only necessary public/runtime-safe files should be selectively
  promoted to `main`.
- Do not merge `main` into `dev` unless explicitly instructed.
- Do not cherry-pick cleanup commits from `main` into `dev` unless explicitly
  instructed.
- This DIFF removes no private/dev files.

## Baseline Facts

- Branch: `dev`.
- HEAD before edits:
  `eae4376 Complete DIFF-180 guided manual text upload flow`.
- `dev` was up to date with `origin/dev`.
- Working tree was clean.
- Highest DIFF found on `dev` before this work was DIFF-180.
- Private/dev files were tracked on `dev`, including `AGENTS.md`, `.codex`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/agents/*`, and `docs/plans/*`.

## Allowed Scope

- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md`
- This DIFF record
- `README.md` or `docs/BRANCH_POLICY.md` only if already present on `dev` and
  needing a minimal branch-policy note
- `AGENTS.md` only if already present on `dev` and needing a minimal
  branch-policy note

## Prohibited Scope

- Do not remove anything from `dev`.
- Do not remove `.codex`.
- Do not remove `AGENTS.md`.
- Do not remove `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`.
- Do not remove `docs/agents`.
- Do not remove `docs/plans`.
- Do not merge `main` into `dev`.
- Do not cherry-pick `main` into `dev`.
- Do not edit runtime code.
- Do not start a feature DIFF.
- Do not rewrite old DIFF content beyond minimal status/notes needed for
  reconciliation.

## Files Inspected

- `AGENTS.md`
- `README.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF_PROCESS.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- tracked private/dev file inventory from `git ls-files AGENTS.md .codex
  Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents
  docs/plans docs/diffs | sort`
- recent git history from `git log --oneline --decorate -12`

## Stale Statuses Found

- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`:
  stale Active status
- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`:
  stale Draft status
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md`:
  stale Draft status

## Status Decisions

- DIFF-176 changed from Active to Complete with a reconciliation note. Git
  history shows DIFF-176 implementation commits before DIFF-180, and later
  DIFFs build on that request-understanding clarification flow as completed
  work.
- DIFF-178 changed from Draft to Complete with a reconciliation note. The DIFF
  already contained completed Result and Verification Result sections, and git
  history includes `5812791 Complete DIFF-178 product roadmap gap audit`.
- DIFF-179 changed from Draft to Complete with a reconciliation note. The DIFF
  already contained completed Result and Verification Result sections, and git
  history includes completed DIFF-179 commits before DIFF-180.

## Files Changed

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md`
- `docs/diffs/DIFF-181-dev-governance-status-reconciliation.md`

## Verification

Required commands:

```bash
git status --short
git diff --check
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep for stale Active, In Progress, and Draft DIFF statuses under docs/diffs
git diff --name-status
```

## Verification Result

Passed.

- `git status --short` showed only the scoped governance/status files changed.
- `git diff --check` passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
  confirmed private/dev files remain tracked on `dev`.
- The required stale-status grep no longer returned DIFF-176, DIFF-178, or
  DIFF-179 as stale.
- The same grep still returned `DIFF-177` and `DIFF-180` as Draft, plus the
  DIFF template. DIFF-177 and DIFF-180 were not in this DIFF's allowed
  status-reconciliation scope and were left unchanged.
- `git diff --name-status` showed only scoped files changed.

Not run:

- No build, test, runtime, Docker Compose, service, merge, cherry-pick, or
  cleanup command was run because this DIFF is governance metadata only.
