# DIFF-088: Rust CLI Foundation

Status: Locked

## Type

Change-bearing Rust foundation

## Objective

Add a local Rust CLI foundation for safe, read-only IGY6 migration commands over
the Rust workspace without replacing Python/FastAPI, changing Docker Compose,
changing `.env`, changing the web UI, archiving legacy code, or wiring Rust into
production runtime.

## Baseline Facts

- DIFF-087 is locked and added the root Rust workspace.
- `crates/igy6-host-bridge/` is already included in the workspace.
- `configs/rust-cutover-manifest.json` has `cli` pending before this DIFF.
- Current active runtime remains Python/FastAPI, Celery worker, Next.js, and
  Docker Compose.

## Allowed Scope

- `docs/diffs/DIFF-088-rust-cli-foundation.md`
- `crates/igy6-cli/`
- Root `Cargo.toml` workspace membership for `crates/igy6-cli`
- `configs/rust-cutover-manifest.json` `cli` phase only after verification
- Minimal tests inside `crates/igy6-cli`
- Narrow additions to `igy6-core`, `igy6-config`, or `igy6-policy` only if
  truly required by the CLI and still foundational
- Minimal README or rust-migration docs only if needed

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose changes.
- No `.env` changes.
- No API backend rewrite.
- No frontend redesign.
- No database migrations.
- No Python removal.
- No archive actions.
- No production runtime wiring.
- No arbitrary shell execution.
- No external model calls.
- No runtime/private data access.

## Required Tags

- Commit message must include `DIFF-088`.
- Final response must identify `DIFF-088`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo test --workspace`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo run -p igy6-cli -- --help`
- `cargo run -p igy6-cli -- version`
- `cargo run -p igy6-cli -- phases --manifest configs/rust-cutover-manifest.json`
- `cargo run -p igy6-cli -- phase-status cli --manifest configs/rust-cutover-manifest.json`
- `cargo run -p igy6-cli -- validate-manifest configs/rust-cutover-manifest.json`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`

## Completion Criteria

- CLI supports `--help`, `version`, `phases`, `phase-status`, and
  `validate-manifest`.
- CLI parses the cutover manifest from a user-provided path.
- CLI reports `cutover_ready` and phase statuses.
- CLI fails cleanly on missing file, invalid manifest structure, unknown phase,
  and unsupported command.
- CLI remains local-only, read-only, deterministic, and does not execute shell
  commands or external model calls.
- Manifest `cli` phase is updated honestly after verification.
- Existing runtime remains unchanged and runnable.

## Verification Result

- `git status --short` checked DIFF-088 scoped files only.
- `git diff --check` passed.
- `cargo test --workspace` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo run -p igy6-cli -- --help` passed.
- `cargo run -p igy6-cli -- version` passed.
- `cargo run -p igy6-cli -- phases --manifest configs/rust-cutover-manifest.json` passed.
- `cargo run -p igy6-cli -- phase-status cli --manifest configs/rust-cutover-manifest.json` passed.
- `cargo run -p igy6-cli -- validate-manifest configs/rust-cutover-manifest.json` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
