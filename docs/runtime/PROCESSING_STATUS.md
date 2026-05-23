# Processing Status Diagnostics

Manual upload processing currently has two paths:

- `POST /collection-runs/manual-upload` creates a completed collection run, a
  raw artifact, and a queued `collection_normalization` work item.
- Python/Celery worker tasks still own the live runtime process for end-to-end
  processing from raw artifacts to normalized documents, chunks, evidence, and
  Qdrant vector memory.

The Rust worker crate now has DIFF-143 `collection_normalization`, DIFF-144
`document_chunking`, and DIFF-145 `chunk_vector_upsert` execution planners plus
SQL/audit/status executor contracts. It preserves Python/Celery semantics for
UTF-8 raw artifact normalization, `normalized_documents` insert shape,
deterministic chunk and evidence item inserts, duplicate skips, deterministic
local chunk vectors, Qdrant collection/status/upsert request planning,
originating work-item status, completion/failure audit events, and chained
work-item creation through the processing pipeline. It does not yet replace
the live Python/Celery worker process. The Rust gateway dispatch route is
safe-limited: it records dispatch metadata and audit events but does not invoke
Celery or arbitrary runtime execution.

DIFF-141 audits worker execution parity and recommends migrating worker
execution to Rust one job family at a time. Until that parity is implemented and
verified, Python/Celery `worker` remains required for live processing. `beat`
also remains in the stack; no repo-defined beat schedule currently exists, but
scheduled-work retirement or replacement requires a later DIFF.

DIFF-142 adds the Rust queue-claim contract only. DIFF-143 adds
`collection_normalization` execution parity planning and executor contracts.
DIFF-144 adds `document_chunking` execution parity planning and executor
contracts. DIFF-145 adds `chunk_vector_upsert` execution parity planning,
including deterministic local vector and Qdrant request contracts. Live worker
process ownership and beat/scheduled-work posture remain Python/Celery-backed.

DIFF-146 decides the worker process cutover is not ready. `crates/igy6-worker`
is still a library crate with contracts and tests, not a long-running worker
binary or container. It does not poll queued work, atomically claim jobs from
PostgreSQL in a runtime loop, read artifacts during execution, apply DB/audit
writes, execute Qdrant HTTP requests, or provide worker health/shutdown/retry
behavior. `worker` and `beat` therefore remain in Docker Compose.

## Pipeline

```text
Raw Artifact
  -> Normalized Document
  -> Chunks
  -> Evidence Items
  -> Vector Memory (Qdrant)
  -> Graph Memory when scoped by graph sync routes
```

## Status Check

Run:

```bash
python3 scripts/processing-status-smoke.py
```

The script checks an already-running stack only. It validates:

- Docker Compose config.
- `worker`, `redis`, `postgres`, `qdrant`, `api`, and `web` are running.
- Redis responds to `PING`.
- Postgres responds to `pg_isready`.
- API readiness responds.
- Work items can be inspected.
- Qdrant vector status can be inspected through the API.

It does not create records, delete records, start services, stop services, or
read private runtime data directly.

## Interpreting Results

- `queued`: work exists but has not been processed.
- `running`: worker task is currently processing.
- `completed`: task completed and may have created the next chained work item.
- `failed`: inspect `error_message` and worker logs.
- No chunks/evidence after upload: normalization or chunking may still be
  queued, or worker processing may have failed.
- Qdrant collection missing: vector collection may not have been ensured or
  vector upsert has not run yet.

## Logs

Worker logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker
```

API logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
```

Redis logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 redis
```

Optional local Ollama check:

```bash
scripts/ollama-local-setup.sh --check
```

Ollama is not required for worker processing. Local model setup is optional and
does not replace deterministic evidence fallback.
