# DIFF-055: Work Item Status API

Status: Locked

## Type

Change-bearing.

## Objective

Add an explicit API route to update work item status with audit logging.

## Baseline Facts

- DIFF-000 through DIFF-054 are locked.
- Work items can be created and read.
- Worker tasks can update work item state directly, but there is no API review
  route for explicit status updates.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-055-work-item-status-api.md`
- `services/api/app/work_items.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /work-items/{work_item_id}/status`.
- Validate status against a local allowlist.
- Store an optional error message.
- Record an audit event with previous and new status.

## Prohibited Scope

This DIFF does not allow worker dispatch, model changes, migrations, dependency
changes, approval execution, source collection, or broad refactors.

## Required Tags

Use `DIFF-055` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate payload construction and status allowlist.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: `WorkItemStatusUpdate` constructs valid payloads.
- Passed targeted venv smoke: status allowlist includes `failed`.

## Completion Criteria

This DIFF is complete when the status endpoint exists, validates status, audits
changes, is documented, and verification passes.
