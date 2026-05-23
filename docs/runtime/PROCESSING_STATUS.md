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

DIFF-147 adds the `igy6-worker` binary as a safe runtime harness. Its default
mode is `--check`, and it also supports `--dry-run`, `--once`, and `--help`.
These modes validate configuration and plan bounded queue/claim behavior
without connecting to PostgreSQL, mutating queue rows, reading artifacts,
writing audits, calling Qdrant, controlling Celery, or replacing beat. Live
execution is not enabled by DIFF-147.

DIFF-148 adds an explicit one-job canary gate:
`--once --canary-live --canary-work-item ID`. Reporting
`live_execution_enabled=true` additionally requires
`IGY6_WORKER_LIVE_CANARY=DIFF-148`. The canary emits structured states and
side-effect verification planning for DB writes, audit writes, artifact reads,
and Qdrant calls. DIFF-148 does not execute those side effects; they remain
planned-only, so Python/Celery remains the production worker path.

DIFF-149 implements the live canary executor behind those same gates. When both
the CLI canary flags and `IGY6_WORKER_LIVE_CANARY=DIFF-148` are present, Rust
may claim and execute exactly one selected canary work item. Implemented live
side effects are PostgreSQL claim/status writes, worker audit events, scoped
artifact reads under `IGY6_DATA_ROOT/artifacts`, job-family DB writes, and Qdrant
collection/point work only for `chunk_vector_upsert`. Broad queue polling,
long-running process ownership, Docker Compose Rust worker wiring, and beat
replacement remain out of scope, so Python/Celery remains the production worker
path.

DIFF-150 audits that posture and chooses Decision B: worker process cutover is
not ready. No controlled real canary was run because no explicitly selected safe
runtime work item was provided. Static verification covers the canary gates,
non-mutating defaults, SQL claim shape, artifact path safety, and Qdrant request
boundaries, but live PostgreSQL/audit/artifact/Qdrant observations remain
unverified. Docker Compose still runs Python/Celery `worker` and `beat`.

DIFF-151 attempts to advance to an observed canary audit, but chooses Decision
B because no explicitly selected safe queued work item ID was available. No live
canary command with `IGY6_WORKER_LIVE_CANARY=DIFF-148` was run. The required
next preparation step is to create or select one non-sensitive queued canary
work item, prevent a Python/Celery race for that item during the canary window,
and then record the single Rust canary command plus observed PostgreSQL,
`audit_events`, artifact, and Qdrant results.

DIFF-152 chooses Decision A and adds a deterministic safe fixture helper:
`scripts/rust-worker-canary-fixture.py`. The selected canary work item ID is
`diff-152-canary-work-item`, with one synthetic `collection_normalization`
fixture path. The helper is non-mutating by default and can emit the seed SQL
and synthetic artifact plan needed for a later controlled canary. DIFF-152 does
not run the live Rust worker canary, does not process broad queues, and does
not change Docker Compose worker or beat ownership.

DIFF-153 chooses Decision A and runs exactly one gated live Rust canary against
the DIFF-152 selected fixture in an isolated local PostgreSQL canary container
and `/tmp/igy6-diff153-canary` data root. Observed side effects: the selected
work item moved to `completed`, claim/start/success audit rows were written,
the synthetic artifact under `IGY6_DATA_ROOT/artifacts` was read, one
`normalized_documents` row was written, and one chained `document_chunking` work
item was queued. Qdrant side effects were not expected because the selected
canary was `collection_normalization`. Python/Celery `worker` and `beat` remain
active because live `document_chunking`, live `chunk_vector_upsert`, broad Rust
worker ownership, and scheduler posture are still not replaced.

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

Rust worker harness check:

```bash
cargo run -p igy6-worker -- --check
```

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
