# DIFF-105: Route Parity Guard

Status: Locked

## Type

Change-bearing

## Objective

Add an automated route parity guard so the repository can verify whether the
Rust gateway still depends on FastAPI fallback routes. This DIFF does not claim
missing route parity is implemented; it makes the dependency mechanically
visible and prevents future cutover work from relying on manual audit alone.

## Baseline Facts

- DIFF-105 route parity guard finds 90 FastAPI APIRouter routes plus `/`.
- DIFF-104 found 7 Rust-native gateway routes.
- FastAPI remains required as `legacy-api` fallback.
- Existing `scripts/rust-cutover.sh --check` validates manifest shape and Rust
  checks, but it does not inventory route parity.

## Allowed Scope

- Add a local stdlib-only route parity script under `scripts/`.
- Update `scripts/rust-cutover.sh` to run the parity script during `--check`.
- Update `configs/rust-cutover-manifest.json` with route parity guard status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` and migration docs
  for the automated guard.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No runtime/private data reads or writes.
- No `.env` content reads or writes.
- No archive actions.
- No deletion.
- No database migrations.
- No dependency additions.
- No Docker Compose rewiring.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-105`.

## Verification

- `git status --short`
- `git diff --check`
- `python3 scripts/rust-route-parity.py --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line if any are changed.

## Completion Criteria

- Route parity script inventories FastAPI, Rust-native, and web-used routes.
- Script exits successfully only when manifest fallback status matches the
  actual route parity result.
- Cutover check runs the parity guard.
- Manifest records the guard and still states that FastAPI fallback is required.
- DIFF-105 is locked after verification passes.

## Completion Notes

- `python3 scripts/rust-route-parity.py --check` reports:
  `fastapi=91`, `rust_native=7`, `web_used=41`,
  `missing_from_rust=85`, `web_requires_fallback=36`.
- `scripts/rust-cutover.sh --check` now runs the route parity guard.
- FastAPI fallback remains required and intentionally documented.

## Out Of Scope Follow-Up

- Implementing missing DB-backed Rust route parity.
- Removing or archiving FastAPI.
- Changing web route behavior.
