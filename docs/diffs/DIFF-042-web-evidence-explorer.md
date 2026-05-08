# DIFF-042: Web Evidence Explorer

Status: Locked

## Type

Change-bearing.

## Objective

Add a read-only evidence explorer section to the web UI so existing normalized
documents, chunks, evidence items, and claims can be inspected from FastAPI.

This DIFF exposes existing evidence records without adding create/generate
actions, backend changes, retrieval, chat, graph traversal, or direct artifact
access.

## Baseline Facts

- DIFF-000 through DIFF-041 are locked.
- The web UI shows API readiness, sources, collection runs, and raw artifacts.
- FastAPI exposes read endpoints for documents, chunks, evidence items, and
  claims.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-042-web-evidence-explorer.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/user-guide.md`

Allowed behavior:

- Fetch `GET /evidence/documents` from FastAPI.
- Fetch `GET /evidence/chunks` from FastAPI.
- Fetch `GET /evidence/items` from FastAPI.
- Fetch `GET /evidence/claims` from FastAPI.
- Render read-only evidence totals and recent evidence metadata.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Backend changes.
- Database model changes.
- Migration changes.
- Frontend POST, PUT, PATCH, or DELETE calls.
- Evidence document creation.
- Chunk generation.
- Evidence item creation.
- Direct artifact reads or local file reads.
- Direct PostgreSQL, Qdrant, Neo4j, Redis, MLflow, or Phoenix access from the
  frontend.
- Chat or answer generation.
- Retrieval planning.
- Embeddings.
- Graph traversal.
- Worker scheduling.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-042` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- Passed: `npm --prefix apps/web run build`
- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

This DIFF is complete when:

- Web UI shows read-only normalized document summaries.
- Web UI shows read-only chunk summaries.
- Web UI shows read-only evidence item summaries.
- Web UI shows read-only claim summaries.
- Frontend calls only FastAPI read endpoints for evidence.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Memory and analysis inspector UI.
- Chat retrieval preview UI.
- Evidence creation or chunk generation controls.
- Any write or approval flows.
