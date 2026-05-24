# DIFF-104 Post-Cutover Route Audit

Date: 2026-05-17

## Summary

DIFF-103 completed cutover governance with Rust primary and FastAPI fallback.
DIFF-138 removes FastAPI fallback after route parity reaches zero missing
FastAPI routes. DIFF-139 archives the tracked legacy FastAPI API source.
DIFF-140 is the final Rust API cutover audit. The current API topology is
Rust-native while Python/Celery workers remain active:

```text
Next.js web
    |
    v
Rust gateway service: api
    |
    +-- Rust-native routes for health, migration status, agent capability,
    |   agent intent, retrieval preview, evidence answer, DIFF-106 and
    |   DIFF-107 read-only DB routes, DIFF-108 status/config routes,
    |   DIFF-109 approval request creation, DIFF-110 feedback/outcome
    |   writes, DIFF-111 source creation, DIFF-112 report creation, and
    |   DIFF-113 analysis pattern writes, DIFF-114 collection dry-run
    |   previews, DIFF-115 settings verify/apply, DIFF-116 work-item
    |   creation, DIFF-117 manual upload collection creation, DIFF-118
    |   agent action request/execution routes, DIFF-120 dynamic web
    |   control routes, DIFF-132 active medium-risk route parity,
    |   DIFF-133 graph/vector memory route parity, DIFF-134 report
    |   work-item route parity, DIFF-135 artifact/collection
    |   ingestion route parity, DIFF-136 experiments/improvements
    |   route parity, and DIFF-137 root route parity
    |
    +-- unsupported routes return deterministic Rust 404 responses
```

FastAPI fallback is no longer configured after DIFF-138. No web-used route
requires FastAPI fallback, and DIFF-140 records the final audit posture while
the route classification remains recorded in
`configs/legacy-fastapi-route-classification.json` and
`docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`.
The legacy FastAPI source is archived at
`archive/legacy-python/services-api`. Python/Celery `worker` and `beat` remain
active runtime services from `services/worker`. DIFF-143 through DIFF-145 add
Rust worker planning and executor contracts for `collection_normalization`,
`document_chunking`, and `chunk_vector_upsert`, but live worker process
ownership and beat/scheduled-work posture are not cut over.

## Runtime Topology

`infra/docker-compose.yml` currently defines:

| Service | Runtime role |
| --- | --- |
| `api` | Rust gateway built from `crates/igy6-gateway/Dockerfile`, published on `127.0.0.1:${APP_PORT:-8000}:8000`. |
| `web` | Next.js UI with `API_BASE_URL=http://api:8000`; browser-side helpers also call `http://127.0.0.1:8000`. |
| `worker` and `beat` | Python/Celery execution remains active. |

The web UI calls the Rust gateway endpoint. The route parity guard reports zero
FastAPI routes missing from Rust and zero web-used routes requiring fallback.
Unsupported gateway routes now return Rust 404 responses instead of proxying to
FastAPI. Full Rust-only repository or runtime operation is not claimed while
Python/Celery `worker` and `beat` remain active.

## Worker Execution Parity

DIFF-141 audits Python/Celery worker and beat usage and recommends migrating
worker execution to Rust one job family at a time. DIFF-142 adds the Rust
queue-claim contract. DIFF-143 adds `collection_normalization` parity
contracts. DIFF-144 adds `document_chunking` parity contracts. DIFF-145 adds
`chunk_vector_upsert` parity contracts, including uncompleted chunk selection,
deterministic local vector planning, Qdrant collection/status/upsert request
planning, chunk metadata/status update planning, and `chunk_vectors.*` audit
events.

This is not a worker runtime cutover. Python/Celery `worker` and `beat` remain
active until a later DIFF replaces live worker process ownership and decides
the scheduler/beat posture.

DIFF-146 makes that decision explicit: choose Decision B, keep Python/Celery
`worker` and `beat`, and do not claim full Rust-only runtime. The blocker is
not job-family parity contracts; those exist for `collection_normalization`,
`document_chunking`, and `chunk_vector_upsert`. The blocker is runtime
ownership: no Rust worker binary/container currently polls queued work,
atomically claims jobs, performs the DB/audit writes, reads artifacts, executes
Qdrant side effects, or replaces/retire scheduled work.

DIFF-147 adds the Rust `igy6-worker` binary and a cutover readiness harness with
`--check`, `--dry-run`, `--once`, and `--help`. The harness validates safe
configuration and plans bounded queue/claim behavior, but it remains
non-mutating. It does not connect to PostgreSQL, mutate runtime queue rows,
read artifacts, write audit events, call Qdrant, control Celery, replace beat,
or claim full Rust-only runtime.

DIFF-148 adds an opt-in one-job canary gate:
`--once --canary-live --canary-work-item ID`. The canary also requires the
runtime acknowledgement `IGY6_WORKER_LIVE_CANARY=DIFF-148` before reporting
`live_execution_enabled=true`. It adds structured canary states and
side-effect verification planning for PostgreSQL claim/writes, audit writes,
artifact reads, and Qdrant collection/point work. DIFF-148 still executes no
real side effects; Python/Celery `worker` and `beat` remain active.

DIFF-149 implements the first live Rust worker side-effect executor behind the
same explicit one-job canary gates. The selected canary work item can now be
claimed with PostgreSQL row locking, marked running/completed/failed, audited,
and executed through the existing Rust parity contracts. `collection_normalization`
can read scoped artifact bytes and write normalized documents plus a chained
chunking work item. `document_chunking` can write chunks, evidence items, and a
chained vector work item. `chunk_vector_upsert` can generate deterministic local
vectors, ensure the Qdrant collection, upsert points, and update chunk embedding
metadata. This is still not a worker process cutover: broad queue polling,
long-running Rust worker ownership, Compose worker replacement, and beat
replacement remain out of scope.

DIFF-150 decision: B, Rust worker process cutover is not ready. No real canary
was run because DIFF-150 did not have an explicitly selected safe runtime work
item. Static verification covers the CLI gates, non-mutating default behavior,
`FOR UPDATE SKIP LOCKED` claim shape, artifact path safety tests, Qdrant
request-boundary tests, and retained Docker Compose `worker`/`beat` services.
Live observations of PostgreSQL writes, audit rows, artifact reads, Qdrant
upserts, and failure rollback remain required before process ownership can move
from Python/Celery to Rust.

DIFF-151 decision: B, safe canary not available. No live Rust canary was run
because no explicitly selected safe queued work item ID was available, and the
DIFF prohibits touching runtime/private data except through that scoped canary.
The next required work is to create or select exactly one non-sensitive queued
canary work item, pause or isolate Python/Celery from racing that item during
the canary window, run the gated Rust canary once, and record observed
PostgreSQL, `audit_events`, artifact, and Qdrant side effects.

DIFF-152 decision: A, safe canary fixture path created. The selected future
canary is `diff-152-canary-work-item`, generated by
`scripts/rust-worker-canary-fixture.py` as a synthetic
`collection_normalization` fixture. The fixture helper is non-mutating by
default and can emit deterministic seed SQL plus the artifact storage path for a
future live canary DIFF. DIFF-152 does not run the live Rust canary and does not
change Python/Celery worker or beat ownership.

DIFF-153 decision: A, controlled live Rust worker canary run completed. The
DIFF-152 fixture was applied to an isolated local PostgreSQL canary container
and `/tmp/igy6-diff153-canary` data root, then Rust ran exactly one selected
work item: `diff-152-canary-work-item`. Observed side effects include
`work_items` completion, claim/start/success audit events, scoped synthetic
artifact read, one `normalized_documents` write, and one chained
`document_chunking` work item. Qdrant side effects were not expected for this
`collection_normalization` canary. Worker process cutover is still not claimed:
live `document_chunking`, live `chunk_vector_upsert`/Qdrant, broad queue
ownership, Compose worker replacement, and beat posture remain unresolved.

DIFF-154 decision: B, successful `document_chunking` canary could not be
completed. Exactly one isolated `document_chunking` canary was attempted for
`diff-154-canary-work-item`, but the fixture used invalid `chunk_size=80`; the
Rust worker marked the item `failed` and wrote claim/start/failure audit events
without writing chunks, evidence items, or a chained `chunk_vector_upsert` work
item. The fixture helper is corrected to emit `chunk_size=100` for a later
canary. Qdrant side effects were not expected or run in DIFF-154.

DIFF-155 decision: A, corrected `document_chunking` canary completed. The
corrected fixture was applied to an isolated local PostgreSQL canary container,
then Rust ran exactly one selected work item: `diff-154-canary-work-item`.
Observed side effects include `work_items` completion, claim/start/success
audit events, three `chunks` rows, three `evidence_items` rows, and one chained
queued `chunk_vector_upsert` work item. Qdrant side effects were not expected
or run because DIFF-155 was scoped to `document_chunking` only. Worker process
cutover is still not claimed: live `chunk_vector_upsert`/Qdrant, broad queue
ownership, Compose worker replacement, and beat posture remain unresolved.

DIFF-156 decision: A, controlled `chunk_vector_upsert` canary was run exactly
once but failed safely at Qdrant point upsert. The selected work item
`work-item-18b25ee83a881458-6` moved to `failed`, claim/start/failure audit
events were observed, Qdrant collection `igy6_diff156_chunks` was created, and
Qdrant point count remained 0 after `PUT /points` returned HTTP 400. Chunk
`embedding_status` and vector metadata remained unchanged because the Qdrant
upsert did not complete. Worker process cutover is still not claimed:
successful Qdrant upsert, broad queue ownership, Compose worker replacement,
and beat posture remain unresolved.

DIFF-157 decision: A, Qdrant point upsert compatibility fixed and corrected
`chunk_vector_upsert` canary completed. Rust now uses deterministic
UUID-shaped Qdrant point IDs derived from chunk IDs while retaining the original
chunk IDs in point payload metadata. The selected work item
`work-item-18b25ee83a881458-6` moved to `completed`, claim/start/success audit
events were observed, Qdrant collection `igy6_diff157_chunks` was created,
`PUT /points` returned HTTP 200, point scroll returned three points, and all
three chunks moved to `embedding_status=completed` with
`embedding_method=local_hash_v1` and `vector_collection=igy6_diff157_chunks`.
Worker process cutover is still not claimed: broad queue ownership, Compose
worker replacement, and beat posture remain unresolved.

DIFF-158 decision: B, worker process cutover is not ready. The Rust worker has
proven isolated one-work-item live canaries for `collection_normalization`,
`document_chunking`, and `chunk_vector_upsert`, but it has not proven
production worker process ownership. Exact blockers are long-running Rust
daemon mode, generic live queue polling without a named canary item,
production retry/backoff and graceful shutdown behavior, worker health/readiness
posture, Rust worker Docker Compose wiring, rollback posture, and
scheduler/beat replacement or retirement. Docker Compose was not changed:
Python/Celery `worker` and `beat` remain active, and full Rust-only runtime is
not claimed.

## Rust-Native Gateway Routes

These routes are handled directly by `crates/igy6-gateway`:

| Method | Route | Rust status |
| --- | --- | --- |
| GET | `/` | Rust-native gateway identity response |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/rust-migration/status` | Rust-native |
| GET | `/settings/env` | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/memory/vector/chunks` | Rust-native read-only status |
| GET | `/memory/graph/schema` | Rust-native read-only status |
| POST | `/memory/graph/schema/ensure` | Rust-native Neo4j schema ensure via bounded service call |
| POST | `/memory/graph/lineage/sync` | Rust-native Neo4j lineage sync via bounded service calls |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | Rust-native Neo4j relationship read with label allowlist |
| POST | `/memory/vector/chunks/ensure` | Rust-native Qdrant collection ensure via bounded service call |
| POST | `/memory/vector/chunks/search` | Rust-native Qdrant chunk vector search |
| POST | `/memory/vector/chunks/upsert` | Rust-native Qdrant chunk vector upsert with DB status update |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/actions/` | Rust-native fixed action request/audit route |
| POST | `/agent/actions/{action_name}/execute` | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| POST | `/agent/intent` | Rust-native |
| POST | `/chat/retrieval-preview` | Rust-native contract response |
| POST | `/chat/evidence-answer` | Rust-native contract response |
| POST | `/approvals` | Rust-native DB write with audit event |
| GET | `/analysis/patterns` | Rust-native DB read |
| GET | `/analysis/patterns/{pattern_id}` | Rust-native DB read |
| POST | `/analysis/patterns` | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Rust-native DB status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Rust-native DB write with deterministic local detection and audit events |
| GET | `/analysis/hypotheses` | Rust-native DB read |
| POST | `/analysis/hypotheses` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/hypotheses/{hypothesis_id}` | Rust-native DB read |
| GET | `/analysis/predictions` | Rust-native DB read |
| POST | `/analysis/predictions` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/predictions/{prediction_id}` | Rust-native DB read |
| GET | `/analysis/recommendations` | Rust-native DB read |
| POST | `/analysis/recommendations` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/recommendations/{recommendation_id}` | Rust-native DB read |
| GET | `/approvals` | Rust-native DB read |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| POST | `/approvals/{approval_id}/decision` | Rust-native pending-only approval decision with audit event |
| GET | `/artifacts` | Rust-native DB read |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| POST | `/artifacts` | Rust-native DB/artifact write with source/run validation and audit event |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| GET | `/collection-runs` | Rust-native DB read |
| GET | `/collection-runs/{collection_run_id}` | Rust-native DB read |
| POST | `/collection-runs` | Rust-native DB write with collection_run.created audit event |
| POST | `/collection-runs/dry-run` | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/local-project` | Rust-native scoped local-project collection with permission, approval, artifact, work-item, and audit records |
| POST | `/collection-runs/manual-upload` | Rust-native DB/artifact write with source permission, approval, and audit events |
| POST | `/collection-runs/manual-upload/ingest` | Rust-native bounded manual text ingest with artifact/document/chunk/evidence/vector and audit behavior |
| GET | `/evidence/documents` | Rust-native DB read |
| POST | `/evidence/documents` | Rust-native DB write with artifact/source validation and audit event |
| GET | `/evidence/documents/{document_id}` | Rust-native DB read |
| POST | `/evidence/documents/{document_id}/chunks` | Rust-native DB chunk/evidence-item generation with audit event |
| GET | `/evidence/items` | Rust-native DB read |
| POST | `/evidence/items` | Rust-native DB write with source/document/chunk validation and audit event |
| GET | `/evidence/items/{evidence_item_id}` | Rust-native DB read |
| GET | `/evidence/chunks` | Rust-native DB read |
| GET | `/evidence/chunks/{chunk_id}` | Rust-native DB read |
| GET | `/evidence/claims` | Rust-native DB read |
| GET | `/evidence/claims/{claim_id}` | Rust-native DB read |
| GET | `/experiments` | Rust-native DB read |
| GET | `/experiments/{experiment_run_id}` | Rust-native DB read |
| POST | `/experiments` | Rust-native DB write with improvement reference validation and audit event |
| POST | `/experiments/{experiment_run_id}/status` | Rust-native DB status update with audit event |
| GET | `/feedback` | Rust-native DB read |
| GET | `/feedback/{feedback_id}` | Rust-native DB read |
| POST | `/feedback` | Rust-native DB write with audit event |
| GET | `/improvements` | Rust-native DB read |
| GET | `/improvements/{improvement_item_id}` | Rust-native DB read |
| POST | `/improvements` | Rust-native DB write with target/priority validation and audit event |
| GET | `/outcomes` | Rust-native DB read |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| POST | `/outcomes` | Rust-native DB write with audit events |
| GET | `/reports` | Rust-native DB read |
| GET | `/reports/{report_id}` | Rust-native DB read |
| POST | `/reports` | Rust-native DB write with audit event |
| POST | `/reports/{report_id}/status` | Rust-native DB status update with audit event |
| POST | `/reports/{report_id}/render` | Rust-native bounded metadata report render with artifact and audit event |
| POST | `/reports/{report_id}/work-item` | Rust-native queued report_generation work item with scaffold-only payload and audit event |
| GET | `/retrieval/chunks/{chunk_id}/trail` | Rust-native DB evidence trail hydration |
| POST | `/retrieval/chunks/search` | Rust-native DB hydrated chunk search |
| GET | `/sources` | Rust-native DB read |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| POST | `/sources` | Rust-native DB write with optional permission and audit event |
| POST | `/sources/{source_id}/permissions` | Rust-native DB permission creation with audit event |
| GET | `/work-items` | Rust-native DB read |
| GET | `/work-items/{work_item_id}` | Rust-native DB read |
| POST | `/work-items` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Rust-native dispatch validation and non-executing audit marker |
| POST | `/work-items/{work_item_id}/status` | Rust-native DB status transition with audit event |

Unsupported routes return a deterministic Rust 404 response. FastAPI fallback
proxying is removed from the runtime gateway path.

Route parity counts:

| Metric | DIFF-105 | DIFF-106 | DIFF-107 | DIFF-108 | DIFF-109 | DIFF-110 | DIFF-111 | DIFF-112 | DIFF-113 | DIFF-114 | DIFF-115 | DIFF-116 | DIFF-117 | DIFF-118 | DIFF-119 | DIFF-120 | DIFF-132 | DIFF-133 | DIFF-134 | DIFF-135 | DIFF-136 | DIFF-137 | DIFF-138 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| FastAPI total routes | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 |
| Rust-native routes | 7 | 24 | 42 | 45 | 46 | 48 | 49 | 50 | 52 | 53 | 55 | 57 | 58 | 60 | 60 | 64 | 75 | 81 | 82 | 86 | 93 | 94 | 94 |
| FastAPI routes missing from Rust | 85 | 68 | 50 | 47 | 46 | 44 | 43 | 42 | 40 | 39 | 37 | 36 | 35 | 34 | 34 | 30 | 19 | 13 | 12 | 8 | 1 | 0 | 0 |
| Web-used routes | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 45 | 45 | 45 | 45 | 45 | 45 | 45 | 45 |
| Web routes requiring fallback | 36 | 28 | 19 | 16 | 14 | 12 | 11 | 9 | 7 | 6 | 4 | 3 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## DIFF-120 Dynamic Web Control Route Parity

DIFF-120 migrates the four dynamically referenced `apps/web` page controls that
DIFF-119 found outside the extractor, and adds them explicitly to
`scripts/rust-route-parity.py` so they cannot be missed again. FastAPI remains
required for the 30 classified routes still missing from Rust, and Rust-only
cannot be claimed.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 11 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 11 |

FastAPI remains required because `intentional_legacy_fallback` and
`unsafe_to_migrate_now` are non-empty. Rust-only cannot honestly be claimed.

## DIFF-132 Active Medium-Risk Route Parity

DIFF-132 migrates the 11 routes previously classified as
`active_parity_required` to Rust-native gateway handlers. The migration covers
analysis creation, evidence document/chunk/item writes, source permission
creation, report/work-item status transitions, and retrieval trail/search
hydration. FastAPI remains required for the 19 classified routes still missing
from Rust, and Rust-only cannot be claimed.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 11 |

## DIFF-133 Graph/Vector Memory Route Parity

DIFF-133 migrates the six graph/vector memory fallback routes to Rust-native
gateway handlers. Graph routes use allowlisted labels and bounded Neo4j
transactional HTTP calls for relationship reads, schema ensure, and lineage
sync. Vector routes use bounded Qdrant HTTP calls plus deterministic local
hash embeddings for collection ensure, search, and chunk upsert. FastAPI
remains required for the 13 classified routes still missing from Rust, and
Rust-only cannot be claimed.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 5 |

## DIFF-134 Report Work-Item Route Parity

DIFF-134 migrates `POST /reports/{report_id}/work-item` to Rust-native gateway
handling. The handler validates the report ID and request body, loads the
existing report, creates a queued `report_generation` work item with a bounded
scaffold-only payload, and inserts the correlated `work_item.created` audit
event in the same database transaction. It does not dispatch Celery work and
does not remove FastAPI fallback for the remaining classified routes.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 4 |

## DIFF-135 Artifact And Collection Ingestion Route Parity

DIFF-135 migrates the four artifact and collection ingestion fallback routes to
Rust-native gateway handlers. The route batch covers raw artifact creation,
generic collection-run creation, scoped local-project collection, and bounded
manual text ingest. The Rust handlers preserve content-addressed artifact
storage, source/run validation, permission checks, approval checks, audit
events, bounded local-project path traversal resistance, manual-upload UTF-8
text limits, and Qdrant vector upsert behavior for manual ingest. FastAPI
fallback remains required for experiments, improvements, and root-route
resolution.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 0 |

## DIFF-136 Experiments And Improvements Route Resolution

DIFF-136 decision: migrate the experiments and improvements route family to
Rust.

DIFF-136 migrates the seven experiments and improvements fallback routes to
Rust-native gateway handlers. The route batch covers experiment list/detail,
experiment creation, experiment status update, improvement list/detail, and
improvement creation. The Rust handlers preserve DB-backed metadata reads,
status and enum validation, improvement reference validation for experiment
creation, and audit events for experiment and improvement writes. FastAPI
fallback remains required until DIFF-137 resolves the duplicate root route and
DIFF-138 evaluates fallback readiness.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 0 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 0 |

## DIFF-137 Duplicate Root Route Resolution

DIFF-137 decision: migrate `GET /` to Rust.

DIFF-137 migrates the duplicate/superseded FastAPI scaffold root route to a
Rust-native gateway identity response. The Rust root identity plus
`/health/live`, `/health/ready`, and `/rust-migration/status` supersede the old
FastAPI scaffold response. FastAPI fallback remains configured until DIFF-138
evaluates readiness and removes fallback only if safe.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 0 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 0 |
| `unsafe_to_migrate_now` | 0 |

## DIFF-138 FastAPI Fallback Readiness Gate

DIFF-138 decision: remove FastAPI fallback from the runtime API path.

DIFF-138 verifies route parity is complete: 91 FastAPI routes are accounted for,
94 Rust-native routes are registered, 0 FastAPI routes are missing from Rust,
and 0 web-used routes require fallback. The Rust gateway no longer accepts or
uses a FastAPI fallback origin at runtime. `infra/docker-compose.yml` no longer
defines `legacy-api`, and unsupported routes return deterministic Rust 404
responses.

This DIFF does not archive `services/api/` and does not change Python/Celery
worker services. DIFF-139 remains responsible for the legacy Python archive or
preservation decision.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 0 |
| `intentional_legacy_fallback` | 0 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 0 |
| `unsafe_to_migrate_now` | 0 |

## DIFF-139 Legacy Python Archive Decision

DIFF-139 decision: archive the legacy FastAPI API source and retain
Python/Celery worker execution.

`services/api/` is no longer needed by an active runtime path after DIFF-138:
Docker Compose no longer defines `legacy-api`, the Rust gateway no longer
accepts a fallback origin, and route parity reports 0 FastAPI routes missing
from Rust. DIFF-139 moves the tracked FastAPI API tree to
`archive/legacy-python/services-api` using `git mv`.

`services/worker/` remains required because Docker Compose still runs:

- `worker`: `celery -A app.celery_app:celery_app worker --loglevel=INFO`
- `beat`: `celery -A app.celery_app:celery_app beat --loglevel=INFO`

The Rust worker crate now includes DIFF-143 `collection_normalization` and
DIFF-144 `document_chunking` execution planning plus SQL/audit/status executor
contracts. It preserves Python/Celery normalization behavior for UTF-8 artifact
normalization, `normalized_documents` insert shape, deterministic chunk and
evidence item inserts, duplicate skips, originating work-item status,
completion/failure audit events, and chained work-item creation through
`chunk_vector_upsert`. It does not replace live Celery process ownership,
execute `chunk_vector_upsert`, call Qdrant, or remove Python/Celery. Full
Rust-only repository/runtime operation is therefore not claimed.

DIFF-141 audits the active Python/Celery worker and beat services. It finds five
registered Celery tasks, no repo-defined beat schedule, and no Python/Celery
Neo4j operations. The recommended path is to migrate worker execution to Rust
one job family at a time while retaining Python/Celery until execution parity is
complete.

DIFF-142 adds the Rust worker queue-claim contract and bounded claim planning.
It supports only `collection_normalization`, `document_chunking`, and
`chunk_vector_upsert` claims, requires queued status and recorded intent
verification, and records an execution-free `work_item.claimed` audit posture.
It does not execute any job family, write worker DB rows, write audit rows, read
artifacts, call Qdrant, call Neo4j, remove Celery, or claim full Rust-only
runtime operation.

DIFF-143 adds the first worker job-family parity layer for
`collection_normalization`. The Rust worker crate can now plan the same
normalization DB writes, status transitions, audit events, duplicate skip
behavior, UTF-8 failure handling, and chained `document_chunking` work-item
creation that the Python/Celery worker performs. The next required parity work
is DIFF-144 for `document_chunking`; until that and later vector/scheduler
parity work complete, Python/Celery `worker` and `beat` remain active.

DIFF-144 adds the second worker job-family parity layer for
`document_chunking`. The Rust worker crate can now plan the same chunk and
evidence-item DB writes, status transitions, audit events, duplicate skip
behavior, empty-document failure handling, and chained `chunk_vector_upsert`
work-item creation that the Python/Celery worker performs. The next required
parity work is DIFF-145 for `chunk_vector_upsert`; until vector and scheduler
parity work complete, Python/Celery `worker` and `beat` remain active.

## Web-Used Route Matrix

| Method | Route | Web usage | Gateway behavior |
| --- | --- | --- | --- |
| GET | `/agent/capabilities` | Next.js proxy and page data load | Rust-native |
| POST | `/agent/intent` | Next.js proxy and page intent preview | Rust-native |
| POST | `/agent/actions/` | Page dynamic action execution prefix detected by route guard | Rust-native fixed action request/audit route |
| POST | `/agent/actions/{action_name}/execute` | Next.js proxy and page action execution | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| GET | `/analysis/patterns` | Page data load | Rust-native DB read |
| POST | `/analysis/patterns` | Page pattern create | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Page pattern review | Rust-native candidate-only status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Page baseline pattern detection | Rust-native DB write with deterministic local detection and audit events |
| GET | `/analysis/hypotheses` | Page data load | Rust-native DB read |
| GET | `/analysis/predictions` | Page data load | Rust-native DB read |
| GET | `/analysis/recommendations` | Page data load | Rust-native DB read |
| GET | `/approvals` | Next.js proxy and page data load | Rust-native DB read |
| POST | `/approvals` | Next.js proxy and page approval request | Rust-native DB write with audit event |
| POST | `/approvals/{approval_id}/decision` | Page approval decision | Rust-native pending-only decision with audit event |
| POST | `/chat/retrieval-preview` | Next.js proxy and page chat preview | Rust-native contract response |
| POST | `/chat/evidence-answer` | Page evidence answer | Rust-native contract response |
| GET | `/artifacts` | Page data load | Rust-native DB read |
| GET | `/audit-events` | Page data load | Rust-native DB read |
| GET | `/collection-runs` | Page data load | Rust-native DB read |
| POST | `/collection-runs/dry-run` | Page collection preview | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/manual-upload` | Page manual upload collection | Rust-native DB/artifact write with source permission, approval, and audit events |
| GET | `/evidence/documents` | Page data load | Rust-native DB read |
| GET | `/evidence/chunks` | Page data load | Rust-native DB read |
| GET | `/evidence/items` | Page data load | Rust-native DB read |
| GET | `/evidence/claims` | Page data load | Rust-native DB read |
| GET | `/feedback` | Page data load | Rust-native DB read |
| GET | `/outcomes` | Page data load | Rust-native DB read |
| POST | `/feedback` | Page review feedback | Rust-native DB write with audit event |
| POST | `/outcomes` | Page review outcome | Rust-native DB write with audit events |
| GET | `/memory/graph/schema` | Page data load | Rust-native read-only status |
| GET | `/memory/vector/chunks` | Page data load | Rust-native read-only status |
| POST | `/reports` | Page report create | Rust-native DB write with audit event |
| GET | `/reports` | Page data load | Rust-native DB read |
| POST | `/reports/{report_id}/render` | Page report render | Rust-native bounded metadata render with artifact and audit event |
| GET | `/settings/env` | Next.js proxy and page settings load | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Next.js proxy and page settings verify | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Next.js proxy and page settings apply | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/sources` | Page data load | Rust-native DB read |
| POST | `/sources` | Page source create | Rust-native DB write with optional permission and audit event |
| GET | `/work-items` | Page data load | Rust-native DB read |
| POST | `/work-items/` | Page work item creation | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Page work dispatch | Rust-native validation and non-executing dispatch audit marker |

## Archived FastAPI Route Inventory

The archived FastAPI source at `archive/legacy-python/services-api/app`
contains `/` plus the following APIRouter routes. DIFF-105 automated the count
and found 90 APIRouter routes plus `/`; this manual table remains a
human-readable inventory of the historical FastAPI route families and current
Rust gateway behavior.

| Method | Route | Gateway behavior |
| --- | --- | --- |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/intent` | Rust-native |
| POST | `/agent/actions/{action_name}/execute` | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| GET | `/analysis/patterns` | Rust-native DB read |
| POST | `/analysis/patterns` | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Rust-native DB status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Rust-native DB write with deterministic local detection and audit events |
| GET | `/analysis/patterns/{pattern_id}` | Rust-native DB read |
| GET | `/analysis/hypotheses` | Rust-native DB read |
| POST | `/analysis/hypotheses` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/hypotheses/{hypothesis_id}` | Rust-native DB read |
| GET | `/analysis/predictions` | Rust-native DB read |
| POST | `/analysis/predictions` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/predictions/{prediction_id}` | Rust-native DB read |
| GET | `/analysis/recommendations` | Rust-native DB read |
| POST | `/analysis/recommendations` | Rust-native DB write with evidence validation and audit event |
| GET | `/analysis/recommendations/{recommendation_id}` | Rust-native DB read |
| GET | `/approvals` | Rust-native DB read |
| POST | `/approvals` | Rust-native DB write with audit event |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| POST | `/approvals/{approval_id}/decision` | Rust-native pending-only decision with audit event |
| GET | `/artifacts` | Rust-native DB read |
| POST | `/artifacts` | Rust-native DB/artifact write with content-addressed storage and audit event |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| POST | `/chat/retrieval-preview` | Rust-native |
| POST | `/chat/evidence-answer` | Rust-native |
| GET | `/collection-runs` | Rust-native DB read |
| POST | `/collection-runs` | Rust-native DB write with source validation and audit event |
| POST | `/collection-runs/dry-run` | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/manual-upload` | Rust-native DB/artifact write with source permission, approval, and audit events |
| POST | `/collection-runs/manual-upload/ingest` | Rust-native bounded manual text ingest with vector upsert behavior |
| POST | `/collection-runs/local-project` | Rust-native scoped local-project collection with path safety checks |
| GET | `/collection-runs/{collection_run_id}` | Rust-native DB read |
| GET | `/evidence/documents` | Rust-native DB read |
| GET | `/evidence/documents/{document_id}` | Rust-native DB read |
| POST | `/evidence/documents` | Rust-native DB write with artifact/source validation and audit event |
| POST | `/evidence/documents/{document_id}/chunks` | Rust-native DB chunk/evidence-item generation with audit event |
| GET | `/evidence/items` | Rust-native DB read |
| GET | `/evidence/items/{evidence_item_id}` | Rust-native DB read |
| GET | `/evidence/chunks` | Rust-native DB read |
| GET | `/evidence/chunks/{chunk_id}` | Rust-native DB read |
| GET | `/evidence/claims` | Rust-native DB read |
| GET | `/evidence/claims/{claim_id}` | Rust-native DB read |
| POST | `/evidence/items` | Rust-native DB write with source/document/chunk validation and audit event |
| GET | `/experiments` | Rust-native DB read |
| POST | `/experiments` | Rust-native DB write with validation and audit event |
| POST | `/experiments/{experiment_run_id}/status` | Rust-native DB status transition with audit event |
| GET | `/experiments/{experiment_run_id}` | Rust-native DB read |
| GET | `/feedback` | Rust-native DB read |
| POST | `/feedback` | Rust-native DB write with audit event |
| GET | `/feedback/{feedback_id}` | Rust-native DB read |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/improvements` | Rust-native DB read |
| POST | `/improvements` | Rust-native DB write with validation and audit event |
| GET | `/improvements/{improvement_item_id}` | Rust-native DB read |
| GET | `/memory/graph/schema` | Rust-native read-only status |
| POST | `/memory/graph/schema/ensure` | Rust-native Neo4j schema ensure via bounded service call |
| POST | `/memory/graph/lineage/sync` | Rust-native Neo4j lineage sync via bounded service calls |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | Rust-native Neo4j relationship read with label allowlist |
| GET | `/memory/vector/chunks` | Rust-native read-only status |
| POST | `/memory/vector/chunks/ensure` | Rust-native Qdrant collection ensure via bounded service call |
| POST | `/memory/vector/chunks/upsert` | Rust-native Qdrant chunk vector upsert with DB status update |
| POST | `/memory/vector/chunks/search` | Rust-native Qdrant chunk vector search |
| GET | `/outcomes` | Rust-native DB read |
| POST | `/outcomes` | Rust-native DB write with audit events |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| GET | `/reports` | Rust-native DB read |
| POST | `/reports` | Rust-native DB write with audit event |
| POST | `/reports/{report_id}/status` | Rust-native DB status update with audit event |
| POST | `/reports/{report_id}/work-item` | Rust-native DB work-item creation with audit event |
| POST | `/reports/{report_id}/render` | Rust-native bounded metadata render with artifact and audit event |
| GET | `/reports/{report_id}` | Rust-native DB read |
| GET | `/retrieval/chunks/{chunk_id}/trail` | Rust-native DB evidence trail hydration |
| POST | `/retrieval/chunks/search` | Rust-native DB hydrated chunk search |
| GET | `/settings/env` | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/sources` | Rust-native DB read |
| POST | `/sources` | Rust-native DB write with optional permission and audit event |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| POST | `/sources/{source_id}/permissions` | Rust-native DB permission creation with audit event |
| GET | `/work-items` | Rust-native DB read |
| POST | `/work-items` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Rust-native validation and non-executing dispatch audit marker |
| POST | `/work-items/{work_item_id}/status` | Rust-native DB status transition with audit event |
| GET | `/work-items/{work_item_id}` | Rust-native DB read |

## Cutover Script Finding

`scripts/rust-cutover.sh` correctly enforces manifest shape, Rust checks,
`cutover_ready=true`, and a clean worktree before `--execute`. Since DIFF-105 it
also runs the route parity guard. DIFF-138 uses that guard plus manifest and
Compose state to remove the FastAPI fallback wiring from the runtime API path.

DIFF-105 adds `scripts/rust-route-parity.py` and runs it from
`scripts/rust-cutover.sh --check`. The guard inventories source-defined routes
and validates that the manifest marks FastAPI fallback as required while parity
is incomplete, and as not required once DIFF-138 completes parity and removes
fallback wiring. DIFF-106 extends the guard to count the Rust gateway route
registry and records the reduced fallback counts. DIFF-107 records the second
DB read batch and reduces web route fallback dependency again. DIFF-108 adds
Rust-native read-only settings/env metadata, vector status, and graph status
routes without reading `.env` contents or mutating Qdrant/Neo4j. DIFF-109 adds
Rust-native approval request creation with audit event insertion. DIFF-110 adds
Rust-native feedback and outcome writes with validation, audit insertion, and
their Python side-effect parity for source trust, weak-feedback improvement
items, and outcome target updates. DIFF-111 adds Rust-native source creation
with optional initial permission insertion and the deterministic
`source.created` audit event. DIFF-112 adds Rust-native report creation with
deterministic validation and the `report.created` audit event. DIFF-113 adds
Rust-native analysis pattern creation and baseline pattern detection with
evidence validation, deterministic local candidate generation, duplicate
detector-key suppression, and `analysis.pattern.created` audit events.
DIFF-114 adds Rust-native collection dry-run preview creation with
source/permission validation, scaffold connector preview parity, and
`collection_run.created` plus `collection_run.dry_run_preview` audit events.
DIFF-115 adds Rust-native settings verify/apply with allowlisted validation,
secret redaction, candidate-hash token compatibility, safe `.env` backup/write
constraints, and `settings.env.updated` audit events. Rust intentionally does
not execute Docker Compose from HTTP request handlers; Compose validation
remains an operator verification step.
DIFF-116 adds Rust-native work-item creation with intent verification context
validation, supported-type validation, deterministic
`pending_intent_verification` status, and `work_item.created` audit events. It
does not dispatch work, execute agents, migrate manual upload, or change
work-item status routes. DIFF-117 adds Rust-native manual upload collection
creation with source type, source permission, approval, text MIME/content, and
safe filename validation; bounded content-addressed artifact writes via
`crates/igy6-artifacts`; collection run, raw artifact, and queued normalization
work item metadata inserts; and `collection_run.created`, `raw_artifact.created`,
and `work_item.created` audit events. It does not execute normalization,
dispatch workers, ingest into vector/graph memory, or execute agents
synchronously. If the artifact write succeeds but the database transaction later
fails, the content-addressed artifact may remain under the configured safe
artifact root without DB metadata; retrying the same upload reuses the same
hash path.
DIFF-118 adds Rust-native web-used agent action request/execution routes. The
Rust gateway accepts only the existing fixed action allowlist, rejects malformed
action names, rejects user-provided `argv`/command surfaces, requires approved
matching `agent_action` approvals for stack-changing actions, writes
`agent.action.requested`, `agent.action.started`, `agent.action.finished`, and
`agent.action.rejected` audit events where persistence is available, and calls
script-backed actions only through the local-only host bridge with a token and
fixed action name. No arbitrary shell command or raw user text execution is
added.
DIFF-120 adds Rust-native dynamic web controls for pattern review, approval
decision, report render, and work-item dispatch. DIFF-132 adds Rust-native
handlers for the active medium-risk route bucket: analysis creation, evidence
document/chunk/item writes, source permission creation, report/work-item status
transitions, and retrieval trail/search hydration. DIFF-133 adds Rust-native
graph/vector memory parity with bounded Neo4j and Qdrant service calls. At the
time of DIFF-133, FastAPI fallback remained required for the 13 classified
routes still missing from Rust. DIFF-138 later removed fallback after route
parity reached zero missing FastAPI routes.

## Manifest Finding

The manifest accurately showed the DIFF-102 gateway phase complete, but it did
not include an explicit post-cutover route-parity phase or a machine-readable
statement for the then-current fallback requirement. DIFF-104 added that status
so future cutover checks could not be read as proof that FastAPI was removable
before route parity completed. DIFF-138 superseded that posture by removing
fallback after route parity reached zero missing FastAPI routes.

## Follow-Up DIFF Plan

Web-used FastAPI fallback was eliminated as of DIFF-118. The following plan
then drove unsupported non-web route migration or retirement through DIFF-138:

1. DIFF-105: add an automated route parity guard so fallback dependency cannot
   become an undocumented manual finding.
2. DIFF-106: implement Rust native handling for the first safe web-critical
   read-only DB route batch: sources, approvals, work-items, reports, and
   evidence document/item/chunk/claim list/detail reads.
3. DIFF-107: continue fallback reduction with the next safest web-critical
   read routes: audit events, artifacts, collection runs, feedback, outcomes,
   and analysis list/detail reads.
4. DIFF-108: migrate Rust-native read-only settings/env metadata,
   vector-memory status, and graph-memory status without `.env` reads or
   Qdrant/Neo4j mutation.
5. DIFF-109: migrate approval request creation with explicit audit-event
   parity.
6. DIFF-110: migrate feedback and outcome write routes with explicit validation
   and audit parity.
7. DIFF-111: migrate source creation with optional permission and explicit
   `source.created` audit parity.
8. DIFF-112: migrate report creation with explicit `report.created` audit
   parity.
9. DIFF-113: migrate analysis pattern creation and baseline detection with
   explicit evidence validation and `analysis.pattern.created` audit parity.
10. DIFF-114: migrate collection dry-run preview creation with explicit
   source/permission validation and audit parity.
11. DIFF-115: migrate settings env verify/apply with safe redaction, token, and
   audit parity.
12. DIFF-116: migrate work-item creation with intent verification and
   `work_item.created` audit parity, without dispatching work.
13. DIFF-117: migrate manual upload collection creation with source permission,
   approval, artifact storage, queued normalization metadata, and audit parity
   without synchronous ingestion or dispatch.
14. DIFF-118: migrate the final web-used agent action request/execution routes
   with fixed allowlists, approval gates, audit events, local-only host bridge
   execution, timeout bounds, redaction, and no user-provided argv.
15. DIFF-119: audit the remaining non-web FastAPI route inventory and decide
   which routes are active, retireable, or still require Rust parity.
16. DIFF-132: migrate the active medium-risk route bucket for analysis,
   evidence, source permission, report/work-item status, and retrieval
   trail/search parity.
17. DIFF-133: migrate graph/vector memory parity routes with scoped
   Neo4j/Qdrant safety and failure tests.
18. DIFF-134 through DIFF-137: resolve report work-item, artifact/collection
   ingestion, experiments/improvements, and duplicate root fallback buckets.
19. DIFF-138: remove `legacy-api` fallback wiring after route parity tests
   prove no active FastAPI route depends on it.
20. DIFF-139: archive legacy FastAPI API source and preserve Python/Celery
   worker and beat as active runtime services.
21. DIFF-140: final audit stating Rust-native API with retained Python worker
    and beat services. Full Rust-only repository or runtime operation is not
    claimed.
22. Recommended next DIFF: worker execution parity, or an explicit long-term
    Python worker retention decision.
23. DIFF-141: audit Python/Celery worker and beat usage and recommend
    one-job-family-at-a-time Rust worker execution parity.
24. Recommended next DIFF: DIFF-142 Rust worker execution contract and
    queue-claim foundation.
25. DIFF-142: add Rust worker queue-claim contract and bounded claim planning
    without job execution.
26. DIFF-143: add Rust `collection_normalization` execution parity planning
    and executor contracts without migrating chunking or vector upsert.
27. DIFF-144: add Rust `document_chunking` execution parity planning and
    executor contracts without migrating vector upsert.
28. Recommended next DIFF: DIFF-145 Rust `chunk_vector_upsert` execution
    parity.
