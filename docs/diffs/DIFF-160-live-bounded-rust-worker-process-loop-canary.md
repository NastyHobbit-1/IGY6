# DIFF-160 Live Bounded Rust Worker Process Loop Canary

## Scope

DIFF-160 adds and runs a live bounded Rust `--canary-loop` process-loop canary
against isolated synthetic PostgreSQL, synthetic `IGY6_DATA_ROOT`, and isolated
local Qdrant.

This DIFF does not process production/private runtime data, does not process an
unbounded queue, does not remove `services/worker/`, does not remove Docker
Compose `worker` or `beat`, does not disable Python/Celery, does not mutate
`.env`, and does not claim full Rust-only runtime.

## Decision

Decision A: live bounded process-loop canary was implemented and run.

## Loop Command

```bash
env DATABASE_URL=postgresql://igy6:diff160-local@127.0.0.1:55437/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff160-canary QDRANT_URL=http://127.0.0.1:16335 QDRANT_CHUNK_COLLECTION=igy6_diff160_chunks QDRANT_CHUNK_VECTOR_SIZE=384 IGY6_WORKER_PROCESS_CANARY=DIFF-159 cargo run -p igy6-worker -- --canary-loop --max-jobs 3 --max-idle-polls 1 --claim-limit 1 --poll-interval-ms 100
```

Loop bounds:

- `max_jobs=3`
- `max_idle_polls=1`
- `claim_limit=1`
- `poll_interval_ms=100`

The loop exited cleanly with:

- `status=process_canary_exited_cleanly`
- `result_state=completed`
- `exit_reason=max_jobs reached`
- `jobs_attempted=3`
- `jobs_completed=3`
- `jobs_failed=0`
- `idle_polls=0`

## Fixture

The canary used the existing synthetic collection fixture:

```bash
scripts/rust-worker-canary-fixture.py --fixture collection_normalization --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff160-canary.sql
scripts/rust-worker-canary-fixture.py --fixture collection_normalization --write-artifact --data-root /tmp/igy6-diff160-canary
```

The fixture was applied only to an isolated PostgreSQL container. Qdrant was a
separate isolated local container with collection `igy6_diff160_chunks`.

## Observed Work Items

The loop processed the synthetic chain in order:

| id | work_type | status |
| --- | --- | --- |
| `diff-152-canary-work-item` | `collection_normalization` | `completed` |
| `work-item-18b270d674236f6e-1` | `document_chunking` | `completed` |
| `work-item-18b270d674cbd7a1-4` | `chunk_vector_upsert` | `completed` |

## Observed Audit Events

Observed audit events included:

- `rust_worker_canary_fixture.selected`
- `work_item.claimed`
- `work_item.started`
- `work_item.created`
- `collection_normalization.completed`
- `document_chunks.generated`
- `chunk_vectors.upserted`

Claim/start events were observed for each processed work item.

## Observed PostgreSQL Side Effects

Observed `normalized_documents`:

- `document-18b270d6741271aa-0`
- `raw_artifact_id=diff-152-canary-raw`
- `source_id=diff-152-canary-source`
- `text_length=188`

Observed `chunks`:

- `chunk-18b270d674b4bf85-2`
- `embedding_status=completed`
- `embedding_method=local_hash_v1`
- `vector_collection=igy6_diff160_chunks`

Observed `evidence_items`:

- `evidence-18b270d674b4c6f0-3`
- `chunk_id=chunk-18b270d674b4bf85-2`
- `evidence_type=document_chunk`

## Observed Qdrant Side Effects

Observed collection:

- `igy6_diff160_chunks`
- status `green`
- vector size `384`
- distance `Cosine`
- `points_count=1`

Qdrant logs showed:

- `GET /collections/igy6_diff160_chunks` returned `404` before ensure
- `PUT /collections/igy6_diff160_chunks` returned `200`
- `PUT /collections/igy6_diff160_chunks/points` returned `200`
- point scroll returned one point

Observed Qdrant point:

| qdrant_point_id | chunk_id | document_id | chunk_index | embedding_method |
| --- | --- | --- | --- | --- |
| `68d8e7c8-6f4e-5d0c-9003-c8518571b2dc` | `chunk-18b270d674b4bf85-2` | `document-18b270d6741271aa-0` | 0 | `local_hash_v1` |

## Runtime Posture

Python/Celery `worker` remains required. DIFF-160 proves one bounded isolated
process-loop canary, not production worker ownership.

Python/Celery `beat` remains required until scheduler/beat replacement or
retirement is explicitly resolved.

Docker Compose was not changed.

Full Rust-only repository or runtime is not claimed.

## Remaining Blockers

Before worker cutover:

- Add a Rust worker Dockerfile or Compose canary service.
- Prove side-by-side worker operation without racing Python/Celery.
- Define health/readiness behavior for a Rust worker service.
- Prove production rollback from Rust worker ownership back to Python/Celery.
- Decide scheduler/beat replacement or retirement.

## Next Recommended DIFF

DIFF-161 should add a Docker/Compose Rust worker canary service or an equivalent
side-by-side service-level canary plan, still without removing Python/Celery.
