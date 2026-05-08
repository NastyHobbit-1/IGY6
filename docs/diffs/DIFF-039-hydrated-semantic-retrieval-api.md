# DIFF-039: Hydrated Semantic Retrieval API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal hydrated semantic retrieval API that calls the existing Qdrant
chunk vector search, then hydrates each returned chunk ID with PostgreSQL
metadata and linked evidence items.

This DIFF turns vector search hits into inspectable evidence-backed retrieval
results without answer generation, chat, graph traversal, or broader retrieval
planning.

## Baseline Facts

- DIFF-000 through DIFF-038 are locked.
- `POST /memory/vector/chunks/search` can return Qdrant chunk vector hits.
- `GET /retrieval/chunks/{chunk_id}/trail` can resolve a known chunk into a
  read-only evidence trail.
- No hydrated semantic retrieval endpoint exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-039-hydrated-semantic-retrieval-api.md`
- `docs/api.md`
- `services/api/app/retrieval.py`

Allowed behavior:

- Add `POST /retrieval/chunks/search`.
- Accept a query and bounded limit.
- Reuse the existing deterministic local Qdrant chunk search helper.
- Hydrate each returned chunk hit from PostgreSQL.
- Return score, Qdrant payload, chunk, document, optional source, optional raw
  artifact, and linked evidence item metadata.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact content reads.
- Changes to the Qdrant embedding algorithm.
- Neo4j traversal.
- External model calls.
- Chat or answer generation.
- Retrieval planning beyond direct vector search plus PostgreSQL hydration.
- Writes to PostgreSQL, Qdrant, Neo4j, or artifact storage.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-039` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api .venv/bin/python -c "from app.retrieval import HydratedChunkSearchRequest; print(HydratedChunkSearchRequest(query='x').limit)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/api .venv/bin/python -c "from app.retrieval import HydratedChunkSearchRequest; print(HydratedChunkSearchRequest(query='x').limit)"`

## Completion Criteria

This DIFF is complete when:

- Hydrated semantic retrieval route exists.
- Query embedding and Qdrant search are delegated to existing vector-memory
  helpers.
- Returned results are bounded.
- Results include vector score/payload plus PostgreSQL trail metadata.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Retrieval-only chat context preview.
- Evidence-backed answer generation.
- Retrieval planning across PostgreSQL, Qdrant, and Neo4j.
