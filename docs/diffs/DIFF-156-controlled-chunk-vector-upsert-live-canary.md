# DIFF-156 Controlled Chunk Vector Upsert Live Canary

## Scope

DIFF-156 runs exactly one controlled Rust worker live canary for
`chunk_vector_upsert` using isolated synthetic PostgreSQL data and an isolated
local Qdrant container. It does not process broad queues, does not process more
than one work item, does not touch production/private runtime data, does not
remove Python/Celery `worker` or `beat`, and does not claim a full Rust-only
runtime.

## Decision

Decision A: safe `chunk_vector_upsert` canary was available and run exactly
once, but the canary failed safely at Qdrant point upsert.

The canary verified PostgreSQL claim/failure audit behavior and Qdrant
collection ensure behavior. It did not verify successful Qdrant point upsert or
chunk embedding metadata/status updates.

## Fixture

Preparation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture chunk_vector_upsert --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff156-canary.sql
```

The fixture was applied to an isolated local PostgreSQL container and paired
with an isolated local Qdrant container:

```bash
docker run -d --rm --name igy6-diff156-postgres -e POSTGRES_USER=igy6 -e POSTGRES_PASSWORD=diff156-local -e POSTGRES_DB=igy6_canary -p 127.0.0.1:55435:5432 postgres:16
docker run -d --rm --name igy6-diff156-qdrant -p 127.0.0.1:16333:6333 qdrant/qdrant:v1.12.5
docker cp /tmp/igy6-diff156-canary.sql igy6-diff156-postgres:/tmp/igy6-diff156-canary.sql
docker exec igy6-diff156-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff156-canary.sql
```

Selected canary work item:

```text
work-item-18b25ee83a881458-6
```

This is the chained `chunk_vector_upsert` work item ID observed in DIFF-155.
The DIFF-156 fixture seeds synthetic chunks with the same selected chunk IDs:

- `chunk-18b25ee83a4d8467-0`
- `chunk-18b25ee83a4d9290-2`
- `chunk-18b25ee83a4d94f0-4`

## Live Canary

Initial non-escalated canary command failed before connecting to the isolated
PostgreSQL container and did not claim or mutate the selected work item. The
work item was confirmed still `queued`, then the same selected one-item canary
was rerun with local container access.

Canary command that reached the isolated services:

```bash
env DATABASE_URL=postgresql://igy6:diff156-local@127.0.0.1:55435/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff156-canary QDRANT_URL=http://127.0.0.1:16333 QDRANT_CHUNK_COLLECTION=igy6_diff156_chunks QDRANT_CHUNK_VECTOR_SIZE=384 IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item work-item-18b25ee83a881458-6
```

Observed Rust result:

- `result_state`: `failed`
- `status`: `canary_failed`
- `work_type`: `chunk_vector_upsert`
- `work_item_id`: `work-item-18b25ee83a881458-6`
- `error_message`: `Qdrant point upsert failed with HTTP 400`

Executed side effects reported by Rust:

- `postgres_work_item_claim`
- `audit_work_item_claimed`
- `audit_work_item_started`
- `postgres_work_item_failed`
- `audit_worker_failure`

Qdrant collection ensure occurred before the point upsert failure. The isolated
Qdrant logs show:

- `GET /collections/igy6_diff156_chunks` returned `404`
- `PUT /collections/igy6_diff156_chunks` returned `200`
- `PUT /collections/igy6_diff156_chunks/points` returned `400`

## Observed PostgreSQL Side Effects

Read-only observation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture chunk_vector_upsert --emit-observation-sql --write-sql /tmp/igy6-diff156-observe.sql
docker cp /tmp/igy6-diff156-observe.sql igy6-diff156-postgres:/tmp/igy6-diff156-observe.sql
docker exec igy6-diff156-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff156-observe.sql
```

Observed `work_items`:

| id | work_type | status | error_message |
| --- | --- | --- | --- |
| `work-item-18b25ee83a881458-6` | `chunk_vector_upsert` | `failed` | `Qdrant point upsert failed with HTTP 400` |

Observed `audit_events`:

| event_type | decision | resource_type | resource_id | correlation_id |
| --- | --- | --- | --- | --- |
| `rust_worker_canary_fixture.selected` | `selected` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `work_item.claimed` | `running` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `work_item.started` | `running` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |
| `chunk_vectors.failed` | `failed` | `work_item` | `work-item-18b25ee83a881458-6` | `work-item-18b25ee83a881458-6` |

Observed `chunks`:

| id | embedding_status | embedding_method | vector_collection |
| --- | --- | --- | --- |
| `chunk-18b25ee83a4d8467-0` | `not_started` | empty | empty |
| `chunk-18b25ee83a4d9290-2` | `not_started` | empty | empty |
| `chunk-18b25ee83a4d94f0-4` | `not_started` | empty | empty |

No chunk metadata/status updates were committed after the failed Qdrant point
upsert.

## Observed Qdrant Side Effects

Qdrant collection status command:

```bash
curl -sS http://127.0.0.1:16333/collections/igy6_diff156_chunks
```

Observed result:

- Collection `igy6_diff156_chunks` exists.
- Collection status is `green`.
- Vector size is `384`.
- Distance is `Cosine`.
- `points_count` is `0`.
- `indexed_vectors_count` is `0`.

Qdrant points observation command:

```bash
curl -sS -X POST http://127.0.0.1:16333/collections/igy6_diff156_chunks/points/scroll -H 'Content-Type: application/json' -d '{"limit":10,"with_payload":true,"with_vector":false}'
```

Observed result:

- `points`: `[]`
- `next_page_offset`: `null`

Qdrant collection ensure was verified. Qdrant point upsert was attempted but
failed with HTTP 400, so no points were stored.

## Current Runtime Posture

Python/Celery `worker` remains required for production live processing.
Python/Celery `beat` remains required until scheduler/beat posture is replaced
or explicitly retired in a later DIFF. Docker Compose was not changed.
`services/worker/` was not removed.

Full Rust-only repository or runtime is not claimed.

## Still Unverified

- Successful live Qdrant point upsert for `chunk_vector_upsert`.
- Chunk `embedding_status=completed` and chunk vector metadata updates after a
  successful Qdrant upsert.
- Long-running Rust worker process ownership.
- Docker Compose Rust worker replacement.
- Beat/scheduler replacement or retirement.
- Production rollback posture after Rust worker process ownership changes.

## Next Recommended DIFF

DIFF-157 should fix or adapt the Rust/Qdrant point upsert compatibility
surfaced by DIFF-156, then rerun exactly one controlled
`chunk_vector_upsert` live canary against isolated synthetic data.
