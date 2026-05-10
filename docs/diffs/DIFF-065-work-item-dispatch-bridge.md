# DIFF-065: Work Item Dispatch Bridge

Status: Locked

## Type

Change-bearing.

## Objective

Add an allowlisted API dispatch bridge that sends queued work items to existing
Celery worker tasks and records an audit event.

## Baseline Facts

- DIFF-000 through DIFF-064 are locked.
- Collection creates queued `collection_normalization` work items.
- Worker tasks already exist for collection normalization, document chunking,
  and chunk vector upsert.
- The API currently records work items but does not dispatch Celery tasks.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-065-work-item-dispatch-bridge.md`
- `services/api/app/work_items.py`
- `services/api/app/config.py`
- `services/api/requirements.txt`
- `infra/docker-compose.yml`
- `docs/api.md`

Allowed behavior:

- Add `POST /work-items/{work_item_id}/dispatch`.
- Dispatch only allowlisted work item types to existing worker task names.
- Derive Celery task arguments from existing work item payloads.
- Record Celery task IDs in work item payload metadata.
- Record dispatch audit events.
- Document the route and limits.

## Prohibited Scope

This DIFF does not allow new worker tasks, worker behavior changes, pipeline
chaining, source collection changes, migrations, UI changes, or broad refactors.

## Required Tags

Use `DIFF-065` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate dispatch plan construction without
contacting Redis.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
from types import SimpleNamespace

sys.path.insert(0, 'services/api')
from app.work_items import build_dispatch_plan

item = SimpleNamespace(id='w1', work_type='collection_normalization', payload_json={'collection_run_id': 'cr1', 'raw_artifact_ids': ['ra1']})
assert build_dispatch_plan(item) == ('collection.normalize_collection_run', ['w1', 'cr1', ['ra1']], {})
item = SimpleNamespace(id='w2', work_type='document_chunking', payload_json={'document_id': 'd1', 'chunk_size': 500})
assert build_dispatch_plan(item) == ('evidence.generate_document_chunks', [['d1']], {'chunk_size': 500, 'work_item_id': 'w2'})
item = SimpleNamespace(id='w3', work_type='chunk_vector_upsert', payload_json={'limit': 10})
assert build_dispatch_plan(item) == ('memory.vector.upsert_chunks', [], {'limit': 10, 'work_item_id': 'w3'})
PY
```

## Completion Criteria

This DIFF is complete when allowlisted queued work items can be dispatched to
Celery, dispatch metadata and audit events are recorded, the route is
documented, and verification passes.
