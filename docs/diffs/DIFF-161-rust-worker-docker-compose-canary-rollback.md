# DIFF-161: Rust Worker Docker/Compose Canary And Rollback

## Status

Complete.

Decision: A. A Rust worker Docker/Compose canary service can be added safely as
an opt-in service. It does not replace the Python/Celery `worker` service and
does not replace or retire `beat`.

## Scope

DIFF-161 adds a container build target for `igy6-worker` and a separate Compose
override file for the Rust worker canary:

- `crates/igy6-worker/Dockerfile`
- `infra/docker-compose.rust-worker-canary.yml`

The base production Compose file remains unchanged. Normal startup with only
`infra/docker-compose.yml` still starts the Rust API plus the Python/Celery
`worker` and `beat` services.

## Canary Service

The canary service is named `rust-worker-canary` and is behind the
`rust-worker-canary` Compose profile. The override also defines isolated
profile-scoped `rust-worker-canary-postgres` and `rust-worker-canary-qdrant`
services so the canary does not use the production worker database or Qdrant
services. It runs:

```bash
igy6-worker --canary-loop --max-jobs 3 --max-idle-polls 1 --claim-limit 1 --poll-interval-ms 100
```

The service requires an explicit synthetic canary data root:

- `RUST_WORKER_CANARY_DATA_ROOT`
- optional `RUST_WORKER_CANARY_QDRANT_CHUNK_COLLECTION`

The container sets `IGY6_WORKER_PROCESS_CANARY=DIFF-159` and
`IGY6_WORKER_COMPOSE_CANARY=DIFF-161`. The worker process still requires the
runtime canary gate before mutating data.

The mounted canary data root is read-only inside the worker container at
`/workspace/canary-storage`. Fixture SQL must therefore point raw artifact
storage paths at relative artifact paths under that data root, matching the
existing `scripts/rust-worker-canary-fixture.py` behavior.

## Startup Command

Use the override only with synthetic canary data. The service should be targeted
directly so Compose starts only the isolated canary dependencies and the canary
worker, not the Python/Celery worker or beat services:

```bash
RUST_WORKER_CANARY_DATA_ROOT=/tmp/igy6-diff161-canary \
RUST_WORKER_CANARY_QDRANT_CHUNK_COLLECTION=igy6_diff161_chunks \
docker compose -f infra/docker-compose.yml \
  -f infra/docker-compose.rust-worker-canary.yml \
  --env-file .env.example \
  --profile rust-worker-canary \
  up --build rust-worker-canary
```

This command is intentionally not the normal production startup command.

## Rollback Command

Rollback is to stop and remove only the canary service. The existing
Python/Celery `worker` and `beat` services remain available in the base Compose
file:

```bash
docker compose -f infra/docker-compose.yml \
  -f infra/docker-compose.rust-worker-canary.yml \
  --env-file .env.example \
  --profile rust-worker-canary \
  stop rust-worker-canary

docker compose -f infra/docker-compose.yml \
  -f infra/docker-compose.rust-worker-canary.yml \
  --env-file .env.example \
  --profile rust-worker-canary \
  rm -f rust-worker-canary
```

If the canary stack dependencies were started only for this test, stop them with
the same explicit Compose file set after collecting observations.

## Safety Posture

- The Rust worker canary service is opt-in only.
- The default production Compose file is unchanged.
- The override uses isolated canary PostgreSQL and Qdrant services.
- Python/Celery `worker` remains active for production processing.
- Python/Celery `beat` remains active until scheduler posture is resolved.
- The canary loop is bounded by `max_jobs=3`, `max_idle_polls=1`,
  `claim_limit=1`, and `poll_interval_ms=100`.
- The canary requires explicit canary environment values and the Rust worker
  process gate.
- The canary data root is synthetic and mounted read-only.
- Full Rust-only runtime is not claimed.

## Remaining Blockers

DIFF-161 adds service wiring and rollback posture, but it does not run the
Compose canary end to end and does not replace production worker ownership.
Before DIFF-162 can remove or replace the Python/Celery worker, the project
still needs:

- one observed Compose-level Rust worker canary run against synthetic data;
- confirmation that the canary service does not race a running Python/Celery
  worker for production work items;
- a production cutover command that replaces worker ownership intentionally;
- a final beat/scheduler replacement, retirement, or retention decision;
- rollback verification from Rust worker ownership back to Python/Celery.

## Runtime Claim

IGY6 remains Rust-primary with a Rust-native API path and retained Python/Celery
worker and beat services. Rust-only runtime is not claimed.

## Next Recommended DIFF

DIFF-162: replace Python/Celery worker and resolve beat/scheduler posture.
