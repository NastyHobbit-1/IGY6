# DIFF-132: Active Medium-Risk Route Parity

Status: Locked

## Type

Change-bearing

## Objective

Migrate only the active medium-risk FastAPI fallback routes classified as
`active_parity_required` to Rust-native gateway handlers while preserving
request/response contracts, validation, audit behavior, permission checks, and
status-transition checks.

## Baseline Facts

- DIFF-131 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 11
  `active_parity_required` routes still missing from Rust.
- Web-used routes currently require no FastAPI fallback, but non-web FastAPI
  fallback remains required.

## Allowed Scope

- `crates/igy6-gateway/`
- Existing Rust support crates only when required by the migrated routes.
- `configs/legacy-fastapi-route-classification.json`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`

The only routes authorized for Rust migration are:

- `GET /retrieval/chunks/{chunk_id}/trail`
- `POST /analysis/hypotheses`
- `POST /analysis/predictions`
- `POST /analysis/recommendations`
- `POST /evidence/documents`
- `POST /evidence/documents/{document_id}/chunks`
- `POST /evidence/items`
- `POST /reports/{report_id}/status`
- `POST /retrieval/chunks/search`
- `POST /sources/{source_id}/permissions`
- `POST /work-items/{work_item_id}/status`

## Prohibited Scope

- No DIFF-133 or later work.
- No graph/vector service mutation route migration.
- No artifact or collection ingestion route migration.
- No experiment/improvement route decision.
- No duplicate root route resolution.
- No FastAPI fallback removal.
- No Rust-only claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, rewiring, redesign,
  dependency changes, data model changes, or migration changes.

## Required Tags

Use `DIFF-132` in change summaries and review notes. Inline comments are only
allowed where useful for a non-obvious route parity behavior.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build` only if web-facing behavior or contracts
  change.
- `npm --prefix apps/web run test:ui-smoke` only if UI workflow/status text
  changes and the script is available.

## Completion Criteria

- The 11 DIFF-132 routes are Rust-native in `crates/igy6-gateway`.
- Validation, success, not-found, invalid-state, and audit behavior are covered
  by Rust tests where applicable.
- Route classification, manifest, and route audit docs reflect the DIFF-132
  migration.
- FastAPI fallback remains required for later DIFF buckets.
- Rust-only operation is not claimed.

## Completion Notes

- DIFF-132 migrated all 11 authorized active medium-risk fallback routes to
  Rust-native gateway handling.
- Route parity is now `fastapi=91`, `rust_native=75`,
  `missing_from_rust=19`, `web_used=45`, and `web_requires_fallback=0`.
- `configs/legacy-fastapi-route-classification.json` now has
  `active_parity_required=0`.
- `fastapi_fallback_required` remains `true`.
- Rust-only operation is not claimed.

## Out Of Scope Follow-Up

- DIFF-133 graph/vector memory parity.
- DIFF-134 report work-item parity.
- DIFF-135 artifact and collection ingestion parity.
- DIFF-136 experiments and improvements fallback resolution.
- DIFF-137 duplicate root route resolution.
- DIFF-138 FastAPI fallback readiness decision.
