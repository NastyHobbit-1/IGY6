# DIFF-030: Chunk And Evidence Generation API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal deterministic API path to generate text chunks and evidence items
from an existing normalized document.

This DIFF does not authorize embeddings, graph writes, claim generation,
pattern detection, worker scheduling, external model calls, or retrieval/chat
integration.

## Baseline Facts

- DIFF-000 through DIFF-029 are locked.
- Normalized documents can be created from raw artifacts.
- Chunk and evidence item tables and read APIs already exist.
- Evidence item creation API already exists.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-030-chunk-evidence-generation-api.md`
- `docs/api.md`
- `services/api/app/evidence.py`

Allowed behavior:

- Add `POST /evidence/documents/{document_id}/chunks`.
- Split normalized document text into deterministic fixed-size chunks.
- Create `Chunk` rows with `embedding_status` set to `not_started`.
- Create one `EvidenceItem` per chunk with source, document, and chunk links.
- Emit audit events for generation.
- Keep the operation idempotent by rejecting generation if chunks already exist
  for the document.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact reads or writes.
- Normalized document creation changes.
- Embeddings.
- Qdrant writes.
- Neo4j writes.
- Claim generation.
- Pattern, hypothesis, prediction, or recommendation creation.
- Chat or retrieval integration.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-030` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.evidence import ChunkGenerationCreate; payload = ChunkGenerationCreate(chunk_size=50); print(payload.chunk_size)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.evidence import ChunkGenerationCreate; payload = ChunkGenerationCreate(chunk_size=50); print(payload.chunk_size)"` because the host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- Chunk/evidence generation route exists.
- Generated chunks are linked to the normalized document.
- Generated evidence items cite the source, document, and chunk.
- Repeated generation for the same document is rejected.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Embedding generation.
- Semantic search.
- Graph extraction.
- Evidence-backed chat.
