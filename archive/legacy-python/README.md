# Legacy Python Archive

DIFF-139 archives legacy Python components that are no longer wired into the
active runtime path.

## Archived

- `services-api/`: former FastAPI API service, including historical app modules,
  tests, Dockerfile, requirements, and Alembic migration history.

The active Docker Compose stack no longer defines `legacy-api`, and the Rust
gateway no longer proxies unsupported routes to FastAPI.

## Retained Elsewhere

- `services/worker/` remains active. Docker Compose still runs `worker` and
  `beat` from this directory using Celery.

Do not delete this archive blindly. It remains useful for route parity history,
migration archaeology, rollback analysis, and future DIFF review.
