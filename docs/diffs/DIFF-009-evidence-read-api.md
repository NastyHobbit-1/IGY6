# DIFF-009: Evidence Read API

Status: Locked

## Type

Change-bearing.

## Objective

Add read-only API routes for inspecting Phase 1 evidence foundation records
that already exist in PostgreSQL.

This DIFF does not authorize creating evidence, collecting sources, normalizing
artifacts, generating claims, embedding content, graph writes, chat, prediction,
or recommendation behavior.

## Baseline Facts

- DIFF-000 through DIFF-008 are locked.
- Phase 1 evidence foundation tables exist from DIFF-001.
- Current API exposes health, sources, work item intent records, and approval
  records.
- No read-only evidence inspection routes exist yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-009-evidence-read-api.md`
- `docs/api.md`
- `services/api/app/evidence.py`
- `services/api/app/main.py`

Allowed behavior:

- Add read-only routes to list and retrieve normalized documents.
- Add read-only routes to list and retrieve evidence items.
- Add read-only routes to list and retrieve claims.
- Use existing SQLAlchemy models and session handling.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Evidence creation or mutation.
- Source collection.
- Artifact writes.
- Browser automation.
- Embeddings.
- Qdrant writes.
- Neo4j writes.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-009` in change summaries, commits, and review notes for this work.

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

- Read-only evidence inspection routes exist.
- API docs list the read-only evidence endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Evidence creation workflows.
- Manual observation APIs.
- Normalization workers.
- Evidence UI.
- Search and retrieval ranking.
