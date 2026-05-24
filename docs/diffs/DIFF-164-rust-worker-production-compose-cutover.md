# DIFF-164 Rust Worker Production Compose Cutover

Date: 2026-05-24

## Decision

Decision A: production worker ownership can move to Rust.

DIFF-163 added the production-capable Rust worker daemon:

```bash
igy6-worker --daemon --claim-limit N --poll-interval-ms MS
```

DIFF-164 updates base Docker Compose so the active `worker` service uses that
daemon instead of Python/Celery.

## Compose Change

`infra/docker-compose.yml` now defines `worker` as:

- build context: `..`
- Dockerfile: `crates/igy6-worker/Dockerfile`
- command:
  `igy6-worker --daemon --claim-limit ${IGY6_WORKER_CLAIM_LIMIT:-4} --poll-interval-ms ${IGY6_WORKER_POLL_INTERVAL_MS:-1000}`
- dependencies: healthy `postgres` and healthy `qdrant`
- data root inside the container: `/workspace/storage`
- shutdown marker env:
  `${IGY6_WORKER_SHUTDOWN_FILE:-worker/control/shutdown}`

The previous Python/Celery `worker` service from `services/worker` is no longer
active in base Compose.

## Beat Posture

`beat` is removed from base Compose.

Reason: `services/worker/app/celery_app.py` defines no `beat_schedule`, and repo
search found no `crontab`, `periodic_task`, `on_after_configure`, or scheduled
Celery registration. There is no repo-defined scheduled Celery work to replace
in this DIFF.

## Retained Files

`services/worker/` is not archived in DIFF-164. It remains in the repository for
rollback/archive review until the final audit DIFF.

## Rollback

Rollback is to revert DIFF-164 or restore the previous Python/Celery `worker`
and `beat` service definitions from git, then validate Compose before restart:

```bash
docker compose -f infra/docker-compose.yml --env-file .env config
docker compose -f infra/docker-compose.yml --env-file .env up --build worker
```

Do not move runtime data into the repository during rollback.

## Rust-Only Claim

Full Rust-only repository/runtime is not finally claimed in DIFF-164. Active
base Compose no longer runs Python/Celery worker or beat, but `services/worker/`
is retained until the final archive/audit DIFF confirms removal and rollback
posture.

## Next

DIFF-165 should archive Python worker source if safe and lock the final Rust-only
runtime audit.
