# Architecture

IGY6 is a local-first adaptive intelligence system. The complete product will
ingest authorized sources, preserve raw artifacts, normalize evidence, store
semantic memory in Qdrant, store relationship memory in Neo4j, support
evidence-backed chat, detect patterns, track predictions/recommendations and
outcomes, and run controlled self-improvement experiments.

Phase 0 only establishes the skeleton:

- Next.js status UI.
- FastAPI gateway.
- PostgreSQL foundational control and audit tables.
- Redis/Celery worker and scheduler.
- Qdrant, Neo4j, MLflow, and Phoenix services.
- Local artifact and export directories.

The UI must call FastAPI only. Workers, collectors, reasoning flows, and future
model tools must go through shared policy checks and audit paths.

## Phase 0 Flow

```text
Next.js status UI
  -> FastAPI /health
      -> PostgreSQL readiness
      -> Redis readiness
      -> Qdrant readiness
      -> Neo4j readiness
      -> MLflow readiness
      -> Phoenix readiness

Celery worker
  -> Redis broker/result backend
  -> phase0.health task only
```

No ingestion, retrieval, prediction, or self-improvement behavior exists in
Phase 0.
