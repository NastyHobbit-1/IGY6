# DIFF-028: Local Project Collection API

Status: Locked

## Type

Change-bearing.

## Objective

Add a scoped local-project collection API path that reads approved files under a
registered local project source, stores them as raw artifacts, links them to a
completed collection run, and records audit events.

This DIFF does not authorize unrestricted filesystem traversal, normalization,
chunking, evidence generation, worker scheduling, browser automation, external
calls, or writes outside the configured artifact store.

## Baseline Facts

- DIFF-000 through DIFF-027 are locked.
- `local_project` source type and connector scaffold exist.
- DIFF-026 added local content-addressed artifact storage.
- DIFF-027 added an API-side collection-run pattern for creating raw artifacts
  from approved user-provided content.
- Source permissions store `scope_json`, but no local project collection rules
  are implemented yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-028-local-project-collection-api.md`
- `docs/api.md`
- `services/api/app/collection_runs.py`
- `services/api/app/local_project_collection.py`

Allowed behavior:

- Add a `POST /collection-runs/local-project` route.
- Require the source to exist, be enabled, and have `source_type` equal to
  `local_project`.
- Require the source permission to belong to the source and allow `collect` or
  `read` when operations are listed.
- Require explicit `scope_json.paths` entries.
- Resolve paths under the source location and reject escaping paths.
- Skip symlinks and non-files.
- Apply conservative `max_files` and `max_file_bytes` limits.
- Store collected file bytes through the existing content-addressed artifact
  helper.
- Create raw artifact metadata rows linked to the collection run.
- Record per-run summary counts and audit events.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Unbounded filesystem traversal.
- Following symlinks.
- Reading files outside the source location.
- Normalization execution.
- Chunk or evidence generation.
- Embedding or graph writes.
- Artifact file serving.
- Export generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-028` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.local_project_collection import collect_local_project_files; result = collect_local_project_files(source_location='/tmp/igy6-local-project-smoke', permission_scope={'paths':['.'], 'max_files': 10, 'max_file_bytes': 100000}, artifact_store_path='/tmp/igy6-local-project-artifacts'); print(result.total_files, result.collected_files)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: temp local project smoke with two files returned `2 2`

## Completion Criteria

This DIFF is complete when:

- Local project collection route exists.
- Collection is constrained to explicit permission scope paths under the source
  location.
- Symlinks and escaping paths are rejected or skipped.
- File count and file size limits are enforced.
- Raw artifact metadata rows are created and linked to a completed collection
  run.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Worker-backed collection.
- Normalized document creation.
- Chunking and evidence generation.
- UI source collection workflows.
