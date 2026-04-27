# DIFF-008: API Phase Metadata

Status: Locked

## Type

Change-bearing.

## Objective

Update API metadata so the service no longer describes itself as Phase 0-only
after Phase 1 source, work item, and approval record endpoints have been added.

## Baseline Facts

- DIFF-000 through DIFF-007 are locked.
- `services/api/app/main.py` still uses version `0.0.0-phase0` and a Phase 0
  skeleton description.
- The API now includes Phase 1 source registry, work item intent, and approval
  record routes.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-008-api-phase-metadata.md`
- `docs/api.md`
- `services/api/app/main.py`

Allowed behavior:

- Update FastAPI version, description, and root metadata.
- Clarify API documentation phase boundary.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Endpoint behavior changes.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact writes.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-008` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker services/collectors
  packages/policy` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- API metadata reflects Phase 1 foundation status.
- API docs describe the current boundary.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.
