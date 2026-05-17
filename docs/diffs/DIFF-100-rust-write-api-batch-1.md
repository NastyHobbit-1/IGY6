# DIFF-100: Rust Write API Batch 1

Status: Locked

## Type

Change-bearing

## Objective

Add a Rust write API batch 1 foundation for sources, approvals, audit,
feedback, and outcomes while keeping Python/FastAPI as the active primary API
gateway.

This DIFF moves typed write contracts and deterministic validation/policy
behavior toward Rust. It does not execute database writes, call external
services, read or write runtime/private data, or make Rust the primary gateway.

## Baseline Facts

- DIFF-099 is locked and marks the `evidence_answer` Rust migration phase
  complete.
- `configs/rust-cutover-manifest.json` has `write_api_batch_1` pending and
  `cutover_ready` false.
- The current Python/FastAPI routes for sources, approvals, audit, feedback,
  and outcomes remain active.
- The root Rust workspace exists and contains Rust migration crates through
  `crates/igy6-evidence-answer`.

## Allowed Scope

- Create `docs/diffs/DIFF-100-rust-write-api-batch-1.md`.
- Add `crates/igy6-write-api/`.
- Update root `Cargo.toml` workspace membership.
- Update `Cargo.lock` as required.
- Update `configs/rust-cutover-manifest.json` only for the
  `write_api_batch_1` phase.
- Update `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for
  accuracy.
- Add or update Rust equivalent snippet JSONL records under:
  - `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
  - `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- Locked DIFF edits.
- Docker Compose changes.
- `.env` changes or `.env` content reads.
- Database migrations.
- Runtime/private data reads or writes.
- File deletion.
- Archive actions.
- Python/FastAPI removal or primary gateway switch.
- Marking `cutover_ready` true.
- Marking future Rust migration phases complete.
- External service calls.
- Arbitrary shell execution.

Unless explicitly allowed here, the following are prohibited:

- Renames.
- Refactors.
- Behavior changes outside the Rust parity foundation.
- Rewiring.
- Redesign.
- Dependency changes beyond this crate's justified workspace-local needs.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

## Required Tags

Use `DIFF-100` in the commit message and final change summary.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-write-api`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- `crates/igy6-write-api/` exists and is included in the workspace.
- The crate defines typed Rust request/response structures for sources,
  approvals, audit, feedback, and outcomes.
- Tests cover source validation, approval transitions, approval-gated actions,
  audit event construction, feedback validation/side-effect planning, and
  outcome validation/side-effect planning.
- `configs/rust-cutover-manifest.json` marks `write_api_batch_1` complete only
  after verification passes.
- `cutover_ready` remains false.
- Python/FastAPI remains active.
- Snippet JSONL additions are valid line-delimited JSON.
- This DIFF is locked after verification passes.

## Out Of Scope Follow-Up

- Rust primary API gateway.
- Live database writes.
- FastAPI route replacement.
- Work queue and report write APIs.
- Docker Compose wiring for a Rust gateway.
- Python archival or deletion.
