# DIFF-051: Worker Normalization Task

Status: Locked

## Type

Change-bearing.

## Objective

Implement the first worker execution path for collection normalization work
items by converting collected UTF-8 raw artifacts into normalized document rows.

This DIFF only allows deterministic UTF-8 artifact normalization. It does not
authorize chunk generation, evidence generation, embeddings, graph sync,
external model calls, report generation, or self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-050 are locked.
- DIFF-050 records `collection_normalization` work items after successful
  non-dry-run collection.
- Raw artifacts store metadata in PostgreSQL and bytes in the local
  content-addressed artifact store.
- The API already has a synchronous normalized document creation route for a
  single raw artifact.
- The worker currently has a non-executing normalization scaffold task.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-051-worker-normalization-task.md`
- `services/api/app/collection_runs.py`
- `services/worker/app/config.py`
- `services/worker/app/tasks.py`
- `services/worker/requirements.txt`
- `docs/api.md`

Allowed behavior:

- Add worker database and artifact-store settings.
- Add worker Python dependencies needed for PostgreSQL access.
- Implement `collection.normalize_collection_run`.
- Update new collection normalization work item payloads to reference the real
  worker task and mark normalization execution as available.
- Validate the work item, collection run, and raw artifact IDs.
- Read only artifact bytes referenced by the work item payload.
- Decode UTF-8 artifacts into normalized document rows.
- Skip raw artifacts that already have normalized documents.
- Mark work items `running`, `completed`, or `failed`.
- Record audit events for normalization completion or failure.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API route changes.
- Database model changes.
- Migration changes.
- Chunk generation.
- Evidence item generation.
- Embedding or vector upserts.
- Graph upserts.
- Report generation.
- Self-improvement queue creation.
- External model calls.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-051` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted worker smoke checks should validate:

- Worker task imports with local `.venv` dependencies.
- Worker normalization helper rejects non-UTF-8 artifact bytes.
- Worker normalization helper returns a deterministic normalized document title.
- Worker task exposes `collection.normalize_collection_run`.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: worker imports with local `.venv` dependencies.
- Passed targeted venv smoke: `collection.normalize_collection_run` is registered with Celery.
- Passed targeted venv smoke: UTF-8 artifact bytes decode successfully.
- Passed targeted venv smoke: non-UTF-8 artifact bytes raise `Artifact is not UTF-8 text`.
- Passed targeted venv smoke: document title derivation uses artifact filename metadata.

## Completion Criteria

This DIFF is complete when:

- Worker settings include database and artifact-store paths.
- Worker requirements include required DB dependencies.
- `collection.normalize_collection_run` can normalize UTF-8 raw artifacts.
- Work item status and audit events are updated for completion/failure.
- New behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker chunk and evidence generation.
- Worker embedding upserts.
- Worker graph sync.
- API-side Celery dispatch if explicitly allowed.
- Retry scheduling and idempotency hardening beyond duplicate document skips.
