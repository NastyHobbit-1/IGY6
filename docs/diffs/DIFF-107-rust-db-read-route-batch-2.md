# DIFF-107: Rust DB Read Route Batch 2

Status: Locked

## Type

Change-bearing

## Objective

Move the next coherent batch of web-critical, read-only, DB-backed routes from
FastAPI fallback to Rust-native gateway handling. This DIFF targets simple
PostgreSQL list/detail reads for audit, artifacts, collection runs, feedback,
outcomes, and analysis records while leaving writes and external-service-backed
routes on FastAPI fallback.

## Baseline Facts

- DIFF-106 reports `fastapi=91`, `rust_native=24`, `web_used=41`,
  `missing_from_rust=68`, and `web_requires_fallback=28`.
- FastAPI fallback remains required after DIFF-106.
- The Rust gateway now has a PostgreSQL client and handles the first DB read
  batch without proxying.
- Remaining web-critical read fallbacks include analysis lists, artifacts,
  audit events, collection runs, feedback, and outcomes.

## Allowed Scope

- Add read-only Rust DB route handling in `crates/igy6-gateway` for:
  - audit events
  - raw artifacts
  - collection runs
  - feedback events
  - outcomes
  - analysis patterns, hypotheses, predictions, and recommendations
- Update route-level tests for the new Rust-native read batch.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust DB read route
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
- No write route migration.
- No approval bypass.
- No FastAPI removal or disabling.
- No settings/env route migration.
- No Qdrant, Neo4j, or artifact file reads.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-107`.

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
- Run existing API/web tests where available.

## Completion Criteria

- Rust gateway serves the selected DB-backed read routes without proxying to
  FastAPI when `DATABASE_URL` is configured.
- Missing `DATABASE_URL` or DB connection/query failures return deterministic
  JSON errors rather than falling through to FastAPI for the selected routes.
- Route-level tests cover migrated routes and error behavior.
- Route parity counts are updated honestly and FastAPI fallback remains
  required while other routes still depend on it.
- DIFF-107 is locked after verification passes.

## Results

- Rust-native routes increased from 24 to 42.
- FastAPI routes missing from Rust decreased from 68 to 50.
- Web routes requiring FastAPI fallback decreased from 28 to 19.
- FastAPI fallback remains required.
- Migrated Rust-native DB reads:
  - `GET /analysis/patterns`
  - `GET /analysis/patterns/{pattern_id}`
  - `GET /analysis/hypotheses`
  - `GET /analysis/hypotheses/{hypothesis_id}`
  - `GET /analysis/predictions`
  - `GET /analysis/predictions/{prediction_id}`
  - `GET /analysis/recommendations`
  - `GET /analysis/recommendations/{recommendation_id}`
  - `GET /artifacts`
  - `GET /artifacts/{artifact_id}`
  - `GET /audit-events`
  - `GET /audit-events/{audit_event_id}`
  - `GET /collection-runs`
  - `GET /collection-runs/{collection_run_id}`
  - `GET /feedback`
  - `GET /feedback/{feedback_id}`
  - `GET /outcomes`
  - `GET /outcomes/{outcome_id}`

## Verification Results

- `git status --short` checked DIFF-107 scoped files only.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=42 web_used=41 missing_from_rust=50 web_requires_fallback=19`.
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
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests passed.

## Out Of Scope Follow-Up

- Settings/env route parity.
- Agent action execution parity.
- Source, approval, report, feedback, outcome, collection, and analysis write
  parity.
- Qdrant, Neo4j, and retrieval hydration route parity.
- Full FastAPI retirement.
