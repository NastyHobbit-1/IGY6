# DIFF-080: Chat Retrieval Missing Vector Collection

Status: Locked

## Type

Change-bearing

## Objective

Handle a missing configured Qdrant chunk vector collection as an empty local
retrieval state for chat retrieval preview, instead of returning `502 Bad
Gateway`.

## Baseline Facts

- `DIFF-079` is locked and must not be edited or reused.
- The dashboard vector collection status endpoint already reports a missing
  collection as `exists: false`.
- `search_chunk_vectors` currently turns any Qdrant search response with status
  code `>= 400` into `502 Bad Gateway`.
- Qdrant can return a `404` search response when the configured collection, such
  as `igy6_chunks`, does not exist.
- `/chat/retrieval-preview` uses hydrated retrieval, which calls
  `search_chunk_vectors`.
- No existing Python test suite files were present before this DIFF.

## Allowed Scope

- `docs/diffs/DIFF-080-chat-retrieval-missing-vector-collection.md`
- `services/api/app/vector_memory.py`
- `services/api/app/retrieval.py` only if response metadata is required there
- `services/api/app/chat.py` only if response metadata is required there
- Narrow tests under `services/api/tests/`

## Prohibited Scope

- No external model calls.
- No automatic Qdrant collection creation from retrieval preview.
- No unrelated UI changes.
- No Docker changes.
- No `.env` changes.
- No ingestion behavior changes.
- No database schema changes.
- No migrations.
- No dependency changes.
- No broad refactor.
- No unrelated cleanup.

## Required Tags

- Commit message must include `DIFF-080`.
- Final response must identify `DIFF-080`.

## Verification

- `git diff --check`
- `python3 -m compileall services/api services/worker`
- Run the narrow missing-collection test.

## Completion Criteria

- Missing Qdrant chunk collection during vector search returns an HTTP 200
  response path for `/chat/retrieval-preview` with zero retrieval hits.
- The vector collection dashboard behavior remains unchanged and can still
  report `exists: false`.
- The fix does not create the missing collection.
- The fix does not call any external model.
- A narrow test covers the missing-collection vector search behavior.
- Verification results are recorded below before locking.

## Verification Result

- `git diff --check` passed.
- `.venv/bin/python services/api/tests/test_vector_memory_missing_collection.py`
  passed.
- `.venv/bin/python -m compileall services/api services/worker` passed.
- Plain `python3 services/api/tests/test_vector_memory_missing_collection.py`
  was not used for final verification because the system Python environment
  does not have project dependency `httpx`; the project venv does.
