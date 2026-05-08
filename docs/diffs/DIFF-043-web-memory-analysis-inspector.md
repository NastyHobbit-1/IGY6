# DIFF-043: Web Memory Analysis Inspector

Status: Locked

## Type

Change-bearing.

## Objective

Add a read-only memory and analysis inspector to the web UI showing vector
collection status, graph schema status, and existing pattern, hypothesis,
prediction, and recommendation records from FastAPI.

This DIFF makes existing memory and analysis state inspectable without adding
write actions, graph traversal controls, vector upserts, answer generation, or
retrieval planning.

## Baseline Facts

- DIFF-000 through DIFF-042 are locked.
- The web UI shows readiness, source/collection/artifact inventory, and an
  evidence explorer.
- FastAPI exposes read endpoints for vector collection status, graph schema
  status, and analysis records.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-043-web-memory-analysis-inspector.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/user-guide.md`

Allowed behavior:

- Fetch `GET /memory/vector/chunks` from FastAPI.
- Fetch `GET /memory/graph/schema` from FastAPI.
- Fetch `GET /analysis/patterns` from FastAPI.
- Fetch `GET /analysis/hypotheses` from FastAPI.
- Fetch `GET /analysis/predictions` from FastAPI.
- Fetch `GET /analysis/recommendations` from FastAPI.
- Render read-only memory and analysis summaries.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Backend changes.
- Database model changes.
- Migration changes.
- Frontend POST, PUT, PATCH, or DELETE calls.
- Vector collection ensure or vector upsert actions.
- Graph schema ensure, lineage sync, or graph relationship traversal controls.
- Direct PostgreSQL, artifact-store, local file, Qdrant, Neo4j, Redis, MLflow,
  or Phoenix access from the frontend.
- Chat or answer generation.
- Retrieval planning.
- Prediction or recommendation generation.
- Worker scheduling.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-043` in change summaries, commits, and review notes for this work.

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

- Web UI shows read-only vector memory collection status.
- Web UI shows read-only graph schema status.
- Web UI shows read-only pattern, hypothesis, prediction, and recommendation
  summaries.
- Frontend calls only FastAPI read endpoints for memory and analysis.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Chat retrieval preview UI.
- Semantic search UI.
- Graph relationship inspection UI.
- Any write, ensure, sync, generation, or approval controls.
