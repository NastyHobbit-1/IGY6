# DIFF-112: Rust Report Create Route

Status: Locked

## Type

Change-bearing

## Objective

Move the next safest web-used write route to Rust-native gateway handling:
`POST /reports`. The Rust gateway should create report metadata and write the
deterministic `report.created` audit event without rendering artifacts,
dispatching work, executing agents, or changing the FastAPI fallback topology.

## Baseline Facts

- DIFF-111 reports `fastapi=91`, `rust_native=49`, `web_used=41`,
  `missing_from_rust=43`, and `web_requires_fallback=11`.
- `POST /reports` is used by the web UI to create report metadata.
- The Python route validates report type, report status, title length, metadata
  object shape, and writes a `report.created` audit event.
- `POST /reports` does not render files, dispatch Celery tasks, execute agent
  actions, read `.env` contents, or call external services.

## Allowed Scope

- Add Rust-native DB-backed handling for `POST /reports`.
- Preserve existing request/response shapes where practical.
- Validate request bodies, title, report type, status, requested actor,
  artifact path type, and metadata object shape.
- Insert the report row and `report.created` audit event in one transaction.
- Update route-level tests for validation, missing DB behavior, and registry
  coverage.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust report
  create/audit patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No report render, report status, or report work-item route migration.
- No work-item dispatch or status route migration.
- No collection, settings, analysis, approval decision, source permission, or
  agent execution route migration.
- No arbitrary shell execution.
- No approval bypass.
- No `.env` content reads or writes.
- No runtime/private data commits.
- No database schema changes or migrations.
- No artifact writes.
- No external service calls.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-112`.

## Verification

- `git status --short`
- `git diff --check`
- `python3 scripts/rust-route-parity.py --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-gateway`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Run existing API/web tests where available.

## Completion Criteria

- `POST /reports` is Rust-native and no longer proxied to FastAPI.
- Invalid report create requests are rejected by Rust without fallback.
- Missing `DATABASE_URL` returns deterministic Rust DB-route errors.
- Successful report creates insert the report row and `report.created` audit
  event in one transaction.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-112 is locked after verification passes.

## Results

- Migrated `POST /reports` to Rust-native DB-backed handling.
- Added Rust validation for title, report type, status, requested actor,
  artifact path type, and metadata object shape.
- Preserved Python response shape where practical by returning a
  `ReportRead`-compatible JSON object.
- Preserved audit behavior by inserting a deterministic `report.created` audit
  event in the same transaction as the report insert.
- Route parity changed from `rust_native=49`, `missing_from_rust=43`, and
  `web_requires_fallback=11` to `rust_native=50`,
  `missing_from_rust=42`, and `web_requires_fallback=9`.
- FastAPI fallback remains required for the remaining write/action routes.
- No report render, work dispatch, or agent execution routes were migrated.

## Verification Results

- `git status --short` checked DIFF-112 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=50 web_used=41 missing_from_rust=42
  web_requires_fallback=9`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 25 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Report rendering, report status, and report work-item routes.
- Work-item dispatch/status routes.
- Settings, collection, analysis, approval decision, source permission, and
  agent execution routes.
- Full FastAPI retirement.
