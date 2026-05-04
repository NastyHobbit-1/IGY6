# DIFF-027: Manual Upload Collection API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal manual-upload collection API path that stores user-provided
base64 content as a raw artifact, links it to a completed collection run, and
records audit events.

This DIFF does not authorize multipart upload dependencies, worker scheduling,
normalization, chunking, evidence generation, external calls, or local project
filesystem collection.

## Baseline Facts

- DIFF-000 through DIFF-026 are locked.
- The worker currently has only a scaffold health task.
- `python-multipart` is not part of the API requirements, so multipart file
  upload is out of scope for this DIFF.
- DIFF-026 added local content-addressed artifact storage and raw artifact
  metadata creation.
- `CollectionRun` and `RawArtifact` models already exist.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-027-manual-upload-collection-api.md`
- `docs/api.md`
- `services/api/app/collection_runs.py`

Allowed behavior:

- Add a minimal `POST /collection-runs/manual-upload` route that accepts
  base64 content.
- Require the source to exist, be enabled, and have `source_type` equal to
  `manual_upload`.
- Require the source permission to belong to the source and allow `collect` or
  `read` when operations are listed.
- Store bytes through the existing content-addressed artifact store helper.
- Create a non-dry-run collection run linked to the source.
- Create a raw artifact metadata row linked to the collection run.
- Emit audit events for the collection run and raw artifact.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Multipart upload handling or dependency changes.
- Local project collection.
- Filesystem traversal outside the configured artifact store.
- Normalization execution.
- Chunk or evidence generation.
- Embedding or graph writes.
- Artifact file serving.
- Export generation.
- Browser automation.
- External model calls.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-027` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.collection_runs import ManualUploadCollectionCreate; payload = ManualUploadCollectionCreate(source_id='src', source_permission_id='perm', content_base64='aGVsbG8=', filename='hello.txt'); print(payload.filename, payload.content_base64)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.collection_runs import ManualUploadCollectionCreate; payload = ManualUploadCollectionCreate(source_id='src', source_permission_id='perm', content_base64='aGVsbG8=', filename='hello.txt'); print(payload.filename, payload.content_base64)"` because the host Python environment does not have `fastapi` installed

## Completion Criteria

This DIFF is complete when:

- Manual upload collection route exists.
- It validates source, source type, enabled state, permission ownership, and
  allowed operations.
- It creates a completed non-dry-run collection run.
- It stores raw bytes using the content-addressed artifact helper.
- It creates linked raw artifact metadata and audit events.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Multipart/manual file upload UX.
- Worker-backed collection.
- Local project collection.
- Normalization, chunking, and evidence generation.
