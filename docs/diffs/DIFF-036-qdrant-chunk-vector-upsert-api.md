# DIFF-036: Qdrant Chunk Vector Upsert API

Status: Locked

## Type

Change-bearing.

## Objective

Add an API path to embed existing chunks with the deterministic local embedding
helper and upsert those vectors into Qdrant.

This DIFF does not authorize external embedding models, semantic search,
worker scheduling, graph writes, retrieval planning, or chat integration.

## Baseline Facts

- DIFF-000 through DIFF-035 are locked.
- Qdrant collection foundation exists.
- Local deterministic embedding helper exists.
- Chunks have `embedding_status`.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-036-qdrant-chunk-vector-upsert-api.md`
- `docs/api.md`
- `services/api/app/vector_memory.py`

Allowed behavior:

- Add a `POST /memory/vector/chunks/upsert` route.
- Ensure the configured Qdrant chunk collection exists.
- Select chunks from PostgreSQL.
- Embed chunk text with the local deterministic helper.
- Upsert Qdrant points with chunk/document metadata.
- Update `embedding_status` to `completed` for successfully upserted chunks.
- Return upsert counts.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- External model calls.
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

Use `DIFF-036` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.vector_memory import qdrant_points_payload; print(qdrant_points_payload([]))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.vector_memory import qdrant_points_payload; print(qdrant_points_payload([]))"` because the host Python environment does not have `httpx` installed

## Completion Criteria

This DIFF is complete when:

- Chunk vector upsert route exists.
- Route uses local deterministic embeddings only.
- Route updates chunk embedding status after successful upsert.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Semantic search API.
- Worker-backed embedding.
- Production embedding model selection.
- Retrieval planning and chat.
