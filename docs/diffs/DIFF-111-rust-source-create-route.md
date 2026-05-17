# DIFF-111: Rust Source Create Route

Status: Locked

## Type

Change-bearing

## Objective

Move the next safest web-used write route to Rust-native gateway handling:
`POST /sources`. The Rust gateway should create source metadata, optionally
create an initial source permission, and write the deterministic
`source.created` audit event without starting collectors or changing the
FastAPI fallback topology.

## Baseline Facts

- DIFF-110 reports `fastapi=91`, `rust_native=48`, `web_used=41`,
  `missing_from_rust=44`, and `web_requires_fallback=12`.
- `POST /sources` is used by the web UI to register source metadata and an
  optional permission scope.
- The Python route validates source type, sensitivity, allowed permission
  operations, external model policy, object-shaped metadata/scope fields, and
  writes a `source.created` audit event.
- `POST /sources` does not run collection, execute agent actions, read `.env`
  contents, or call external services.

## Allowed Scope

- Add Rust-native DB-backed handling for `POST /sources`.
- Preserve existing request/response shapes where practical, including nested
  source permission response data.
- Validate request bodies, missing/empty IDs, source type, sensitivity,
  metadata object shape, permission scope object shape, allowed operations,
  external model policy, boolean fields, and actor IDs.
- Insert the source row, optional source permission row, and `source.created`
  audit event in one transaction.
- Update route-level tests for validation, missing DB behavior, and registry
  coverage.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust source
  create/audit patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No source permission create route migration.
- No collection, dry-run, manual-upload, settings, report, work-item,
  analysis, approval decision, or agent execution route migration.
- No arbitrary shell execution.
- No approval bypass.
- No `.env` content reads or writes.
- No runtime/private data commits.
- No database schema changes or migrations.
- No external service calls.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-111`.

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

- `POST /sources` is Rust-native and no longer proxied to FastAPI.
- Invalid source create requests are rejected by Rust without fallback.
- Missing `DATABASE_URL` returns deterministic Rust DB-route errors.
- Successful source creates insert the source row, optional permission row, and
  `source.created` audit event in one transaction.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-111 is locked after verification passes.

## Results

- Migrated `POST /sources` to Rust-native DB-backed handling.
- Added Rust validation for source name, source type, sensitivity, owner actor,
  trust level, enabled flag, metadata shape, optional permission shape,
  permission scope shape, allowed operations, external model policy,
  approval-required flag, and created-by actor.
- Preserved Python response shape where practical by returning a
  `SourceRead`-compatible JSON object with nested permissions.
- Preserved audit behavior by inserting a deterministic `source.created` audit
  event in the same transaction as the source and optional permission inserts.
- Route parity changed from `rust_native=48`, `missing_from_rust=44`, and
  `web_requires_fallback=12` to `rust_native=49`,
  `missing_from_rust=43`, and `web_requires_fallback=11`.
- FastAPI fallback remains required for the remaining write/action routes.
- No agent execution routes were migrated.

## Verification Results

- `git status --short` checked DIFF-111 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=49 web_used=41 missing_from_rust=43
  web_requires_fallback=11`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 23 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Source permission create route.
- Collection dry-run and manual upload writes.
- Report, work-item, settings, approval decision, analysis, and agent execution
  routes.
- Full FastAPI retirement.
