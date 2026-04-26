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

Expected readiness checks include PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and
Phoenix. A healthy Phase 0 stack returns overall status `ok`.

## Worker

```bash
docker compose -f infra/docker-compose.yml --env-file .env exec -T worker celery -A app.celery_app:celery_app inspect ping
```

Expected result: one Celery node responds with `pong`.

## Migrations

The API container runs `alembic upgrade head` before starting Uvicorn.

Manual migration command:

```bash
docker compose -f infra/docker-compose.yml --env-file .env run --rm api alembic upgrade head
```

Check current migration:

```bash
docker compose -f infra/docker-compose.yml --env-file .env exec -T api alembic current
```

Expected Phase 0 revision: `0001_phase0_foundation`.

List foundational tables:

```bash
docker compose -f infra/docker-compose.yml --env-file .env exec -T postgres psql -U adaptive -d adaptive_intelligence -c "\dt"
```

Expected Phase 0 tables:

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
