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

`dev` may contain development-only build instructions and agent coordination material.

`dev` must not be merged directly into `main`.

Runtime/product changes from `dev` must be moved to `main` through a clean branch or cherry-pick that excludes dev-only instruction files.

## Clean Promotion Rule

When a runtime/product DIFF is completed on `dev`, promote it to `main` by:

1. creating a clean branch from `main`;
2. cherry-picking only the runtime/product commit;
3. verifying forbidden files are absent;
4. opening a pull request from the clean branch into `main`;
5. merging only that clean branch.

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

Tracked product docs such as `docs/agents/README.md` and `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md` are allowed when they do not contain build-agent private instructions.
