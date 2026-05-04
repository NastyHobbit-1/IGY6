# DIFF-037: Semantic Chunk Search API

Status: Active

## Type

Change-bearing.

## Objective

Add a minimal semantic chunk search API backed by the configured Qdrant chunk
collection and deterministic local query embedding.

This DIFF does not authorize chat integration, answer generation, external
model calls, retrieval planning, graph traversal, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-036 are locked.
- Chunk vectors can be upserted to Qdrant with local deterministic embeddings.
- No semantic search API exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-037-semantic-chunk-search-api.md`
- `docs/api.md`
- `services/api/app/vector_memory.py`

Allowed behavior:

- Add `POST /memory/vector/chunks/search`.
- Embed the query with the local deterministic embedding helper.
- Query Qdrant for nearest chunk vectors.
- Return bounded chunk IDs, document IDs, scores, and payloads.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- External model calls.
- Chat or answer generation.
- Retrieval planning beyond direct vector search.
- Neo4j traversal.
- Source collection.
- Artifact reads or writes.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-037` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.vector_memory import qdrant_search_payload; print(qdrant_search_payload([0.1, 0.2], 3)['limit'])"
```

## Completion Criteria

This DIFF is complete when:

- Semantic chunk search route exists.
- Query embedding uses the local deterministic helper.
- Results are bounded.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Evidence-backed chat.
- Retrieval planning across PostgreSQL, Qdrant, and Neo4j.
- Production embedding model selection.
