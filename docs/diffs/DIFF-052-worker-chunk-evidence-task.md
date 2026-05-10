# DIFF-052: Worker Chunk And Evidence Task

Status: Locked

## Type

Change-bearing.

## Objective

Implement a worker task that deterministically creates chunks and evidence
items for existing normalized documents.

This DIFF only allows chunk and evidence generation from already-normalized
documents. It does not authorize embeddings, vector upserts, graph sync,
external model calls, API route changes, report generation, or
self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-051 are locked.
- The API already has a deterministic chunk/evidence generation route.
- DIFF-051 added worker-side normalized document creation.
- Chunks and evidence items already exist in the database schema.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-052-worker-chunk-evidence-task.md`
- `services/worker/app/tasks.py`
- `docs/api.md`

Allowed behavior:

- Implement `evidence.generate_document_chunks`.
- Split normalized document text into deterministic fixed-size chunks.
- Create one evidence item for each generated chunk.
- Reject empty document text.
- Skip documents that already have chunks.
- Optionally update a supplied `document_chunking` work item state.
- Record audit events for chunk/evidence generation completion or failure.
- Document the worker task and limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API route changes.
- Database model changes.
- Migration changes.
- API-side Celery dispatch.
- Embedding or vector upserts.
- Graph upserts.
- Report generation.
- Self-improvement queue creation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-052` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted worker smoke checks should validate:

- Worker task exposes `evidence.generate_document_chunks`.
- Chunk splitting is deterministic.
- Empty text is rejected.
- Existing chunks cause a skip path.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: `evidence.generate_document_chunks` is registered with Celery.
- Passed targeted venv smoke: `_split_text_chunks("abcdef", 2)` returns `["ab", "cd", "ef"]`.
- Passed targeted venv smoke: `_split_text_chunks("", 100)` returns `[]`, supporting the task's empty-document rejection path.
- Existing-chunk skip behavior is implemented by checking for an existing chunk before inserts; live database smoke was not run.

## Completion Criteria

This DIFF is complete when:

- Worker chunk/evidence task exists.
- Task creates deterministic chunks and evidence items for normalized documents.
- Task skips already-chunked documents.
- Task writes audit events for completion/failure.
- New behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker embedding upserts.
- Worker graph sync.
- Automatic task dispatch.
- UI controls for worker execution.
- Retry scheduling and broader idempotency controls.
