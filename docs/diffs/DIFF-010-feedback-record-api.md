# DIFF-010: Feedback Record API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes for recording and inspecting feedback events on existing records.

This DIFF does not authorize outcome automation, prediction evaluation,
self-improvement queueing, reasoning changes, or worker execution.

## Baseline Facts

- DIFF-000 through DIFF-009 are locked.
- The `feedback_events` table exists from DIFF-001.
- Current API can inspect evidence but cannot record user feedback labels.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-010-feedback-record-api.md`
- `docs/api.md`
- `services/api/app/feedback.py`
- `services/api/app/main.py`

Allowed behavior:

- Add routes to create, list, and retrieve feedback events.
- Validate feedback target type and label against local allowlists.
- Record audit events for feedback creation.
- Do not trigger downstream behavior.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Self-improvement queueing.
- Prediction or recommendation evaluation.
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

Use `DIFF-010` in change summaries, commits, and review notes for this work.

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

- Feedback event routes exist.
- Feedback creation writes an audit event.
- No downstream execution or self-improvement behavior is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Feedback UI controls.
- Outcome APIs.
- Self-improvement candidate creation from feedback.
- Feedback-driven ranking or memory updates.
