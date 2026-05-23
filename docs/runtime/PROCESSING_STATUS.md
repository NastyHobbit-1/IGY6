# Processing Status Diagnostics

Manual upload processing currently has two paths:

- `POST /collection-runs/manual-upload` creates a completed collection run, a
  raw artifact, and a queued `collection_normalization` work item.
- Python/Celery worker tasks own live runtime processing from raw artifacts to
  normalized documents, chunks, evidence, and Qdrant vector memory.

The Rust worker crate is a deterministic planning foundation. It does not
replace the live Python/Celery worker yet. The Rust gateway dispatch route is
safe-limited: it records dispatch metadata and audit events but does not invoke
Celery or arbitrary runtime execution.

DIFF-141 audits worker execution parity and recommends migrating worker
execution to Rust one job family at a time. Until that parity is implemented and
verified, Python/Celery `worker` remains required for live processing. `beat`
also remains in the stack; no repo-defined beat schedule currently exists, but
scheduled-work retirement or replacement requires a later DIFF.

DIFF-142 adds the Rust queue-claim contract only. It validates claim eligibility
for `collection_normalization`, `document_chunking`, and `chunk_vector_upsert`
work items, but it does not execute those jobs, read artifacts, write database
rows, write audit events, call Qdrant, or replace Celery.

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
