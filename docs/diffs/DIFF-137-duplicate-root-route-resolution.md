# DIFF-137: Duplicate Root Route Resolution

Status: Locked

## Type

Change-bearing

## Objective

Resolve the duplicate/superseded FastAPI root route with minimal scope.

Decision: migrate `GET /` to a Rust-native gateway identity response.

The only route authorized for resolution is:

- `GET /`

## Baseline Facts

- DIFF-136 is complete and locked.
- IGY6 is Rust-primary, not Rust-only.
- `fastapi_fallback_required` remains `true`.
- `configs/legacy-fastapi-route-classification.json` records 1 FastAPI route
  still missing from Rust before this DIFF: `GET /`.
- The FastAPI root route returns a scaffold service identity only.
- Rust already exposes stronger operational identity and readiness surfaces at
  `/health/live`, `/health/ready`, and `/rust-migration/status`.

## Allowed Scope

- `crates/igy6-gateway/`
- `configs/legacy-fastapi-route-classification.json`
- `configs/rust-cutover-manifest.json`
- `scripts/rust-route-parity.py`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`
- This DIFF document.

Tests may be updated inside the Rust gateway scope to verify root route-native
handling and fallback posture.

## Prohibited Scope

- No DIFF-138 or later work until DIFF-137 is locked.
- No FastAPI fallback removal.
- No legacy-api Compose removal.
- No Rust-only claim unless factually true.
- No artifact, collection, experiment, improvement, worker, or UI behavior
  changes.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT` during tests or
  verification.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, rewiring, redesign,
  dependency changes, data model changes, or migration changes.

## Required Tags

Use `DIFF-137` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious root-route behavior.

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
- HTTP root smoke using `curl -i http://127.0.0.1:<port>/`

Run `npm --prefix apps/web run build` only if DIFF-137 changes web-facing
contracts, UI workflow behavior, or status text. Run
`npm --prefix apps/web run test:ui-smoke` only if UI workflow/status text
changes and the script is available.

## Completion Criteria

- `GET /` is resolved by minimal Rust-native migration or formal retirement.
  This DIFF chooses Rust-native migration.
- Rust health/status/root behavior is documented as superseding the old FastAPI
  scaffold root response.
- Route classification, manifest, route parity guard state, and route audit docs
  reflect zero FastAPI routes missing from Rust.
- FastAPI fallback is not removed in this DIFF.
- Rust-only operation is not claimed unless DIFF-138 readiness proves fallback
  can be removed.

## Completion Notes

DIFF-137 migrated the duplicate/superseded FastAPI `GET /` scaffold route to
Rust-native gateway handling.

The Rust root route now returns gateway identity and operational posture:

```json
{"service":"igy6-gateway","phase":"rust-primary","status":"ok","primary_gateway":true,"fallback":"fastapi"}
```

This supersedes the FastAPI root scaffold response while preserving FastAPI
fallback configuration for the separate DIFF-138 readiness decision.

Route parity now records:

- FastAPI routes: 91
- Rust-native routes: 94
- Web-used FastAPI fallback routes: 0
- FastAPI routes missing from Rust: 0
- Web routes requiring fallback: 0

`fastapi_fallback_required` remains `true` and `rust_only_claim_allowed`
remains `false` until DIFF-138 evaluates whether fallback removal is safe.

## Verification Results

- `git status --short` inspected before changes and before lock.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- `python3 -m py_compile scripts/rust-route-parity.py` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test -p igy6-gateway` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- HTTP root smoke used `curl -i http://127.0.0.1:18037/` against the local
  Rust gateway and returned `HTTP/1.1 200 OK` with the Rust gateway identity
  payload shown above. The first non-escalated localhost bind failed due sandbox
  network restrictions; the successful smoke used an approved local bind only.

## Out Of Scope Follow-Up

- DIFF-138 FastAPI fallback readiness decision.
- DIFF-139 legacy Python archive plan or execution.
- DIFF-140 final Rust completion audit.
