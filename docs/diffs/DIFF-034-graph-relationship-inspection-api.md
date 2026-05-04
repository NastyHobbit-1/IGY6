# DIFF-034: Graph Relationship Inspection API

Status: Locked

## Type

Change-bearing.

## Objective

Add a bounded read-only API for inspecting deterministic Neo4j lineage
relationships for a known graph node.

This DIFF does not authorize new graph writes, inferred relationships, entity
extraction, pattern detection, embeddings, retrieval planning, or chat
integration.

## Baseline Facts

- DIFF-000 through DIFF-033 are locked.
- Deterministic graph lineage sync can upsert source, artifact, document,
  chunk, and evidence nodes plus provenance relationships.
- No relationship inspection API exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-034-graph-relationship-inspection-api.md`
- `docs/api.md`
- `services/api/app/graph_memory.py`

Allowed behavior:

- Add a read-only `GET /memory/graph/nodes/{node_label}/{node_id}/relationships`
  route.
- Allow only known deterministic node labels.
- Return bounded incoming and outgoing relationships with neighbor IDs and
  labels.
- Use existing Neo4j settings.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Graph writes.
- Inferred relationships.
- Entity extraction.
- Pattern, hypothesis, prediction, or recommendation generation.
- Embedding generation.
- Qdrant writes.
- Source collection.
- Artifact reads or writes.
- External model calls.
- Chat or retrieval integration.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-034` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.graph_memory import allowed_graph_node_labels; print(sorted(allowed_graph_node_labels())[0])"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.graph_memory import allowed_graph_node_labels; print(sorted(allowed_graph_node_labels())[0])"` because the host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- A bounded graph relationship inspection route exists.
- Only known deterministic labels are accepted.
- The route performs no graph writes.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- UI graph views.
- Worker-backed graph sync.
- Entity and claim extraction.
- Relationship inference and pattern detection.
