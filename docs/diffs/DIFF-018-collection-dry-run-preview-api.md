# DIFF-018: Collection Dry-Run Preview API

Status: Locked

## Type

Change-bearing.

## Objective

Add an API route that produces a metadata-only dry-run preview for a source and
permission pair before any collection occurs.

This DIFF does not authorize collector execution, artifact collection,
normalization, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-017 are locked.
- The `sources`, `source_permissions`, and `collection_runs` tables exist.
- Collection-run record routes already exist.
- No API currently produces a dry-run preview summary.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-018-collection-dry-run-preview-api.md`
- `docs/api.md`
- `services/api/app/collection_runs.py`
- `services/api/app/main.py`

Allowed behavior:

- Add a dry-run preview route for a source and permission pair.
- Validate that the source exists.
- Validate that the permission exists and belongs to the source.
- Create a dry-run collection-run record with a synthesized summary only.
- Record audit events.

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

Use `DIFF-018` in change summaries, commits, and review notes for this work.

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

- Dry-run preview route exists.
- Preview creation writes an audit event.
- No worker execution or collector behavior is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Actual collector execution.
- Collector-specific dry-run implementations.
- Preview UI.
- Artifact and normalization workflow integration.
