# DIFF-050: Collection Enqueue Scaffold

Status: Locked

## Type

Change-bearing.

## Objective

Record a durable normalization work item after successful non-dry-run
collection so collected artifacts have an auditable next-step queue marker.

This DIFF only adds a queue scaffold. It does not implement normalization,
Celery dispatch from the API, embeddings, graph sync, report generation, or
self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-049 are locked.
- Manual upload and local project collection create raw artifacts.
- Collection summaries currently say `would_normalize: False` and
  `would_enqueue_worker: False`.
- Existing `WorkItem` records can persist queued work intent.
- The worker currently only exposes a health task.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-050-collection-enqueue-scaffold.md`
- `services/api/app/collection_runs.py`
- `services/worker/app/tasks.py`
- `docs/api.md`

Allowed behavior:

- Create a `WorkItem` after successful manual upload collection.
- Create a `WorkItem` after successful local project collection.
- Include collection run ID and raw artifact IDs in the work item payload.
- Add audit events for the queued normalization work item.
- Add a worker task stub for future collection normalization work.
- Document the queue scaffold and limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Dependency changes.
- API-side Celery dispatch.
- Normalization execution.
- Chunk or evidence generation.
- Embedding or vector upserts.
- Graph upserts.
- Report generation.
- Self-improvement queue creation.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-050` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted API smoke checks should validate:

- Normalization work item payloads include collection run ID and artifact IDs.
- Manual upload collection summary can include a queued normalization work item.
- Local project collection summary can include a queued normalization work item.
- Worker task stub returns a non-executing scaffold response.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: normalization work item helper creates `collection_normalization` work item payload with collection run ID, source permission ID, artifact IDs, artifact count, scaffold flags, and audit data.
- Passed targeted docs/source check: collection summaries expose `would_enqueue_worker: False` and `normalization_work_item_id`.
- Blocked targeted worker task runtime smoke: local `.venv` does not have `celery` installed, so importing `services/worker/app/tasks.py` fails at `from celery import Celery`. Compile verification still passed.

## Completion Criteria

This DIFF is complete when:

- Successful non-dry-run collection records a normalization work item.
- Collection summaries expose the queued work item ID.
- Work item creation is audited.
- Worker task stub exists.
- New behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker normalization execution.
- Worker chunk and evidence generation.
- Worker embedding upserts.
- Worker graph sync.
- API-side or worker-side Celery dispatch behavior if explicitly allowed.
