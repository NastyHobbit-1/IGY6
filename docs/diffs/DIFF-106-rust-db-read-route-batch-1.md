# DIFF-106: Rust DB Read Route Batch 1

Status: Locked

## Type

Change-bearing

## Objective

Move the first coherent batch of web-critical, read-only, DB-backed routes from
FastAPI fallback to Rust-native gateway handling. This DIFF reduces required
fallback by serving stable list/detail metadata routes directly from
PostgreSQL while leaving dangerous or system-changing writes on FastAPI
fallback.

## Baseline Facts

- DIFF-105 reports `fastapi=91`, `rust_native=7`, `web_used=41`,
  `missing_from_rust=85`, and `web_requires_fallback=36`.
- The Rust gateway currently has no PostgreSQL client.
- `infra/docker-compose.yml` already provides `DATABASE_URL` to `legacy-api`,
  `worker`, and `beat`, but not to Rust `api`.
- Web-critical read routes include sources, work-items, reports, and evidence
  list/detail endpoints.

## Allowed Scope

- Add a narrow synchronous PostgreSQL dependency to `crates/igy6-gateway`.
- Update `Cargo.lock`.
- Update `infra/docker-compose.yml` only to pass `DATABASE_URL` to the Rust
  gateway.
- Add read-only Rust DB route handling in `crates/igy6-gateway`.
- Update `scripts/rust-route-parity.py` only to recognize Rust native route
  declarations accurately.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust DB route
  patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No runtime/private data reads or writes.
- No `.env` content reads or writes.
- No archive actions.
- No deletion.
- No destructive migrations.
- No database schema changes.
- No write route migration except route declarations required for parity
  accounting.
- No approval bypass.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-106`.

## Verification

- `git status --short`
- `git diff --check`
- `python3 scripts/rust-route-parity.py --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

- Rust gateway serves the selected DB-backed read routes without proxying to
  FastAPI when `DATABASE_URL` is configured.
- Missing `DATABASE_URL` or DB connection/query failures return deterministic
  JSON errors rather than falling through to FastAPI for the selected routes.
- Route-level tests cover migrated routes and error behavior.
- Route parity counts are updated honestly and FastAPI fallback remains
  required while other routes still depend on it.
- DIFF-106 is locked after verification passes.

## Results

- Rust-native routes increased from 7 to 24.
- FastAPI routes missing from Rust decreased from 85 to 68.
- Web routes requiring FastAPI fallback decreased from 36 to 28.
- FastAPI fallback remains required.
- Migrated Rust-native DB reads:
  - `GET /sources`
  - `GET /sources/{source_id}`
  - `GET /sources/{source_id}/permissions`
  - `GET /approvals`
  - `GET /approvals/{approval_id}`
  - `GET /work-items`
  - `GET /work-items/{work_item_id}`
  - `GET /reports`
  - `GET /reports/{report_id}`
  - `GET /evidence/documents`
  - `GET /evidence/documents/{document_id}`
  - `GET /evidence/items`
  - `GET /evidence/items/{evidence_item_id}`
  - `GET /evidence/chunks`
  - `GET /evidence/chunks/{chunk_id}`
  - `GET /evidence/claims`
  - `GET /evidence/claims/{claim_id}`

## Verification Results

- `git status --short` checked DIFF-106 scoped files only.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=24 web_used=41 missing_from_rust=68 web_requires_fallback=28`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 13 tests passed.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `python3 -m unittest discover services/api/tests` failed in the host Python
  environment because `fastapi` and `httpx` were not installed there.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests passed.

## Out Of Scope Follow-Up

- Source, approval, report, work-item, settings, and collection write route
  parity.
- Full FastAPI retirement.
- Rust worker replacement for Celery execution.
