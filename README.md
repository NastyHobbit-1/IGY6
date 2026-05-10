# IGY6

Local-first foundation for the Adaptive Intelligence System.

This repository is intentionally local-first. The current foundation includes
the monorepo structure, Docker Compose wiring, health checks, PostgreSQL state
and audit tables, source and approval metadata, artifact and collection
metadata, evidence records, deterministic chunking, local deterministic
embedding scaffolds, Qdrant and Neo4j memory foundations, retrieval previews,
review metadata, worker tasks for normalization/chunking/vector upsert, report
metadata, and self-improvement queue/experiment metadata. It does not implement
answer generation, autonomous collection dispatch, browser automation,
production self-improvement execution, or system-changing actions.

## Services

- Web UI: Next.js, bound to `127.0.0.1:3000`
- API: FastAPI, bound to `127.0.0.1:8000`
- Worker: Celery worker
- Beat: Celery Beat scheduler
- PostgreSQL: state, audit, foundational control tables
- Redis: Celery broker/result backend
- Qdrant: vector memory service for deterministic chunk embeddings
- Neo4j: graph memory service for deterministic lineage relationships
- MLflow: experiment tracking service reserved for controlled experiments
- Phoenix: observability service reserved for trace review

## Run Locally

1. Copy the environment template:

```bash
cp .env.example .env
```

2. Review local-only placeholder values in `.env`.

3. Start the local stack:

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

The foundation is verified with:

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

## Current Boundaries

The system remains read-only and scaffolded by default:

- No autonomous source collection dispatch.
- No browser automation.
- No external embedding or answer-generation model calls.
- No generated evidence-backed answers.
- No automatic prediction/advice generation.
- No self-improvement experiment execution.
- No production method changes without approval.
- No ruvnet components.
- No remote service exposure by default.
- No hard-coded secrets.

Every future source access and important decision must go through policy,
approval, and audit paths.
