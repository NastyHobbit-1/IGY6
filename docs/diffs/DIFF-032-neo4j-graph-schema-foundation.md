# DIFF-032: Neo4j Graph Schema Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal API-managed Neo4j schema foundation for future relationship
memory.

This DIFF does not authorize graph data extraction, node upserts from evidence,
relationship writes, pattern detection, retrieval planning, or chat
integration.

## Baseline Facts

- DIFF-000 through DIFF-031 are locked.
- Neo4j is already part of Docker Compose and API readiness checks.
- API dependencies already include the Neo4j Python driver.
- PostgreSQL contains sources, raw artifacts, normalized documents, chunks, and
  evidence items, but no graph sync exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-032-neo4j-graph-schema-foundation.md`
- `docs/api.md`
- `services/api/app/graph_memory.py`
- `services/api/app/main.py`

Allowed behavior:

- Add Neo4j helper functions to list constraints and create id uniqueness
  constraints for source, raw artifact, document, chunk, and evidence nodes.
- Add API routes to inspect graph schema status and ensure baseline graph
  constraints exist.
- Use existing Neo4j settings from API config.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Graph node or relationship upserts from PostgreSQL data.
- Pattern, claim, hypothesis, prediction, or recommendation generation.
- Retrieval or chat integration.
- Embeddings or Qdrant writes.
- Source collection.
- Artifact reads or writes.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-032` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.graph_memory import graph_constraint_statements; print(len(graph_constraint_statements()))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.graph_memory import graph_constraint_statements; print(len(graph_constraint_statements()))"` because the host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- Graph memory API routes exist.
- Baseline graph constraint statements are centralized.
- Schema inspection and ensure helpers exist.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Graph node upserts.
- Evidence lineage relationships.
- Entity/claim extraction.
- Relationship query APIs.
