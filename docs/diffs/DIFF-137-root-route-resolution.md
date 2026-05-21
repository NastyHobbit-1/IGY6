# DIFF-137: Root Route Resolution

Status: Locked

## Type

Change-bearing route parity governance update.

## Objective

Resolve the final FastAPI route parity blocker, `GET /`, without changing runtime behavior or removing the FastAPI fallback service.

## Decision

`GET /` is retired/superseded for route-parity purposes.

The FastAPI root route only returns service identity. Rust already exposes supported runtime identity and readiness surfaces through:

- `GET /health/live`
- `GET /health/ready`
- `GET /rust-migration/status`

No web-used route depends on FastAPI `GET /`.

## Changed Scope

- `scripts/rust-route-parity.py`
- `configs/legacy-fastapi-route-classification.json`
- `docs/diffs/DIFF-137-root-route-resolution.md`

## Notes

DIFF-137 does not remove `legacy-api` from Docker Compose and does not archive `services/api/` or `services/worker/`. DIFF-138 remains responsible for the FastAPI fallback readiness decision and any runtime wiring removal.

## Verification

Connector-based edit only. Required local verification remains:

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

## Final Posture

After local manifest/doc completion and verification, expected route parity is:

- `fastapi=91`
- `rust_native=93`
- `web_used=45`
- `missing_from_rust=0`
- `web_requires_fallback=0`

Rust-only is not claimed by this DIFF. DIFF-138 must decide whether fallback wiring can be removed.
