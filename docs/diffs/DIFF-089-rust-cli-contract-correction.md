# DIFF-089: Rust CLI Contract Correction

Status: Locked

## Type

Change-bearing Rust CLI correction

## Objective

Correct DIFF-088 CLI contract drift by providing the required local IGY6 CLI
binary named `igy6` with the operational command surface required by the Rust
migration plan, while preserving useful DIFF-088 manifest commands when they do
not conflict.

## Baseline Facts

- DIFF-088 added the initial Rust CLI foundation.
- DIFF-088 implemented manifest-oriented commands and a binary named
  `igy6-cli`.
- DIFF-089 corrects the CLI contract drift.
- Required binary is `igy6`.
- Required commands are `health`, `run`, `stop`, `run-last-healthy`,
  `config check`, and `snapshot show`.
- Bash wrappers remain active until a later DIFF proves Rust CLI parity and
  explicitly allows replacement.

## Allowed Scope

- `docs/diffs/DIFF-089-rust-cli-contract-correction.md`
- `crates/igy6-cli/`
- `configs/rust-cutover-manifest.json` `cli` phase verification notes only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/bash/snippets.jsonl`
- `snippet-vault/rust-equivalents/by-source-language/other/snippets.jsonl` only
  if source language is unclear
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No backend rewrite.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads from `IGY6_DATA_ROOT` except safe
  existence/path checks if already required by existing repo behavior.
- No file deletion.
- No archive execution.
- No broad refactor.
- No unrelated formatting churn.
- Do not replace or archive `scripts/run.sh`, `scripts/stop.sh`,
  `scripts/run-last-healthy-config.sh`, or `scripts/lib/igy6-ops.sh`.

## Required Tags

- Commit message must include `DIFF-089`.
- Final response must identify `DIFF-089`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo run -p igy6-cli --bin igy6 -- --help`
- `cargo run -p igy6-cli --bin igy6 -- health`
- `cargo run -p igy6-cli --bin igy6 -- config check`
- `cargo run -p igy6-cli --bin igy6 -- snapshot show`
- `cargo run -p igy6-cli --bin igy6 -- version`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- If snippet-vault JSONL files are created or updated, validate that each added
  JSONL line parses as valid JSON.

## Completion Criteria

- `cargo run -p igy6-cli --bin igy6 -- --help` documents the required
  commands.
- `igy6 health` performs safe local repo/tooling checks only.
- `igy6 run`, `igy6 stop`, and `igy6 run-last-healthy` call existing Bash
  scripts with fixed argv arrays and no shell string execution.
- `igy6 config check` validates repo-visible config structure without reading
  or printing `.env` secrets.
- `igy6 snapshot show` is non-destructive and either shows safe supported
  snapshot status or clearly documents deferred behavior.
- Existing DIFF-088 manifest commands remain available if non-conflicting.
- The `cli` phase in `configs/rust-cutover-manifest.json` records DIFF-089 as
  the corrective completion DIFF after verification passes.
- `cutover_ready` remains false and no other pending Rust phase is marked
  complete.

## Verification Result

- `git status --short` checked DIFF-089 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo run -p igy6-cli --bin igy6 -- --help` passed.
- `cargo run -p igy6-cli --bin igy6 -- health` passed.
- `cargo run -p igy6-cli --bin igy6 -- config check` passed.
- `cargo run -p igy6-cli --bin igy6 -- snapshot show` passed with the
  intentional non-destructive placeholder.
- `cargo run -p igy6-cli --bin igy6 -- version` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Bash operator scripts.
- Reading runtime/private snapshot data from `IGY6_DATA_ROOT` in Rust.
- Rust API/container Docker socket access.
- Any production runtime rewiring.
