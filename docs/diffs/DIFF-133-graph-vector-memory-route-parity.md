# DIFF-133: Graph And Vector Memory Route Parity

Status: Locked

## Type

Change-bearing

## Objective

Migrate only the graph and vector memory FastAPI fallback routes assigned to
DIFF-133 into Rust-native gateway handlers while preserving missing-service
behavior, missing-collection behavior, vector-size/config validation, graph
schema safety, bounded service calls, and honest fallback posture.

## Baseline Facts

- DIFF-132 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 19 FastAPI routes
  still missing from Rust, including six DIFF-133 graph/vector memory routes.
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

- `GET /memory/graph/nodes/{node_label}/{node_id}/relationships`
- `POST /memory/graph/lineage/sync`
- `POST /memory/graph/schema/ensure`
- `POST /memory/vector/chunks/ensure`
- `POST /memory/vector/chunks/search`
- `POST /memory/vector/chunks/upsert`

## Prohibited Scope

- No DIFF-134 or later work.
- No artifact or collection ingestion route migration.
- No report work-item route migration.
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

Use `DIFF-133` in change summaries and review notes. Inline comments are only
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

- The six DIFF-133 graph/vector memory routes are Rust-native in
  `crates/igy6-gateway`.
- Validation, missing service, missing collection, bounded request, and response
  shape behavior are covered by Rust tests where applicable.
- Route classification, manifest, and route audit docs reflect the DIFF-133
  migration.
- FastAPI fallback remains required for later DIFF buckets.
- Rust-only operation is not claimed.

## Completion Notes

- DIFF-133 migrated all six authorized graph/vector memory fallback routes to
  Rust-native gateway handling.
- Route parity is now `fastapi=91`, `rust_native=81`,
  `missing_from_rust=13`, `web_used=45`, and `web_requires_fallback=0`.
- `configs/legacy-fastapi-route-classification.json` now has
  `unsafe_to_migrate_now=5`.
- `fastapi_fallback_required` remains `true`.
- Rust-only operation is not claimed.

## Out Of Scope Follow-Up

- DIFF-134 report work-item parity.
- DIFF-135 artifact and collection ingestion parity.
- DIFF-136 experiments and improvements fallback resolution.
- DIFF-137 duplicate root route resolution.
- DIFF-138 FastAPI fallback readiness decision.
