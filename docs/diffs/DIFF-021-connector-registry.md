# DIFF-021: Connector Registry

Status: Locked

## Type

Change-bearing.

## Objective

Add a connector registry for discovering supported connector scaffolds by
source type.

This DIFF does not authorize connector execution, artifact collection,
normalization execution, API integration, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-020 are locked.
- Manual-upload and local-project connector scaffolds exist.
- No central connector registry exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-021-connector-registry.md`
- `services/collectors/README.md`
- `services/collectors/app/__init__.py`
- `services/collectors/app/registry.py`

Allowed behavior:

- Add registry helpers to list connector source types.
- Add registry helper to retrieve a connector by source type.
- Register the current scaffold connectors only.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Connector execution.
- Artifact collection.
- File content extraction.
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

Use `DIFF-021` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/collectors python3 -c "from app.registry import list_source_types; print(list_source_types())"`

## Completion Criteria

This DIFF is complete when:

- A connector registry module exists.
- Existing connector scaffolds are registered.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker integration.
- API integration.
- Collector execution.
- Real source-specific connector implementations.
