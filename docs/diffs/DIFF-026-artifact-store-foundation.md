# DIFF-026: Artifact Store Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal local content-addressed artifact store foundation so raw bytes can
be written under the configured local artifact directory and recorded as raw
artifact metadata in PostgreSQL.

This DIFF does not authorize manual upload workflow, source collection,
normalization, chunking, evidence generation, worker scheduling, artifact file
serving, export generation, filesystem traversal outside the artifact store, or
external calls.

## Baseline Facts

- DIFF-000 through DIFF-025 are locked.
- `raw_artifacts` metadata table already exists.
- Read-only artifact metadata endpoints already exist.
- `ARTIFACT_STORE_PATH` is configured in `.env.example` and
  `services/api/app/config.py`.
- Local `storage/artifacts` exists with a `.gitkeep`.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-026-artifact-store-foundation.md`
- `docs/api.md`
- `services/api/app/artifact_store.py`
- `services/api/app/artifacts.py`

Allowed behavior:

- Add a local content-addressed artifact storage helper using SHA-256.
- Store bytes under the configured artifact store path with deterministic
  sharded paths.
- Use exclusive writes and verify existing hash files instead of silently
  overwriting content.
- Add a minimal `POST /artifacts` route that accepts base64 content, stores it
  locally, creates a `RawArtifact` metadata row, and emits an audit event.
- Validate referenced source and collection-run IDs when provided.
- Keep artifact reads metadata-only; do not serve file contents.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Manual upload workflow or multipart upload handling.
- Normalization execution.
- Chunk or evidence generation.
- Embedding or graph writes.
- Artifact file serving.
- Export generation.
- Filesystem traversal outside the configured artifact store.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-026` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.artifact_store import store_artifact_bytes; stored = store_artifact_bytes(b'hello', '/tmp/igy6-artifacts-smoke'); print(stored.content_hash, stored.size_bytes, stored.existed)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/api python3 -c "from app.artifact_store import store_artifact_bytes; stored = store_artifact_bytes(b'hello', '/tmp/igy6-artifacts-smoke'); print(stored.content_hash, stored.storage_path, stored.size_bytes, stored.existed)"`
- Passed repeat-write smoke: same `/tmp/igy6-artifacts-smoke` content returned `existed=True`

## Completion Criteria

This DIFF is complete when:

- A content-addressed artifact store helper exists.
- Artifact bytes are stored by SHA-256 under the configured artifact path.
- Existing matching hash files are verified rather than overwritten.
- `POST /artifacts` creates raw artifact metadata and audit events.
- Artifact read endpoints remain metadata-only.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Manual upload collection workflow.
- Local project collection.
- Artifact-to-normalized-document conversion.
- Chunking and evidence generation.
- Artifact file serving or export, if explicitly approved.
