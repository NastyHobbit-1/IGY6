# DIFF-091: Rust Artifact Store

Status: Locked

## Type

Change-bearing Rust artifact store foundation

## Objective

Add a Rust content-addressed artifact store bounded by caller-provided
`IGY6_DATA_ROOT`, without replacing the existing Python artifact behavior.

## Baseline Facts

- DIFF-090 completed the Rust `config` phase.
- The manifest shows `artifact_store` as the next pending Rust phase.
- Python/FastAPI artifact behavior remains active in `services/api/app`.
- Existing Python artifact storage writes content-addressed files by SHA-256 and
  reads only bounded relative storage paths.
- This DIFF adds Rust parity beside Python only.

## Allowed Scope

- `docs/diffs/DIFF-091-rust-artifact-store.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-artifacts`
- `Cargo.lock` workspace package metadata
- `crates/igy6-artifacts/`
- `configs/rust-cutover-manifest.json` `artifact_store` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python/FastAPI replacement.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private reads from `IGY6_DATA_ROOT` contents.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-091`.
- Final response must identify `DIFF-091`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-artifacts`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust artifact store hashes raw bytes deterministically.
- Rust artifact store creates content-addressed write plans.
- Rust artifact store writes content-addressed artifacts under
  `IGY6_DATA_ROOT/artifacts` only.
- Duplicate writes are avoided and reported.
- Artifact metadata includes hash, relative storage path, size, and existed flag.
- Reads by hash are bounded to the artifact root.
- Path traversal and invalid hashes are rejected.
- Tests prove stable hashes, duplicate avoidance, bounded writes, and traversal
  rejection.
- Manifest `artifact_store` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-091 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo clippy --workspace --all-targets` passed after resolving the
  `needless_range_loop` warning.
- `cargo test --workspace` passed.
- `cargo test -p igy6-artifacts` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Python artifact behavior.
- API route rewiring.
- Database metadata changes or migrations.
- Runtime/private `IGY6_DATA_ROOT` data inspection outside explicit caller test
  temp directories.
