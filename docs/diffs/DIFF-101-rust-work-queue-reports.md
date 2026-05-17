# DIFF-101: Rust Work Queue Reports

Status: Locked

## Type

Change-bearing

## Objective

Add a Rust work queue and reports foundation for work-items, dispatch planning,
reports, and report rendering while keeping Python/FastAPI and Celery active.

This DIFF moves typed contracts and deterministic validation/planning behavior
toward Rust. It does not execute Celery tasks, write databases, read runtime
data, render from live PostgreSQL records, call external services, or make Rust
the primary gateway.

## Baseline Facts

- DIFF-100 is locked and marks `write_api_batch_1` complete.
- `configs/rust-cutover-manifest.json` has `work_queue_reports` pending and
  `cutover_ready` false.
- Python/FastAPI work-item and report routes remain active.
- The current Rust workspace includes `crates/igy6-write-api` with reusable
  audit event draft structures.

## Allowed Scope

- Create `docs/diffs/DIFF-101-rust-work-queue-reports.md`.
- Add `crates/igy6-work-queue-reports/`.
- Update root `Cargo.toml` workspace membership.
- Update `Cargo.lock` as required.
- Update `configs/rust-cutover-manifest.json` only for the
  `work_queue_reports` phase.
- Update `docs/rust-migration/RUST_MIGRATION_PLAN.md` only for accuracy.
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
- Python/FastAPI or Celery removal.
- Primary API gateway switch.
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
- Dependency changes beyond workspace-local crate usage.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

## Required Tags

Use `DIFF-101` in the commit message and final change summary.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-work-queue-reports`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- `crates/igy6-work-queue-reports/` exists and is included in the workspace.
- The crate defines typed Rust request/response structures for work-items,
  dispatch, reports, report status updates, report work-items, and report
  rendering plans.
- Tests cover work-item intent verification, status transitions, dispatch
  payload validation, report type/status validation, report work-item planning,
  report markdown rendering, and audit event construction.
- `configs/rust-cutover-manifest.json` marks `work_queue_reports` complete only
  after verification passes.
- `cutover_ready` remains false.
- Python/FastAPI and Celery remain active.
- Snippet JSONL additions are valid line-delimited JSON.
- This DIFF is locked after verification passes.

## Out Of Scope Follow-Up

- Rust primary API gateway.
- Live database writes.
- Celery replacement or execution.
- Docker Compose wiring.
- Python archival or deletion.
- Final cutover execution.
