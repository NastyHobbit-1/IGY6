# DIFF-045: Web Review Operations Inspector

Status: Locked

## Type

Change-bearing.

## Objective

Add a read-only review and operations inspector to the web UI for work items,
approvals, feedback events, outcomes, reports, and audit events from FastAPI.

This DIFF makes the existing control, review, and audit layer visible without
adding decisions, feedback submission, report creation, worker actions, or other
writes.

## Baseline Facts

- DIFF-000 through DIFF-044 are locked.
- The web UI shows inventory, evidence, memory, analysis, and retrieval preview
  state.
- FastAPI exposes read endpoints for work items, approvals, feedback, outcomes,
  reports, and audit events.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-045-web-review-operations-inspector.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/user-guide.md`

Allowed behavior:

- Fetch `GET /work-items` from FastAPI.
- Fetch `GET /approvals` from FastAPI.
- Fetch `GET /feedback` from FastAPI.
- Fetch `GET /outcomes` from FastAPI.
- Fetch `GET /reports` from FastAPI.
- Fetch `GET /audit-events` from FastAPI.
- Render read-only review and operations summaries.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Backend changes.
- Database model changes.
- Migration changes.
- Frontend POST, PUT, PATCH, or DELETE calls.
- Work item creation.
- Approval decisions.
- Feedback submission.
- Outcome creation.
- Report creation.
- Audit event creation.
- Worker scheduling.
- Source collection.
- Chat answer generation.
- Direct PostgreSQL, artifact-store, local file, Qdrant, Neo4j, Redis, MLflow,
  or Phoenix access from the frontend.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-045` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- Passed: `npm --prefix apps/web run build`
- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

This DIFF is complete when:

- Web UI shows read-only work item summaries.
- Web UI shows read-only approval summaries.
- Web UI shows read-only feedback, outcome, report, and audit summaries.
- Frontend calls only FastAPI read endpoints for review and operations state.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Approval decision controls.
- Feedback submission controls.
- Report generation.
- Work item creation or execution.
- Any worker-backed actions.
