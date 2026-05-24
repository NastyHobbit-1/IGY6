# DIFF-155 Corrected Document Chunking Live Canary

## Scope

DIFF-155 runs exactly one corrected Rust worker live canary for
`document_chunking` against isolated synthetic data. It does not process broad
queues, does not run `chunk_vector_upsert`, does not touch production/private
runtime data, does not remove Python/Celery `worker` or `beat`, and does not
claim a full Rust-only runtime.

## Decision

Decision A: corrected `document_chunking` canary available and run.

The DIFF-154 fixture correction is effective: the synthetic
`document_chunking` fixture now uses `chunk_size=100`, which satisfies the Rust
contract bound of 100 through 5000.

## Fixture

Preparation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture document_chunking --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff155-canary.sql
```

The fixture was applied to an isolated local PostgreSQL container:

```bash
docker run -d --rm --name igy6-diff155-postgres -e POSTGRES_USER=igy6 -e POSTGRES_PASSWORD=diff155-local -e POSTGRES_DB=igy6_canary -p 127.0.0.1:55434:5432 postgres:16
docker cp /tmp/igy6-diff155-canary.sql igy6-diff155-postgres:/tmp/igy6-diff155-canary.sql
docker exec igy6-diff155-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff155-canary.sql
```

Selected canary work item:

```text
diff-154-canary-work-item
```

The ID retains the DIFF-154 prefix because it is the deterministic
`document_chunking` fixture ID added and corrected during DIFF-154. DIFF-155
uses the corrected fixture output and runs exactly one live canary for that
selected work item.

## Live Canary

Canary command:

```bash
env DATABASE_URL=postgresql://igy6:diff155-local@127.0.0.1:55434/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff155-canary QDRANT_URL=http://127.0.0.1:6333 IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item diff-154-canary-work-item
```

Observed Rust result:

- `result_state`: `completed`
- `status`: `canary_completed`
- `work_type`: `document_chunking`
- `work_item_id`: `diff-154-canary-work-item`
- `created_chunk_ids`: `chunk-18b25ee83a4d8467-0`,
  `chunk-18b25ee83a4d9290-2`, `chunk-18b25ee83a4d94f0-4`
- `created_evidence_ids`: `evidence-18b25ee83a4d8fbc-1`,
  `evidence-18b25ee83a4d93e6-3`, `evidence-18b25ee83a4d95d5-5`
- `chunk_vector_upsert_work_item_id`: `work-item-18b25ee83a881458-6`
- `skipped_document_ids`: `[]`

Executed side effects reported by Rust:

- `postgres_work_item_claim`
- `audit_work_item_claimed`
- `audit_work_item_started`
- `postgres_chunk_writes`
- `postgres_evidence_item_writes`
- `postgres_chained_work_item_write`
- `postgres_work_item_completed`
- `audit_worker_success`

Planned-only side effects reported by Rust:

- `artifact_store_read`
- `qdrant_collection_and_points`

For `document_chunking`, artifact reads are not expected. Qdrant side effects
are expected only for the chained `chunk_vector_upsert` work item, which this
DIFF is prohibited from running.

## Observed Database Side Effects

Read-only observation command:

```bash
scripts/rust-worker-canary-fixture.py --fixture document_chunking --emit-observation-sql --write-sql /tmp/igy6-diff155-observe.sql
docker cp /tmp/igy6-diff155-observe.sql igy6-diff155-postgres:/tmp/igy6-diff155-observe.sql
docker exec igy6-diff155-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff155-observe.sql
```

Observed `work_items`:

| id | work_type | status | error_message |
| --- | --- | --- | --- |
| `diff-154-canary-work-item` | `document_chunking` | `completed` | empty |

Observed chained work item:

| id | work_type | status | parent_work_item_id |
| --- | --- | --- | --- |
| `work-item-18b25ee83a881458-6` | `chunk_vector_upsert` | `queued` | `diff-154-canary-work-item` |

Observed `audit_events`:

| event_type | decision | resource_type | resource_id | correlation_id |
| --- | --- | --- | --- | --- |
| `rust_worker_canary_fixture.selected` | `selected` | `work_item` | `diff-154-canary-work-item` | `diff-154-canary-work-item` |
| `work_item.claimed` | `running` | `work_item` | `diff-154-canary-work-item` | `diff-154-canary-work-item` |
| `work_item.started` | `running` | `work_item` | `diff-154-canary-work-item` | `diff-154-canary-work-item` |
| `work_item.created` | `queued` | `work_item` | `work-item-18b25ee83a881458-6` | `diff-154-canary-work-item` |
| `document_chunks.generated` | `completed` | `work_item` | `diff-154-canary-work-item` | `diff-154-canary-work-item` |

Observed `chunks`:

| id | document_id | chunk_index | embedding_status | text_length |
| --- | --- | --- | --- | --- |
| `chunk-18b25ee83a4d8467-0` | `diff-154-canary-document` | 0 | `not_started` | 100 |
| `chunk-18b25ee83a4d9290-2` | `diff-154-canary-document` | 1 | `not_started` | 100 |
| `chunk-18b25ee83a4d94f0-4` | `diff-154-canary-document` | 2 | `not_started` | 61 |

Observed `evidence_items`:

| id | document_id | chunk_id | evidence_type | statement_length |
| --- | --- | --- | --- | --- |
| `evidence-18b25ee83a4d8fbc-1` | `diff-154-canary-document` | `chunk-18b25ee83a4d8467-0` | `document_chunk` | 100 |
| `evidence-18b25ee83a4d93e6-3` | `diff-154-canary-document` | `chunk-18b25ee83a4d9290-2` | `document_chunk` | 100 |
| `evidence-18b25ee83a4d95d5-5` | `diff-154-canary-document` | `chunk-18b25ee83a4d94f0-4` | `document_chunk` | 61 |

The normalized source document remained the synthetic fixture document:
`diff-154-canary-document`, `text`, `internal`, text length 261.

The isolated PostgreSQL canary container was stopped after observation.

## Current Runtime Posture

Python/Celery `worker` remains required for production live processing.
Python/Celery `beat` remains required until scheduler/beat posture is replaced
or explicitly retired in a later DIFF. Docker Compose was not changed.
`services/worker/` was not removed.

Full Rust-only repository or runtime is not claimed.

## Still Unverified

- Live `chunk_vector_upsert` canary, including Qdrant collection ensure and
  point upsert side effects.
- Chunk `embedding_status` and metadata updates after a successful Qdrant
  canary.
- Long-running Rust worker process ownership.
- Docker Compose Rust worker replacement.
- Beat/scheduler replacement or retirement.
- Production rollback posture after Rust worker process ownership changes.

## Next Recommended DIFF

DIFF-156 should run exactly one controlled `chunk_vector_upsert` live Rust
worker canary against isolated synthetic data and audit observed Qdrant,
PostgreSQL, and audit side effects.
