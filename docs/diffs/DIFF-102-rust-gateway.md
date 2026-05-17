# DIFF-102: Rust Gateway

Status: Locked

## Type

Change-bearing

## Objective

Make a Rust API gateway the primary local Compose `api` service while keeping
the existing FastAPI application available as `legacy-api` fallback. The gateway
must preserve local-first behavior, expose deterministic Rust-owned health and
migration routes, route safe Rust contract endpoints where parity exists, and
proxy unsupported routes to the FastAPI fallback instead of removing Python.

This DIFF uses a std-only Rust gateway because the current workspace has no
established Axum/Tokio dependency and the Rust migration rules prefer std-only
unless an existing workspace dependency is already established and justified.
An Axum implementation remains an out-of-scope follow-up unless dependency
approval is added in a later DIFF.

## Baseline Facts

- DIFF-101 is locked and marks `work_queue_reports` complete.
- `configs/rust-cutover-manifest.json` has `rust_gateway` pending and
  `cutover_ready` false.
- The Next.js UI calls `API_BASE_URL`, currently `http://api:8000` in Compose.
- The current Compose `api` service is FastAPI; this DIFF may split it into a
  Rust `api` gateway plus `legacy-api` FastAPI fallback.

## Allowed Scope

- Create `docs/diffs/DIFF-102-rust-gateway.md`.
- Add `crates/igy6-gateway/`.
- Update root `Cargo.toml` workspace membership.
- Update `Cargo.lock` as required.
- Update `infra/docker-compose.yml` to make Rust `api` primary and FastAPI
  fallback explicit.
- Update `configs/rust-cutover-manifest.json` only for the `rust_gateway`
  phase and safe final cutover readiness metadata if justified by verification.
- Update `docs/rust-migration/RUST_MIGRATION_PLAN.md` for accuracy.
- Add or update Rust equivalent snippet JSONL records under:
  - `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
  - `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- Locked DIFF edits.
- `.env` changes or `.env` content reads.
- Database migrations.
- Runtime/private data reads or writes.
- File deletion.
- Archive actions.
- Python/FastAPI removal.
- Celery removal.
- Disabling FastAPI fallback.
- Unapproved external dependency downloads.
- External service calls during tests.
- Arbitrary shell execution.
- Marking future phases complete.

Unless explicitly allowed here, the following are prohibited:

- Renames outside the explicit Compose service split.
- Refactors outside touched Rust gateway files.
- Behavior changes outside the gateway/fallback routing surface.
- Redesign.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

## Required Tags

Use `DIFF-102` in the commit message and final change summary.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-gateway`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

- `crates/igy6-gateway/` exists and is included in the workspace.
- The Rust gateway handles local health, migration status, agent capabilities,
  agent intent classification, retrieval preview, and evidence answer contract
  routes without external calls.
- Unsupported routes produce a deterministic local fallback proxy plan in tests
  and proxy to the configured FastAPI fallback at runtime.
- Compose names the Rust gateway as `api` and preserves FastAPI as
  `legacy-api`.
- Web UI continues to target `http://api:8000` in Compose.
- `configs/rust-cutover-manifest.json` marks `rust_gateway` complete only after
  verification passes.
- `cutover_ready` may be set true only if all required phases are complete and
  final cutover remains explicitly gated by a later final cutover DIFF.
- Python/FastAPI and Celery remain present.
- This DIFF is locked after verification passes.

## Out Of Scope Follow-Up

- Axum/Tokio rewrite.
- Direct PostgreSQL route implementation.
- Python archival or deletion.
- Final cutover archive execution.
- Runtime stack startup.
