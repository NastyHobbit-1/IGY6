# DIFF-143 Collection Normalization Execution Parity Prompt

Use this prompt with Codex after DIFF-142 is complete and locked.

## Mission

Start DIFF-143 only.

Implement Rust `collection_normalization` execution parity as the first real Rust worker job family.

## Context

- DIFF-142 is complete and locked.
- DIFF-142 added Rust worker execution contract and queue-claim foundation only.
- Python/Celery worker and beat remain active.
- Full Rust-only runtime is not claimed.
- Do not start DIFF-144.

## Target Job Family

- `collection.normalize_collection_run`
- `collection_normalization`

## Primary Objective

Migrate collection normalization execution behavior to Rust while preserving Python worker semantics.

## Required First Reads

- `AGENTS.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/diffs/DIFF-141-worker-execution-parity-audit.md`
- `docs/diffs/DIFF-142-worker-execution-contract-queue-claim.md`
- `configs/rust-cutover-manifest.json`
- `crates/igy6-worker/`
- `services/worker/`
- `docs/runtime/PROCESSING_STATUS.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`

## Allowed

- Add Rust execution logic for `collection_normalization` only.
- Read raw artifact metadata/content needed for collection normalization.
- Create `normalized_documents` records matching existing semantics.
- Update `work_items` status/error fields for this job family.
- Write worker audit events matching Python/Celery behavior.
- Create chained `document_chunking` work items if Python behavior does this.
- Add tests for success, missing artifact, invalid payload, DB failure planning/handling, audit shape, chained work-item creation, and status transitions.
- Update docs and manifest honestly.

## Prohibited

- Do not migrate `document_chunking` yet.
- Do not migrate `chunk_vector_upsert` yet.
- Do not remove `services/worker/`.
- Do not remove worker or beat from Docker Compose.
- Do not remove Python/Celery yet.
- Do not claim full Rust-only runtime.
- Do not mutate `.env`.
- Do not touch runtime/private data except through scoped tested worker behavior.
- Do not add external model calls, secrets, credentials, or cloud providers.
- Do not edit locked DIFFs.
- Do not start DIFF-144.

## Preserve These Python Worker Behaviors

- Reads raw artifacts for a collection run.
- Writes `normalized_documents`.
- Updates the originating `work_item`.
- Creates chained `document_chunking` work items when appropriate.
- Writes audit events.
- Handles missing/invalid inputs deterministically.
- Keeps bounded input behavior.
- Does not silently skip failed records unless existing Python behavior explicitly does.

## Important Constraint

If full live DB execution is too broad for this DIFF, implement a clearly separated execution planner plus a DB executor for only `collection_normalization`, with tests proving SQL/audit/status plan shape. Do not fake-complete parity without documenting exactly what remains.

## Expected DIFF-143 Output

- `docs/diffs/DIFF-143-collection-normalization-execution-parity.md`
- Rust worker `collection_normalization` execution or narrowly scoped executor foundation
- Manifest update showing `collection_normalization` parity status
- Python/Celery worker and beat still active unless explicitly proven safe otherwise
- Next recommended DIFF: DIFF-144 document_chunking execution parity

## Required Verification

Run:

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-worker`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build` if UI/status source text changed

## Final Response Must Include

- Active DIFF number.
- Files changed.
- Exact `collection_normalization` behavior migrated.
- What remains Python/Celery-backed.
- Whether worker/beat remain required.
- Verification results.
- Whether full Rust-only runtime is claimed.
- Next recommended DIFF.
