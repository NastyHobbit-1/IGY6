# DIFF-142 Worker Execution Contract Queue-Claim Prompt

Use this prompt with Codex after DIFF-141 is complete and locked.

## Required First Read

- `AGENTS.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/diffs/DIFF-141-worker-execution-parity-audit.md`
- `configs/rust-cutover-manifest.json`
- `docs/runtime/PROCESSING_STATUS.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`

## Mission

Start DIFF-142 only.

Create the Rust worker execution contract and queue-claim foundation.

## Context

- DIFF-141 is complete and locked.
- Active API path is Rust-native.
- FastAPI fallback is removed.
- `services/api/` is archived.
- Python/Celery worker and beat remain active.
- Rust worker execution parity is not complete.
- Full Rust-only repo/runtime is not claimed.

## Primary Objective

Build the safe Rust foundation for claiming queued work items and executing worker jobs later, without migrating any job family in this DIFF.

## Target Scope

- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-142-worker-execution-contract-queue-claim-foundation.md`
- `docs/runtime/PROCESSING_STATUS.md` if stale
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` if stale
- `README.md` if stale

## Allowed

- Add Rust worker execution contract types.
- Add queue-claim planning/foundation logic.
- Add deterministic validation for claimable work item states.
- Add safe status transition planning for queued/running/failed/completed.
- Add audit event planning for worker claim/start/failure/completion.
- Add tests for claim eligibility, invalid status, unsupported work type, stale claim handling if modeled, and audit/status plan shape.
- Update manifest/docs to say DIFF-142 adds foundation only.
- Keep Python/Celery worker and beat active.

## Prohibited

- Do not remove `services/worker/`.
- Do not remove worker or beat from Docker Compose.
- Do not migrate `collection.normalize_collection_run` yet.
- Do not migrate `evidence.generate_document_chunks` yet.
- Do not migrate `memory.vector.upsert_chunks` yet.
- Do not perform live background execution yet unless already scoped as planning-only.
- Do not write runtime/private data under `IGY6_DATA_ROOT`.
- Do not mutate `.env`.
- Do not add cloud providers, credentials, secrets, or external model calls.
- Do not remove Python/Celery.
- Do not claim full Rust-only runtime.
- Do not edit locked DIFFs.
- Do not start DIFF-143.

## Required Worker Facts From DIFF-141

Python/Celery tasks currently found:

- `phase0.health`
- `collection.normalization_scaffold`
- `collection.normalize_collection_run`
- `evidence.generate_document_chunks`
- `memory.vector.upsert_chunks`

Beat/scheduled work:

- No repo-defined `beat_schedule`, `crontab`, periodic task, or scheduled task config was found.
- Beat remains active until a later DIFF explicitly retires or replaces scheduled-work support.

Rust already covers:

- Deterministic planning in `crates/igy6-worker`.
- Normalization/chunk/vector planning through Rust crates.
- Rust gateway work-item creation and dispatch metadata.
- Rust gateway bounded Qdrant vector upsert route.

Rust does not yet cover:

- Background queue claiming/execution.
- Live worker DB writes for queued jobs.
- Worker audit event parity.
- Artifact reads from queued work.
- Chained work-item creation during background execution.
- Qdrant upsert from queued background work with worker audit/failure semantics.
- Scheduler/beat replacement.

## Expected DIFF-142 Output

- A Rust worker execution contract/foundation.
- No actual migration of the three live job families yet.
- Python/Celery worker and beat remain required.
- Manifest records Rust worker execution parity is still incomplete.
- Next recommended DIFF is DIFF-143 collection_normalization execution parity.

## Suggested Contract Concepts

- `WorkItemExecutionPlan`
- `WorkItemClaimPlan`
- `WorkerAuditPlan`
- `WorkerStatusTransition`
- `ClaimEligibility`
- `UnsupportedWorkType` handling
- Claimable statuses: `queued` or equivalent current worker-ready state
- Non-claimable statuses: `running`, `completed`, `failed`, `canceled`, `pending_intent_verification` unless explicitly intended
- Supported future job types:
  - `collection_normalization`
  - `document_chunking`
  - `chunk_vector_upsert`
- Health/scaffold/no-op behavior may be modeled, but must not distract from queue claim foundation.

## Verification

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
- What worker execution foundation was added.
- What remains Python/Celery-backed.
- Whether worker/beat remain required.
- Verification results.
- Whether full Rust-only runtime is claimed.
- Next recommended DIFF.
