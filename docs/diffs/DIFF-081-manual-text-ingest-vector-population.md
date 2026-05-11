# DIFF-081: Manual Text Ingest Vector Population

Status: Locked

## Type

Change-bearing

## Objective

Add the next minimal ingestion path that can take an authorized local manual
UTF-8 text source through raw artifact storage, normalized document creation,
chunk/evidence creation, and configured Qdrant chunk vector collection
creation/population.

## Baseline Facts

- `DIFF-080` is locked and must not be edited.
- The current fixed commit before this DIFF is
  `7314509 DIFF-080 chat retrieval missing vector collection`.
- `/chat/retrieval-preview` handles a missing configured Qdrant collection as
  empty retrieval context.
- The dashboard can still show zero sources, documents, chunks, evidence, and
  `igy6_chunks` missing before ingestion.
- Source registration and source permissions exist in `services/api/app/sources.py`.
- Manual upload and local project collection create raw artifacts and queued
  normalization work items in `services/api/app/collection_runs.py`.
- Worker tasks in `services/worker/app/tasks.py` already normalize UTF-8 text
  artifacts, generate chunks/evidence, and upsert deterministic local hash
  vectors into Qdrant.
- API vector memory in `services/api/app/vector_memory.py` already has local
  deterministic hash embeddings and collection creation helpers.
- The existing MVP Action Console exposes source creation, manual upload
  collection, and work item dispatch, but there is no single API path that
  completes manual text ingestion through vector upsert synchronously.
- During inspection, `upsert_chunk_vectors` in
  `services/api/app/vector_memory.py` contained a runtime-invalid branch in the
  upsert path; DIFF-081 may correct it because reliable vector population is in
  scope.

## Allowed Scope

- `docs/diffs/DIFF-081-manual-text-ingest-vector-population.md`
- `services/api/app/collection_runs.py`
- `services/api/app/vector_memory.py`
- Existing API routes related to source ingestion or collection runs
- Narrow tests under `services/api/tests/`
- Minimal UI change only if an existing button/form is present but not wired

## Prohibited Scope

- Do not edit locked DIFF files.
- Do not create the vector collection from chat retrieval preview.
- Do not change Docker, Compose, ports, or `.env`.
- Do not add external model calls.
- Do not add broad architecture, auth, cloud services, external SaaS, or
  unrelated features.
- Do not refactor unrelated files.
- Do not rename existing concepts unless required by current code.
- Do not make destructive database or Qdrant changes.
- Do not wipe existing volumes or data.
- Do not add dependencies.

## Required Tags

- Commit message must include `DIFF-081`.
- Final response must identify `DIFF-081`.

## Verification

- `git status --short`
- `git diff --check`
- Compile check for `services/api` and `services/worker`
- Narrow ingestion/vector test
- Direct API check for `/chat/retrieval-preview`
- Direct API or curl check for `/memory/vector/chunks`

## Completion Criteria

- Manual UTF-8 text can be ingested through a deterministic local path into raw
  artifact, normalized document, chunk, evidence item, and Qdrant vector state.
- Repeating the same manual text ingest is safe: existing artifact/document/chunk
  records are reused where possible and Qdrant point upsert remains idempotent.
- Missing `igy6_chunks` may be created by ingestion/vector upsert, not chat
  preview.
- Missing source path/input, empty content, unsupported file type, Qdrant
  unavailability, and already-existing collection cases have explicit handling.
- Dashboard state can reflect sources/documents/chunks/evidence and vector
  collection existence after successful ingestion.
- `/chat/retrieval-preview` continues returning HTTP 200.
- Verification results are recorded below before locking.

## Verification Result

- `git status --short` was run before edits and showed a clean worktree.
- `git status --short` after edits showed only DIFF-081 scoped files changed.
- `git diff --check` passed.
- `python3 -m compileall services/api services/worker` passed.
- `.venv/bin/python -m compileall services/api services/worker` passed.
- `.venv/bin/python services/api/tests/test_vector_memory_missing_collection.py`
  passed with two tests.
- Direct `curl http://127.0.0.1:8000/memory/vector/chunks` returned HTTP 200
  with `{"collection_name":"igy6_chunks","exists":false,"detail":null}` before
  live ingestion.
- Direct `curl -X POST http://127.0.0.1:8000/chat/retrieval-preview ...`
  returned HTTP 200 with `answer_status: not_generated`,
  `collection_exists: false`, and zero hits before live ingestion.
- A later live OpenAPI check could not connect because no API server was
  listening on `127.0.0.1:8000`; the full live ingest endpoint was not executed
  without restarting/rebuilding the local API stack.
