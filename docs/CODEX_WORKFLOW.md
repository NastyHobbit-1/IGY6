# IGY6 Codex Workflow

This document is the tracked, public-safe workflow instruction source for
Codex and future coding agents working in IGY6.

`AGENTS.md` is local-only and ignored. Do not rely on it as the tracked
instruction source on `main`.

## Branch Rules

- Work from `main` unless the owner explicitly requests a feature branch.
- For routine work on `main`, no clean cherry-pick promotion branch is required.
- For risky or larger work, recommend a feature branch from `main` and a pull
  request back to `main`.
- Do not merge `dev` into `main`.
- Treat `dev` as obsolete/archive-only unless the owner explicitly says
  otherwise.

## Before Editing

Verify repo state before changing files:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -6
```

Identify the active DIFF under `docs/diffs/` before changing files.

Read the active DIFF and obey its allowed and prohibited scope. If a requested
change is outside the active DIFF, do not make it unless the owner updates the
DIFF scope.

## Private And Runtime Data Boundaries

Do not add ignored private/build-agent files to `main`.

Do not create or track `AGENTS.md` on `main`.

Do not add these private/local files or paths to `main`:

- `.codex`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- private prompt files under `docs/agents/`
- `docs/plans/DEV_BRANCH_POLICY.md`
- `docs/plans/IGY6_DEV_BUILD_PLAN.md`

Keep runtime/private data out of the repository.

Do not change `.env`, runtime data, Docker volumes, databases, Qdrant, Neo4j,
Redis, MLflow, or Phoenix unless the active DIFF explicitly allows it.

## Product And Runtime Scope

Do not change runtime, UI, or backend behavior unless the active DIFF allows it.

Do not change Rust crates, Next.js UI code, Docker Compose, migrations, or
service behavior unless the active DIFF explicitly allows those changes.

Do not start, stop, or restart services unless the active DIFF explicitly
requires it.

## Final Response

Codex final responses must include:

- active branch;
- DIFF ID;
- files changed;
- summary of changes;
- verification commands run and results;
- prohibited scope avoided;
- whether the work is ready to commit.
