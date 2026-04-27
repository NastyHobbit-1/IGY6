# DIFF-016: Raw Artifact Read API

Status: Locked

## Type

Change-bearing.

## Objective

Add read-only API routes for inspecting raw artifact metadata records already
present in PostgreSQL.

This DIFF does not authorize artifact creation, artifact file reads, file
writes, collection, normalization, or export behavior.

## Baseline Facts

- DIFF-000 through DIFF-015 are locked.
- The `raw_artifacts` table exists from Phase 0.
- No API currently exposes raw artifact metadata records.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-016-raw-artifact-read-api.md`
- `docs/api.md`
- `services/api/app/artifacts.py`
- `services/api/app/main.py`

Allowed behavior:

- Add read-only routes to list and retrieve raw artifact metadata.
- Use existing SQLAlchemy models and session handling.
- Document that artifact files are not read or written.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Artifact creation, mutation, deletion, or file reads.
- File writes.
- Source collection.
- Normalization execution.
- Export generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-016` in change summaries, commits, and review notes for this work.

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

- Read-only raw artifact metadata routes exist.
- API docs list the artifact metadata endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Artifact creation.
- Content-addressed storage writes.
- Artifact file serving or export.
- Collection workflow integration.
