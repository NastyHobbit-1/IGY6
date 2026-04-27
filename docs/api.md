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
GET /evidence/chunks
GET /evidence/chunks/{chunk_id}
GET /evidence/claims
GET /evidence/claims/{claim_id}
GET /feedback
POST /feedback
GET /feedback/{feedback_id}
GET /outcomes
POST /outcomes
GET /outcomes/{outcome_id}
GET /reports
POST /reports
GET /reports/{report_id}
GET /analysis/patterns
GET /analysis/patterns/{pattern_id}
GET /analysis/hypotheses
GET /analysis/hypotheses/{hypothesis_id}
GET /analysis/predictions
GET /analysis/predictions/{prediction_id}
GET /analysis/recommendations
GET /analysis/recommendations/{recommendation_id}
GET /audit-events
GET /audit-events/{audit_event_id}
GET /artifacts
GET /artifacts/{artifact_id}
GET /collection-runs
POST /collection-runs
GET /collection-runs/{collection_run_id}
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
chunks, evidence items, and claims already present in PostgreSQL. They do not
create evidence, run collectors, normalize artifacts, embed content, or perform
retrieval ranking.

Feedback endpoints record user labels for existing records and emit audit
events. Feedback creation does not trigger outcome evaluation, ranking changes,
or self-improvement jobs.

Outcome endpoints record what happened after a prediction, recommendation, work
item, hypothesis, pattern, or report. Outcome creation emits an audit event but
does not update prediction/recommendation status or start self-improvement.

Report endpoints record report metadata and emit audit events. They do not
render reports, write artifacts, or create exports.

Analysis endpoints are read-only inspection routes for existing patterns,
hypotheses, predictions, and recommendations. They do not generate new records,
score confidence, create recommendations, or update outcomes.

Audit endpoints are read-only inspection routes for audit events already present
in PostgreSQL. They do not create, modify, or delete audit records.

Artifact endpoints are read-only inspection routes for raw artifact metadata
already present in PostgreSQL. They do not read artifact files, write artifacts,
or create exports.

Collection-run endpoints record dry-run planning metadata only. They do not
execute collectors, create raw artifacts, normalize content, or start worker
jobs.

Future endpoints for chat and self-improvement are intentionally not implemented
yet.
