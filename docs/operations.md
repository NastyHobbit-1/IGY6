# Operations

## Local Startup

```bash
cp .env.example .env
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

## Health

```bash
curl http://127.0.0.1:8000/health/live
curl http://127.0.0.1:8000/health/ready
```

The Rust gateway readiness endpoint reports gateway status and current fallback
posture. Service-specific readiness should be checked through Docker Compose
health and logs.

## Worker

```bash
docker compose -f infra/docker-compose.yml --env-file .env exec -T worker celery -A app.celery_app:celery_app inspect ping
```

Expected result: one Celery node responds with `pong`.

## Migrations

The active API container is the Rust gateway and does not run Alembic on
startup. The legacy FastAPI Alembic history is archived under
`archive/legacy-python/services-api/migrations` and must be preserved until a
later migration-governance DIFF replaces or retires it.

Legacy migration inspection, if needed for archaeology, should use the archived
Python API environment explicitly rather than the active `api` container.

Historical command shape:

```bash
PYTHONPATH=archive/legacy-python/services-api alembic -c archive/legacy-python/services-api/alembic.ini current
```

The active Compose services no longer include a FastAPI API container.

List foundational tables directly:

```bash
docker compose -f infra/docker-compose.yml --env-file .env exec -T postgres psql -U adaptive -d adaptive_intelligence -c "\dt"
```

Expected foundational tables include:

- `sources`
- `source_permissions`
- `work_items`
- `approvals`
- `audit_events`
- `collection_runs`
- `raw_artifacts`
- `reports`

## MLflow Storage

Phase 0 uses local SQLite/file-backed MLflow storage inside the `mlflow_data`
Docker volume. A later experiment phase can migrate MLflow metadata to
PostgreSQL after the experiment schema and backup plan are finalized.

## Shutdown

```bash
docker compose -f infra/docker-compose.yml --env-file .env down
```

## Backups

Phase 0 defines named Docker volumes for PostgreSQL, Qdrant, Neo4j, MLflow, and
Phoenix. Formal backup/restore scripts are a later hardening task.
