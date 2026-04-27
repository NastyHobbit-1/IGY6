# DIFF-011: Outcome Record API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes for recording and inspecting outcomes for predictions,
recommendations, work items, or other target records.

This DIFF does not authorize automated scoring, prediction evaluation,
recommendation updates, self-improvement queueing, or worker execution.

## Baseline Facts

- DIFF-000 through DIFF-010 are locked.
- The `outcomes` table exists from DIFF-001.
- Feedback events can be recorded, but outcome records cannot yet be created
  through the API.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-011-outcome-record-api.md`
- `docs/api.md`
- `services/api/app/main.py`
- `services/api/app/outcomes.py`

Allowed behavior:

- Add routes to create, list, and retrieve outcomes.
- Validate outcome target type and status against local allowlists.
- Record audit events for outcome creation.
- Do not trigger downstream behavior.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Prediction or recommendation status updates.
- Self-improvement queueing.
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

Use `DIFF-011` in change summaries, commits, and review notes for this work.

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

- Outcome routes exist.
- Outcome creation writes an audit event.
- No downstream evaluation or execution is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Prediction lifecycle updates.
- Recommendation usefulness tracking.
- Self-improvement candidates from outcomes.
- Outcome review UI.
