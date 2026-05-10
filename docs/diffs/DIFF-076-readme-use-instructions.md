# DIFF-076: README Use Instructions

Status: Locked

## Type

Change-bearing documentation

## Objective

Make `README.md` accurate and detailed enough for a user or coding agent to
understand what IGY6 can currently do, what it cannot do yet, how to run it
locally, how to use the current UI/API workflows, and how to verify or
troubleshoot the local stack.

## Baseline Facts

- The worktree was clean before this DIFF started.
- No active or in-progress DIFF existed before this DIFF.
- IGY6 currently exposes a Next.js web UI, FastAPI API, PostgreSQL-backed state
  tables, Redis/Celery workers, Qdrant vector memory foundation, Neo4j graph
  memory foundation, and reserved MLflow/Phoenix services through local Docker
  Compose.
- The implemented chat behavior is deterministic local retrieval preview and
  deterministic evidence-summary packets. It does not generate LLM answers.
- Manual upload collection validates UTF-8 text before queuing normalization.
- Local project collection stores scoped files from container-visible paths, and
  worker normalization currently supports UTF-8 text artifacts only.
- Work-item dispatch currently supports `collection_normalization`,
  `document_chunking`, and `chunk_vector_upsert`.
- The current web UI is a dark AI-console visual shell that preserves existing
  IGY6 data loading and FastAPI-backed controls.

## Allowed Scope

- `docs/diffs/DIFF-076-readme-use-instructions.md`
- `README.md`

## Prohibited Scope

- No backend code changes.
- No frontend code changes.
- No Docker changes.
- No migration changes.
- No dependency changes.
- No package or script changes.
- No API behavior changes.
- No database model changes.
- No feature implementation.
- No broad cleanup.
- No unrelated documentation files unless a README link is broken and the user
  is notified first.

## Required Tags

- Commit message must include `DIFF-076`.
- Final response must identify `DIFF-076`.

## Verification

- `git diff --check`
- `python3 -m compileall services/api services/worker`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  when Docker Compose is available.

## Completion Criteria

- README has detailed current-use instructions.
- README accurately separates current capabilities from not-yet-implemented
  features.
- README gives realistic startup, usage, API, UI, troubleshooting, and
  verification instructions.
- No code or behavior was changed.
- No dependencies were added.
- Prohibited scope was avoided.
- Verification results are recorded below.

## Verification Result

- `git diff --check` passed.
- `python3 -m compileall services/api services/worker` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- Full stack start was not run because this DIFF is documentation-only and
  Compose configuration validation was sufficient for the requested scope.
