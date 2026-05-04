# DIFF-035: Local Embedding Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add a deterministic local embedding helper for text chunks so Phase 2 can
exercise vector-memory flows without external model calls or new dependencies.

This DIFF does not authorize Qdrant point upserts, semantic search endpoints,
external model calls, worker scheduling, or production embedding model
selection.

## Baseline Facts

- DIFF-000 through DIFF-034 are locked.
- Qdrant collection foundation exists.
- Chunks exist in PostgreSQL and start with `embedding_status="not_started"`.
- No embedding helper exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-035-local-embedding-foundation.md`
- `services/api/app/vector_memory.py`

Allowed behavior:

- Add a deterministic local text-to-vector helper.
- Use only Python standard-library hashing/math.
- Match the configured Qdrant chunk vector size.
- Normalize output vectors for cosine distance.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Qdrant point upserts.
- Semantic search.
- External model calls.
- Dependency changes.
- Source collection.
- Artifact reads or writes.
- Graph writes.
- Chat or retrieval integration.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-035` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.vector_memory import embed_text_local; vector = embed_text_local('hello world', 16); print(len(vector), round(sum(value * value for value in vector), 3))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.vector_memory import embed_text_local; vector = embed_text_local('hello world', 16); print(len(vector), round(sum(value * value for value in vector), 3))"` because the host Python environment does not have `httpx` installed

## Completion Criteria

This DIFF is complete when:

- Deterministic local embedding helper exists.
- Helper returns the requested vector size.
- Helper produces normalized nonzero vectors for non-empty text.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Qdrant point upserts.
- Semantic search.
- Worker-backed embedding.
- Production embedding model selection.
