# DIFF-119: Non-Web FastAPI Route Classification

Status: Locked

## Type

Change-bearing

## Objective

Audit every remaining FastAPI route that is not Rust-native after DIFF-118,
classify the non-web fallback posture, and enforce that no missing FastAPI route
is left unclassified before any future FastAPI removal is attempted.

## Baseline Facts

- DIFF-118 is locked.
- `scripts/rust-route-parity.py --json` reports:
  - `fastapi_routes`: 91
  - `rust_native_routes`: 60
  - `fastapi_routes_missing_from_rust`: 34
  - `web_used_routes`: 41
  - `web_routes_requiring_fallback`: 0
- Rust is primary for web-used routes, but the repository is not Rust-only.
- `infra/docker-compose.yml` still runs `api` as the Rust gateway and
  `legacy-api` as the FastAPI fallback.
- FastAPI must remain while any active, intentional, or unsafe-to-migrate
  non-web fallback route remains.

## Allowed Scope

- Add a machine-readable non-web FastAPI route classification file under
  `configs/`.
- Add a human-readable non-web route classification document under
  `docs/rust-migration/`.
- Update `scripts/rust-route-parity.py` to enforce:
  - `web_routes_requiring_fallback` remains `0`.
  - Every FastAPI route missing from Rust is classified.
  - No unexpected or unclassified missing route is allowed.
  - Rust-only is not claimed while intentional legacy fallback or
    unsafe-to-migrate routes remain.
- Add focused tests for the route parity/classification script if practical.
- Update `configs/rust-cutover-manifest.json` honestly for DIFF-119.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` with the
  classification summary and DIFF-119 counts.
- Update this DIFF document with verification results and lock it after checks
  pass.

## Prohibited Scope

- No FastAPI removal, disablement, or archival.
- No Docker Compose rewiring.
- No Rust route migration batch.
- No new API route behavior.
- No approval bypass.
- No arbitrary shell execution.
- No user-provided argv execution.
- No `.env` reads or writes.
- No secrets, tokens, private keys, runtime/private data commits, or raw secret
  exposure.
- No database migrations.
- No dependency changes.
- No unrelated refactor, broad cleanup, renames, or redesign.
- No locked DIFF edits.

## Required Tags

Use `DIFF-119` in the commit message and final change summary.

## Verification

- `git status --short` checked DIFF-119 scoped files before commit.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=60 web_used=41 missing_from_rust=34
  web_requires_fallback=0`.
- `python3 scripts/test-rust-route-parity.py` passed, 3 tests.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 49 tests.
- `cargo test -p igy6-agent-api` passed, 6 tests.
- `cargo test -p igy6-host-bridge` passed, 7 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- No snippet-vault files changed, so snippet-vault JSONL validation was not
  applicable.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- No web test script exists beyond the build script in `apps/web/package.json`.
- `docker run --rm -v
  /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api
  python -m unittest discover tests` passed, 8 tests.

## Completion Criteria

- Every route in `fastapi_routes_missing_from_rust` is classified.
- Classification counts match route parity output.
- `web_routes_requiring_fallback` remains `0`.
- `fastapi_fallback_required` remains `true` while intentional fallback or
  unsafe-to-migrate routes remain.
- The manifest does not claim Rust-only.
- Required verification passes.
- This DIFF is locked only after verification passes.

## Out Of Scope Follow-Up

- DIFF-120 should migrate or retire the next explicit non-web route batch based
  on the DIFF-119 classification.
- FastAPI removal can only be scoped after route parity, runtime topology, and
  classification prove no required fallback remains.
