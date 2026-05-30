# IGY6 Branch Policy

## Purpose

This document defines the current branch boundary for IGY6.

## Main Branch

`main` is the clean product/runtime branch.

`main` may contain:

- runtime source code;
- product documentation;
- DIFF records;
- verification scripts;
- runtime configuration templates;
- archived legacy runtime source kept for history and rollback.

`main` must not contain:

- build-agent private prompts;
- Codex-only operating prompts;
- root `AGENTS.md` build instructions;
- `.codex`;
- private implementation instructions;
- non-runtime coordination scratch files.

## Dev Branch

`dev` is the ongoing work branch for development and build-agent assisted work.

`dev` may contain development-only build instructions and agent coordination
material.

Do not remove private/dev/build instruction files from `dev`.

`dev` must not be merged directly into `main`.

Do not merge `main` into `dev` unless the owner explicitly instructs it.

Do not cherry-pick cleanup commits from `main` into `dev` unless the owner
explicitly instructs it.

## Main Branch Promotion

`main` is the public/runtime-clean branch.

Later, only necessary public/runtime-safe files should be selectively promoted
to `main` when the owner explicitly requests promotion. Any promotion must
exclude private/dev/build instruction files.

## Forbidden Main Files

The following are not allowed on `main` unless a later DIFF explicitly changes this policy:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/agents/CODEX_DIFF_123_129_PROMPT.md`
- `docs/agents/CODEX_DIFF_130_OLLAMA_ROUTING_PROMPT.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/plans/DEV_BRANCH_POLICY.md`
- `docs/plans/IGY6_DEV_BUILD_PLAN.md`

Private/dev/build instruction files stay on `dev`.
