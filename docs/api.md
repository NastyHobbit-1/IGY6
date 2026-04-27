# API

Phase 0 exposes health and skeleton metadata. Phase 1 starts the source
registry API without implementing collection or ingestion.

## Endpoints

```text
GET /
GET /health/live
GET /health/ready
GET /sources
POST /sources
GET /sources/{source_id}
GET /sources/{source_id}/permissions
POST /sources/{source_id}/permissions
```

`/health/live` confirms the API process is running.

`/health/ready` checks PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix
reachability.

Source registry endpoints record authorized source metadata and permissions.
Creating sources or permissions writes audit events. These endpoints do not run
collectors, perform dry-runs, write artifacts, normalize content, call external
models, or start worker jobs.

Future endpoints for approvals, work items, evidence, chat, reports, patterns,
predictions, outcomes, and self-improvement are intentionally not implemented
yet.
