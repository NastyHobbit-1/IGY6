# DIFF-053: Worker Chunk Vector Upsert

Status: Locked

## Type

Change-bearing.

## Objective

Implement a worker task that embeds existing chunks with the deterministic local
hash embedding helper and upserts chunk vectors into Qdrant.

This DIFF only allows worker-backed chunk vector upserts. It does not authorize
external embedding models, graph sync, retrieval planning, generated chat,
report generation, API-side Celery dispatch, or self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-052 are locked.
- DIFF-036 added API-side deterministic Qdrant chunk vector upsert.
- DIFF-052 added worker-side chunk and evidence generation.
- Chunks already have `embedding_status`.
- The worker has database access from DIFF-051.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-053-worker-chunk-vector-upsert.md`
- `services/worker/app/config.py`
- `services/worker/app/tasks.py`
- `services/worker/requirements.txt`
- `docs/api.md`

Allowed behavior:

- Add worker Qdrant settings.
- Add worker HTTP dependency needed to call Qdrant.
- Implement `memory.vector.upsert_chunks`.
- Ensure the configured Qdrant chunk collection exists.
- Select chunks whose `embedding_status` is not `completed`.
- Embed chunk text using deterministic local hash embeddings only.
- Upsert Qdrant points with chunk/document metadata.
- Mark successfully upserted chunks `completed`.
- Optionally update a supplied `chunk_vector_upsert` work item state.
- Record audit events for completion or failure.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API route changes.
- Database model changes.
- Migration changes.
- API-side Celery dispatch.
- External embedding model calls.
- Semantic search changes.
- Retrieval planning.
- Chat answer generation.
- Graph upserts.
- Report generation.
- Self-improvement queue creation.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-053` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted worker smoke checks should validate:

- Worker task exposes `memory.vector.upsert_chunks`.
- Local embedding is deterministic and normalized for non-empty text.
- Qdrant collection payload shape matches the API behavior.
- Qdrant point payload includes chunk/document metadata.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: `memory.vector.upsert_chunks` is registered with Celery.
- Passed targeted venv smoke: local embedding returns the requested vector length and normalized magnitude for non-empty text.
- Passed targeted venv smoke: Qdrant collection payload uses cosine distance.
- Passed targeted venv smoke: Qdrant point payload preserves chunk/document metadata.

## Completion Criteria

This DIFF is complete when:

- Worker chunk vector upsert task exists.
- Worker uses deterministic local embeddings only.
- Worker ensures Qdrant collection before upserting.
- Worker updates chunk embedding status after successful upsert.
- New behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker graph sync.
- Automatic task dispatch.
- Production embedding model selection.
- Retrieval planning and evidence-backed chat generation.
