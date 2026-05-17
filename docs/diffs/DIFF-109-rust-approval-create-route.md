# DIFF-109: Rust Approval Create Route

Status: Locked

## Type

Change-bearing

## Objective

Move the safest write fallback route to Rust-native gateway handling by
implementing DB-backed `POST /approvals` approval request creation with an
audit event. This reduces fallback dependency without migrating approval
decisions, agent execution, settings apply/verify, collection writes, or other
system-changing behavior.

## Baseline Facts

- DIFF-108 reports `fastapi=91`, `rust_native=45`, `web_used=41`,
  `missing_from_rust=47`, and `web_requires_fallback=16`.
- `POST /approvals` is used by the web UI to create pending approval records.
- Python FastAPI creates an approval row and an `approval.requested` audit
  event for this route.
- Approval decisions and action execution remain more sensitive than approval
  request creation and require separate parity work.

## Allowed Scope

- Add Rust-native DB-backed handling for `POST /approvals`.
- Preserve the approval response shape where practical.
- Validate required approval request fields.
- Insert a deterministic audit event structure for approval creation.
- Keep missing DB configuration and DB errors deterministic.
- Add route-level tests for validation, missing DB, and Rust-native routing.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for the reusable Python-to-Rust approval
  write/audit route pattern.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No approval decision migration.
- No agent action execution migration.
- No settings/env verify or apply migration.
- No source, collection, feedback, outcome, report, work-item, or analysis
  write route migration.
- No arbitrary shell execution.
- No approval bypass.
- No `.env` content reads or writes.
- No runtime/private data commits.
- No database schema changes or migrations.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-109`.

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

- `POST /approvals` is Rust-native and no longer proxied to FastAPI.
- Rust validation rejects malformed approval-create requests without fallback.
- Rust DB handling creates an approval row and matching audit event when
  `DATABASE_URL` is configured.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-109 is locked after verification passes.

## Results

- Migrated `POST /approvals` to Rust-native DB-backed handling.
- Added JSON validation for approval request creation without fallback.
- Added a PostgreSQL transaction that inserts the pending approval row and the
  matching `approval.requested` audit event.
- Route parity changed from `rust_native=45`, `missing_from_rust=47`, and
  `web_requires_fallback=16` to `rust_native=46`, `missing_from_rust=46`, and
  `web_requires_fallback=14`.
- FastAPI fallback remains required for approval decisions and remaining
  write/action routes.
- No agent execution routes were migrated.

## Verification Results

- `git status --short` checked DIFF-109 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=46 web_used=41 missing_from_rust=46
  web_requires_fallback=14`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 18 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Approval decisions.
- Agent action execution.
- Settings/env verify and apply.
- Source, collection, feedback, outcome, report, work-item, and analysis
  writes.
- Full FastAPI retirement.
