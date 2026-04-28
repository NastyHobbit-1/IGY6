# DIFF-022: Collector Dry-Run Runner

Status: Locked

## Type

Change-bearing.

## Objective

Add a collector-side dry-run runner that uses the connector registry to execute
the existing scaffold dry-run methods against typed source and permission
references.

This DIFF does not authorize API integration, worker scheduling, real
collection, artifact writes, normalization execution, or filesystem scanning.

## Baseline Facts

- DIFF-000 through DIFF-021 are locked.
- Manual-upload and local-project connector scaffolds exist.
- A connector registry exists for current scaffold connectors.
- Collection dry-run preview records currently exist in the API, but connector
  execution is not wired into API or worker flows.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-022-collector-dry-run-runner.md`
- `services/collectors/README.md`
- `services/collectors/app/__init__.py`
- `services/collectors/app/runner.py`

Allowed behavior:

- Add a collector-side helper that retrieves a connector by source type.
- Validate that the source type matches the selected connector.
- Call the connector scaffold `dry_run` method.
- Return the existing `DryRunResult` contract object.
- Export the runner helper from the collectors package.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Real connector collection.
- Artifact collection.
- File content extraction.
- Filesystem traversal.
- Normalization execution.
- Artifact writes.
- Export generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-022` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/collectors python3 -c "from app.contracts import SourcePermissionRef, SourceRef; from app.runner import run_dry_run; source = SourceRef(id='src-test', source_type='manual_upload', name='Manual', location=None); permission = SourcePermissionRef(id='perm-test', source_id='src-test', allowed_operations=['read']); result = run_dry_run(source, permission); print(result.connector_name, result.allowed)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/collectors python3 -c "from app.contracts import SourcePermissionRef, SourceRef; from app.runner import run_dry_run; source = SourceRef(id='src-test', source_type='manual_upload', name='Manual', location=None); permission = SourcePermissionRef(id='perm-test', source_id='src-test', allowed_operations=['read']); result = run_dry_run(source, permission); print(result.connector_name, result.allowed)"`

## Completion Criteria

This DIFF is complete when:

- A collector dry-run runner module exists.
- The runner uses the connector registry.
- The runner returns the existing `DryRunResult` contract.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- API integration with connector-backed dry-run previews.
- Worker integration.
- Real source-specific connector implementations.
- Collection and normalization execution.
