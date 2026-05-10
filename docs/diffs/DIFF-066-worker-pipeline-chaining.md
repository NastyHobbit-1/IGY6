# DIFF-066: Worker Pipeline Chaining

Status: Locked

## Type

Change-bearing.

## Objective

Have worker tasks create the next queued work item in the ingestion pipeline so
MVP ingestion can progress from normalization to chunking to vector upsert.

## Baseline Facts

- DIFF-000 through DIFF-065 are locked.
- Collection creates queued `collection_normalization` work items.
- `collection.normalize_collection_run` creates normalized documents and stops.
- `evidence.generate_document_chunks` creates chunks/evidence and stops.
- `memory.vector.upsert_chunks` already completes vector upsert work items.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-066-worker-pipeline-chaining.md`
- `services/worker/app/tasks.py`
- `docs/api.md`

Allowed behavior:

- After successful normalization, create a queued `document_chunking` work item
  when new documents were created.
- After successful chunking, create a queued `chunk_vector_upsert` work item
  when new chunks were created.
- Use `chunk_ids` from a `chunk_vector_upsert` work item payload when present.
- Record audit events for chained work item creation.
- Include chained work item IDs in task return payloads.
- Document the pipeline chaining.

## Prohibited Scope

This DIFF does not allow automatic Celery dispatch, API behavior changes, new
worker tasks, graph sync changes, vector algorithm changes beyond honoring
explicit `chunk_ids` already stored in a vector work item payload, migrations,
dependency changes, or broad refactors.

## Required Tags

Use `DIFF-066` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate chained work item payload builders.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
sys.path.insert(0, 'services/worker')
from app.tasks import _chained_document_chunking_payload, _chained_vector_upsert_payload

chunking = _chained_document_chunking_payload(['d1', 'd2'], 'w1')
assert chunking['document_ids'] == ['d1', 'd2']
assert chunking['worker_task_name'] == 'evidence.generate_document_chunks'
vector = _chained_vector_upsert_payload(['c1', 'c2'], 'w2')
assert vector['chunk_ids'] == ['c1', 'c2']
assert vector['limit'] == 2
assert vector['worker_task_name'] == 'memory.vector.upsert_chunks'
PY
```

## Completion Criteria

This DIFF is complete when successful normalization and chunking create the
next queued work item markers, audit events are recorded, docs are updated, and
verification passes.
