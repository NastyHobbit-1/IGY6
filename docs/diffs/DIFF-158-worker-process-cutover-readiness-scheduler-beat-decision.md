# DIFF-158 Worker Process Cutover Readiness Scheduler Beat Decision

## Scope

DIFF-158 audits worker process cutover readiness after successful isolated Rust
worker live canaries for:

- `collection_normalization`
- `document_chunking`
- `chunk_vector_upsert`

This DIFF decides whether the Rust worker can replace Python/Celery worker
process ownership now, and whether Celery Beat can be removed, retained, or
replaced. This DIFF does not process broad queues, does not remove
`services/worker/`, does not remove Docker Compose `worker` or `beat`, does
not disable Python/Celery, and does not claim full Rust-only runtime.

## Decision

Decision B: Rust worker process cutover is not ready.

The isolated live canaries prove that the Rust worker can execute exactly one
explicitly selected canary work item for each covered job family. They do not
prove production worker process ownership.

## Current Evidence

DIFF-153 verified `collection_normalization` against isolated synthetic
PostgreSQL and a synthetic `IGY6_DATA_ROOT`.

DIFF-155 verified `document_chunking` against isolated synthetic PostgreSQL.

DIFF-157 fixed Qdrant point ID compatibility and verified
`chunk_vector_upsert` against isolated synthetic PostgreSQL and isolated local
Qdrant.

`crates/igy6-worker` now has a Rust binary with safe modes:

- `--help`
- `--check`
- `--dry-run`
- `--once`
- `--once --canary-live --canary-work-item ID` with
  `IGY6_WORKER_LIVE_CANARY=DIFF-148`

The live side-effect executor remains intentionally bounded to one explicitly
selected canary work item. It is not a generic worker daemon.

## Compose And Runtime Posture

`infra/docker-compose.yml` still wires:

- `worker`: `celery -A app.celery_app:celery_app worker --loglevel=INFO`
- `beat`: `celery -A app.celery_app:celery_app beat --loglevel=INFO`

Docker Compose was not changed in DIFF-158.

`services/worker/` remains the active Python/Celery worker source tree for
production worker and beat containers.

## Exact Blockers

Rust worker process cutover is blocked by:

- No long-running Rust worker daemon mode is implemented.
- No generic live queue polling loop processes queued work items without a
  named canary work item.
- Live Rust execution still requires `--once --canary-live --canary-work-item
  ID` and `IGY6_WORKER_LIVE_CANARY=DIFF-148`.
- No Dockerfile or Compose service exists for a production Rust worker
  container.
- No production retry, backoff, and repeated polling behavior has been proven
  for the Rust worker process.
- No graceful shutdown or in-flight job handoff behavior has been proven for a
  long-running Rust worker process.
- No worker health/readiness posture has been defined for Docker Compose.
- No production rollback plan has been proven for replacing the Python/Celery
  worker container with a Rust worker container.
- Celery Beat scheduler posture has not been replaced or explicitly retired.
- No side-by-side canary deployment has shown the Rust worker owning broad queue
  processing without racing Python/Celery.

## Worker And Beat Decision

Python/Celery `worker` remains required.

Reason: production live processing still needs a long-running worker process
that owns queue polling, retries, shutdown behavior, and repeated execution.
Rust has proven only isolated one-item canary execution for the covered job
families.

Python/Celery `beat` remains required.

Reason: even though DIFF-141 found no repo-defined beat schedule, the scheduler
posture has not been replaced or formally retired. Removing `beat` would be a
runtime ownership decision that DIFF-158 does not prove safe.

## Full Rust-Only Runtime Claim

Full Rust-only repository or runtime is not claimed.

The active API path is Rust-native and FastAPI fallback is removed, but
Python/Celery worker and beat remain active runtime services.

## Next Recommended DIFF

DIFF-159 should add or prove a production-shaped Rust worker process canary:

- long-running Rust worker mode
- bounded generic queue polling for covered job families
- production retry/backoff and graceful shutdown behavior
- Dockerfile and optional Compose canary service
- explicit scheduler/beat replacement or retirement decision
- rollback posture for returning worker ownership to Python/Celery

Only after that is proven should a later DIFF remove or replace the Python
`worker` and `beat` services.
