# DIFF-015: Chunk Read API

Status: Locked

## Type

Change-bearing.

## Objective

Add read-only API routes for inspecting normalized document chunks already
present in PostgreSQL.

This DIFF does not authorize chunk creation, normalization execution,
embedding, retrieval ranking, Qdrant writes, or source collection.

## Baseline Facts

- DIFF-000 through DIFF-014 are locked.
- The `chunks` table exists from DIFF-001.
- DIFF-009 added read-only evidence routes for documents, evidence items, and
  claims, but not chunks.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-015-chunk-read-api.md`
- `docs/api.md`
- `services/api/app/evidence.py`

Allowed behavior:

- Add read-only routes to list and retrieve chunks.
- Use existing SQLAlchemy models and session handling.
- Document the endpoints and read-only boundary.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Chunk creation or mutation.
- Normalization execution.
- Embeddings.
- Qdrant writes.
- Retrieval ranking.
- Source collection.
- Artifact writes.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-015` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker services/collectors
  packages/policy` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- Read-only chunk routes exist.
- API docs list the chunk endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Chunk creation.
- Embedding status transitions.
- Retrieval/search endpoints.
- Evidence UI.
