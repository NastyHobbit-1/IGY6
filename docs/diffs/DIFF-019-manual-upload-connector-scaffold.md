# DIFF-019: Manual Upload Connector Scaffold

Status: Locked

## Type

Change-bearing.

## Objective

Add a manual-upload connector scaffold that can validate scope and produce dry-
run metadata without performing collection or normalization.

This DIFF does not authorize real artifact collection, file reads for content
extraction, normalization execution, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-018 are locked.
- Connector contracts already exist in `services/collectors/app/contracts.py`.
- No concrete connector implementation exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-019-manual-upload-connector-scaffold.md`
- `services/collectors/README.md`
- `services/collectors/app/manual_upload.py`
- `services/collectors/app/__init__.py`

Allowed behavior:

- Add a `ManualUploadConnector` class implementing the connector protocol.
- Validate source type and permission operations for manual uploads.
- Produce dry-run summaries only.
- Leave `collect`, `normalize`, and `cleanup` as explicit placeholders or
  lightweight no-op/error boundaries.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Real artifact collection.
- File content extraction.
- Normalization execution.
- Artifact writes.
- Export generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-019` in change summaries, commits, and review notes for this work.

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

- A manual-upload connector class exists.
- Dry-run validation and summary generation work.
- No collection or normalization execution is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Real manual upload collection.
- Normalization of uploaded artifacts.
- Artifact-store integration.
- Source-specific connector variants.
