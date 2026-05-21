# Architecture

IGY6 is a local-first adaptive intelligence system. The complete product will
ingest authorized sources, preserve raw artifacts, normalize evidence, store
semantic memory in Qdrant, store relationship memory in Neo4j, support
evidence-backed chat, detect patterns, track predictions/recommendations and
outcomes, and run controlled self-improvement experiments.

Current runtime posture after DIFF-139:

- Next.js UI.
- Rust API gateway.
- PostgreSQL foundational control and audit tables.
- Redis/Celery Python worker and scheduler.
- Qdrant, Neo4j, MLflow, and Phoenix services.
- Local artifact and export directories.

The UI must call the Rust API gateway only. Workers, collectors, reasoning
flows, and future model tools must go through shared policy checks and audit
paths.

## Phase 0 Flow

```text
Next.js UI
  -> Rust gateway /health and application routes

Celery worker
  -> Redis broker/result backend
  -> normalization, chunk generation, and vector upsert tasks
```

The legacy FastAPI API source is archived under
`archive/legacy-python/services-api` after DIFF-139. Python/Celery worker and
beat services remain active runtime components until Rust worker execution
parity is implemented and verified.
