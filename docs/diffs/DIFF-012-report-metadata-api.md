# DIFF-012: Report Metadata API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes for recording and inspecting report metadata records.

This DIFF does not authorize report rendering, artifact writes, exports, worker
jobs, or generated summaries.

## Baseline Facts

- DIFF-000 through DIFF-011 are locked.
- The `reports` table exists from Phase 0.
- No API currently exposes report metadata records.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-012-report-metadata-api.md`
- `docs/api.md`
- `services/api/app/main.py`
- `services/api/app/reports.py`

Allowed behavior:

- Add routes to create, list, and retrieve report metadata.
- Validate report type and status against local allowlists.
- Record audit events for report record creation.
- Do not render or export report files.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Report rendering.
- Artifact writes.
- Export generation.
- Source collection.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-012` in change summaries, commits, and review notes for this work.

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

- Report metadata routes exist.
- Report record creation writes an audit event.
- No rendering, export, or artifact write behavior is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Report rendering.
- Export artifact creation.
- Report UI.
- Evidence-backed report generation.
