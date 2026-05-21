# DIFF-135: Artifact And Collection Ingestion Route Parity

Status: Locked

## Type

Change-bearing

## Objective

Migrate only the remaining artifact and collection ingestion FastAPI fallback
routes to Rust-native gateway handling while preserving artifact storage safety,
source permission checks, approval checks, audit events, content-addressing,
bounded input behavior, and honest FastAPI fallback posture.

The only routes authorized for Rust migration are:

- `POST /artifacts`
- `POST /collection-runs`
- `POST /collection-runs/local-project`
- `POST /collection-runs/manual-upload/ingest`

## Baseline Facts

- DIFF-134 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 12 FastAPI routes
  still missing from Rust.
- The four DIFF-135 routes are currently classified as `unsafe_to_migrate_now`
  with `recommended_future_diff` set to `DIFF-135`.
- Web-used routes currently require no FastAPI fallback, but non-web FastAPI
  fallback remains required.
- Python/Celery workers remain active; this DIFF must not remove or replace
  worker execution broadly.

## Allowed Scope

- `crates/igy6-gateway/`
- Existing Rust support crates only when required by the migrated routes.
- `configs/legacy-fastapi-route-classification.json`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`

The DIFF may update tests inside the allowed Rust gateway/support crate scope
when needed to verify route validation, safety boundaries, database behavior,
audit behavior, and fallback posture.

## Prohibited Scope

- No DIFF-136 or later work.
- No experiments or improvements route migration or retirement decision.
- No duplicate root route resolution.
- No FastAPI fallback removal.
- No Rust-only claim.
- No broad collector redesign.
- No broad worker replacement.
- No direct Celery dispatch or arbitrary worker execution from the HTTP
  gateway unless a migrated Python route already performs an equivalent bounded
  enqueue marker and the Rust behavior is explicitly non-executing or
  audit-preserving.
- No PDF, image, audio, browser, router, or advanced parser expansion unless
  required to preserve existing route parity.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT` during tests or
  verification.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, rewiring, redesign,
  dependency changes, data model changes, or migration changes.

## Required Tags

Use `DIFF-135` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious artifact, collection, permission, approval,
or storage-boundary behavior.

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

Run `npm --prefix apps/web run build` only if DIFF-135 changes web-facing
contracts, UI workflow behavior, or status text. Run
`npm --prefix apps/web run test:ui-smoke` only if UI workflow/status text
changes and the script is available.

## Completion Criteria

- `POST /artifacts` is Rust-native in `crates/igy6-gateway` with source/run
  reference validation, storage path/content hash safety, bounded metadata, and
  audit parity where expected.
- `POST /collection-runs` is Rust-native with bounded scaffold behavior,
  source validation, source permission and approval behavior preserved where
  applicable, and audit parity.
- `POST /collection-runs/local-project` is Rust-native with scoped local
  project collection semantics, read-only/default safety, path traversal
  resistance, source permission checks, approval checks, artifact metadata, and
  audit parity.
- `POST /collection-runs/manual-upload/ingest` is Rust-native with bounded
  ingest behavior, artifact safety, normalization/evidence metadata behavior,
  failure recovery, work-item/audit behavior, and no unscoped parser expansion.
- Route classification, manifest, and route audit docs reflect the DIFF-135
  migration.
- FastAPI fallback remains required for later DIFF buckets unless route parity
  and classification state prove otherwise in a later DIFF.
- Rust-only operation is not claimed.

## Completion Notes

- Migrated exactly the four DIFF-135 artifact and collection ingestion routes
  into the Rust gateway:
  - `POST /artifacts`
  - `POST /collection-runs`
  - `POST /collection-runs/local-project`
  - `POST /collection-runs/manual-upload/ingest`
- Preserved FastAPI fallback for remaining unsupported routes. DIFF-135 does
  not claim Rust-only operation.
- Preserved bounded content-addressed artifact storage behavior through the
  Rust artifact store and existing data-root boundary checks.
- Preserved source existence/type checks, source permission checks, approval
  checks, collection run writes, raw artifact writes, normalization/evidence
  metadata writes, work-item creation where applicable, vector failure
  handling, and audit-event insertion for the migrated route surfaces.
- Updated route classification, cutover manifest, and rust-migration route
  audit docs to reflect `86` Rust-native routes, `8` FastAPI routes still
  missing from Rust, and `0` web-used routes requiring FastAPI fallback.
- Left experiments/improvements, duplicate root route resolution, fallback
  removal, and Rust-only readiness for later DIFF scopes.

## Verification Results

- `git status --short` checked DIFF-135 scoped files; Cargo-generated
  `target/` remains untracked and is not part of the DIFF.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test -p igy6-gateway` passed, including DIFF-135 route-native and
  validation coverage.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=86 web_used=45 missing_from_rust=8
  web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `npm --prefix apps/web run build` was not run because DIFF-135 did not
  change web contracts, UI workflow behavior, or status text.

## Out Of Scope Follow-Up

- DIFF-136 experiments and improvements fallback resolution.
- DIFF-137 duplicate root route resolution.
- DIFF-138 FastAPI fallback readiness decision.
- DIFF-139 legacy Python archive plan or execution.
- DIFF-140 final Rust completion audit.
