# DIFF-144 Document Chunking Execution Parity Prompt

Use this prompt with Codex after DIFF-143 is complete, locked, committed, and pushed.

## Mission

Start DIFF-144 only.

Implement Rust `document_chunking` execution parity as the next Rust worker job family.

## Context

- DIFF-142 added Rust worker execution contract and queue-claim foundation.
- DIFF-143 migrated `collection_normalization` planning/executor contracts.
- Python/Celery worker and beat remain active.
- Full Rust-only runtime is not claimed.
- Do not start DIFF-145.

## Target Job Family

- `evidence.generate_document_chunks`
- `document_chunking`

## Primary Objective

Migrate document chunking worker execution behavior to Rust while preserving Python/Celery worker semantics.

## Required First Reads

- `AGENTS.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/diffs/DIFF-141-worker-execution-parity-audit.md`
- `docs/diffs/DIFF-142-worker-execution-contract-queue-claim.md`
- `docs/diffs/DIFF-143-collection-normalization-execution-parity.md`
- `configs/rust-cutover-manifest.json`
- `crates/igy6-worker/`
- `crates/igy6-chunking/`
- `services/worker/`
- `docs/runtime/PROCESSING_STATUS.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`

## Allowed

- Add Rust execution logic for `document_chunking` only.
- Read normalized document data needed for chunk generation.
- Generate chunks using existing Rust chunking behavior.
- Create `chunks` records matching existing semantics.
- Create `evidence_items` records matching existing semantics if Python worker does so.
- Update originating `work_items` status/error fields for this job family.
- Write worker audit events matching Python/Celery behavior.
- Create chained `chunk_vector_upsert` work items when appropriate.
- Add tests for success, missing document, invalid payload, empty/oversized document behavior, chunk/evidence insert plan shape, chained work-item creation, status transitions, audit shape, and failure behavior.
- Update docs and manifest honestly.

## Prohibited

- Do not migrate `chunk_vector_upsert` yet.
- Do not perform Qdrant upsert work in this DIFF.
- Do not remove `services/worker/`.
- Do not remove worker or beat from Docker Compose.
- Do not remove Python/Celery yet.
- Do not claim full Rust-only runtime.
- Do not mutate `.env`.
- Do not touch runtime/private data except through scoped tested worker behavior.
- Do not add external model calls, secrets, credentials, or cloud providers.
- Do not edit locked DIFFs.
- Do not start DIFF-145.

## Preserve These Python Worker Behaviors

- Reads the normalized document for the work item.
- Splits document text into deterministic chunks.
- Writes `chunks` records.
- Writes `evidence_items` for generated chunks if existing Python behavior does this.
- Updates the originating `work_item`.
- Creates chained `chunk_vector_upsert` work items when chunks are created.
- Writes audit events.
- Handles missing/invalid inputs deterministically.
- Keeps bounded input behavior.
- Does not silently skip failed records unless existing Python behavior explicitly does.

## Important Constraint

If full live DB execution is too broad for this DIFF, implement a clearly separated execution planner plus a DB executor for only `document_chunking`, with tests proving SQL/audit/status/chained-work plan shape. Do not fake-complete parity without documenting exactly what remains.

## Expected DIFF-144 Output

- `docs/diffs/DIFF-144-document-chunking-execution-parity.md`
- Rust worker `document_chunking` execution or narrowly scoped executor foundation
- Manifest update showing `document_chunking` parity status
- Python/Celery worker and beat still active unless explicitly proven safe otherwise
- Next recommended DIFF: DIFF-145 chunk_vector_upsert execution parity

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
- Exact `document_chunking` behavior migrated.
- What remains Python/Celery-backed.
- Whether worker/beat remain required.
- Verification results.
- Whether full Rust-only runtime is claimed.
- Next recommended DIFF.
