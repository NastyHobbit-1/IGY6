# DIFF-096: Rust Read-Only API

Status: Locked

## Type

Change-bearing Rust read-only API sidecar foundation

## Objective

Add a minimal Rust read-only API sidecar crate that exposes safe local health
and Rust migration status responses while FastAPI remains the primary API.

## Baseline Facts

- DIFF-095 completed the Rust `worker` phase.
- The manifest shows `read_only_api` as the next pending Rust phase.
- FastAPI remains the primary policy-enforced API gateway.
- This DIFF adds a standalone Rust sidecar foundation only.

## Allowed Scope

- `docs/diffs/DIFF-096-rust-read-only-api.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-read-only-api`
- `Cargo.lock` workspace package metadata
- `crates/igy6-read-only-api/`
- `configs/rust-cutover-manifest.json` `read_only_api` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No FastAPI replacement.
- No API gateway rewiring.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads.
- No external service health calls.
- No live sidecar startup during verification beyond `--help`.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-096`.
- Final response must identify `DIFF-096`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-read-only-api`
- `cargo run -p igy6-read-only-api -- --help`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust sidecar crate provides deterministic read-only route handling.
- `GET /health/live` returns safe local liveness JSON.
- `GET /health/ready` returns safe local readiness JSON and states FastAPI is
  still primary.
- `GET /rust-migration/status` returns safe manifest-derived status without
  reading secrets or runtime/private data.
- Unsupported routes and methods fail predictably.
- Binary help documents local-only read-only sidecar usage.
- Tests cover health routes, manifest status summarization, unsupported routes,
  unsupported methods, and request parsing.
- Manifest `read_only_api` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-096 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-read-only-api` passed.
- `cargo run -p igy6-read-only-api -- --help` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- FastAPI replacement, gateway cutover, database-backed read routes, auth,
  browser/API gateway wiring, Docker Compose changes, or live service
  deployment.
