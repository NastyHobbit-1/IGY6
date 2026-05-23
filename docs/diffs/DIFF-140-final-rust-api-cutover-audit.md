# DIFF-140: Final Rust API Cutover Audit

Status: Locked

## Type

Final runtime audit and documentation update.

## Objective

Record the current true post-cutover runtime state after DIFF-139.

Decision:

- The active API runtime path is Rust-native.
- FastAPI fallback was removed in DIFF-138.
- `services/api/` was archived in DIFF-139.
- API route parity is complete with zero FastAPI routes missing from Rust and
  zero web-used routes requiring fallback.
- Python/Celery `worker` and `beat` remain active runtime components.
- Full Rust-only repository or runtime operation is not claimed.

## Baseline Facts

- DIFF-139 is complete and locked.
- `infra/docker-compose.yml` no longer defines or wires `legacy-api`.
- `archive/legacy-python/services-api/` contains the archived former FastAPI
  API source.
- Route parity reports `missing_from_rust=0` and `web_requires_fallback=0`.
- Docker Compose still defines active `worker` and `beat` services built from
  `services/worker/`.
- `crates/igy6-worker` remains planning-only and does not replace live
  Python/Celery execution, database writes, audit writes, or Qdrant work.

## Allowed Scope

- `docs/diffs/DIFF-140-final-rust-api-cutover-audit.md`
- `configs/rust-cutover-manifest.json`
- `README.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/agents/DIFF-131_LOCAL_LLM_TASK_ROUTING.md`
- Other live documentation that incorrectly states FastAPI fallback is still
  required.

## Prohibited Scope

- No worker execution migration.
- No removal of `services/worker/`.
- No removal of `worker` or `beat` from Docker Compose.
- No full Rust-only repository or runtime claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, redesign, data model changes,
  migration changes, or dependency changes.

## Required Tags

Use `DIFF-140` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious runtime posture documentation.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`

## Completion Criteria

- Manifest and live documentation state that the active API path is Rust-native
  and FastAPI fallback is removed.
- Manifest and live documentation state that `services/api/` is archived.
- Manifest and live documentation state that Python/Celery `worker` and `beat`
  remain active because Rust worker execution parity is not implemented or
  verified.
- No full Rust-only repository or runtime operation is claimed.
- Required verification passes or blocked checks are recorded precisely.

## Completion Notes

DIFF-140 records the final Rust API cutover audit.

Current runtime posture:

- Active API runtime path: Rust-native gateway.
- Route parity: complete, with `missing_from_rust=0` and
  `web_requires_fallback=0`.
- FastAPI fallback: removed in DIFF-138.
- Legacy FastAPI source: archived at `archive/legacy-python/services-api/` in
  DIFF-139.
- Remaining Python runtime: `worker` and `beat` from `services/worker/`.
- Rust worker status: planning-only; it does not replace live worker execution,
  database writes, audit writes, or Qdrant work.
- Full Rust-only repository or runtime operation: not claimed.

Next recommended DIFF:

- DIFF-141 worker execution parity, or an explicit long-term Python worker
  retention decision.

## Verification Results

- `git status --short` inspected scoped DIFF-140 changes.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed `api`, `worker`, and `beat`, with no `legacy-api`.
- `npm --prefix apps/web run build` passed.
