# DIFF-038: Retrieval Evidence Trail API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal read-only retrieval evidence trail API that resolves a known chunk
into its source trail: chunk metadata, normalized document metadata, source
metadata, raw artifact metadata, and linked evidence items.

This DIFF creates inspectable retrieval context without semantic search
hydration, answer generation, chat, graph traversal, artifact content reads, or
worker scheduling.

## Baseline Facts

- DIFF-000 through DIFF-037 are locked.
- Chunk, document, source, raw artifact, and evidence item records already exist
  in PostgreSQL.
- Vector chunk search exists but returns Qdrant payloads only.
- No retrieval evidence trail API exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-038-retrieval-evidence-trail-api.md`
- `docs/api.md`
- `services/api/app/retrieval.py`
- `services/api/app/main.py`

Allowed behavior:

- Add `GET /retrieval/chunks/{chunk_id}/trail`.
- Return the requested chunk.
- Return the chunk's normalized document metadata.
- Return the related source metadata when present.
- Return the related raw artifact metadata when present.
- Return evidence items linked to the chunk and document.
- Keep the endpoint read-only and bounded to PostgreSQL metadata.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact content reads.
- Qdrant calls.
- Neo4j traversal.
- External model calls.
- Chat or answer generation.
- Retrieval ranking or query planning.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-038` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api .venv/bin/python -c "from app.retrieval import RetrievalTrailRead; print(RetrievalTrailRead.__name__)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/api .venv/bin/python -c "from app.retrieval import RetrievalTrailRead; print(RetrievalTrailRead.__name__)"`

## Completion Criteria

This DIFF is complete when:

- Retrieval evidence trail route exists.
- Route returns chunk, document, optional source, optional raw artifact, and
  linked evidence item metadata.
- Route is read-only and does not read artifact file contents.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Hydrated semantic retrieval.
- Evidence-backed chat.
- Retrieval planning across PostgreSQL, Qdrant, and Neo4j.
