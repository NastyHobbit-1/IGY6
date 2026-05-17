# DIFF-099: Rust Evidence Answer

Status: Locked

## Type

Change-bearing Rust deterministic evidence-answer foundation

## Objective

Add Rust deterministic evidence-answer packet construction over hydrated
retrieval context without replacing Python/FastAPI chat behavior.

## Baseline Facts

- DIFF-098 completed the Rust `retrieval_preview` phase.
- The manifest shows `evidence_answer` as the next pending Rust phase.
- `services/api/app/chat.py` remains the active evidence-answer API route and
  response implementation.
- This DIFF adds Rust in-memory answer-packet parity only.

## Allowed Scope

- `docs/diffs/DIFF-099-rust-evidence-answer.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-evidence-answer`
- `Cargo.lock` workspace package metadata
- `crates/igy6-evidence-answer/`
- `configs/rust-cutover-manifest.json` `evidence_answer` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python/FastAPI replacement.
- No DB reads or writes.
- No Qdrant or graph calls.
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

- Commit message must include `DIFF-099`.
- Final response must identify `DIFF-099`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-evidence-answer`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust evidence-answer packet returns `evidence_summary` when facts exist.
- Rust evidence-answer packet returns `insufficient_evidence` when no facts
  exist.
- Evidence items are preferred over chunk excerpts for facts.
- Chunk fallback facts are excerpted deterministically.
- Source trails are deduplicated by document/chunk.
- Confidence is bounded and combines retrieval score with evidence confidence
  when present.
- Tests cover evidence facts, chunk fallback, insufficient evidence, source
  trail dedupe, confidence bounds, and inference text.
- Manifest `evidence_answer` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-099 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-evidence-answer` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- FastAPI route replacement, live retrieval execution, graph inference,
  prediction/recommendation generation, external model use, or write APIs.
