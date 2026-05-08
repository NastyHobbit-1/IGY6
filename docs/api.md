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
POST /evidence/documents
GET /evidence/documents/{document_id}
POST /evidence/documents/{document_id}/chunks
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
POST /artifacts
GET /artifacts/{artifact_id}
GET /collection-runs
POST /collection-runs
POST /collection-runs/dry-run
POST /collection-runs/manual-upload
POST /collection-runs/local-project
GET /collection-runs/{collection_run_id}
GET /memory/vector/chunks
POST /memory/vector/chunks/ensure
POST /memory/vector/chunks/upsert
POST /memory/vector/chunks/search
GET /retrieval/chunks/{chunk_id}/trail
GET /memory/graph/schema
POST /memory/graph/schema/ensure
POST /memory/graph/lineage/sync
GET /memory/graph/nodes/{node_label}/{node_id}/relationships
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

Evidence document read endpoints inspect normalized documents already present in
PostgreSQL. `POST /evidence/documents` creates a normalized UTF-8 text document
from an existing raw artifact in the local artifact store. Evidence item
endpoints can create or inspect immutable evidence items. These routes do not
run collectors, create chunks, embed content, write graph records, or perform
retrieval ranking.

`POST /evidence/documents/{document_id}/chunks` deterministically splits an
existing normalized document into text chunks and creates one evidence item per
chunk. Generated chunks are not embedded and are rejected if chunks already
exist for the document.

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

Artifact endpoints list, inspect, and create raw artifact metadata. `POST
/artifacts` accepts base64 content, stores bytes in the local content-addressed
artifact store, records PostgreSQL metadata, and emits an audit event. Artifact
read routes remain metadata-only; they do not read artifact files or create
exports.

Collection-run endpoints record dry-run planning metadata only. They do not
create raw artifacts, normalize content, or start worker jobs.

The `POST /collection-runs/dry-run` route runs connector-backed dry-run
validation for a source and permission pair, then records the preview result.
The route rejects disabled sources and permissions that do not allow dry-run or
read preview. It does not execute collection, read source content, write
artifacts, normalize records, or queue work.

The `POST /collection-runs/manual-upload` route accepts base64 content for a
registered `manual_upload` source, stores it in the local content-addressed
artifact store, creates a completed collection-run record, records raw artifact
metadata, and emits audit events. It does not normalize content, generate
chunks/evidence, or enqueue worker jobs.

The `POST /collection-runs/local-project` route reads only files covered by
explicit `scope_json.paths` for a registered `local_project` source. It rejects
disabled or mismatched sources, skips symlinks and non-files, enforces file
count and size limits, stores collected bytes in the local content-addressed
artifact store, and records raw artifact metadata. It does not normalize
content, generate chunks/evidence, or enqueue worker jobs.

Vector memory endpoints inspect and create the configured Qdrant chunk
collection, upsert existing chunk text using the deterministic local embedding
helper, and run direct semantic chunk search against the Qdrant-backed chunk
vector collection.

`POST /memory/vector/chunks/search` embeds the request `query` locally with the
same deterministic embedding helper used for chunk vector upserts, then queries
Qdrant for nearest chunk vectors. The request body accepts `query` and an
optional `limit`, which defaults to `10` and is bounded to `1` through `50`.
The response returns the original `query` plus bounded `hits` containing
`chunk_id`, `document_id`, `score`, and the stored Qdrant `payload`. The route
does not call external embedding models, generate answers, perform retrieval
planning, traverse Neo4j, or read artifact content.

Retrieval trail endpoints inspect existing PostgreSQL metadata for a known
chunk. `GET /retrieval/chunks/{chunk_id}/trail` returns the chunk metadata, its
normalized document metadata, optional source metadata, optional raw artifact
metadata, and linked evidence item metadata for the chunk and document. The
route is read-only metadata inspection only. It does not read artifact contents,
search Qdrant, traverse Neo4j, generate answers, or plan retrieval.

Graph memory endpoints inspect and create baseline Neo4j uniqueness constraints
for future source, artifact, document, chunk, and evidence nodes. They do not
infer relationships, generate patterns, or call models. The lineage sync route
upserts only deterministic source/artifact/document/chunk/evidence provenance
links already present in PostgreSQL. Relationship inspection is read-only and
bounded to deterministic graph node labels.

Future endpoints for chat and self-improvement are intentionally not implemented
yet.
