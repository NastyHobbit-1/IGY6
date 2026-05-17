# DIFF-098: Rust Retrieval Preview

Status: Locked

## Type

Change-bearing Rust retrieval-preview foundation

## Objective

Add Rust retrieval-preview response planning that preserves
`answer_status: not_generated` while Python/FastAPI retrieval remains active.

## Baseline Facts

- DIFF-097 completed the Rust `agent_api` phase.
- The manifest shows `retrieval_preview` as the next pending Rust phase.
- `services/api/app/retrieval.py` and `services/api/app/chat.py` remain the
  active DB/Qdrant/FastAPI retrieval implementation.
- This DIFF adds Rust in-memory retrieval-preview parity only.

## Allowed Scope

- `docs/diffs/DIFF-098-rust-retrieval-preview.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-retrieval-preview`
- `Cargo.lock` workspace package metadata
- `crates/igy6-retrieval-preview/`
- `configs/rust-cutover-manifest.json` `retrieval_preview` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python/FastAPI replacement.
- No DB reads or writes.
- No Qdrant calls.
- No external model calls.
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

- Commit message must include `DIFF-098`.
- Final response must identify `DIFF-098`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-retrieval-preview`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust preview planning returns `answer_status: not_generated`.
- Limit handling is bounded to the current vector search maximum.
- Disabled sources are filtered from hydrated hits.
- Source trails preserve chunk, document, source, raw artifact, evidence, and
  score metadata supplied by the caller.
- Tests cover answer status, limit clamping, disabled-source filtering, missing
  source allowance, and source-trail preservation.
- Manifest `retrieval_preview` phase is marked complete only after
  verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-098 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-retrieval-preview` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- DB hydration, Qdrant search execution, FastAPI route replacement,
  evidence-answer generation, graph retrieval, or external model use.
