# DIFF-094: Rust Vector Memory

Status: Locked

## Type

Change-bearing Rust vector-memory foundation

## Objective

Add deterministic Rust vector-memory helpers for local embedding, Qdrant
collection payloads, upsert payloads, search payloads, and bounded HTTP request
planning beside the existing Python API behavior.

## Baseline Facts

- DIFF-093 completed the Rust `chunking` phase.
- The manifest shows `vector_memory` as the next pending Rust phase.
- `services/api/app/vector_memory.py` remains the active API/database/Qdrant
  runtime path.
- This DIFF adds Rust parity helpers only; it does not replace Python behavior.

## Allowed Scope

- `docs/diffs/DIFF-094-rust-vector-memory.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-vector-memory`
- `Cargo.lock` workspace package metadata
- `crates/igy6-vector-memory/`
- `configs/rust-cutover-manifest.json` `vector_memory` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python API/worker replacement.
- No FastAPI rewrite.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads.
- No live Qdrant mutation during verification.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-094`.
- Final response must identify `DIFF-094`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-vector-memory`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust vector helpers build deterministic vectors for text.
- Empty text returns a zero vector.
- Invalid vector sizes are rejected.
- Qdrant collection payloads preserve cosine-distance collection semantics.
- Qdrant points payloads include chunk, document, chunk-index, and embedding
  method metadata.
- Qdrant search payloads clamp limits to the current maximum and do not request
  vectors in responses.
- Qdrant HTTP request planning is bounded to plain local/client-provided
  `http://host[:port]` origins and rejects malformed origins or path traversal.
- Tests cover deterministic vectors, normalization, invalid sizes, payload
  shape, search limit bounds, URL/request planning, and unsafe collection names.
- Manifest `vector_memory` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-094 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-vector-memory` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Python API vector routes, database writes, or worker upserts.
- Live Qdrant integration tests.
- Rust worker orchestration for normalization, chunking, and vector upserts.
