# IGY6 Branch Policy

## Purpose

This document defines the current branch and local-file policy for IGY6.

For public-safe Codex and coding-agent operating instructions, see
[`docs/CODEX_WORKFLOW.md`](CODEX_WORKFLOW.md).

## Main Branch

`main` is the normal working branch for product, runtime, documentation, and
DIFF-governed development.

`main` may contain:

- runtime source code;
- product documentation;
- DIFF records;
- verification scripts;
- runtime configuration templates;
- archived legacy runtime source kept for history and rollback.

`main` must not contain:

- build-agent private prompts;
- local Codex or agent operating prompts;
- root `AGENTS.md` build instructions;
- `.codex`;
- private implementation instructions;
- non-runtime coordination scratch files.

## Optional Feature Branches

Feature branches may branch from `main` and return to `main` through the normal
review or merge process.

Feature branches must follow the same tracked-file boundary as `main`: private
agent files, local build-agent prompts, and private coordination notes must stay
out of tracked history.

## Local-Only Agent Files

Private/build-agent coordination files are local-only and are ignored by
`.gitignore`.

Ignored local-only paths include:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- `docs/agents/`
- `docs/plans/DEV_BRANCH_POLICY.md`
- `docs/plans/IGY6_DEV_BUILD_PLAN.md`

`.gitignore` prevents new untracked local-only files from being added. It does
not remove files that are already tracked on another branch. Do not merge or
copy tracked private files from another branch into `main`.

## Dev Branch

`dev` is obsolete as the normal working branch.

Do not merge `dev` into `main`.

The old routine `dev` to clean-main to `main` cherry-pick promotion loop is no
longer required for runtime/product changes. Normal work happens on `main`, or
on optional feature branches created from `main`.

The existing `dev` branch may contain private/build-agent planning material.
It may be deleted later only after a separate explicit owner-approved cleanup.
DIFF-181 does not delete the local or remote `dev` branch.

## Forbidden Main Files

The following are not allowed on `main` unless a later DIFF explicitly changes
this policy:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- private prompt files under `docs/agents/`
- `docs/plans/DEV_BRANCH_POLICY.md`
- `docs/plans/IGY6_DEV_BUILD_PLAN.md`

Private/build-agent files must not be committed to `main`, even when normal
development happens on `main`.
