# DIFF-056: Report Status API

Status: Locked

## Type

Change-bearing.

## Objective

Add an explicit API route to update report status and artifact path metadata
with audit logging.

## Baseline Facts

- DIFF-000 through DIFF-055 are locked.
- Reports can be created and read.
- Report rendering/export remains out of scope.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-056-report-status-api.md`
- `services/api/app/reports.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /reports/{report_id}/status`.
- Validate status against the existing report status allowlist.
- Optionally update `artifact_path`.
- Record audit events with previous and new status.

## Prohibited Scope

This DIFF does not allow report rendering, artifact creation, worker dispatch,
model changes, migrations, dependency changes, or broad refactors.

## Required Tags

Use `DIFF-056` in change summaries, commits, and review notes.

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
- Passed targeted venv smoke: `ReportStatusUpdate` constructs valid payloads.
- Passed targeted venv smoke: status allowlist includes `archived`.

## Completion Criteria

This DIFF is complete when the status endpoint exists, validates status, audits
changes, is documented, and verification passes.
