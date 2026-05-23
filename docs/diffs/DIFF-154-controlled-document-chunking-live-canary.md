# DIFF-154: Controlled Document Chunking Live Canary

Status: Locked

## Type

Controlled live canary attempt and observed side-effect audit.

## Objective

Run exactly one controlled Rust worker live canary for `document_chunking` using
isolated synthetic data, then audit observed side effects.

## Decision

Decision B: the `document_chunking` canary could not be completed successfully.

Exactly one live canary was attempted. It failed safely because the synthetic
fixture used `chunk_size=80`, while the Rust document chunking contract requires
chunk size between 100 and 5000. DIFF-154 does not run a second canary.

## Fixture Command

The fixture was prepared with:

```bash
scripts/rust-worker-canary-fixture.py --fixture document_chunking --emit-schema-sql --emit-sql --emit-observation-sql --write-sql /tmp/igy6-diff154-canary.sql
```

The fixture was applied to an isolated local PostgreSQL container:

```bash
docker run -d --rm --name igy6-diff154-postgres -e POSTGRES_USER=igy6 -e POSTGRES_PASSWORD=diff154-local -e POSTGRES_DB=igy6_canary -p 127.0.0.1:55433:5432 postgres:16
docker cp /tmp/igy6-diff154-canary.sql igy6-diff154-postgres:/tmp/igy6-diff154-canary.sql
docker exec igy6-diff154-postgres psql -U igy6 -d igy6_canary -f /tmp/igy6-diff154-canary.sql
```

The container was stopped after observation:

```bash
docker stop igy6-diff154-postgres
```

## Canary Command

Exactly one live Rust canary was run:

```bash
env DATABASE_URL=postgresql://igy6:diff154-local@127.0.0.1:55433/igy6_canary IGY6_DATA_ROOT=/tmp/igy6-diff154-canary QDRANT_URL=http://127.0.0.1:6333 IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item diff-154-canary-work-item
```

## Selected Work Item

- Work item ID: `diff-154-canary-work-item`
- Work type: `document_chunking`
- Document ID: `diff-154-canary-document`
- Source ID: `diff-154-canary-source`
- Fixture chunk size used during the attempted canary: `80`
- Corrected fixture chunk size after audit: `100`

## Canary Result

Rust reported:

- `result_state`: `failed`
- `status`: `canary_failed`
- `error_message`: `chunk size must be between 100 and 5000, got 80`
- `side_effects_executed`:
  - `postgres_work_item_claim`
  - `audit_work_item_claimed`
  - `audit_work_item_started`
  - `postgres_work_item_failed`
  - `audit_worker_failure`

## Observed PostgreSQL Effects

Read-only observation SQL showed:

- `work_items`: `diff-154-canary-work-item` is `failed` with error
  `chunk size must be between 100 and 5000, got 80`.
- `audit_events` rows:
  - `rust_worker_canary_fixture.selected`
  - `work_item.claimed`
  - `work_item.started`
  - `document_chunks.failed`
- `chunks`: no rows written.
- `evidence_items`: no rows written.
- Chained `chunk_vector_upsert` work item: no row written.
- `normalized_documents`: the synthetic document fixture remained available
  with text length `261`.

## Qdrant

No Qdrant side effects were expected or run. DIFF-154 is scoped to
`document_chunking`; `chunk_vector_upsert` and Qdrant are explicitly prohibited
in this DIFF.

## Fixture Correction

After observing the failure, `scripts/rust-worker-canary-fixture.py` was updated
so `--fixture document_chunking` emits `chunk_size=100`, which satisfies the
Rust chunking contract. DIFF-154 does not rerun the live canary because it is
limited to one canary work item.

## Still Unverified

- Successful live `document_chunking` chunk writes.
- Successful live `document_chunking` evidence item writes.
- Successful chained `chunk_vector_upsert` work item creation.
- Live `chunk_vector_upsert` Qdrant collection ensure and point upsert.
- Long-running Rust worker process ownership.
- Docker Compose Rust worker replacement.
- Python/Celery beat replacement or retirement.

## Runtime Posture

Python/Celery `worker` remains required. Python/Celery `beat` remains required.
Docker Compose is unchanged. `services/worker/` is retained. Full Rust-only
runtime is not claimed.

## Next Recommended DIFF

DIFF-155 corrected `document_chunking` live canary verification.

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
- `scripts/rust-worker-canary-fixture.py --emit-sql`: passed and preserved the
  default DIFF-152 collection-normalization fixture output.
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
