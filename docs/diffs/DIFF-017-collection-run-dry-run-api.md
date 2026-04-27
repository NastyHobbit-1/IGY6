# DIFF-017: Collection Run Dry-Run API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes for creating and inspecting collection-run records that represent
dry-run planning only.

This DIFF does not authorize collector execution, artifact collection,
normalization, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-016 are locked.
- The `collection_runs` table exists from Phase 0.
- No API currently exposes collection-run planning records.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-017-collection-run-dry-run-api.md`
- `docs/api.md`
- `services/api/app/collection_runs.py`
- `services/api/app/main.py`

Allowed behavior:

- Add routes to create, list, and retrieve collection-run records.
- Validate that the referenced source exists before creating a collection run.
- Default created records to dry-run mode.
- Record audit events for dry-run record creation.
- Keep the API as metadata only; do not trigger workers.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Collector execution.
- Artifact creation.
- Normalization execution.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-017` in change summaries, commits, and review notes for this work.

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

- Collection-run dry-run routes exist.
- Dry-run creation writes an audit event.
- No worker execution or collector behavior is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Actual dry-run execution summaries.
- Collector implementation.
- Collection workflow UI.
- Artifact and normalization worker behavior.
