# DIFF-002: Source Registry API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal policy-aware source registry API for Phase 1 so authorized
sources and permissions can be recorded before any future collection work.

This DIFF does not authorize real collection, dry-runs, ingestion,
normalization execution, embeddings, graph writes, chat, predictions,
recommendations, or self-improvement execution.

## Baseline Facts

- DIFF-000 and DIFF-001 are locked.
- Source and source permission tables already exist from Phase 0.
- Audit events already exist from Phase 0.
- The Phase 1 plan requires a source registry and permission model before
  connectors or ingestion.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-002-source-registry-api.md`
- `docs/api.md`
- `services/api/app/main.py`
- `services/api/app/sources.py`

Allowed behavior:

- Add Pydantic request/response models for source registry operations.
- Add FastAPI routes to create and list sources.
- Add FastAPI routes to create and list source permissions.
- Record audit events for source and permission creation.
- Keep the API local-first and database-backed through the existing SQLAlchemy
  session.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Real source collection.
- Dry-run execution.
- Browser automation.
- Artifact writes.
- Normalization execution.
- Embeddings.
- Qdrant writes.
- Neo4j writes.
- Chat or reasoning routes.
- Prediction or recommendation execution.
- Self-improvement execution.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-002` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- Source registry routes are available through FastAPI.
- Source permission routes are available through FastAPI.
- Source and permission creation write audit events.
- API docs describe the new endpoints and Phase 1 boundaries.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Source dry-run workflow.
- Manual upload connector.
- Local project connector.
- Evidence inspection API.
- Web UI for source management.
