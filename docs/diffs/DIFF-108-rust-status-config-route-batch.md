# DIFF-108: Rust Status And Config Route Batch

Status: Locked

## Type

Change-bearing

## Objective

Move the safest remaining web fallback routes to Rust-native gateway handling:
read-only status/config routes that do not mutate state, do not read `.env`
contents, do not expose secrets, and do not execute actions.

## Baseline Facts

- DIFF-107 reports `fastapi=91`, `rust_native=42`, `web_used=41`,
  `missing_from_rust=50`, and `web_requires_fallback=19`.
- Remaining web fallback routes include settings/env, vector/graph status,
  write routes, and agent action execution.
- The Rust gateway already has local request handling and DB-backed read route
  support.
- `GET /settings/env`, `GET /memory/vector/chunks`, and
  `GET /memory/graph/schema` are read-only web-used routes.

## Allowed Scope

- Add Rust-native read-only handlers in `crates/igy6-gateway` for:
  - `GET /settings/env`
  - `GET /memory/vector/chunks`
  - `GET /memory/graph/schema`
- Read only process environment variables needed for safe route metadata.
- Perform only safe local TCP reachability checks for vector/graph status.
- Redact secret and sensitive configuration values.
- Preserve web-used response shapes where practical.
- Update route-level tests for the migrated routes.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust status/config
  route patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No runtime/private data reads or writes.
- No `.env` content reads or writes.
- No secret exposure.
- No archive actions.
- No deletion.
- No database schema changes.
- No write route migration.
- No settings/env verify or apply route migration.
- No agent action execution migration.
- No arbitrary shell execution.
- No Qdrant or Neo4j writes.
- No artifact file reads.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-108`.

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

- The three selected read-only routes are Rust-native and not proxied to
  FastAPI.
- Settings response shape remains usable by the web UI and redacts secrets.
- Vector and graph status responses are honest about Rust gateway status checks.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while write/action routes still depend on
  it.
- DIFF-108 is locked after verification passes.

## Results

- Migrated `GET /settings/env` to a Rust-native redacted process-environment
  metadata response.
- Migrated `GET /memory/vector/chunks` to a Rust-native read-only vector status
  response with bounded TCP reachability metadata.
- Migrated `GET /memory/graph/schema` to a Rust-native read-only graph status
  response with bounded TCP reachability metadata.
- Route parity changed from `rust_native=42`, `missing_from_rust=50`, and
  `web_requires_fallback=19` to `rust_native=45`, `missing_from_rust=47`, and
  `web_requires_fallback=16`.
- FastAPI fallback remains required for the remaining write/action routes.
- No write routes were migrated.
- No agent execution routes were migrated.

## Verification Results

- `git status --short` checked DIFF-108 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=45 web_used=41 missing_from_rust=47
  web_requires_fallback=16`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Settings/env verify and apply.
- Agent action execution.
- Approval, source, collection, report, feedback, outcome, analysis, and work
  item write routes.
- Full FastAPI retirement.
