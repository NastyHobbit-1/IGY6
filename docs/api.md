# API

Phase 0 exposes health and skeleton metadata only.

## Endpoints

```text
GET /
GET /health/live
GET /health/ready
```

`/health/live` confirms the API process is running.

`/health/ready` checks PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix
reachability.

Future endpoints for sources, approvals, work items, evidence, chat, reports,
patterns, predictions, outcomes, and self-improvement are intentionally not
implemented in Phase 0.
