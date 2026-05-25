# Processing Status Diagnostics

Manual upload processing currently has two paths:

- `POST /collection-runs/manual-upload` creates a completed collection run, a
  raw artifact, and a queued `collection_normalization` work item.
- The production Docker Compose `worker` service now runs the Rust worker
  daemon for end-to-end processing from raw artifacts to normalized documents,
  chunks, evidence, and Qdrant vector memory.

The Rust worker crate preserves the former Python/Celery processing semantics
for UTF-8 raw artifact normalization, normalized-document inserts,
deterministic chunk and evidence item inserts, duplicate skips, deterministic
local chunk vectors, Qdrant collection/upsert behavior, originating work-item
status, completion/failure audit events, and chained work-item creation through
the processing pipeline. DIFF-153 through DIFF-160 verified those side effects
through isolated Rust canaries. DIFF-163 added production daemon mode, DIFF-164
made the daemon the active production Compose worker, and DIFF-165 archived the
inactive Python/Celery worker source under
`archive/legacy-python/services-worker`.

No Python/Celery `worker` service and no Celery `beat` service remain active in
base Docker Compose. `beat` is retired because the archived worker source has no
repo-defined beat schedule or periodic task registration. The Rust gateway
dispatch route is safe-limited: it records dispatch metadata and audit events
but does not invoke Celery or arbitrary runtime execution.

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
- `worker`, `postgres`, `qdrant`, `api`, and `web` are running.
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

Rust worker harness check:

```bash
cargo run -p igy6-worker -- --check
```

Post-cutover runtime audit:

```bash
python3 scripts/post-cutover-runtime-audit.py
```

Post-cutover Rust-only runtime smoke suite:

```bash
scripts/post-cutover-smoke.sh --check
```

The smoke suite validates current Rust-only runtime posture, Docker Compose
ownership, route parity, the post-cutover audit, and Rust worker help/check
without starting services, stopping services, running broad worker queues,
mutating `.env`, or touching `IGY6_DATA_ROOT`. Live API health probes are
optional unless `--require-running` is supplied.

Rust worker canary plan:

```bash
cargo run -p igy6-worker -- --once --canary-live --canary-work-item example-work-item
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
