# DIFF-014: Audit Read API

Status: Locked

## Type

Change-bearing.

## Objective

Add read-only API routes for inspecting audit events already recorded in
PostgreSQL.

This DIFF does not authorize audit mutation, policy enforcement changes, worker
execution, source collection, or any sensitive/system-changing action.

## Baseline Facts

- DIFF-000 through DIFF-013 are locked.
- The `audit_events` table exists from Phase 0.
- Several Phase 1 record APIs now write audit events.
- No API currently exposes audit events for inspection.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-014-audit-read-api.md`
- `docs/api.md`
- `services/api/app/audit.py`
- `services/api/app/main.py`

Allowed behavior:

- Add read-only routes to list and retrieve audit events.
- Use existing SQLAlchemy models and session handling.
- Document the endpoints and read-only boundary.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Audit creation, mutation, or deletion routes.
- Policy enforcement rewiring.
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

Use `DIFF-014` in change summaries, commits, and review notes for this work.

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

- Read-only audit routes exist.
- API docs list the audit endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Audit filtering and pagination.
- Policy decision views.
- UI audit inspection.
