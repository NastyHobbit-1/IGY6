# DIFF-033: Deterministic Graph Lineage Upserts

Status: Locked

## Type

Change-bearing.

## Objective

Add a deterministic API path to sync known PostgreSQL evidence lineage into
Neo4j.

This DIFF does not authorize entity extraction, inferred relationships, pattern
detection, embeddings, semantic search, worker scheduling, external model calls,
or chat integration.

## Baseline Facts

- DIFF-000 through DIFF-032 are locked.
- Neo4j graph schema foundation exists.
- PostgreSQL has sources, raw artifacts, normalized documents, chunks, and
  evidence items.
- Chunks and evidence items carry deterministic provenance links.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-033-deterministic-graph-lineage-upserts.md`
- `docs/api.md`
- `services/api/app/graph_memory.py`

Allowed behavior:

- Add a `POST /memory/graph/lineage/sync` route.
- Upsert graph nodes for source, raw artifact, document, chunk, and evidence
  item records already present in PostgreSQL.
- Upsert deterministic provenance relationships only:
  - `SOURCE_HAS_ARTIFACT`
  - `ARTIFACT_HAS_DOCUMENT`
  - `DOCUMENT_HAS_CHUNK`
  - `DOCUMENT_HAS_EVIDENCE`
  - `CHUNK_HAS_EVIDENCE`
- Return sync counts.
- Use existing SQLAlchemy models and Neo4j settings.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Entity extraction.
- Inferred relationships.
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

Use `DIFF-033` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.graph_memory import lineage_relationship_types; print(len(lineage_relationship_types()))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.graph_memory import lineage_relationship_types; print(len(lineage_relationship_types()))"` because the host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- Deterministic graph lineage sync route exists.
- Sync only uses explicit PostgreSQL provenance links.
- Sync returns node and relationship counts.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Graph relationship inspection APIs.
- Worker-backed graph sync.
- Entity and claim extraction.
- Relationship inference and pattern detection.
