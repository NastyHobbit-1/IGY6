# API

The API currently exposes health checks plus Phase 1 foundation endpoints for
source registry records, work item intent records, and approval records. It does
not implement collection, ingestion, evidence review, chat, prediction, or
self-improvement execution yet.

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
GET /approvals
POST /approvals
GET /approvals/{approval_id}
POST /approvals/{approval_id}/decision
GET /evidence/documents
GET /evidence/documents/{document_id}
GET /evidence/items
GET /evidence/items/{evidence_item_id}
GET /evidence/claims
GET /evidence/claims/{claim_id}
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

Approval endpoints record approval requests and decisions with audit events.
Approval decisions do not execute work or trigger worker jobs.

Evidence endpoints are read-only inspection routes for normalized documents,
evidence items, and claims already present in PostgreSQL. They do not create
evidence, run collectors, normalize artifacts, embed content, or perform
retrieval ranking.

Future endpoints for chat, reports, patterns, predictions, outcomes, and
self-improvement are intentionally not implemented yet.
