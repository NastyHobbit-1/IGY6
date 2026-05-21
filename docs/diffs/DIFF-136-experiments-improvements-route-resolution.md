# DIFF-136: Experiments And Improvements Route Resolution

Status: Locked

## Type

Change-bearing

## Objective

Resolve the experiments and improvements FastAPI fallback route family with one
explicit decision.

Decision: migrate to Rust.

The only routes authorized for Rust migration are:

- `GET /experiments`
- `GET /experiments/{experiment_run_id}`
- `POST /experiments`
- `POST /experiments/{experiment_run_id}/status`
- `GET /improvements`
- `GET /improvements/{improvement_item_id}`
- `POST /improvements`

## Baseline Facts

- DIFF-135 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 8 FastAPI routes
  still missing from Rust before this DIFF.
- The seven experiments/improvements routes are classified as
  `intentional_legacy_fallback` with `recommended_future_diff` set to
  `DIFF-136`.
- The only other missing route is the duplicate/superseded FastAPI root route,
  which is reserved for DIFF-137.

## Allowed Scope

- `crates/igy6-gateway/`
- `configs/legacy-fastapi-route-classification.json`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`
- This DIFF document.

Tests may be updated inside the Rust gateway scope to verify route-native
handling, validation, audit behavior, and fallback posture.

## Prohibited Scope

- No DIFF-137 or later work.
- No `GET /` root route resolution.
- No artifact route changes.
- No collection ingestion route changes.
- No FastAPI fallback removal unless this DIFF fully proves it is safe.
- No Rust-only claim unless manifest and route parity prove it.
- No MLflow, Optuna, Celery, experiment-runner, or self-improvement execution
  expansion beyond the existing HTTP metadata route semantics.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT` during tests or
  verification.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, rewiring, redesign,
  dependency changes, data model changes, or migration changes.

## Required Tags

Use `DIFF-136` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious experiments/improvements route behavior.

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

Run `npm --prefix apps/web run build` only if DIFF-136 changes web-facing
contracts, UI workflow behavior, or status text. Run
`npm --prefix apps/web run test:ui-smoke` only if UI workflow/status text
changes and the script is available.

## Completion Criteria

- The seven experiments/improvements routes are either Rust-native, retired, or
  intentionally kept with documented reason and retirement condition. This DIFF
  chooses Rust-native migration.
- Rust-native handlers preserve existing HTTP metadata semantics, validation,
  database writes, status updates, read ordering, and audit events.
- Route classification, manifest, and route audit docs reflect the DIFF-136
  decision.
- FastAPI fallback remains required if any FastAPI route is still unresolved.
- Rust-only operation is not claimed unless every remaining route is migrated
  or retired and the manifest proves fallback is no longer required.

## Completion Notes

- DIFF-136 family decision: migrate to Rust.
- Migrated exactly the seven experiments and improvements routes:
  - `GET /experiments`
  - `GET /experiments/{experiment_run_id}`
  - `POST /experiments`
  - `POST /experiments/{experiment_run_id}/status`
  - `GET /improvements`
  - `GET /improvements/{improvement_item_id}`
  - `POST /improvements`
- Preserved DB-backed list/detail read ordering and response fields for
  experiments and improvements.
- Preserved experiment creation validation for optional improvement item
  references and inserted `experiment_run.created` audit events.
- Preserved experiment status updates, optional metrics/artifacts/metadata
  replacement behavior, and inserted `experiment_run.status_updated` audit
  events.
- Preserved improvement target area and priority validation, `proposed` status
  creation, and inserted `improvement_item.created` audit events.
- Updated route classification, cutover manifest, and rust-migration route
  audit docs to reflect `93` Rust-native routes, `1` FastAPI route still
  missing from Rust, and `0` web-used routes requiring FastAPI fallback.
- Left `GET /` root route resolution for DIFF-137.
- Kept `fastapi_fallback_required=true`; Rust-only operation is not claimed.

## Verification Results

- `git status --short` checked DIFF-136 scoped files; Cargo-generated
  `target/` remains untracked and is not part of the DIFF.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- `cargo fmt --all --check` passed.
- `cargo test -p igy6-gateway` passed with 63 tests.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=93 web_used=45 missing_from_rust=1
  web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `npm --prefix apps/web run build` was not run because DIFF-136 did not
  change web contracts, UI workflow behavior, or status text.

## Out Of Scope Follow-Up

- DIFF-137 duplicate root route resolution.
- DIFF-138 FastAPI fallback readiness decision.
- DIFF-139 legacy Python archive plan or execution.
- DIFF-140 final Rust completion audit.
