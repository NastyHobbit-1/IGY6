# DIFF-163: Production Rust Worker Daemon Mode

## Status

Complete.

Decision: A. A production-capable Rust worker daemon mode can be implemented
safely. It is added as an explicit runtime mode, but production Docker Compose
worker ownership is not changed in this DIFF.

## Runtime Mode Added

The Rust worker binary now accepts:

```bash
igy6-worker --daemon --claim-limit 1 --poll-interval-ms 1000
```

`--daemon` is separate from `--canary-loop`:

- it does not require `IGY6_WORKER_PROCESS_CANARY`;
- it is opt-in by command invocation;
- it reuses the verified PostgreSQL claim, DB write, audit write, artifact
  read, and Qdrant execution path;
- it polls repeatedly until a shutdown marker is observed or a fatal
  PostgreSQL/runtime error stops the process;
- it keeps `claim_limit` bounded to `1..16`;
- it keeps `poll_interval_ms` bounded to `100..60000`;
- it validates `DATABASE_URL`, `QDRANT_URL`, `IGY6_DATA_ROOT`, Qdrant
  collection, vector size, and the shutdown marker path before execution.

The safe default remains:

```bash
igy6-worker --check
```

## Graceful Shutdown

Daemon mode checks a relative shutdown marker under `IGY6_DATA_ROOT` between
polls. The default marker is:

```text
worker/control/shutdown
```

The marker path can be configured with `IGY6_WORKER_SHUTDOWN_FILE`, but it must
remain relative and must not contain parent traversal. This keeps shutdown
control inside the configured data root.

## Failure And Retry Posture

Daemon mode does not retry a failed job unboundedly. If a claimed job fails
during execution, the existing worker failure path marks the work item failed
and writes a failure audit event. The daemon then continues polling for other
eligible queued jobs.

Idle polling is retried by sleeping for `poll_interval_ms` before polling again.
Fatal PostgreSQL connection or claim errors stop the daemon so the process
supervisor can restart it.

## Compose And Python Posture

`infra/docker-compose.yml` is unchanged in DIFF-163. Production still runs:

- `worker`: Python/Celery from `services/worker`
- `beat`: Python/Celery beat from `services/worker`

The Rust daemon mode is ready for a later Compose cutover DIFF, but that cutover
is not performed here. `services/worker/` is not archived.

## Rollback Posture

Because production Compose is unchanged, rollback is to stop any manually run
Rust daemon and keep the Python/Celery `worker` and `beat` services active.
Future Compose cutover rollback must restore the current Python/Celery worker
and beat definitions before restart.

## Runtime Claim

IGY6 remains Rust-primary with a Rust-native API path and retained Python/Celery
worker and beat services. Rust-only runtime is not claimed.

## Next Recommended DIFF

DIFF-164: cut production Docker Compose worker ownership from Python/Celery to
the Rust `--daemon` worker and resolve beat/scheduler posture.
