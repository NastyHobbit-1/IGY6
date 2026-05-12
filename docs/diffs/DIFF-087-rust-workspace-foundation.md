# DIFF-087: Rust Workspace Foundation

Status: Locked

## Type

Change-bearing Rust foundation

## Objective

Add the root Rust workspace and foundational shared crates for IGY6 without
rewriting Python services, changing Docker Compose, changing `.env`, changing
the web UI, archiving legacy code, or wiring Rust into production runtime.

## Baseline Facts

- DIFF-086 is locked and added `crates/igy6-host-bridge/`.
- The repository does not have a root `Cargo.toml` before this DIFF.
- `configs/rust-cutover-manifest.json` has `host_bridge` complete and
  `workspace` pending before this DIFF.
- Current active runtime remains Python/FastAPI, Celery worker, Next.js, and
  Docker Compose.

## Allowed Scope

- `docs/diffs/DIFF-087-rust-workspace-foundation.md`
- Root `Cargo.toml`
- `crates/igy6-core/`
- `crates/igy6-config/`
- `crates/igy6-policy/`
- Minimal updates to `crates/igy6-host-bridge/` only if required for workspace
  membership without behavior change
- `configs/rust-cutover-manifest.json` workspace phase only
- Minimal README or rust-migration docs if needed
- `scripts/rust-cutover.sh` only for the workspace-aware `cargo fmt --all --check`
  verification correction required by the root virtual Cargo workspace

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose changes.
- No `.env` changes.
- No API backend rewrite.
- No frontend redesign.
- No database migrations.
- No Python removal.
- No archive actions.
- No runtime behavior change.
- No arbitrary shell execution.
- No external model calls.
- No runtime/private data reads or moves.

## Required Tags

- Commit message must include `DIFF-087`.
- Final response must identify `DIFF-087`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo test --workspace`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`

## Completion Criteria

- Root `Cargo.toml` defines a workspace.
- Existing `crates/igy6-host-bridge` is included in the workspace without
  behavior changes.
- `igy6-core` contains shared basic types/utilities only.
- `igy6-config` contains safe config parsing/validation primitives only.
- `igy6-policy` contains approval/safety/policy primitives only.
- Each new crate has minimal deterministic unit tests.
- Manifest `workspace` phase is updated honestly after verification.
- Existing runtime remains unchanged and runnable.

## Verification Result

- `git status --short` checked DIFF-087 scoped files only.
- `git diff --check` passed.
- `cargo test --workspace` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
