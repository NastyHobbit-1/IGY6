# DIFF-003: Connector Contracts

Status: Locked

## Type

Change-bearing.

## Objective

Add typed connector contracts for Phase 1 so future source collectors share a
read-only, permission-validated interface before any real connector
implementation exists.

This DIFF does not authorize collection, dry-run execution, artifact writes,
normalization execution, or worker scheduling.

## Baseline Facts

- DIFF-000, DIFF-001, and DIFF-002 are locked.
- `services/collectors` currently contains only a placeholder README.
- The build instructions require every connector to validate scope, support
  dry-run, collect raw artifacts, normalize artifacts, classify sensitivity,
  extract metadata, and clean up.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-003-connector-contracts.md`
- `services/collectors/README.md`
- `services/collectors/app/__init__.py`
- `services/collectors/app/contracts.py`

Allowed behavior:

- Add Python type contracts and value objects for future connectors.
- Document that no real collectors are implemented yet.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- UI changes.
- Worker task changes.
- Database model changes.
- Migration changes.
- Real source collection.
- Dry-run execution.
- Artifact writes.
- Normalization execution.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-003` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker services/collectors`
  passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- Connector contract types exist.
- Collector README references the contracts and repeats the no-implementation
  boundary.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Source dry-run planning.
- Manual upload connector.
- Local project connector.
- Worker integration.
- Audit-backed collection execution.
