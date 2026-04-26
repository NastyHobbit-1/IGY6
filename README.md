# IGY6

Phase 0 skeleton for the Adaptive Intelligence System.

This repository is intentionally local-first. Phase 0 only provides the
monorepo structure, service skeletons, Docker Compose wiring, health checks,
initial PostgreSQL migration, and documentation placeholders. It does not
implement ingestion, browser automation, embeddings, graph extraction,
evidence-backed chat, prediction/advice logic, or self-improvement experiments.

## Services

- Web UI: Next.js, bound to `127.0.0.1:3000`
- API: FastAPI, bound to `127.0.0.1:8000`
- Worker: Celery worker
- Beat: Celery Beat scheduler
- PostgreSQL: state, audit, foundational control tables
- Redis: Celery broker/result backend
- Qdrant: vector memory service placeholder
- Neo4j: graph memory service placeholder
- MLflow: experiment tracking placeholder, using local file-backed storage in Phase 0
- Phoenix: observability placeholder

## Run Locally

1. Copy the environment template:

```bash
cp .env.example .env
```

2. Review local-only placeholder values in `.env`.

3. Start the Phase 0 stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

4. Check API health:

```bash
curl http://127.0.0.1:8000/health/live
curl http://127.0.0.1:8000/health/ready
```

5. Open the web status page:

```text
http://127.0.0.1:3000
```

6. Stop the stack when finished:

```bash
docker compose -f infra/docker-compose.yml --env-file .env down
```

## Migrations

The API container applies Alembic migrations on startup. To run manually:

```bash
docker compose -f infra/docker-compose.yml --env-file .env run --rm api alembic upgrade head
```

## Verification

Phase 0 was verified with:

```bash
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
docker compose -f infra/docker-compose.yml --env-file .env.example up -d
curl http://127.0.0.1:8000/health/ready
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T api alembic current
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T worker celery -A app.celery_app:celery_app inspect ping
```

Expected API readiness status:

```json
{"status":"ok"}
```

## Phase 0 Boundaries

Phase 0 must remain read-only and skeletal:

- No real source ingestion.
- No browser automation.
- No embeddings.
- No graph extraction.
- No evidence-backed chat.
- No prediction/advice logic.
- No self-improvement experiments.
- No ruvnet components.
- No remote service exposure by default.
- No hard-coded secrets.

Every future source access and important decision must go through policy,
approval, and audit paths.
