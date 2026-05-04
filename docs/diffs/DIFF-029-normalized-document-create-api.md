# DIFF-029: Normalized Document Create API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal API path to create a normalized document from an existing raw
artifact stored in the local content-addressed artifact store.

This DIFF does not authorize chunking, evidence generation, embeddings, graph
writes, worker scheduling, source collection, or external model calls.

## Baseline Facts

- DIFF-000 through DIFF-028 are locked.
- Raw artifacts can now be stored and linked to collection runs.
- Normalized document read endpoints and database model already exist.
- The collector package has normalization scaffolding, but API runtime sharing
  with collectors is still intentionally deferred.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-029-normalized-document-create-api.md`
- `docs/api.md`
- `services/api/app/artifact_store.py`
- `services/api/app/evidence.py`

Allowed behavior:

- Add an artifact-store helper to read bytes for an existing stored artifact
  path while keeping reads inside the configured artifact store.
- Add `POST /evidence/documents` to create a normalized document from a raw
  artifact.
- Validate raw artifact existence and source linkage.
- Decode artifact bytes as UTF-8 text.
- Create a `NormalizedDocument` row with title, document type, language,
  sensitivity, text content, and metadata.
- Emit an audit event for normalized document creation.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact writes.
- Reading files outside the configured artifact store.
- Chunk creation.
- Evidence item creation beyond the existing endpoint.
- Claim, pattern, hypothesis, prediction, or recommendation creation.
- Embedding or graph writes.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-029` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.artifact_store import store_artifact_bytes, read_artifact_bytes; stored = store_artifact_bytes(b'hello', '/tmp/igy6-normalize-artifacts'); print(read_artifact_bytes(stored.storage_path, '/tmp/igy6-normalize-artifacts').decode('utf-8'))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: artifact-store read smoke returned `hello`

## Completion Criteria

This DIFF is complete when:

- `POST /evidence/documents` can create a normalized document from a raw
  artifact.
- Artifact reads are constrained to the configured artifact store.
- Raw artifact and source link consistency is validated.
- Normalized document creation is audited.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Chunk generation.
- Evidence generation from normalized documents.
- Worker-backed normalization.
- Non-text document parsing.
