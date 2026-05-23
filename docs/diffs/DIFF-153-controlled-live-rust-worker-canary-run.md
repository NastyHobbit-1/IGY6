# DIFF-153: Controlled Live Rust Worker Canary Run

Status: Locked

## Type

Controlled live canary verification and observed side-effect audit.

## Objective

Apply the DIFF-152 synthetic fixture, run exactly one gated Rust worker canary
against `diff-152-canary-work-item`, and document observed side effects.

## Decision

Decision A: fixture applied and live canary run.

The canary ran against an isolated local PostgreSQL container and synthetic
`IGY6_DATA_ROOT`, not the normal runtime/private data root.

## Fixture Commands

Synthetic fixture and artifact preparation:

```bash
scripts/rust-worker-canary-fixture.py --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff153-canary.sql --write-artifact --data-root /tmp/igy6-diff153-canary
```

Isolated PostgreSQL canary container:

```bash
docker run -d --rm --name igy6-diff153-postgres -e POSTGRES_USER=igy6 -e POSTGRES_PASSWORD=diff153-local -e POSTGRES_DB=igy6_canary -p 127.0.0.1:55432:5432 postgres:16
docker cp /tmp/igy6-diff153-canary.sql igy6-diff153-postgres:/tmp/igy6-diff153-canary.sql
docker exec igy6-diff153-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff153-canary.sql
```

The container was stopped after observation:

```bash
docker stop igy6-diff153-postgres
```

## Canary Command

Exactly one live Rust canary was run:

```bash
env DATABASE_URL=postgresql://igy6:diff153-local@127.0.0.1:55432/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff153-canary QDRANT_URL=http://127.0.0.1:6333 IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item diff-152-canary-work-item
```

The first non-escalated attempt could not connect to the isolated local
PostgreSQL port from the sandbox. The successful run used the same command with
permission to connect to `127.0.0.1:55432`.

## Selected Work Item

- Work item ID: `diff-152-canary-work-item`
- Work type: `collection_normalization`
- Fixture source ID: `diff-152-canary-source`
- Fixture collection run ID: `diff-152-canary-run`
- Fixture raw artifact ID: `diff-152-canary-raw`
- Data root: `/tmp/igy6-diff153-canary`

## Canary Result

Rust reported:

- `result_state`: `completed`
- `status`: `canary_completed`
- `mutates_runtime_data`: `true`
- `created_document_ids`: `document-18b22ec6d500b8d3-0`
- `document_chunking_work_item_id`: `work-item-18b22ec6d50fc91f-1`
- `side_effects_executed`:
  - `postgres_work_item_claim`
  - `audit_work_item_claimed`
  - `audit_work_item_started`
  - `artifact_store_read`
  - `postgres_normalized_document_writes`
  - `postgres_chained_work_item_write`
  - `postgres_work_item_completed`
  - `audit_worker_success`

## Observed PostgreSQL Effects

Read-only observation SQL showed:

- `work_items`: `diff-152-canary-work-item` is `completed` with empty
  `error_message`.
- Chained `work_items`: `work-item-18b22ec6d50fc91f-1` is
  `document_chunking`, `queued`, and has parent
  `diff-152-canary-work-item`.
- `audit_events` rows:
  - `rust_worker_canary_fixture.selected`
  - `work_item.claimed`
  - `work_item.started`
  - `work_item.created`
  - `collection_normalization.completed`
- `normalized_documents`: `document-18b22ec6d500b8d3-0` was written for
  `diff-152-canary-raw`, source `diff-152-canary-source`, title
  `diff-152-rust-worker-canary.txt`, type `text`, sensitivity `internal`,
  text length `188`.
- `raw_artifacts`: `diff-152-canary-raw` retained the expected storage path,
  content hash, and size.

## Observed Artifact Effects

The synthetic artifact existed under the isolated data root:

```text
/tmp/igy6-diff153-canary/artifacts/sha256/26/15/26157f4935b30dbed9c3801007fea903db0c7192cc60422602703a5f22320256
```

Observed SHA-256:

```text
26157f4935b30dbed9c3801007fea903db0c7192cc60422602703a5f22320256
```

Observed byte count:

```text
188
```

## Qdrant

No Qdrant side effects were expected for this canary. The selected work type was
`collection_normalization`; Qdrant collection ensure and point upsert are only
expected for `chunk_vector_upsert`.

## Still Unverified

- Live `document_chunking` execution side effects.
- Live `chunk_vector_upsert` Qdrant collection ensure and point upsert.
- Live failure rollback posture.
- Long-running Rust worker process ownership.
- Docker Compose Rust worker replacement.
- Python/Celery beat replacement or retirement.

## Runtime Posture

Python/Celery `worker` remains required. Python/Celery `beat` remains required.
Docker Compose is unchanged. `services/worker/` is retained. Full Rust-only
runtime is not claimed.

## Next Recommended DIFF

DIFF-154 controlled `document_chunking` live canary verification.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m py_compile scripts/rust-worker-canary-fixture.py`
- `scripts/rust-worker-canary-fixture.py --emit-sql`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-worker`
- `cargo run -p igy6-worker -- --help`
- `cargo run -p igy6-worker -- --check`
- `cargo run -p igy6-worker -- --dry-run --once`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`

## Verification Results

- `git diff --check`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `python3 -m py_compile scripts/rust-worker-canary-fixture.py`: passed.
- `scripts/rust-worker-canary-fixture.py --emit-sql`: passed.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `cargo test -p igy6-worker`: passed.
- `cargo run -p igy6-worker -- --help`: passed.
- `cargo run -p igy6-worker -- --check`: passed; non-mutating check mode.
- `cargo run -p igy6-worker -- --dry-run --once`: passed; non-mutating
  one-job plan.
- `python3 scripts/rust-route-parity.py --check`: passed with
  `missing_from_rust=0` and `web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`:
  passed; `worker` and `beat` remain configured.
- `npm --prefix apps/web run build`: passed.
