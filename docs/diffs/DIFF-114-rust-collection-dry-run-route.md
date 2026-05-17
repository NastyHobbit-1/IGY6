# DIFF-114: Rust Collection Dry-Run Route

Status: Locked

## Type

Change-bearing

## Objective

Move the next safest web-used collection write route to Rust-native gateway
handling: `POST /collection-runs/dry-run`. The Rust gateway should validate
source and permission metadata, create a dry-run collection run, and write
deterministic audit events without collecting artifacts, normalizing content,
dispatching workers, reading `.env`, or calling external services.

## Baseline Facts

- DIFF-113 reports `fastapi=91`, `rust_native=52`, `web_used=41`,
  `missing_from_rust=40`, and `web_requires_fallback=7`.
- `POST /collection-runs/dry-run` is used by the web UI to preview whether a
  source/permission pair can be collected.
- The Python route validates source existence/enabled state, permission
  existence/ownership, dry-run or read permission, scaffold connector support,
  and writes `collection_run.created` and `collection_run.dry_run_preview`
  audit events.
- The route is preview-only: it does not create artifacts, normalize content,
  enqueue workers, execute agents, or call external services.

## Allowed Scope

- Add Rust-native DB-backed handling for `POST /collection-runs/dry-run`.
- Preserve existing request/response shapes where practical.
- Validate request bodies, source IDs, permission IDs, actor IDs, and notes
  object shape.
- Validate source/permission existence, enabled state, ownership, and allowed
  operations.
- Preserve scaffold connector dry-run behavior for `manual_upload` and
  `local_project` source types and failed-preview behavior for unsupported
  source types.
- Insert collection run and audit events in one transaction.
- Update route-level tests for validation, missing DB behavior, registry
  coverage, forbidden, conflict, and not-found behavior.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust collection
  dry-run/audit patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No manual upload collection or ingest route migration.
- No artifact writes.
- No normalization or worker dispatch.
- No settings, report render, work-item dispatch/status, approval decision,
  source permission, pattern review, or agent execution route migration.
- No arbitrary shell execution.
- No approval bypass.
- No `.env` content reads or writes.
- No runtime/private data commits.
- No database schema changes or migrations.
- No external service calls.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-114`.

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

- `POST /collection-runs/dry-run` is Rust-native and no longer proxied to
  FastAPI.
- Invalid dry-run requests are rejected by Rust without fallback.
- Missing `DATABASE_URL` returns deterministic Rust DB-route errors.
- Successful and failed previews insert collection run rows and audit events in
  one transaction.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-114 is locked after verification passes.

## Results

- Added Rust-native handling for `POST /collection-runs/dry-run` in
  `crates/igy6-gateway`.
- Preserved the Python request shape for `source_id`, `source_permission_id`,
  `requested_by_actor_id`, and `notes`.
- Preserved preview-only semantics: no artifact writes, no normalization, no
  worker dispatch, no `.env` reads, no external service calls, and no agent
  execution.
- Added source and permission validation for missing source, disabled source,
  missing permission, source/permission mismatch, and missing `dry_run`/`read`
  permission.
- Added scaffold connector dry-run parity for `manual_upload` and
  `local_project`; unsupported source types persist a failed preview rather
  than collecting.
- Added deterministic `collection_run.created` and
  `collection_run.dry_run_preview` audit event insertion in the same
  transaction as the collection run row.
- Updated route parity counts to `rust_native=53`,
  `fastapi_routes_missing_from_rust=39`, and
  `web_routes_requiring_fallback=6`.
- FastAPI remains required for remaining web-used fallbacks:
  `POST /agent/actions/`,
  `POST /agent/actions/${encodeURIComponent(actionName)}/execute`,
  `POST /collection-runs/manual-upload`, `POST /settings/env/apply`,
  `POST /settings/env/verify`, and `POST /work-items/`.

## Verification Results

- `git status --short` checked DIFF-114 scoped changes.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=53 web_used=41 missing_from_rust=39 web_requires_fallback=6`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 34 tests.
- `scripts/rust-cutover.sh --check` passed and ran the route parity guard.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- Existing legacy API tests passed:
  `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  ran 8 tests successfully.

## Out Of Scope Follow-Up

- Manual upload collection/ingest, report render, work dispatch, settings
  verify/apply, approval decision, source permission, pattern review, and agent
  execution routes.
- Full FastAPI retirement.
