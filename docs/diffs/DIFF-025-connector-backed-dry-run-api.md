# DIFF-025: Connector-Backed Dry-Run API

Status: Locked

## Type

Change-bearing.

## Objective

Wire the existing collection-run dry-run API to connector-backed dry-run
execution so a registered source and permission produce an auditable read-only
preview from the connector dry-run contract.

This DIFF does not authorize real collection, worker scheduling, artifact
writes, normalization execution, filesystem traversal, browser automation, or
external model calls.

## Baseline Facts

- DIFF-000 through DIFF-024 are locked.
- `POST /collection-runs/dry-run` currently synthesizes metadata-only preview
  summaries in `services/api/app/collection_runs.py`.
- Collector scaffold dry-run contracts and a `run_dry_run` helper exist under
  `services/collectors/app`.
- The current API container only copies `services/api`, while the collector
  service uses a separate top-level `app` package, so direct runtime sharing
  requires an explicit narrow bridge.
- The currently registered scaffold connectors are `manual_upload` and
  `local_project`.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-025-connector-backed-dry-run-api.md`
- `docs/api.md`
- `services/api/app/collection_runs.py`
- `services/api/app/collector_dry_run.py`

Allowed behavior:

- Add a narrow API-side dry-run adapter that mirrors the existing collector
  dry-run contract for currently scaffolded connector types.
- Build connector dry-run input from `Source` and `SourcePermission` records.
- Make `POST /collection-runs/dry-run` execute connector-backed dry-run
  validation instead of returning only a synthetic preview.
- Reject disabled sources.
- Reject source permissions that do not allow `dry_run` or `read`.
- Persist a collection-run summary that includes connector name, allowed flag,
  summary text, estimated item count, warnings, connector metadata, and request
  notes.
- Persist failed connector dry-runs as dry-run collection-run records with an
  error message and audit event.
- Keep the endpoint read-only: no workers, artifact writes, normalization, file
  reads, browser actions, or external calls.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Real connector collection.
- Artifact collection or writes.
- File content extraction.
- Filesystem traversal.
- Normalization execution.
- Chunk or evidence generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-025` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.collector_dry_run import run_connector_dry_run; print(run_connector_dry_run(source_id='src', source_type='manual_upload', source_name='Manual', source_location=None, source_metadata={}, permission_id='perm', permission_source_id='src', permission_scope={}, allowed_operations=['dry_run'], external_model_policy='blocked', approval_required=True).connector_name)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/api python3 -c "from app.collector_dry_run import run_connector_dry_run; print(run_connector_dry_run(source_id='src', source_type='manual_upload', source_name='Manual', source_location=None, source_metadata={}, permission_id='perm', permission_source_id='src', permission_scope={}, allowed_operations=['dry_run'], external_model_policy='blocked', approval_required=True).connector_name)"`
- Passed expected-failure smoke: unsupported `web_public` connector raises `ValueError: No connector registered for source type: web_public`
- Blocked extra API route import smoke: host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- `POST /collection-runs/dry-run` uses connector-backed dry-run validation.
- Disabled sources are rejected.
- Permissions must belong to the source and allow dry-run/read preview.
- Successful dry-runs persist connector result details in `summary_json`.
- Failed connector dry-runs persist an auditable failed dry-run record.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Packaging collectors for shared API/worker imports without top-level package
  conflicts.
- Real collection execution.
- Artifact store writes.
- Manual upload ingestion.
- Local project filesystem traversal.
- Normalization, chunking, and evidence generation from collection runs.
