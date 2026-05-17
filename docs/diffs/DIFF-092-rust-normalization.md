# DIFF-092: Rust Normalization

Status: Locked

## Type

Change-bearing Rust normalization foundation

## Objective

Add a Rust normalization crate that mirrors the current connector
normalization scaffold and provides deterministic UTF-8 text normalization,
without replacing Python collector or API behavior.

## Baseline Facts

- DIFF-091 completed the Rust `artifact_store` phase.
- The manifest shows `normalization` as the next pending Rust phase.
- Python collector normalization remains active in `services/collectors/app`.
- Existing Python normalization builds normalized document references from raw
  artifact references and classifies sensitivity labels.
- This DIFF adds Rust normalization parity beside Python only.

## Allowed Scope

- `docs/diffs/DIFF-092-rust-normalization.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-normalization`
- `Cargo.lock` workspace package metadata
- `crates/igy6-normalization/`
- `configs/rust-cutover-manifest.json` `normalization` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python collector replacement.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-092`.
- Final response must identify `DIFF-092`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-normalization`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust normalization builds deterministic normalized document refs from raw
  artifact refs.
- Sensitivity labels are classified with the same known-label/fallback behavior
  as the Python scaffold.
- UTF-8 byte normalization is deterministic and reports whether replacement
  characters were needed.
- Normalization metadata preserves raw artifact lineage.
- Tests cover sensitivity fallback, raw lineage metadata, deterministic IDs,
  valid UTF-8, lossy UTF-8, and caller metadata merging.
- Manifest `normalization` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-092 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-normalization` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Python collectors or worker normalization tasks.
- Database writes or API route rewiring.
- Chunk generation, evidence generation, embeddings, or graph writes.
