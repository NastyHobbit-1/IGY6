# Architecture (grok branch)

IGY6 is a local-first evidence and collection workspace. It ingests authorized
sources, preserves raw artifacts, normalizes evidence, stores semantic memory in
Qdrant, stores relationship memory in Neo4j, and exposes a password-protected
web UI for collection, media viewing, evidence review, and security settings.

## Current Runtime (grok branch)

Active application services:

- **Next.js web UI** (`apps/web/`) — browser interface.
- **Rust API gateway** (`crates/igy6-gateway/`, Compose service `api`) — all HTTP API routes.
- **Rust worker daemon** (`crates/igy6-worker/`, Compose service `worker`) — claims queued work from PostgreSQL and executes pipeline steps.
- **PostgreSQL** — relational state, work items, evidence, audit records.
- **Qdrant** — vector memory for chunk retrieval.
- **Neo4j** — graph/relationship memory surfaces.
- **MLflow / Phoenix** — local observability/experiment support (optional supporting services).

Retired from active Compose on `grok`:

- Legacy Python/FastAPI API (archived under `archive/legacy-python/services-api/`).
- Legacy Python/Celery worker and beat (archived under `archive/legacy-python/services-worker/`).
- Redis (was Celery broker only; work queue is PostgreSQL-based).

The UI calls the Rust API gateway only. Workers claim work from PostgreSQL; they
do not use Celery or Redis.

## Request Flow

```text
Browser (Next.js UI)
  -> Rust gateway (host APP_PORT, default 8000)
  -> PostgreSQL / Qdrant / Neo4j / local artifact store

Rust worker daemon
  -> claims queued work_items from PostgreSQL
  -> normalization, chunking, vector upsert, audit writes
  -> Qdrant HTTP API
```

## Ports and URLs

Default host ports are `WEB_PORT=3000` (UI) and `APP_PORT=8000` (API). If those
are busy, `igy6 start` and the bootstrap scripts pick the next free ports and
write `WEB_BASE_URL` / `API_BASE_URL` into `.env`. Always open the URL printed
by `igy6` or read `WEB_BASE_URL` from `.env`.

## Data Layout

Runtime data lives under `IGY6_DATA_ROOT` (outside the repo). The repository
contains source, docs, scripts, and archived history only.