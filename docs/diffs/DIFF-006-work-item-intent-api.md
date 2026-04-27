# DIFF-006: Work Item Intent API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal work-item API that records proposed work and intent-verification
context without executing tasks.

This supports the project rule that major work must summarize intent,
assumptions, expected output, and safety requirements before execution.

## Baseline Facts

- DIFF-000 through DIFF-005 are locked.
- The `work_items` and `audit_events` tables already exist.
- No API currently exposes work item creation or listing.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-006-work-item-intent-api.md`
- `docs/api.md`
- `services/api/app/main.py`
- `services/api/app/work_items.py`

Allowed behavior:

- Add Pydantic request/response models for work items.
- Add routes to create, list, and retrieve work items.
- Default new work items to an intent-verification status.
- Record audit events for work item creation.
- Do not execute work items.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Celery scheduling.
- Source collection.
- Artifact writes.
- Browser automation.
- External model calls.
- Approval decision APIs.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-006` in change summaries, commits, and review notes for this work.

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

- Work item routes exist.
- New work items record intent-verification context.
- Work item creation writes an audit event.
- No worker execution or scheduling is added.
- Required verification passes or any blockage is recorded.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Approval APIs.
- Worker-side execution.
- Work item status transitions.
- UI review of intent verification.
