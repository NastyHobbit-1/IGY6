# DIFF-182: Public Codex Workflow Instructions For Main-Only Policy

Status: Complete

## Type

Documentation / governance cleanup

## Objective

Add a tracked, public-safe instruction document that tells Codex and future
coding agents how to work in IGY6 after DIFF-181.

The instructions must reflect the main-only workflow and must not depend on
`AGENTS.md` being tracked.

## Baseline Facts

- `main` is the normal working branch after DIFF-181.
- `dev` is obsolete as the normal working branch and remains only as an old
  archive branch unless the owner explicitly says otherwise.
- Private/build-agent/local coordination files are ignored by `.gitignore` and
  must not be added to `main`.
- `AGENTS.md` is local-only/ignored and must not be relied on as the tracked
  instruction source on `main`.

## Allowed Scope

- Add `docs/CODEX_WORKFLOW.md`.
- Add this DIFF record.
- Update `README.md` only to point Codex/future agents to
  `docs/CODEX_WORKFLOW.md`.
- Update `docs/BRANCH_POLICY.md` only if needed to reference
  `docs/CODEX_WORKFLOW.md`.
- Update this DIFF Result and Verification Result.

## Prohibited Scope

- Do not merge `dev` into `main`.
- Do not delete `dev`.
- Do not add `AGENTS.md` to `main`.
- Do not add `.codex` to `main`.
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
- Do not start, stop, or restart services.

## Required Public Workflow Content

`docs/CODEX_WORKFLOW.md` must state that Codex:

1. must work from `main` unless the owner explicitly requests a feature branch;
2. must verify repo state before editing with `git status --short`,
   `git branch --show-current`, and `git log --oneline --decorate -6`;
3. must identify the active DIFF under `docs/diffs/` before changing files;
4. must obey the active DIFF allowed/prohibited scope;
5. must not merge `dev` into `main`;
6. must treat `dev` as obsolete/archive-only unless the owner explicitly says
   otherwise;
7. must not add ignored private/build-agent files to `main`;
8. must not create or track `AGENTS.md` on `main`;
9. must not add `.codex`,
   `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, private
   `docs/agents` prompts, or private `docs/plans` files;
10. must keep runtime/private data out of the repo;
11. must not change `.env`, runtime data, Docker volumes, databases, Qdrant,
    Neo4j, Redis, MLflow, or Phoenix unless an active DIFF explicitly allows it;
12. must not change runtime/UI/backend behavior unless the active DIFF allows
    it;
13. must include active branch, DIFF ID, files changed, summary, verification
    commands/results, prohibited scope avoided, and commit readiness in final
    responses;
14. does not need a clean cherry-pick promotion branch for routine work on
    `main`;
15. may recommend a feature branch from `main` and PR back to `main` for risky
    or larger work.

## Result

Completed.

- Added `docs/CODEX_WORKFLOW.md` as the tracked, public-safe workflow
  instruction source for Codex and future coding agents.
- Documented main-first work, active-DIFF scope discipline, obsolete/archive
  `dev` posture, private ignored-file boundaries, runtime/private data
  boundaries, behavior-change limits, and final-response requirements.
- Updated `README.md` to link to `docs/CODEX_WORKFLOW.md`.
- Updated `docs/BRANCH_POLICY.md` to point to `docs/CODEX_WORKFLOW.md`.
- Did not touch `AGENTS.md`, `.codex`, private `docs/agents` prompt files, or
  private `docs/plans` files.

## Verification Result

Passed:

- `git fetch --all --prune` completed.
- `git switch main` confirmed the working branch is `main`. The first
  sandboxed attempt could not create `.git/index.lock`; rerunning with approved
  git switch permissions succeeded and reported `Already on 'main'`.
- `git pull --ff-only origin main` completed and reported `Already up to date`.
- `git status --short` showed only `README.md`, `docs/BRANCH_POLICY.md`,
  `docs/CODEX_WORKFLOW.md`, and this DIFF record changed.
- `git branch --show-current` returned `main`.
- `git log --oneline --decorate -6` showed `main` at DIFF-181 before these
  working-tree edits.
- `git diff --name-status` showed tracked edits only in `README.md` and
  `docs/BRANCH_POLICY.md`; `git status --short` showed the two new untracked
  docs files to be added for DIFF-182.
- `git diff --check` passed.
- `git check-ignore -v .codex AGENTS.md Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents/example.md docs/plans/DEV_BRANCH_POLICY.md docs/plans/IGY6_DEV_BUILD_PLAN.md`
  confirmed all required private/build-agent paths are ignored by `.gitignore`.
- `git ls-files .codex AGENTS.md Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans/DEV_BRANCH_POLICY.md docs/plans/IGY6_DEV_BUILD_PLAN.md`
  returned only the existing public `docs/agents/README.md`; it did not show
  `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/plans/DEV_BRANCH_POLICY.md`, or
  `docs/plans/IGY6_DEV_BUILD_PLAN.md` tracked on `main`.
- A direct `git ls-files` check for the listed private prompt files returned no
  tracked files.

Not run:

- No npm or cargo build was run because this DIFF changed documentation only
  and did not touch runtime, UI, or Rust files.
- No services were started, stopped, or restarted.
