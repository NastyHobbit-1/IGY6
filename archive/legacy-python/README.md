# Legacy Python Archive

DIFF-139 and DIFF-165 archive legacy Python components that are no longer wired
into the active runtime path.

## Archived

- `services-api/`: former FastAPI API service, including historical app modules,
  tests, Dockerfile, requirements, and Alembic migration history.
- `services-worker/`: former Python/Celery worker service, including historical
  Celery app modules, Dockerfile, and requirements.

The active Docker Compose stack no longer defines `legacy-api`, no longer
defines a Python/Celery `worker`, and no longer defines Celery `beat`. The Rust
gateway no longer proxies unsupported routes to FastAPI, and the active
production worker is the Rust daemon from `crates/igy6-worker/Dockerfile`.

## Retained Elsewhere

- No legacy Python application service is active in base Docker Compose.

Do not delete this archive blindly. It remains useful for route parity history,
migration archaeology, rollback analysis, and future DIFF review. Rollback to
the historical Python/Celery worker topology requires restoring matching Compose
service definitions from git history and validating Compose before restart.
