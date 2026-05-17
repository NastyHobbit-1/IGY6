# DIFF-095: Rust Worker MVP

Status: Locked

## Type

Change-bearing Rust worker planning foundation

## Objective

Add a Rust worker MVP that composes artifact normalization, chunk planning, and
vector-upsert request planning without replacing the active Python/Celery
worker runtime.

## Baseline Facts

- DIFF-094 completed the Rust `vector_memory` phase.
- The manifest shows `worker` as the next pending Rust phase.
- `services/worker/app/tasks.py` remains the active worker runtime for
  database updates, audit writes, work-item status transitions, and Qdrant
  calls.
- This DIFF adds deterministic Rust worker planning only.

## Allowed Scope

- `docs/diffs/DIFF-095-rust-worker-mvp.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-worker`
- `Cargo.lock` workspace package metadata
- `crates/igy6-worker/`
- `configs/rust-cutover-manifest.json` `worker` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python/Celery worker replacement.
- No worker process rewiring.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads.
- No live Qdrant calls during verification.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-095`.
- Final response must identify `DIFF-095`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-worker`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust worker planning can normalize UTF-8 artifact bytes into a normalized
  document reference.
- Invalid UTF-8 artifact bytes are rejected to preserve current Python worker
  behavior.
- Rust worker planning can chunk a normalized document and create deterministic
  chunk/evidence IDs.
- Rust worker planning can plan Qdrant vector points and an upsert request
  without making network calls.
- Chunk size and vector size errors are surfaced as structured worker errors.
- Tests cover valid end-to-end planning, invalid UTF-8, invalid chunk size,
  empty document text, deterministic IDs, and vector-upsert request planning.
- Manifest `worker` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-095 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-worker` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Celery, SQLAlchemy database writes, audit writes, work-item status
  updates, or live Qdrant calls.
- Rust worker service process management.
- Docker Compose changes for Rust worker runtime.
