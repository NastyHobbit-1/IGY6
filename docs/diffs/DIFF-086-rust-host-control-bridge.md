# DIFF-086: Rust Host Control Bridge

Status: Locked

## Type

Change-bearing Rust implementation

## Objective

Add a local-only Rust host control bridge that can execute only fixed IGY6
operator script actions from the host environment. The bridge is a narrow
building block for later approved API integration; it does not change Docker,
FastAPI, Next.js, `.env`, database schema, or existing runtime behavior.

## Baseline Facts

- DIFF-085 is locked and created the Rust migration control plan.
- No tracked Cargo workspace existed before this DIFF.
- Existing host operator scripts are:
  - `scripts/run.sh`
  - `scripts/stop.sh`
  - `scripts/run-last-healthy-config.sh`
- DIFF-084 reports stack-control actions blocked from the API runtime because
  Docker CLI/socket access is unavailable there.
- This DIFF adds a host-side bridge only; it does not wire FastAPI to the bridge
  yet.

## Allowed Scope

- `docs/diffs/DIFF-086-rust-host-control-bridge.md`
- `crates/igy6-host-bridge/`
- `scripts/start-host-bridge.sh`
- `scripts/stop-host-bridge.sh`
- `configs/rust-cutover-manifest.json`
- README documentation for running the bridge

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose changes.
- No `.env` changes.
- No Docker socket mounting.
- No API backend rewrite.
- No frontend redesign.
- No database migrations.
- No Python removal.
- No archive actions.
- No arbitrary shell execution.
- No external model calls.
- No runtime/private data moves.

## Required Tags

- Commit message must include `DIFF-086`.
- Final response must identify `DIFF-086`.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/start-host-bridge.sh`
- `bash -n scripts/stop-host-bridge.sh`
- `cargo test --manifest-path crates/igy6-host-bridge/Cargo.toml`
- `cargo fmt --manifest-path crates/igy6-host-bridge/Cargo.toml --check`
- `cargo clippy --manifest-path crates/igy6-host-bridge/Cargo.toml --all-targets`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`

## Completion Criteria

- Bridge binds only to `127.0.0.1`.
- Bridge requires token authentication.
- Unknown and unauthenticated actions are rejected.
- Only fixed allowlisted actions map to existing operator script argv arrays.
- No user-provided argv or shell strings are accepted.
- Output is bounded and sensitive-looking lines are redacted.
- Health and capabilities endpoints exist and require authentication.
- Host scripts start and stop the bridge without printing the token.
- Manifest `host_bridge` phase is updated only after verification.
- Existing Python/FastAPI/Next.js/Docker behavior is not replaced.

## Verification Result

- `git status --short`: showed only DIFF-086 scoped files before staging.
- `git diff --check`: passed.
- `bash -n scripts/start-host-bridge.sh`: passed.
- `bash -n scripts/stop-host-bridge.sh`: passed.
- `scripts/start-host-bridge.sh --help`: passed.
- `scripts/stop-host-bridge.sh --help`: passed.
- `cargo test --manifest-path crates/igy6-host-bridge/Cargo.toml`: passed,
  7 tests passed.
- `cargo fmt --manifest-path crates/igy6-host-bridge/Cargo.toml --check`:
  passed.
- `cargo clippy --manifest-path crates/igy6-host-bridge/Cargo.toml --all-targets`:
  passed.
- `git diff --check`: passed.
- `bash -n scripts/start-host-bridge.sh`: passed.
- `bash -n scripts/stop-host-bridge.sh`: passed.
- `scripts/rust-cutover.sh --check`: passed. It warned that root `Cargo.toml`
  does not exist yet and `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `host_bridge` is marked complete in the cutover manifest after Rust
  verification passed.
