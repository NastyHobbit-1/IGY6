# DIFF-151: Controlled Rust Worker Canary Observed Side-Effect Audit

Status: Locked

## Type

Facts-and-documentation canary audit decision.

## Objective

Run or prepare exactly one controlled Rust worker live canary against one
explicitly selected safe work item, then document observed side effects.

## Decision

Decision B: safe canary not available.

No live Rust worker canary was run in DIFF-151.

## Blocker

No explicitly selected safe queued work item ID was provided. DIFF-151 prohibits
touching runtime/private data except through the explicitly scoped canary, and
it prohibits broad queue processing or processing more than one work item.

Because no safe `work_item_id` was available, this DIFF did not run:

```bash
IGY6_WORKER_LIVE_CANARY=DIFF-148 cargo run -p igy6-worker -- --once --canary-live --canary-work-item <work_item_id>
```

## Observed Side Effects

None. No live canary was run.

## Still Unverified

- PostgreSQL claim and `work_items` status update for one selected canary.
- `audit_events` claim/start/success/failure rows for one selected canary.
- Artifact bytes read under `IGY6_DATA_ROOT/artifacts` for a selected
  `collection_normalization` canary.
- `normalized_documents` writes for a selected `collection_normalization`
  canary.
- `chunks` and `evidence_items` writes for a selected `document_chunking`
  canary.
- Qdrant collection ensure and point upsert for a selected
  `chunk_vector_upsert` canary.
- Chunk `embedding_status` and metadata updates after a selected Qdrant canary.
- Failure rollback posture from an intentionally safe failing canary.

## Preparation Required

Before a live canary can be run, a later DIFF must:

1. Start from an approved local stack with PostgreSQL and Qdrant running.
2. Create or select one non-sensitive test source and one test raw artifact
   under `IGY6_DATA_ROOT/artifacts`.
3. Create exactly one queued `work_items` row for `collection_normalization`,
   `document_chunking`, or `chunk_vector_upsert` with recorded intent
   verification.
4. Confirm Python/Celery worker will not race the selected canary item during
   the Rust canary window.
5. Record the selected `work_item_id`.
6. Run exactly one gated Rust canary command.
7. Query only the selected work item, correlated `audit_events`, expected
   downstream DB rows, and expected Qdrant points.
8. Document before/after state, observed side effects, and rollback posture.

## Runtime Posture

Python/Celery `worker` remains required. Python/Celery `beat` remains required.
Docker Compose is unchanged. `services/worker/` is retained. Full Rust-only
runtime is not claimed.

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery worker and beat services. Rust-only is not claimed.

## Next Recommended DIFF

DIFF-152 safe canary fixture creation and explicit work item selection.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
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

- `git diff --check`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `cargo test -p igy6-worker`: passed.
- `cargo run -p igy6-worker -- --help`: passed.
- `cargo run -p igy6-worker -- --check`: passed; reported non-mutating check
  mode with Python/Celery worker and beat still required.
- `cargo run -p igy6-worker -- --dry-run --once`: passed; planned one
  non-mutating worker cycle without runtime mutation.
- `python3 scripts/rust-route-parity.py --check`: passed with
  `missing_from_rust=0` and `web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`:
  passed; `worker` and `beat` remain configured.
- `npm --prefix apps/web run build`: passed.
