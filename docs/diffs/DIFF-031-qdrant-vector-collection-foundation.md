# DIFF-031: Qdrant Vector Collection Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal API-managed Qdrant collection foundation for future chunk
embeddings.

This DIFF does not authorize embedding generation, model calls, vector upserts,
semantic search, retrieval planning, graph writes, or chat integration.

## Baseline Facts

- DIFF-000 through DIFF-030 are locked.
- Qdrant is already part of Docker Compose and API readiness checks.
- Chunks now exist in PostgreSQL with `embedding_status`.
- API dependencies already include `httpx`, which can call Qdrant's HTTP API.
- No Qdrant collection-management API exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-031-qdrant-vector-collection-foundation.md`
- `docs/api.md`
- `services/api/app/config.py`
- `services/api/app/main.py`
- `services/api/app/vector_memory.py`

Allowed behavior:

- Add configurable Qdrant chunk collection name and vector size settings.
- Add a small Qdrant HTTP helper for collection status and creation.
- Add API routes to inspect vector collection status and ensure the chunk
  vector collection exists.
- Use cosine distance and a fixed dense vector size.
- Do not write vectors or read chunk text.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Embedding generation.
- External model calls.
- Qdrant point upserts.
- Semantic search.
- Retrieval or chat integration.
- Neo4j writes.
- Source collection.
- Artifact reads or writes.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-031` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.vector_memory import qdrant_collection_payload; print(qdrant_collection_payload(384)['vectors']['distance'])"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.vector_memory import qdrant_collection_payload; print(qdrant_collection_payload(384)['vectors']['distance'])"` because the host Python environment does not have `httpx` installed

## Completion Criteria

This DIFF is complete when:

- Vector-memory API routes exist.
- Qdrant collection status and ensure helpers exist.
- Chunk vector collection config is centralized in settings.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Embedding workers.
- Qdrant point upserts.
- Semantic search API.
- Retrieval planning.
