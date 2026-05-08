# DIFF-041: Web Source Collection Inventory

Status: Locked

## Type

Change-bearing.

## Objective

Replace the Phase 0-only web status screen with a read-only operational
inventory that shows API readiness plus source, collection-run, and raw artifact
summaries fetched only through FastAPI.

This DIFF makes existing Phase 1 collection state inspectable without adding
frontend write actions or direct storage/database access.

## Baseline Facts

- DIFF-000 through DIFF-040 are locked.
- The web app currently renders a minimal health status page.
- FastAPI exposes read endpoints for health, sources, collection runs, and raw
  artifact metadata.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-041-web-source-collection-inventory.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/user-guide.md`

Allowed behavior:

- Fetch `GET /health/ready` from FastAPI.
- Fetch `GET /sources` from FastAPI.
- Fetch `GET /collection-runs` from FastAPI.
- Fetch `GET /artifacts` from FastAPI.
- Render read-only status, source, collection-run, and raw artifact summaries.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Backend changes.
- Database model changes.
- Migration changes.
- Frontend POST, PUT, PATCH, or DELETE calls.
- Direct PostgreSQL, artifact-store, local file, Qdrant, Neo4j, Redis, MLflow,
  or Phoenix access from the frontend.
- Source creation.
- Collection execution.
- Upload.
- Normalization.
- Worker scheduling.
- Approval decisions.
- Chat or answer generation.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-041` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- Passed after installing existing web dependencies: `npm --prefix apps/web run build`
- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

This DIFF is complete when:

- Web UI shows API readiness.
- Web UI shows read-only source summaries.
- Web UI shows read-only collection-run summaries.
- Web UI shows read-only raw artifact summaries.
- Frontend calls only FastAPI read endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Evidence explorer UI.
- Memory and analysis inspector UI.
- Chat retrieval preview UI.
- Any create, upload, approval, or collection execution controls.
