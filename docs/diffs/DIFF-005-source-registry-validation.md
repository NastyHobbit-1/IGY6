# DIFF-005: Source Registry Validation

Status: Locked

## Type

Change-bearing.

## Objective

Add basic request validation to the Phase 1 source registry API so source types,
sensitivity labels, external model policies, and allowed operations are checked
before database writes.

## Baseline Facts

- DIFF-000 through DIFF-004 are locked.
- `packages/policy` now contains shared policy constants, but it is not wired
  into the API runtime container.
- Source registry endpoints exist from DIFF-002.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-005-source-registry-validation.md`
- `docs/api.md`
- `services/api/app/sources.py`

Allowed behavior:

- Add local validation constants to `services/api/app/sources.py`.
- Validate source type, sensitivity, external model policy, and allowed
  operations at request parsing time.
- Document the validation boundary.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Collector changes.
- Importing or packaging `packages/policy` into the API container.
- Real source collection.
- Dry-run execution.
- Artifact writes.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-005` in change summaries, commits, and review notes for this work.

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

- Source registry request validation exists.
- API docs mention the validation boundary.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Packaging shared policy code for API runtime use.
- Approval APIs.
- Dry-run planning.
