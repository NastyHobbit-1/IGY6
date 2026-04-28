# DIFF-020: Local Project Connector Scaffold

Status: Locked

## Type

Change-bearing.

## Objective

Add a local-project connector scaffold that can validate scope and produce
dry-run metadata without performing collection or normalization.

This DIFF does not authorize repository reads for content extraction,
artifact collection, normalization execution, or worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-019 are locked.
- Connector contracts and a manual-upload connector scaffold already exist.
- No concrete local-project connector implementation exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-020-local-project-connector-scaffold.md`
- `services/collectors/README.md`
- `services/collectors/app/local_project.py`
- `services/collectors/app/__init__.py`

Allowed behavior:

- Add a `LocalProjectConnector` class implementing the connector protocol.
- Validate scope and permission operations for local project sources.
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
- Real repository traversal.
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

Use `DIFF-020` in change summaries, commits, and review notes for this work.

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

- A local-project connector class exists.
- Dry-run validation and summary generation work.
- No collection or normalization execution is added.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Real local project collection.
- Repository traversal rules.
- Artifact-store integration.
- Source-specific connector variants.
