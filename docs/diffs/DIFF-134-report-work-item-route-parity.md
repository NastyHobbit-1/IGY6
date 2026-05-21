# DIFF-134: Report Work-Item Route Parity

Status: Active

## Type

Change-bearing

## Objective

Migrate only `POST /reports/{report_id}/work-item` to Rust-native gateway
handling while preserving report work-item creation semantics, report/audit
expectations, and honest fallback posture.

## Baseline Facts

- DIFF-133 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 13 FastAPI routes
  still missing from Rust, including `POST /reports/{report_id}/work-item`.
- Web-used routes currently require no FastAPI fallback, but non-web FastAPI
  fallback remains required.

## Allowed Scope

- `crates/igy6-gateway/`
- Existing Rust support crates only when required by the migrated route.
- `configs/legacy-fastapi-route-classification.json`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`

The only route authorized for Rust migration is:

- `POST /reports/{report_id}/work-item`

## Prohibited Scope

- No DIFF-135 or later work.
- No artifact route migration.
- No collection ingestion route migration.
- No experiment/improvement route decision.
- No duplicate root route resolution.
- No Celery dispatch or direct worker execution from the HTTP gateway.
- No FastAPI fallback removal.
- No Rust-only claim.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, rewiring, redesign,
  dependency changes, data model changes, or migration changes.

## Required Tags

Use `DIFF-134` in change summaries and review notes. Inline comments are only
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

## Completion Criteria

- `POST /reports/{report_id}/work-item` is Rust-native in
  `crates/igy6-gateway`.
- The Rust handler creates a queued `report_generation` work item with bounded
  scaffold-only payload and no Celery dispatch.
- The Rust handler inserts the expected `work_item.created` audit event
  correlated to the report.
- Route classification, manifest, and route audit docs reflect the DIFF-134
  migration.
- FastAPI fallback remains required for later DIFF buckets.
- Rust-only operation is not claimed.

## Out Of Scope Follow-Up

- DIFF-135 artifact and collection ingestion parity.
- DIFF-136 experiments and improvements fallback resolution.
- DIFF-137 duplicate root route resolution.
- DIFF-138 FastAPI fallback readiness decision.
