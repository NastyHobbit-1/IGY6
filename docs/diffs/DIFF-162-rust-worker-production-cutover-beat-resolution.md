# DIFF-162: Rust Worker Production Cutover And Beat Resolution

## Status

Complete.

Decision: B. Rust worker replacement is not safe yet. Production Docker Compose
worker ownership remains Python/Celery, and `beat` remains active.

## Decision Evidence

DIFF-160 proved a bounded live Rust worker process-loop canary against isolated
synthetic PostgreSQL, synthetic `IGY6_DATA_ROOT`, and isolated local Qdrant.
DIFF-161 added Docker/Compose canary service wiring and rollback posture.

Those are necessary cutover steps, but the current Rust worker binary is still
canary-only for live loop execution:

- `igy6-worker --check` is the safe default.
- `--dry-run` and `--once` remain non-mutating planning modes unless paired
  with explicit one-item canary gates.
- `--canary-loop` requires `IGY6_WORKER_PROCESS_CANARY=DIFF-159`.
- `--canary-loop` is bounded by `--max-jobs`, `--max-idle-polls`,
  `--claim-limit`, and `--poll-interval-ms`.
- `--max-jobs` is bounded to 1 through 16, so the process intentionally exits.
- The Rust worker Dockerfile defaults to `igy6-worker --check`.
- The DIFF-161 Compose override is canary-only and profile-gated.

That behavior is appropriate for canaries, but it is not production worker
ownership. Replacing the production `worker` service with the current Rust
canary loop would create a worker that intentionally stops after a bounded job
or idle budget.

## Beat/Scheduler Posture

`services/worker/app/celery_app.py` does not define a `beat_schedule`, and the
repo search found no `crontab`, `periodic_task`, `on_after_configure`, or
scheduled Celery registration. That means no repo-defined beat schedule was
found.

Beat is still retained in this DIFF because production worker ownership itself
was not replaced. Removing beat while retaining the Python/Celery worker would
be a partial runtime topology change without a successful production Rust worker
cutover. Full Rust-only runtime is therefore not claimed.

## Compose Decision

`infra/docker-compose.yml` is unchanged:

- `worker` still builds `../services/worker` and runs Celery worker.
- `beat` still builds `../services/worker` and runs Celery beat.
- `api` remains the Rust gateway.
- The DIFF-161 canary override remains available for isolated canary work.
- The canary override remains profile-gated and uses a safe
  `/tmp/igy6-rust-worker-canary` default data root so Compose config validation
  can run without mutating `.env`.

## Exact Blockers

Before production worker ownership can move to Rust, the repo needs:

- a production Rust worker mode distinct from `--canary-loop`;
- long-running polling that does not intentionally stop after a 1..16 job
  budget;
- documented retry/backoff behavior for transient PostgreSQL, artifact, and
  Qdrant failures;
- graceful shutdown behavior suitable for Docker Compose;
- production health/readiness expectations for the worker service;
- a successful Compose-level Rust worker canary run against synthetic data;
- an explicit scheduler decision: remove beat after proving no scheduled work is
  required, or replace scheduled work with a Rust scheduler if one becomes
  necessary;
- rollback verification from production Rust worker ownership back to
  Python/Celery.

## Rollback Posture

No production worker replacement was performed, so rollback is preserving the
current base Compose file. If a future cutover changes production worker
ownership, rollback must restore the `worker` and `beat` service definitions
from `services/worker` and validate Compose before restart.

The DIFF-161 canary rollback remains:

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

## Runtime Claim

IGY6 remains Rust-primary with a Rust-native API path and retained Python/Celery
worker and beat services. Rust-only runtime is not claimed.

## Next Recommended DIFF

DIFF-163 cannot safely archive `services/worker` until production Rust worker
ownership and scheduler posture are resolved. The next DIFF should either add a
production Rust worker daemon and remove/replace Python worker plus beat, or
convert DIFF-163 into a final blocker audit instead of an archive.
