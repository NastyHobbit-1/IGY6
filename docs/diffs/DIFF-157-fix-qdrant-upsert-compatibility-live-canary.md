# DIFF-157 Fix Qdrant Upsert Compatibility Live Canary

## Scope

DIFF-157 fixes the Rust/Qdrant point upsert compatibility issue observed in
DIFF-156, then reruns exactly one controlled `chunk_vector_upsert` Rust worker
live canary against isolated synthetic PostgreSQL data and an isolated local
Qdrant container.

This DIFF does not process broad queues, does not process more than one work
item, does not touch production/private runtime data, does not remove
Python/Celery `worker` or `beat`, and does not claim a full Rust-only runtime.

## Decision

Decision A: Qdrant upsert compatibility fixed and the controlled
`chunk_vector_upsert` canary succeeded.

## Compatibility Fix

DIFF-156 used chunk IDs such as `chunk-18b25ee83a4d8467-0` as Qdrant point IDs.
Qdrant rejected the point upsert with HTTP 400 because point IDs must be
Qdrant-compatible IDs, not arbitrary chunk ID strings.

DIFF-157 updates `crates/igy6-vector-memory/src/lib.rs` so Qdrant point payloads
use deterministic UUID-shaped point IDs derived from the chunk ID. The original
chunk ID remains in the Qdrant payload as `chunk_id`, preserving chunk lineage
and query metadata while satisfying Qdrant point ID validation.

Tests were added/adjusted to verify:

- Qdrant point payloads use deterministic UUID-shaped IDs.
- Original `chunk_id`, `document_id`, `chunk_index`, and `embedding_method`
  remain in point payload metadata.

## Fixture

Preparation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture chunk_vector_upsert --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff157-canary.sql
```

The fixture was applied to isolated local PostgreSQL and Qdrant containers:

```bash
docker run -d --rm --name igy6-diff157-postgres -e POSTGRES_USER=igy6 -e POSTGRES_PASSWORD=diff157-local -e POSTGRES_DB=igy6_canary -p 127.0.0.1:55436:5432 postgres:16
docker run -d --rm --name igy6-diff157-qdrant -p 127.0.0.1:16334:6333 qdrant/qdrant:v1.12.5
docker cp /tmp/igy6-diff157-canary.sql igy6-diff157-postgres:/tmp/igy6-diff157-canary.sql
docker exec igy6-diff157-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff157-canary.sql
```

Selected work item:

```text
work-item-18b25ee83a881458-6
```

Selected chunk IDs:

- `chunk-18b25ee83a4d8467-0`
- `chunk-18b25ee83a4d9290-2`
- `chunk-18b25ee83a4d94f0-4`

## Live Canary

Canary command:

```bash
env DATABASE_URL=postgresql://igy6:diff157-local@127.0.0.1:55436/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff157-canary QDRANT_URL=http://127.0.0.1:16334 QDRANT_CHUNK_COLLECTION=igy6_diff157_chunks QDRANT_CHUNK_VECTOR_SIZE=384 IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item work-item-18b25ee83a881458-6
```

Observed Rust result:

- `result_state`: `completed`
- `status`: `canary_completed`
- `work_type`: `chunk_vector_upsert`
- `work_item_id`: `work-item-18b25ee83a881458-6`
- `chunks_selected`: 3
- `chunks_upserted`: 3

Executed side effects reported by Rust:

- `postgres_work_item_claim`
- `audit_work_item_claimed`
- `audit_work_item_started`
- `qdrant_collection_ensure`
- `qdrant_points_upsert`
- `postgres_chunk_embedding_updates`
- `postgres_work_item_completed`
- `audit_worker_success`

## Observed PostgreSQL Side Effects

Read-only observation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture chunk_vector_upsert --emit-observation-sql --write-sql /tmp/igy6-diff157-observe.sql
docker cp /tmp/igy6-diff157-observe.sql igy6-diff157-postgres:/tmp/igy6-diff157-observe.sql
docker exec igy6-diff157-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff157-observe.sql
```

Observed `work_items`:

| id | work_type | status | error_message |
| --- | --- | --- | --- |
| `work-item-18b25ee83a881458-6` | `chunk_vector_upsert` | `completed` | empty |

Observed `audit_events`:

| event_type | decision | resource_type | resource_id | correlation_id |
| --- | --- | --- | --- | --- |
| `rust_worker_canary_fixture.selected` | `selected` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `work_item.claimed` | `running` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `work_item.started` | `running` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `chunk_vectors.upserted` | `completed` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |

Observed `chunks`:

| id | embedding_status | embedding_method | vector_collection |
| --- | --- | --- | --- |
| `chunk-18b25ee83a4d8467-0` | `completed` | `local_hash_v1` | `igy6_diff157_chunks` |
| `chunk-18b25ee83a4d9290-2` | `completed` | `local_hash_v1` | `igy6_diff157_chunks` |
| `chunk-18b25ee83a4d94f0-4` | `completed` | `local_hash_v1` | `igy6_diff157_chunks` |

## Observed Qdrant Side Effects

Qdrant logs from the isolated `igy6-diff157-qdrant` container show:

- `GET /collections/igy6_diff157_chunks` returned `404`
- `PUT /collections/igy6_diff157_chunks` returned `200`
- `PUT /collections/igy6_diff157_chunks/points` returned `200`

Qdrant collection status command:

```bash
curl -sS http://127.0.0.1:16334/collections/igy6_diff157_chunks
```

Observed result:

- Collection `igy6_diff157_chunks` exists.
- Collection status is `green`.
- Vector size is `384`.
- Distance is `Cosine`.
- `points_count` is `3`.

Qdrant points observation command:

```bash
curl -sS -X POST http://127.0.0.1:16334/collections/igy6_diff157_chunks/points/scroll -H 'Content-Type: application/json' -d '{"limit":10,"with_payload":true,"with_vector":false}'
```

Observed points:

| qdrant_point_id | chunk_id | document_id | chunk_index | embedding_method |
| --- | --- | --- | --- | --- |
| `bf7a0ac7-8293-5ae9-8c85-62430be5fa39` | `chunk-18b25ee83a4d8467-0` | `diff-156-canary-document` | 0 | `local_hash_v1` |
| `5842bd3d-9fb4-5f16-a474-94b9284db426` | `chunk-18b25ee83a4d9290-2` | `diff-156-canary-document` | 1 | `local_hash_v1` |
| `e2373440-7d5c-5d49-92c9-951e3c69ccf9` | `chunk-18b25ee83a4d94f0-4` | `diff-156-canary-document` | 2 | `local_hash_v1` |

The isolated PostgreSQL and Qdrant canary containers were stopped after
observation.

## Current Runtime Posture

Python/Celery `worker` remains required for production live processing.
Python/Celery `beat` remains required until scheduler/beat posture is replaced
or explicitly retired in a later DIFF. Docker Compose was not changed.
`services/worker/` was not removed.

Full Rust-only repository or runtime is not claimed.

## Still Unverified

- Long-running Rust worker process ownership.
- Docker Compose Rust worker replacement.
- Beat/scheduler replacement or retirement.
- Multi-job queue processing outside the one-item canary gate.
- Production rollback posture after Rust worker process ownership changes.

## Next Recommended DIFF

DIFF-158 should decide Rust worker process cutover readiness after successful
one-item live canaries for `collection_normalization`, `document_chunking`, and
`chunk_vector_upsert`, with explicit handling for long-running worker ownership
and beat/scheduler posture.
