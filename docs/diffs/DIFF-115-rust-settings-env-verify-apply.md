# DIFF-115: Rust Settings Env Verify/Apply Routes

Status: Locked

## Type

Change-bearing

## Objective

Move the web-used settings routes `POST /settings/env/verify` and
`POST /settings/env/apply` to Rust-native gateway handling while preserving
secret redaction, key allowlists, verification-token semantics, safe `.env`
path constraints, and auditability.

## Baseline Facts

- DIFF-114 reports `fastapi=91`, `rust_native=53`, `web_used=41`,
  `missing_from_rust=39`, and `web_requires_fallback=6`.
- `GET /settings/env` is already Rust-native and redacts values from process
  environment metadata without reading `.env`.
- The Python verify/apply routes read the configured `.env`, validate changes
  against an allowlist, reject unknown and read-only keys, return sanitized
  candidate settings, use the candidate hash as the verification token, write
  `.env` only after a matching token, create a backup, and audit successful
  writes.
- The current Rust `api` Compose service does not mount the project or storage
  paths needed for safe `.env` writes.

## Allowed Scope

- Add Rust-native gateway handling for `POST /settings/env/verify`.
- Add Rust-native gateway handling for `POST /settings/env/apply`.
- Add deterministic Rust settings parsing, validation, sanitization,
  redaction, candidate hashing, safe render, backup, atomic write, and audit
  helpers.
- Preserve Python request/response shapes where practical for the web UI.
- Preserve verification-token compatibility semantics by using the rendered
  candidate SHA-256 hash as the token.
- Reject unknown keys and read-only keys.
- Reject unsafe storage paths, unsafe host data roots, invalid ports, invalid
  booleans, invalid URLs, invalid external model policies, and invalid audit
  log levels.
- Redact secret values in all responses and audit details.
- Insert `settings.env.updated` audit events on successful Rust apply.
- Update `infra/docker-compose.yml` only to give the Rust `api` service the
  same safe settings env vars and mounts required for settings apply.
- Update route-level tests for valid verify, invalid verify, redaction,
  unsafe-key rejection, apply token validation, missing database handling, and
  audit-planning behavior where unit-testable.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust settings
  verify/apply/redaction patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No manual upload, artifact write, normalization, or collection execution
  route migration.
- No work-item dispatch/status route migration.
- No agent execution route migration.
- No arbitrary shell execution from the Rust gateway.
- No Docker Compose execution from HTTP request handlers.
- No approval bypass.
- No raw `.env` values in responses or audit details.
- No `.env` content commits.
- No runtime/private data commits.
- No database schema changes or migrations.
- No external service calls.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-115`.

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
- Existing API/web tests where available, including the established legacy API
  test command from DIFF-114.

## Completion Criteria

- `POST /settings/env/verify` is Rust-native and no longer proxied to FastAPI.
- `POST /settings/env/apply` is Rust-native and no longer proxied to FastAPI.
- Invalid verify/apply requests are rejected by Rust without fallback.
- Secret settings are redacted from response and audit payloads.
- Apply writes only the configured safe `.env` path and creates a backup under
  the configured safe backup root.
- Successful apply writes a `settings.env.updated` audit event.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining manual upload,
  work-item dispatch, and agent execution routes depend on it.
- DIFF-115 is locked after verification passes.

## Results

- Added Rust-native handling for `POST /settings/env/verify`.
- Added Rust-native handling for `POST /settings/env/apply`.
- Preserved the web-used request shape: `values`, `actor_id`, and
  `verification_token` for apply.
- Preserved the web-used response shape for verify/apply, including
  `passed`, `errors`, `warnings`, `normalized_candidate`, `changed_keys`,
  `restart_required`, `restart_notes`, `verification_token`,
  `candidate_hash`, `compose_validation`, `saved`, `backup_path`, and
  `current`.
- Added allowlisted settings validation for unknown keys, read-only keys,
  non-string values, ports, booleans, URLs, service host/port agreement,
  storage paths, host data root, external model policy, audit log level, and
  Qdrant vector size.
- Preserved candidate-hash token semantics using the rendered candidate
  SHA-256 hash.
- Added safe `.env` apply behavior constrained to
  `/workspace/project/.env`, with backups under `/workspace/storage`.
- Added `settings.env.updated` audit event insertion on successful apply with
  `secret_values_recorded=false`.
- Added Rust `api` service mounts and env vars needed for the safe settings
  path and backup path.
- Intentionally did not execute Docker Compose from HTTP request handlers;
  `compose_validation` is present with `available=false`, and Compose config
  remains a verification/operator command.
- Updated route parity counts to `rust_native=55`,
  `fastapi_routes_missing_from_rust=37`, and
  `web_routes_requiring_fallback=4`.
- FastAPI remains required for remaining web-used fallbacks:
  `POST /agent/actions/`,
  `POST /agent/actions/${encodeURIComponent(actionName)}/execute`,
  `POST /collection-runs/manual-upload`, and `POST /work-items/`.

## Verification Results

- `git status --short` checked DIFF-115 scoped changes.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=55 web_used=41 missing_from_rust=37 web_requires_fallback=4`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 39 tests.
- `scripts/rust-cutover.sh --check` passed and ran the route parity guard.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- Existing legacy API tests passed:
  `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  ran 8 tests successfully.

## Intentional Parity Limit

Rust verify/apply does not execute `docker compose config` from HTTP request
handlers. The response keeps the `compose_validation` field with
`available=false`, `passed=null`, and an explicit message. Compose validation
remains part of DIFF verification and operator workflow, not a request-time
side effect.

## Out Of Scope Follow-Up

- Manual upload collection/ingest, report render, work dispatch, settings
  process restart, approval decision, source permission, pattern review, and
  agent execution routes.
- Full FastAPI retirement.
