# DIFF-138: FastAPI Fallback Readiness Gate

Status: Locked

## Type

Change-bearing fallback readiness and runtime wiring update.

## Objective

Determine whether FastAPI fallback can be removed after DIFF-137 completed
route parity.

Decision: remove FastAPI fallback wiring from the runtime API path.

## Baseline Facts

- DIFF-137 is complete and locked.
- Route parity records 91 FastAPI routes, 94 Rust-native routes, 0 FastAPI
  routes missing from Rust, 45 web-used routes, and 0 web-used routes requiring
  fallback.
- `GET /` is Rust-native.
- `fastapi_fallback_required` remains `true` only because DIFF-138 had not yet
  performed the readiness decision.
- Python/Celery worker services remain required and are not part of the FastAPI
  fallback removal decision.

## Allowed Scope

- `crates/igy6-gateway/`
- `infra/docker-compose.yml`
- `configs/rust-cutover-manifest.json`
- `configs/legacy-fastapi-route-classification.json`
- `scripts/rust-route-parity.py`
- `README.md`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`
- `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`
- This DIFF document.

## Prohibited Scope

- No DIFF-139 or later work until DIFF-138 is locked.
- No archive or deletion of `services/api/`.
- No archive or deletion of `services/worker/`.
- No worker parity claims.
- No `.env` mutation.
- No runtime/private data access under `IGY6_DATA_ROOT`.
- No cloud providers, credentials, or secrets.
- No locked DIFF edits.
- No unrelated cleanup, broad refactors, renames, redesign, data model changes,
  migration changes, or dependency changes.

## Required Tags

Use `DIFF-138` in change summaries and review notes. Inline comments are only
allowed where useful for non-obvious fallback removal behavior.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
- `python3 -m py_compile scripts/rust-route-parity.py`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

Run web build or UI smoke only if DIFF-138 changes frontend code or web-facing
contracts.

## Completion Criteria

- Manifest and route classification state no longer mark FastAPI fallback as
  required.
- Compose no longer starts or wires `legacy-api` as the Rust gateway fallback.
- Unsupported runtime API routes return a deterministic Rust error instead of
  proxying to FastAPI.
- Documentation states the API is Rust-native without FastAPI fallback, while
  not archiving legacy Python files or claiming Python worker parity.
- DIFF-139 remains the separate legacy Python archive/preservation decision.

## Completion Notes

DIFF-138 removes FastAPI fallback from the runtime API path.

The readiness decision is based on route parity reaching:

- FastAPI routes: 91
- Rust-native routes: 94
- FastAPI routes missing from Rust: 0
- Web-used routes: 45
- Web-used routes requiring fallback: 0

Runtime changes:

- `infra/docker-compose.yml` no longer defines or wires `legacy-api`.
- The Rust gateway no longer accepts `--fallback` or
  `FALLBACK_API_BASE_URL`.
- Unsupported API routes return deterministic Rust `404 Not Found` responses
  instead of attempting FastAPI proxying.
- Root and readiness responses report `fallback:"none"` and
  `fastapi_fallback.status:"removed"`.

Scope limits preserved:

- `services/api/` was not archived or deleted.
- `services/worker/` was not archived or deleted.
- Python/Celery worker parity is not claimed.
- DIFF-139 remains the legacy Python archive or preservation decision.

## Verification Results

- `git status --short` inspected before DIFF-138 and before lock.
- `git diff --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- `python3 -m py_compile scripts/rust-route-parity.py` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo test -p igy6-gateway` passed with 61 tests.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `Route parity: fastapi=91 rust_native=94 web_used=45 missing_from_rust=0 web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed and showed no `legacy-api` service or `FALLBACK_API_BASE_URL`.
- Runtime smoke with `cargo run -p igy6-gateway -- --bind 127.0.0.1:18038`
  and `curl` passed:
  - `GET /` returned `HTTP/1.1 200 OK` with `fallback:"none"`.
  - `GET /health/ready` returned `HTTP/1.1 200 OK` with
    `fastapi_fallback.status:"removed"`.
  - `GET /unimplemented-route` returned `HTTP/1.1 404 Not Found` from Rust.
