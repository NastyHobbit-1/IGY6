# DIFF-007: Approval Record API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes for recording approval requests and decisions without executing
the approved action.

## Baseline Facts

- DIFF-000 through DIFF-006 are locked.
- The `approvals` and `audit_events` tables already exist.
- Work items can now be recorded with intent-verification context, but approval
  request APIs do not exist yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-007-approval-record-api.md`
- `docs/api.md`
- `services/api/app/approvals.py`
- `services/api/app/main.py`

Allowed behavior:

- Add approval request/response models.
- Add routes to create, list, and retrieve approvals.
- Add a route to record an approval decision.
- Record audit events for approval request and decision.
- Do not execute any approved work.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Work item execution.
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

Use `DIFF-007` in change summaries, commits, and review notes for this work.

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

- Approval routes exist.
- Approval request and decision writes emit audit events.
- Approval decisions do not trigger execution.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Work item approval linkage.
- Worker-side approval enforcement.
- UI approval review.
