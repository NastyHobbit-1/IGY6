# DIFF-070: Web MVP Action Console

Status: Locked

## Type

Change-bearing.

## Objective

Add a web operator console for the bounded MVP workflows exposed by the API.

## Baseline Facts

- DIFF-000 through DIFF-069 are locked.
- The web UI currently shows read-only inventory and retrieval preview.
- Backend routes exist for source creation, approvals, dry-run, collection,
  work dispatch, evidence answers, feedback, outcomes, pattern detection, and
  report rendering.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-070-web-mvp-action-console.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `README.md`

Allowed behavior:

- Add browser-side forms that call existing FastAPI endpoints.
- Keep all data access through the API.
- Display operation results/errors in the page.
- Add minimal styles for the action console.
- Document the web operator console.

## Prohibited Scope

This DIFF does not allow new backend routes, direct database access from the
frontend, direct Qdrant/Neo4j/Redis/MLflow/Phoenix access from the frontend,
dependency changes, Docker changes, or broad UI redesign.

## Required Tags

Use `DIFF-070` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Completed verification:

```bash
npm --prefix apps/web run build
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

## Completion Criteria

This DIFF is complete when the web UI exposes the main MVP operator workflows
through FastAPI-only controls, verification passes, and no prohibited access is
introduced.
