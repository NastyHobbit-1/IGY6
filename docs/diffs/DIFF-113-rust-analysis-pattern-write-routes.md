# DIFF-113: Rust Analysis Pattern Write Routes

Status: Locked

## Type

Change-bearing

## Objective

Move the next safest web-used analysis write routes to Rust-native gateway
handling: `POST /analysis/patterns` and
`POST /analysis/patterns/detect-baseline`. The Rust gateway should preserve
local evidence validation, deterministic baseline pattern detection, pattern
row creation, and `analysis.pattern.created` audit events without executing
agents, collectors, settings writes, or external services.

## Baseline Facts

- DIFF-112 reports `fastapi=91`, `rust_native=50`, `web_used=41`,
  `missing_from_rust=42`, and `web_requires_fallback=9`.
- `POST /analysis/patterns` creates a pattern after validating referenced
  evidence item IDs.
- `POST /analysis/patterns/detect-baseline` reads local evidence items,
  computes deterministic recurrence/cross-source/missing-information
  candidates, skips detector keys already present, and creates candidate
  pattern rows with audit events.
- These routes do not call external services, execute shell commands, read
  `.env` contents, or mutate runtime/private data outside PostgreSQL state.

## Allowed Scope

- Add Rust-native DB-backed handling for:
  - `POST /analysis/patterns`
  - `POST /analysis/patterns/detect-baseline`
- Preserve existing request/response shapes where practical.
- Validate request bodies, evidence IDs, confidence bounds, status/actor
  fields, metadata object shape, and recurrence threshold bounds.
- Validate referenced evidence items for explicit pattern creation.
- Preserve deterministic baseline pattern detection and duplicate detector key
  suppression.
- Insert pattern rows and `analysis.pattern.created` audit events in
  transactions.
- Update route-level tests for validation, missing DB behavior, and registry
  coverage.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust pattern
  write/detection/audit patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No pattern review route migration.
- No hypothesis, prediction, or recommendation write route migration.
- No collection, settings, report render, work-item dispatch/status, approval
  decision, source permission, or agent execution route migration.
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

Commit messages and final summaries must include `DIFF-113`.

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

- `POST /analysis/patterns` and `POST /analysis/patterns/detect-baseline` are
  Rust-native and no longer proxied to FastAPI.
- Invalid pattern requests are rejected by Rust without fallback.
- Missing `DATABASE_URL` returns deterministic Rust DB-route errors.
- Successful pattern writes insert pattern rows and audit events in one
  transaction.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-113 is locked after verification passes.

## Results

- Migrated `POST /analysis/patterns` to Rust-native DB-backed handling.
- Migrated `POST /analysis/patterns/detect-baseline` to Rust-native DB-backed
  handling.
- Added Rust validation for pattern type, summary, evidence IDs, confidence,
  status, actor, metadata object shape, and recurrence threshold bounds.
- Preserved explicit pattern evidence validation against `evidence_items`.
- Preserved deterministic baseline detection for missing-information,
  recurrence by evidence type, and cross-source normalized statements.
- Preserved duplicate detector-key suppression for baseline detection.
- Preserved audit behavior by inserting `analysis.pattern.created` audit events
  in the same transaction as pattern inserts.
- Route parity changed from `rust_native=50`, `missing_from_rust=42`, and
  `web_requires_fallback=9` to `rust_native=52`,
  `missing_from_rust=40`, and `web_requires_fallback=7`.
- FastAPI fallback remains required for the remaining write/action routes.
- No pattern review, settings, collection, work dispatch, or agent execution
  routes were migrated.

## Verification Results

- `git status --short` checked DIFF-113 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=52 web_used=41 missing_from_rust=40
  web_requires_fallback=7`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 29 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Pattern review and other analysis writes.
- Collection, settings, report render, work-item dispatch/status, approval
  decision, source permission, and agent execution routes.
- Full FastAPI retirement.
