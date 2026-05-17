# DIFF-104: Post-Cutover Reality Audit

Status: Locked

## Type

Change-bearing

## Objective

Audit the actual post-cutover runtime state after DIFF-103 and correct the
manifest/docs if they overstate Rust route parity or FastAPI retirement. This
DIFF must produce an explicit route parity matrix, runtime topology summary,
and follow-up implementation plan if legacy FastAPI fallback remains required.

## Baseline Facts

- DIFF-103 is locked and committed.
- `infra/docker-compose.yml` currently defines Rust gateway as `api` and
  FastAPI as `legacy-api`.
- `configs/rust-cutover-manifest.json` currently has `cutover_ready` true and
  `target_architecture` `rust-primary`.
- The Rust gateway was implemented as a std-only HTTP gateway and proxies
  unsupported routes to `legacy-api`.

## Allowed Scope

- Add or update route parity and runtime topology documentation under
  `docs/rust-migration/`.
- Update `configs/rust-cutover-manifest.json` only to honestly represent current
  post-cutover status and follow-up phases.
- Add small verification scripts/tests for route parity under `scripts/` or
  Rust crate tests if useful.
- Inspect FastAPI, Rust gateway, web route usage, Compose, README/docs, and
  `scripts/rust-cutover.sh`.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No runtime/private data reads or writes.
- No `.env` content reads or writes.
- No archive actions.
- No deletion.
- No database migrations.
- No dependency additions.
- No Docker Compose rewiring in this audit DIFF.
- No FastAPI removal or disabling in this audit DIFF.
- No claims that FastAPI is removable unless route parity proves it.

## Required Tags

Commit messages and final summaries must include `DIFF-104`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line if any are changed.

## Completion Criteria

- Every FastAPI route is inventoried.
- Every Rust gateway route is inventoried.
- Every route used by `apps/web` is inventoried.
- Route parity identifies each route as Rust-handled, proxied to FastAPI, or
  unsupported.
- Runtime topology is documented.
- Manifest/docs honestly describe current operational state.
- Follow-up DIFF plan exists if FastAPI fallback remains required.
- DIFF-104 is locked after verification passes.

## Completion Notes

- FastAPI exposes `/` plus 82 APIRouter routes.
- Rust gateway directly handles 7 routes.
- Active web workflows still require proxied FastAPI routes.
- Runtime state is Rust-primary with required FastAPI fallback, not Rust-only.
- DIFF-105 is required for the first web-critical Rust route parity batch.

## Out Of Scope Follow-Up

- Implementing missing Rust route parity.
- Removing or archiving FastAPI.
- Changing Docker Compose service topology.
