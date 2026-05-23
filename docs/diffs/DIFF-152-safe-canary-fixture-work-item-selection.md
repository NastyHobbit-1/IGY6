# DIFF-152: Safe Canary Fixture Work Item Selection

Status: Locked

## Type

Fixture and documentation update.

## Objective

Create a deterministic, synthetic, non-production canary fixture path and
explicitly select one future Rust worker canary work item.

## Decision

Decision A: safe canary fixture can be created.

DIFF-152 adds `scripts/rust-worker-canary-fixture.py`. The helper is
non-mutating by default and emits the selected canary metadata. When explicitly
requested, it emits deterministic seed SQL and can write one synthetic artifact
under a canary/diff152 test data root.

## Selected Canary

- Work item ID: `diff-152-canary-work-item`
- Work type: `collection_normalization`
- Source ID: `diff-152-canary-source`
- Collection run ID: `diff-152-canary-run`
- Raw artifact ID: `diff-152-canary-raw`
- Artifact content hash:
  `26157f4935b30dbed9c3801007fea903db0c7192cc60422602703a5f22320256`
- Artifact storage path:
  `sha256/26/15/26157f4935b30dbed9c3801007fea903db0c7192cc60422602703a5f22320256`

## Selection Method

The selected `work_item_id` is deterministic and comes from the fixture helper:

```bash
scripts/rust-worker-canary-fixture.py --emit-sql
```

The helper emits one `collection_normalization` work item with a payload
containing exactly one raw artifact ID. It does not run Rust worker execution.

## Future Canary Command

The future DIFF that performs live verification should run exactly one selected
canary after applying the fixture to a safe local test stack:

```bash
IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item diff-152-canary-work-item
```

DIFF-152 does not run this command.

## Safety

- Fixture data is synthetic and non-production.
- The helper default mode prints selection metadata only.
- Seed SQL is emitted only with `--emit-sql`.
- Artifact writing requires `--write-artifact` and a canary/diff152 data root.
- No production/private runtime data is read or written by DIFF-152.
- No broad queue processing is performed.
- Python/Celery `worker` remains active.
- Python/Celery `beat` remains active.
- Docker Compose is unchanged.
- Full Rust-only runtime is not claimed.

## Remaining Live Verification

The following side effects still require a later controlled live canary:

- PostgreSQL claim and `work_items` status update for
  `diff-152-canary-work-item`.
- `audit_events` claim/start/success/failure rows for the selected canary.
- Artifact bytes read under `IGY6_DATA_ROOT/artifacts`.
- `normalized_documents` writes for the selected `collection_normalization`
  canary.
- Chained `document_chunking` work item creation after normalization.
- Failure rollback posture if a later safe failing canary is introduced.

Qdrant side effects are not expected from this selected fixture because it is a
`collection_normalization` canary. Qdrant remains unverified for a future
`chunk_vector_upsert` canary.

## Next Recommended DIFF

DIFF-153 controlled live canary run against the DIFF-152 selected fixture.

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

`npm --prefix apps/web run build` is not required unless UI/status source text
changes.

## Verification Results

- `git status --short`: expected DIFF-152 files only, with generated `target/`
  removed before final status.
- `git diff --check`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `python3 -m py_compile scripts/rust-worker-canary-fixture.py`: passed.
- `scripts/rust-worker-canary-fixture.py --emit-sql`: passed and emitted
  `selected_work_item_id=diff-152-canary-work-item`.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `cargo test -p igy6-worker`: passed.
- `cargo run -p igy6-worker -- --help`: passed.
- `cargo run -p igy6-worker -- --check`: passed; non-mutating check mode.
- `cargo run -p igy6-worker -- --dry-run --once`: passed; non-mutating one-job
  plan.
- `python3 scripts/rust-route-parity.py --check`: passed with
  `missing_from_rust=0` and `web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`:
  passed; `worker` and `beat` remain configured.
- `npm --prefix apps/web run build`: passed.
