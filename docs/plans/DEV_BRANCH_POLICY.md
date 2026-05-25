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

## Current Branch Meaning

Expected posture:

    main = clean product repo
    dev  = local working branch with prompts and agent files restored

If Codex sees `docs/agents/` prompt files on `dev`, that is expected.

If Codex sees those files on `main`, that is a problem unless the user explicitly approves it.

## Required Preflight Before Changes

Before editing files, Codex must run:

    git branch --show-current
    git status --short
    git log --oneline --decorate -6

Codex must identify which branch is active.

If the active branch is `main`, Codex must not add or restore prompt/agent files.

If the active branch is `dev`, Codex may use and update prompt/agent files only when the user asks.

## Main Protection Rule

Codex must never move dev-only prompt files into `main`.

Do not merge `dev` into `main` blindly.

Do not run:

    git checkout main
    git merge dev

unless the user explicitly confirms that dev-only files have been removed or excluded.

Preferred promotion path from `dev` to `main`:

    git checkout main
    git pull origin main
    git checkout dev -- <specific product files only>
    git status --short
    git diff --check
    git commit -m "<scoped product change>"
    git push origin main

Only promote explicitly named product files.

Never promote all of `docs/agents/` to `main`.

## Pulling main into dev

To keep dev updated with the clean product branch:

    git checkout dev
    git merge main

This is allowed.

If conflicts occur, preserve:

- product-facing files from `main` when they are cleaner/current
- dev-only prompt/agent files from `dev`

## Remote Policy

`dev` should remain local unless the user explicitly says to push it.

Do not run:

    git push origin dev

unless the user explicitly requests it.

If `dev` accidentally gets an upstream, remove it:

    git branch --unset-upstream dev

If `origin/dev` exists and the user did not intend it, stop and ask before deleting the remote branch.

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

The README may mention contributor governance briefly, but the main focus must be:

- what IGY6 is
- what it does
- how to run it
- how to use it
- current supported features
- current limitations
- where user/operator docs live

## DIFF Governance

DIFF governance still applies.

Rules:

- one active DIFF at a time
- locked DIFFs are never edited
- no skipped/mixed DIFF work
- every DIFF must leave the repo runnable
- runtime/private data must not be committed
- `.env` must not be mutated unless explicitly scoped

Branch policy does not override DIFF governance.

## Runtime Data Rule

Runtime/private data belongs outside the repository under `IGY6_DATA_ROOT`.

Do not commit:

- `.env`
- storage roots
- artifacts
- private exports
- credentials
- tokens
- cookies
- collected personal data
- Docker volume data

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

## Safe Status Check

Use this anytime branch state is unclear:

    git branch
    git branch -r
    git status --short
    git log --oneline --decorate -8
    ls docs/agents 2>/dev/null || true

Expected local development state:

    * dev
      main

Expected public branch state:

    main tracks origin/main
    dev has no upstream unless explicitly requested
