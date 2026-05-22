# DIFF-141: Worker Execution Parity Audit

Status: Locked

## Type

Audit-only worker parity planning update.

## Objective

Audit Python/Celery `worker` and `beat` usage after DIFF-140 and decide the
safest Rust worker execution parity path.

Decision:

- Migrate worker execution to Rust one job family at a time.
- Keep Python/Celery `worker` and `beat` active until execution parity is
  implemented and verified.
- Do not remove `services/worker/`, `worker`, or `beat` in this DIFF.
- Do not claim full Rust-only repository or runtime operation in this DIFF.

## Baseline Facts

- DIFF-140 is complete and locked.
- The active API path is Rust-native.
- FastAPI fallback is removed.
- `services/api/` is archived.
- Python/Celery `worker` and `beat` remain active Docker Compose services.
- `crates/igy6-worker` is planning-only and does not perform live worker
  execution, database writes, audit writes, artifact reads or writes, Qdrant
  updates, Neo4j operations, queue consumption, or scheduled work.

## Allowed Scope

- `docs/diffs/DIFF-141-worker-execution-parity-audit.md`
- `configs/rust-cutover-manifest.json`
- Live worker-related documentation if stale.

## Prohibited Scope

- No worker execution migration.
- No removal of `services/worker/`.
- No removal of `worker` or `beat` from Docker Compose.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, redesign, data model changes,
  migration changes, or dependency changes.

## Inventory

### Python/Celery Tasks

1. `phase0.health`
   - Function: `health`.
   - Behavior: returns a static worker health payload.
   - DB writes: none.
   - Audit writes: none.
   - Artifact writes: none.
   - Qdrant operations: none.
   - Neo4j operations: none.
   - Rust coverage: API health exists in Rust gateway; no Rust worker health
     task execution exists.
   - Parity need: low; can be replaced by Rust worker readiness/status once a
     Rust worker process exists.

2. `collection.normalization_scaffold`
   - Function: `normalization_scaffold`.
   - Behavior: returns a scaffold-only `not_executed` payload.
   - DB writes: none.
   - Audit writes: none.
   - Artifact writes: none.
   - Qdrant operations: none.
   - Neo4j operations: none.
   - Rust coverage: not needed for live processing unless historical task
     compatibility is required.
   - Parity need: retire or keep as a compatibility no-op in Rust only if
     queued historical work depends on it.

3. `collection.normalize_collection_run`
   - Function: `normalize_collection_run`.
   - Behavior: validates a `collection_normalization` work item, reads raw
     artifact bytes from local artifact storage, decodes UTF-8 text, inserts
     normalized documents, marks the work item running/completed/failed, creates
     a chained `document_chunking` work item, and writes completion/failure audit
     events.
   - DB tables written: `work_items`, `normalized_documents`, `audit_events`.
   - DB tables read: `work_items`, `collection_runs`, `raw_artifacts`,
     `normalized_documents`.
   - Audit writes: `work_item.created`,
     `collection_normalization.completed`,
     `collection_normalization.failed`.
   - Artifact paths read: relative `raw_artifacts.storage_path` under
     `ARTIFACT_STORE_PATH`.
   - Artifact writes: none.
   - Qdrant operations: none.
   - Neo4j operations: none.
   - Rust coverage: `crates/igy6-worker::plan_utf8_pipeline` can plan UTF-8
     normalization; Rust gateway creates `collection_normalization` work items.
   - Missing Rust parity: queue claiming, artifact file read, idempotent DB
     inserts, work-item transitions, chained work item creation, audit writes,
     and failure handling.

4. `evidence.generate_document_chunks`
   - Function: `generate_document_chunks`.
   - Behavior: validates a `document_chunking` work item when supplied, splits
     normalized document text, inserts chunks and evidence items, marks the work
     item running/completed/failed, creates a chained `chunk_vector_upsert` work
     item, and writes completion/failure audit events.
   - DB tables written: `work_items`, `chunks`, `evidence_items`,
     `audit_events`.
   - DB tables read: `work_items`, `normalized_documents`, `chunks`.
   - Audit writes: `work_item.created`, `document_chunks.generated`,
     `document_chunks.failed`.
   - Artifact writes: none.
   - Qdrant operations: none.
   - Neo4j operations: none.
   - Rust coverage: `crates/igy6-chunking` plans deterministic chunks and
     evidence; `crates/igy6-worker::plan_utf8_pipeline` composes chunk/evidence
     planning after normalization.
   - Missing Rust parity: queued task execution, document reads from Postgres,
     idempotent chunk/evidence DB inserts, work-item transitions, chained vector
     work item creation, audit writes, and failure handling.

5. `memory.vector.upsert_chunks`
   - Function: `upsert_chunk_vectors`.
   - Behavior: validates a `chunk_vector_upsert` work item when supplied, selects
     chunks whose embeddings are not completed, creates deterministic local hash
     embeddings, ensures the Qdrant collection, upserts points, marks chunks
     completed, marks the work item completed/failed, and writes
     completion/failure audit events.
   - DB tables written: `work_items`, `chunks`, `audit_events`.
   - DB tables read: `work_items`, `chunks`.
   - Audit writes: `chunk_vectors.upserted`, `chunk_vectors.failed`.
   - Artifact writes: none.
   - Qdrant operations:
     - `GET /collections/{QDRANT_CHUNK_COLLECTION}`
     - `PUT /collections/{QDRANT_CHUNK_COLLECTION}` when missing
     - `PUT /collections/{QDRANT_CHUNK_COLLECTION}/points`
   - Neo4j operations: none.
   - Rust coverage: `crates/igy6-vector-memory` plans collection ensure,
     deterministic embeddings, search requests, and point upsert requests;
     Rust gateway has live `/memory/vector/chunks/upsert` behavior for a bounded
     batch with DB status updates.
   - Missing Rust parity: queue claiming by work item, requested chunk ID
     scoping from work payload, exact worker audit events, failure status
     handling, and background execution outside an HTTP request.

### Beat And Scheduled Tasks

- No `beat_schedule`, `crontab`, periodic task registration, or scheduled task
  configuration was found in `services/worker/app/celery_app.py` or
  `services/worker/app/tasks.py`.
- The `beat` container is active in Docker Compose but currently has no
  repo-defined scheduled work to execute.
- Because scheduled work is a required product capability, `beat` should not be
  removed until a future DIFF either proves it is unused and retires it, or
  replaces it with an explicit Rust scheduler.

### Worker Environment Variables

Required by `worker` and `beat` in Docker Compose:

- `CELERY_BROKER_URL`
- `CELERY_RESULT_BACKEND`
- `DATABASE_URL`
- `ARTIFACT_STORE_PATH`
- `QDRANT_URL`
- `QDRANT_CHUNK_COLLECTION`
- `QDRANT_CHUNK_VECTOR_SIZE`

Additional deployment input:

- `IGY6_DATA_ROOT` bind-mounted to `/workspace/storage`.

### Artifact, Qdrant, And Neo4j Summary

- Artifact reads: `collection.normalize_collection_run` reads relative raw
  artifact paths under `ARTIFACT_STORE_PATH`.
- Artifact writes: none found in Python/Celery worker tasks.
- Qdrant writes: `memory.vector.upsert_chunks` ensures the chunk collection and
  upserts deterministic chunk vectors.
- Neo4j operations: none found in Python/Celery worker tasks.

## Recommendation

Migrate worker execution to Rust one job family at a time.

Reasoning against the decision criteria:

- Faster runtime: Rust can avoid Python interpreter and Celery serialization
  overhead for deterministic local processing.
- Lower memory/resource usage: a Rust worker can eventually replace the Python
  worker image and Celery worker process; a Rust scheduler can remove the beat
  process if scheduled work remains needed.
- Simpler deployment: replacing Celery/beat can reduce Python package, Redis
  queue, and multi-process operational surface, but Redis should remain until a
  later DIFF proves all queue semantics have a Rust replacement.
- Fewer containers/processes: long-term target is one Rust worker process, or
  Rust gateway plus a Rust worker/scheduler, instead of Python worker plus beat.
- No behavior loss: migration must preserve DB writes, audit writes, artifact
  reads, Qdrant updates, idempotency, failure handling, and scheduled-work
  posture before removal.

The safest near-term posture is hybrid: keep Python/Celery active while Rust
execution parity is added and verified family by family.

## Future DIFF Plan

1. DIFF-142: Rust worker execution contract and queue-claim foundation.
   - Add a Rust executable or worker mode that can claim queued work items
     without invoking Celery.
   - Preserve intent verification, allowed work types, running/completed/failed
     transitions, audit shape, bounded concurrency, and local-only behavior.
   - Do not remove Python worker or beat.

2. DIFF-143: Rust `collection_normalization` execution parity.
   - Implement artifact read safety, UTF-8 normalization, normalized document
     inserts, duplicate skipping, chained `document_chunking` work item
     creation, and `collection_normalization.*` audit events.
   - Verify against existing Python semantics.

3. DIFF-144: Rust `document_chunking` execution parity.
   - Implement normalized document reads, chunk/evidence inserts, duplicate
     skipping, chained `chunk_vector_upsert` work item creation, and
     `document_chunks.*` audit events.

4. DIFF-145: Rust `chunk_vector_upsert` execution parity.
   - Implement requested chunk scoping, deterministic embeddings, Qdrant
     collection ensure, point upsert, chunk metadata/status updates, and
     `chunk_vectors.*` audit events in background execution.

5. DIFF-146: Scheduler and beat decision.
   - Either retire `beat` after proving no scheduled work exists, or implement a
     Rust scheduler for required scheduled rechecks.
   - Preserve scheduled-work product requirements before removing beat.

6. DIFF-147: Worker cutover readiness gate.
   - Run side-by-side or targeted parity verification.
   - Only then decide whether to remove Python/Celery `worker`, `beat`, and
     Celery-specific dependencies from Compose.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

`npm --prefix apps/web run build` is not required unless web-facing status text
changes.

## Completion Criteria

- Every Python/Celery task is inventoried.
- Beat/scheduled task status is documented.
- Worker DB writes, audit writes, artifact behavior, Qdrant operations, Neo4j
  operations, and environment variables are documented.
- Rust planning coverage and missing execution parity are documented.
- Manifest states Python/Celery worker remains active until execution parity is
  complete.
- Full Rust-only repository or runtime operation is not claimed.

## Completion Notes

DIFF-141 audits worker execution parity and recommends a hybrid transition:
retain Python/Celery while migrating worker execution to Rust one job family at
a time.

Current worker posture:

- Python/Celery `worker` remains required for live processing.
- Python/Celery `beat` remains in Docker Compose, but no repo-defined beat
  schedule was found.
- Rust worker code covers deterministic planning only.
- Rust does not yet cover background queue claiming, live worker DB writes,
  worker audit writes, artifact reads from queued work, Qdrant writes from
  queued work, failure handling, or scheduler replacement.
- Full Rust-only repository or runtime operation is not claimed.

Next recommended DIFF:

- DIFF-142 Rust worker execution contract and queue-claim foundation.

## Verification Results

- `git status --short` inspected scoped DIFF-141 changes and existing uncommitted
  DIFF-140 documentation changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed retained `worker` and `beat` services.
- `npm --prefix apps/web run build` was not run because DIFF-141 changed no
  web-facing UI/status source text.
