# DIFF-093: Rust Chunking

Status: Locked

## Type

Change-bearing Rust chunking foundation

## Objective

Add deterministic Rust text chunking and evidence-item planning beside the
existing Python API/worker chunk generation, without replacing Python behavior.

## Baseline Facts

- DIFF-092 completed the Rust `normalization` phase.
- The manifest shows `chunking` as the next pending Rust phase.
- Python API and worker chunking split normalized document text by fixed
  character windows and create one evidence item per chunk.
- This DIFF adds Rust parity beside Python only.

## Allowed Scope

- `docs/diffs/DIFF-093-rust-chunking.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-chunking`
- `Cargo.lock` workspace package metadata
- `crates/igy6-chunking/`
- `configs/rust-cutover-manifest.json` `chunking` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python API/worker replacement.
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

- Commit message must include `DIFF-093`.
- Final response must identify `DIFF-093`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-chunking`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust chunking splits text deterministically by character window size.
- Chunk size is bounded to the current Python API range.
- Chunk locations include `char_start` and `char_end`.
- Chunk metadata preserves `chunk_size`.
- Evidence planning creates one `document_chunk` evidence item per chunk.
- Empty text and invalid chunk sizes are rejected.
- Tests cover deterministic splitting, boundaries, empty text, invalid sizes,
  evidence count, and non-ASCII character boundaries.
- Manifest `chunking` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-093 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-chunking` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Database writes, worker execution, embeddings, vector upserts, or graph writes.
- Replacing Python API/worker chunk generation.
