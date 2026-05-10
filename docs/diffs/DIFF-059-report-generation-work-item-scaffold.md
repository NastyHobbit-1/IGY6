# DIFF-059: Report Generation Work Item Scaffold

Status: Locked

## Type

Change-bearing.

## Objective

Add a report endpoint that creates a queued report-generation work item marker
without rendering report files or dispatching workers.

## Baseline Facts

- DIFF-000 through DIFF-058 are locked.
- Report metadata routes already exist.
- Work item metadata and status routes already exist.
- Report rendering and export generation are not implemented.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-059-report-generation-work-item-scaffold.md`
- `services/api/app/reports.py`
- `docs/api.md`

Allowed behavior:

- Add a route that creates a queued `report_generation` work item for an
  existing report.
- Include the report ID, report type, and explicit scaffold-only flags in the
  work item payload.
- Record an audit event for the created work item.
- Document the route and limits.

## Prohibited Scope

This DIFF does not allow report rendering, artifact writes, export writes,
worker dispatch, Celery calls, template changes, model changes, migrations,
dependency changes, or broad refactors.

## Required Tags

Use `DIFF-059` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate report work item payload construction.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python -c "import sys; sys.path.insert(0, 'services/api'); from app.reports import ReportWorkItemCreate; payload = ReportWorkItemCreate(notes='queue draft'); assert payload.requested_by_actor_id == 'local-owner'; assert payload.notes == 'queue draft'"
```

## Completion Criteria

This DIFF is complete when an existing report can produce a queued
`report_generation` work item marker, the action is audited, the route is
documented, and verification passes.
