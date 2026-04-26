# Phase 0 Implementation Plan

Approved clarifications:

- Keep the repo root as `IGY6`.
- Use Python 3.12.
- Use `uv` if available; otherwise use `requirements.txt`. `uv` is not available in this environment, so Phase 0 uses `requirements.txt`.
- Bind services to localhost by default.
- Stub authentication only, while including actor/user fields where needed.
- Use local file-backed MLflow storage in Phase 0 and document later PostgreSQL backend migration.
- Do not build Phase 1 or later features.

## Phase 0 Scope

Build the local-first skeleton only:

1. Monorepo directories.
2. Docker Compose stack.
3. FastAPI service with health checks.
4. Next.js status UI.
5. PostgreSQL, Redis/Celery, Qdrant, Neo4j, MLflow, and Phoenix services.
6. Initial Alembic migration for foundational control and audit tables.
7. Placeholder packages and docs.

## Out Of Scope

- Source ingestion.
- Browser automation.
- Embeddings.
- Graph extraction.
- Evidence-backed chat.
- Pattern detection.
- Prediction/advice logic.
- Outcome feedback workflows.
- Self-improvement experiments.
- External model use.
- Remote exposure.

## Step-By-Step Tasks

1. Create the approved repository structure under `IGY6`.
2. Add root `.env.example`, `.gitignore`, and README run instructions.
3. Scaffold FastAPI with config, health routes, database session, and foundational models.
4. Add Alembic and the initial Phase 0 migration.
5. Scaffold Celery worker and health task.
6. Scaffold Next.js status UI that calls FastAPI only.
7. Add Dockerfiles for API, worker, and web services.
8. Add Docker Compose with localhost-bound ports.
9. Add placeholders for policy, schemas, collectors, ML, reports, and self-improvement.
10. Add Qdrant and Neo4j config placeholders.
11. Add docs for architecture, security, API, operations, and user guide.
12. Verify migrations and service health when Docker dependencies are available.

## Implementation Verification

Completed Phase 0 verification:

- Python syntax compilation passed for `services/api` and `services/worker`.
- Docker Compose configuration rendered successfully with `.env.example`.
- API, worker, and web images built successfully.
- Docker Compose stack started locally with localhost-bound ports.
- `/health/live` returned `ok`.
- `/health/ready` returned `ok` for PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix.
- Alembic current revision is `0001_phase0_foundation`.
- PostgreSQL contains the foundational Phase 0 tables.
- Celery worker inspection returned `pong`.
- Web status page rendered and displayed service readiness.
- Web dependency install reported zero npm vulnerabilities after patched Next/React pins and a PostCSS override.
