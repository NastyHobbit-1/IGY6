# Dev Branch Policy

## Purpose

This repo uses two branch roles:

- `main` is the clean public/product branch.
- `dev` is the private local working branch for Codex prompts, agent files, coordination notes, and experimental build guidance.

Codex must preserve this separation.

## Branch Rules

### main

`main` must stay clean and product-facing.

Allowed on `main`:

- product source code
- runtime scripts
- user/operator documentation
- configs and templates
- tests
- locked DIFF records
- archived legacy source needed for rollback/history

Not allowed on `main`:

- Codex prompt files
- agent instruction files
- local coordination prompts
- private build notes
- personal workflow notes
- experimental prompt cards
- internal planning prompts that are not product documentation

### dev

`dev` is the local working branch.

Allowed on `dev`:

- everything allowed on `main`
- `docs/agents/`
- Codex prompts
- agent prompt files
- local coordination instructions
- experimental instructions
- temporary build guidance
- private workflow notes

`dev` may contain files that must not be pushed to `main`.

## Required Preflight Before Changes

Before editing files, Codex must run:

    git branch --show-current
    git status --short
    git log --oneline --decorate -6

If the active branch is `main`, Codex must not add or restore prompt/agent files.

If the active branch is `dev`, Codex may use and update prompt/agent files only when the user asks.

## Main Protection Rule

Codex must never move dev-only prompt files into `main`.

Do not merge `dev` into `main` blindly.

Only promote explicitly named product files from `dev` to `main`.

Never promote all of `docs/agents/` to `main`.

## Pulling main into dev

To keep dev updated with the clean product branch:

    git checkout dev
    git merge main

This is allowed.

## Remote Policy

`dev` should remain local unless the user explicitly says to push it.

Do not run:

    git push origin dev

unless the user explicitly requests it.

If `dev` accidentally gets an upstream, remove it:

    git branch --unset-upstream dev

## Prompt and Agent File Policy

Dev-only prompt files include, but are not limited to:

    docs/agents/AGENT_PROMPT.md
    docs/agents/AGENT_PROMPT_CODING.md
    docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md
    docs/agents/DIFF-*_*.md
    docs/agents/CODEX_*_PROMPT.md

These may exist on `dev`.

These should not appear on `main`.

## README Policy

The root `README.md` on `main` must describe the actual IGY6 product.

It should not primarily describe:

- how the user instructed the build
- Codex prompt history
- internal agent workflow
- private coordination notes
- migration prompts

## DIFF Governance

DIFF governance still applies.

Rules:

- one active DIFF at a time
- locked DIFFs are never edited
- no skipped/mixed DIFF work
- every DIFF must leave the repo runnable
- runtime/private data must not be committed
- `.env` must not be mutated unless explicitly scoped

## Runtime Data Rule

Runtime/private data belongs outside the repository under `IGY6_DATA_ROOT`.

Do not commit `.env`, storage roots, artifacts, private exports, credentials, tokens, cookies, collected personal data, or Docker volume data.

## Codex Behavior Requirements

Codex must:

1. Check the active branch before changes.
2. Respect the branch role.
3. Keep `main` clean and product-facing.
4. Keep prompt/agent files on `dev` only.
5. Never blindly merge `dev` into `main`.
6. Never push `dev` unless explicitly requested.
7. Keep README product-facing.
8. Keep DIFF records locked once completed.
9. Ask before destructive branch, history, or remote operations.
