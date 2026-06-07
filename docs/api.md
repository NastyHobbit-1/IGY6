# API

The API currently exposes local-first foundation endpoints for health checks,
source registry records, work item intent records, approvals, artifact and
collection metadata, evidence records, review records, vector and graph memory,
retrieval preview, reports, improvement items, and experiment run metadata. It
does not implement answer generation, autonomous collection dispatch,
self-improvement execution, production method changes, or system-changing
actions.

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
POST /work-items/{work_item_id}/dispatch
POST /work-items/{work_item_id}/status
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
POST /reports/{report_id}/status
POST /reports/{report_id}/work-item
POST /reports/{report_id}/render
GET /reports/{report_id}
GET /analysis/patterns
POST /analysis/patterns
POST /analysis/patterns/{pattern_id}/review
POST /analysis/patterns/detect-baseline
GET /analysis/patterns/{pattern_id}
GET /analysis/hypotheses
POST /analysis/hypotheses
GET /analysis/hypotheses/{hypothesis_id}
GET /analysis/predictions
POST /analysis/predictions
GET /analysis/predictions/{prediction_id}
GET /analysis/recommendations
POST /analysis/recommendations
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
POST /retrieval/chunks/search
GET /retrieval/chunks/{chunk_id}/trail
POST /chat/retrieval-preview
POST /chat/evidence-answer
GET /memory/graph/schema
POST /memory/graph/schema/ensure
POST /memory/graph/lineage/sync
GET /memory/graph/nodes/{node_label}/{node_id}/relationships
GET /improvements
POST /improvements
GET /improvements/{improvement_item_id}
GET /experiments
POST /experiments
POST /experiments/{experiment_run_id}/status
GET /experiments/{experiment_run_id}
```

`/health/live` confirms the API process is running.

`/health/ready` reports Rust gateway readiness (`primary_gateway: rust`). For
dependency reachability, use Docker Compose health checks and service logs.

Source registry endpoints record authorized source metadata and permissions.
Creating sources or permissions writes audit events. These endpoints do not run
collectors, perform dry-runs, write artifacts, normalize content, call external
models, or start worker jobs.

Source registry requests validate known source types, sensitivity labels,
allowed operations, and external model policy values before database writes.

Work item endpoints record proposed work and intent-verification context. New
work items are created with `pending_intent_verification` status and do not
execute worker jobs.

`POST /work-items/{work_item_id}/status` explicitly updates a work item status
within the local allowlist and records an audit event with previous and new
status. It does not dispatch workers or execute queued work.

`POST /work-items/{work_item_id}/dispatch` marks a queued, allowlisted work
item for the Rust worker daemon and records dispatch metadata plus an audit
event. Supported work types are `collection_normalization`,
`document_chunking`, and `chunk_vector_upsert`. The route does not create new
work items, bypass worker validation, approve work, or execute unsupported task
types.

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
events. Source-target feedback labels `trusted`, `noisy`, and `rejected` also
update the target source: `trusted` sets source `trust_level` to `trusted`,
`noisy` sets source `trust_level` to `noisy`, and `rejected` sets source
`trust_level` to `rejected` and disables the source. These source trust side
effects require the source to exist and emit a source audit event. Weak
non-source feedback labels `not_useful`, `wrong`, `incomplete`, and `rejected`
create proposed improvement items and audit that proposal. Feedback creation
does not trigger outcome evaluation, ranking changes, graph or vector updates,
worker jobs, experiment execution, or production method changes.

Outcome endpoints record what happened after a prediction, recommendation, work
item, hypothesis, pattern, or report. `POST /outcomes` validates that
`target_type` is supported and that `target_id` exists for that target table
before inserting the outcome. Optional `evidence_ids` must reference existing
immutable evidence items; duplicate IDs are deduplicated in first-seen order.
Validation failures return `422` and do not create an outcome or audit event.
Outcome creation emits an audit event but does not update
graph or vector memory, enqueue workers, or start self-improvement. For target
records that expose review status and metadata, outcome creation updates the
target's status and latest-outcome metadata, and audits that local review-state
change.

Report endpoints record report metadata and emit audit events. They do not
render reports, write artifacts, or create exports.

`POST /reports/{report_id}/status` updates report status within the local
allowlist, can attach an existing artifact path, and records an audit event. It
does not render report content, write artifacts, or create exports.

`POST /reports/{report_id}/work-item` creates a queued `report_generation`
work item marker for an existing report and records an audit event. The work
item payload is marked scaffold-only and non-executing. The route does not
render report content, write artifacts, create exports, or execute worker jobs
directly.

`POST /reports/{report_id}/render` creates a deterministic Markdown report from
existing local metadata records, stores it in the content-addressed artifact
store, records raw artifact metadata, marks the report `ready`, attaches the
artifact path, and writes an audit event. It does not read raw artifact
contents, call external models, use template engines, dispatch workers, or
execute actions.

Analysis endpoints support explicit record entry and inspection for patterns,
hypotheses, predictions, and recommendations. Create routes are human/API entry
only. They validate every referenced evidence ID against existing immutable
evidence items before inserting records and emit audit events when records are
created.

Pattern creation accepts `pattern_type`, `summary`, `evidence_ids`, optional
`status`, optional `confidence`, optional `metadata_json`, and optional
`actor_id`.

Pattern review accepts `status`, optional `reviewed_by_actor_id`, and optional
`review_note` at `POST /analysis/patterns/{pattern_id}/review`. Review status
must be `verified` or `rejected`, and only existing `candidate` patterns can be
reviewed. Review returns the updated pattern and writes an audit event with the
previous status, new status, reviewer actor, and review note. It does not
update outcomes, feedback, graph records, vector records, worker queues, or
self-improvement state.

Baseline pattern detection at `POST /analysis/patterns/detect-baseline` creates
candidate patterns from existing local evidence using deterministic rules for
recurrence, repeated statements across multiple sources, and missing evidence.
It avoids duplicate detector keys and audits created candidates. It does not
call external models, run advanced ML, write graph/vector memory, generate
predictions or recommendations, dispatch workers, or change reviewed patterns.

Hypothesis creation accepts `hypothesis_text`, `supporting_evidence_ids`,
optional `status`, optional `missing_evidence_json`, optional `confidence`,
optional `metadata_json`, and optional `actor_id`.

Prediction creation accepts `prediction_text`, `expected_result`,
`evidence_ids`, optional `disproof_condition`, optional `status`, optional
`confidence`, optional `metadata_json`, and optional `actor_id`.

Recommendation creation accepts `recommendation_text`, `evidence_ids`, optional
`risk_level`, optional `approval_required`, optional `expected_result`,
optional `status`, optional `confidence`, optional `metadata_json`, and
optional `actor_id`.

Analysis create routes do not detect patterns, generate hypotheses, score
confidence automatically, execute predictions or recommendations, update
outcomes, enqueue workers, upsert vector or graph records, call external
models, or trigger self-improvement.

Audit endpoints are read-only inspection routes for audit events already present
in PostgreSQL. They do not create, modify, or delete audit records.

Artifact endpoints list, inspect, and create raw artifact metadata. `POST
/artifacts` accepts base64 content, stores bytes in the local content-addressed
artifact store, records PostgreSQL metadata, and emits an audit event. Artifact
read routes remain metadata-only; they do not read artifact files or create
exports.

Collection-run endpoints support dry-run planning and approved non-dry-run
collection routes. Non-dry-run collection can create raw artifacts and a durable
normalization work item marker. Collection routes do not normalize content,
execute worker jobs directly from the API.

The `POST /collection-runs/dry-run` route runs connector-backed dry-run
validation for a source and permission pair, then records the preview result.
The route rejects disabled sources and permissions that do not explicitly allow
dry-run or read preview. Empty operation lists are not permissive. It does not
execute collection, read source content, write
artifacts, normalize records, or queue work.

The `POST /collection-runs/manual-upload` route accepts base64 content for a
registered `manual_upload` source, stores it in the local content-addressed
artifact store, creates a completed collection-run record, records raw artifact
metadata, records a queued `collection_normalization` work item with the raw
artifact ID, and emits audit events. It does not normalize content, generate
chunks/evidence, or execute worker jobs directly from the API. The
collection summary exposes `normalization_work_item_id`. The route requires
the source permission to explicitly allow collection or read. When the
permission has `approval_required: true`, the request must include an approved
approval ID whose optional payload metadata matches the source, permission, and
`manual_upload_collection` operation.

The `POST /collection-runs/local-project` route reads only files covered by
explicit `scope_json.paths` for a registered `local_project` source. It rejects
disabled or mismatched sources, skips symlinks and non-files, enforces file
count and size limits, stores collected bytes in the local content-addressed
artifact store, records raw artifact metadata, records a queued
`collection_normalization` work item with collected raw artifact IDs, and emits
audit events. It does not normalize content, generate chunks/evidence, dispatch
worker jobs directly from the API. The collection summary
exposes `normalization_work_item_id`. The route requires the source permission
to explicitly allow collection or read. When the permission has
`approval_required: true`, the request must include an approved approval ID
whose optional payload metadata matches the source, permission, and
`local_project_collection` operation.

The Rust worker daemon handles queued
`collection_normalization` work items. It validates the work item, collection
run, and raw artifact IDs, reads only referenced artifact bytes, decodes UTF-8
text, creates normalized document rows, skips already-normalized raw artifacts,
updates work item state, creates a queued `document_chunking` work item when
new documents were created, and emits completion or failure audit events. It
does not dispatch the chained work item automatically, upsert vector or graph
memory, generate reports, call external models, or trigger self-improvement.

The Rust worker also handles `document_chunking` for existing
normalized documents. It deterministically splits document text into fixed-size
chunks, creates one evidence item per chunk, skips already-chunked documents,
optionally updates a supplied `document_chunking` work item, creates a queued
`chunk_vector_upsert` work item when new chunks were created, and emits
completion or failure audit events. Work is claimed from PostgreSQL by the Rust
worker daemon; the API dispatch route can mark a queued item ready for claim. It
does not write Qdrant or Neo4j records during chunking,
call external models, generate reports, or trigger self-improvement.

The Rust worker handles `chunk_vector_upsert` for worker-backed chunk vector
upserts. It selects chunks whose `embedding_status` is not `completed`, embeds
chunk text with the deterministic local hash embedding helper, ensures the
configured Qdrant chunk collection exists, upserts Qdrant points with
chunk/document metadata, marks successfully upserted chunks completed, and
emits completion or failure audit events. It does not call external embedding
models, perform semantic search, update Neo4j, plan retrieval, generate chat
answers, dispatch itself from the API, or trigger self-improvement.

Docker Compose passes worker runtime settings explicitly, including
`DATABASE_URL`, `ARTIFACT_STORE_PATH`, `QDRANT_URL`,
`QDRANT_CHUNK_COLLECTION`, and `QDRANT_CHUNK_VECTOR_SIZE`, so worker execution
uses the same local service configuration declared in the environment file.

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

Retrieval chunk search accepts a request `query` and optional bounded `limit`,
delegates semantic matching to the existing Qdrant-backed chunk vector search,
then hydrates each returned chunk ID from PostgreSQL. `POST
/retrieval/chunks/search` returns the original `query` plus bounded `results`
containing each vector `score`, stored Qdrant `payload`, chunk metadata,
normalized document metadata, optional source metadata, optional raw artifact
metadata, and linked evidence item metadata. The route does not generate
answers, traverse Neo4j, read artifact content, or perform broader retrieval
planning.

Retrieval trail endpoints inspect existing PostgreSQL metadata for a known
chunk. `GET /retrieval/chunks/{chunk_id}/trail` returns the chunk metadata, its
normalized document metadata, optional source metadata, optional raw artifact
metadata, and linked evidence item metadata for the chunk and document. The
route is read-only metadata inspection only. It does not read artifact contents,
search Qdrant, traverse Neo4j, generate answers, or plan retrieval.

Chat retrieval preview accepts a user `message` and optional bounded `limit`,
then reuses hydrated semantic retrieval to return retrieval context for the
message. `POST /chat/retrieval-preview` returns `answer_status:
"not_generated"` with the original message and hydrated context containing
chunk metadata, normalized document metadata, optional source metadata, optional
raw artifact metadata, linked evidence item metadata, vector scores, and stored
Qdrant payloads. The route does not call models, generate answers, persist
conversations, trigger actions, read artifact contents, traverse Neo4j, or
write to PostgreSQL, Qdrant, Neo4j, or artifact storage.

Chat evidence answer accepts a user `message` and optional bounded `limit`,
then reuses hydrated semantic retrieval to return a deterministic answer packet
with facts, assumptions, inferences, uncertainty, missing-information notes,
source trails, and the underlying retrieval context. Every fact and inference
carries evidence item or chunk citations. `POST /chat/evidence-answer` does not
call external models, persist conversations, execute tools, trigger actions,
read artifact contents, traverse Neo4j, or write to PostgreSQL, Qdrant, Neo4j,
or artifact storage.

Graph memory endpoints inspect and create Neo4j uniqueness constraints for
source, artifact, document, chunk, evidence, claim, analysis, outcome, and
report nodes. They do not infer relationships, generate patterns, or call
models. The lineage sync route upserts deterministic provenance, evidence
support, and outcome relationships already present in PostgreSQL. Relationship
inspection is read-only and bounded to deterministic graph node labels.
Chunk-evidence lineage is derived directly from each evidence item's stored
`chunk_id`.

Improvement endpoints create and inspect self-improvement queue items for
parsing, retrieval, scoring, prediction, reporting, reasoning, and safety
targets. Item creation records an audit event but does not execute
experiments, dispatch workers, call MLflow or Optuna, alter methods, or change
production behavior.

Experiment endpoints create and inspect self-improvement experiment run
metadata, optionally link a run to an existing improvement item, and explicitly
update run status plus recorded metrics/artifact metadata. They record audit
events but do not execute experiments, dispatch workers, call MLflow or Optuna,
alter methods, or change production behavior.

Future endpoints for answer-generating chat and self-improvement execution are
intentionally not implemented yet.
