# DIFF-181: Simplify Branch Policy To Main-Only With Local-Ignored Agent Files

Status: Complete

## Type

Documentation / governance cleanup

## Objective

Simplify the IGY6 workflow so normal development happens on `main`.

Private/build-agent/local coordination files must stay local-only through
`.gitignore`. Stop the routine `dev` to clean-main to `main` promotion loop.

## Baseline Facts

- `main` is clean and contains DIFF-180.
- `dev` exists, but `dev` contains private/build-agent planning material that
  must not be merged into `main`.
- `.gitignore` prevents new untracked files from being added. It does not
  remove files already tracked on `dev`.
- `dev` must not be merged into `main`.
- Private/build-agent files must not be copied from `dev` into `main`.

## Allowed Scope

- Update `.gitignore`.
- Update `docs/BRANCH_POLICY.md`.
- Add this DIFF record.
- Update `README.md` only if it references the old `dev`/`main` workflow.
- Update this DIFF Result and Verification Result.

## Prohibited Scope

- Do not merge `dev` into `main`.
- Do not add `.codex` to `main`.
- Do not add `AGENTS.md` to `main`.
- Do not add `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md` to
  `main`.
- Do not add private prompt files under `docs/agents/` to `main`.
- Do not add `docs/plans/DEV_BRANCH_POLICY.md` to `main`.
- Do not add `docs/plans/IGY6_DEV_BUILD_PLAN.md` to `main`.
- Do not change runtime code.
- Do not change UI code.
- Do not change Rust crates.
- Do not change Docker Compose.
- Do not change `.env` or `.env.example`.
- Do not change migrations.
- Do not delete local or remote `dev` branch in this DIFF.
- Do not start, stop, or restart services.

## New Branch Policy

- `main` is the normal working branch.
- Optional feature branches may branch from `main` and PR or merge back to
  `main`.
- Private/local agent files are ignored and should stay outside tracked history.
- `dev` is obsolete as the normal working branch.
- `dev` may be deleted later only after a separate explicit owner-approved
  cleanup.
- Runtime/product changes no longer require the `dev` to clean-main cherry-pick
  loop.
- Private/build-agent files still must not be committed to `main`.

## Result

Completed.

- Added `.gitignore` rules for local-only/private agent coordination files:
  `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/agents/`, `docs/plans/DEV_BRANCH_POLICY.md`, and
  `docs/plans/IGY6_DEV_BUILD_PLAN.md`.
- Rewrote `docs/BRANCH_POLICY.md` to make `main` the normal working branch,
  allow optional feature branches from `main`, mark `dev` obsolete as the
  normal working branch, and keep private/build-agent files out of tracked
  history.
- Updated `README.md` to remove the obsolete instruction that private
  build-agent instructions belong on a local development branch.
- Did not add
  `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md` to `main`;
  this DIFF does not make that dev-side plan public product documentation.

## Verification Result

Passed:

- `git status --short` confirmed the only changed paths are `.gitignore`,
  `README.md`, `docs/BRANCH_POLICY.md`, and this DIFF record.
- `git branch --show-current` returned `main`.
- `git log --oneline --decorate -6` confirmed `main` remains at DIFF-180 before
  these working-tree edits and no `dev` merge commit was created.
- `git diff --name-status main..dev | sed -n '1,160p'` showed `dev` still
  contains private/build-agent paths such as `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/agents/*`, `docs/plans/DEV_BRANCH_POLICY.md`,
  `docs/plans/IGY6_DEV_BUILD_PLAN.md`, and
  `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`; these were not
  merged or copied into `main`.
- `git diff --name-status` showed only tracked documentation/policy edits:
  `.gitignore`, `README.md`, and `docs/BRANCH_POLICY.md`. The new DIFF record is
  visible in `git status --short` until staged.
- `git diff --check` passed.
- `git check-ignore -v .codex AGENTS.md Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents/example.md docs/plans/DEV_BRANCH_POLICY.md docs/plans/IGY6_DEV_BUILD_PLAN.md`
  confirmed all required local-only/private paths match `.gitignore`.
- `git ls-files .codex AGENTS.md Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans/DEV_BRANCH_POLICY.md docs/plans/IGY6_DEV_BUILD_PLAN.md`
  returned only `docs/agents/README.md`; it did not show `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, private prompt
  files, `docs/plans/DEV_BRANCH_POLICY.md`, or
  `docs/plans/IGY6_DEV_BUILD_PLAN.md` tracked on `main`.
- A direct `git ls-files` check for the listed private paths plus
  `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md` returned no
  tracked files.

Not run:

- No npm or cargo build was run because this DIFF did not change runtime, UI,
  Rust, dependency, Docker Compose, `.env`, or migration files.
- No services were started, stopped, or restarted.
