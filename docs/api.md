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
GET /work-items
POST /work-items
GET /work-items/{work_item_id}
```

`/health/live` confirms the API process is running.

`/health/ready` checks PostgreSQL, Redis, Qdrant, Neo4j, MLflow, and Phoenix
reachability.

Source registry endpoints record authorized source metadata and permissions.
Creating sources or permissions writes audit events. These endpoints do not run
collectors, perform dry-runs, write artifacts, normalize content, call external
models, or start worker jobs.

Source registry requests validate known source types, sensitivity labels,
allowed operations, and external model policy values before database writes.

Work item endpoints record proposed work and intent-verification context. New
work items are created with `pending_intent_verification` status and do not
execute worker jobs.

Future endpoints for approvals, evidence, chat, reports, patterns, predictions,
outcomes, and self-improvement are intentionally not implemented yet.
