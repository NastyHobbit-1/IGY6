# DIFF-085: Rust Migration Cutover Plan

Status: Locked

## Type

Change-bearing documentation and script scaffold

## Objective

Create the Rust migration documentation, cutover manifest, and safe final
cutover script scaffold without replacing current Python, worker, Docker, or web
behavior.

## Baseline Facts

- DIFF-084 is locked.
- No tracked Cargo workspace or `crates/` tree exists before this DIFF.
- `docs/rust-migration/` does not exist before this DIFF.
- `IGY6_Rust_Migration_and_Cutover_Plan.md` was referenced by the user but is
  not present in the repository root at inspection time.
- Current runtime behavior is Python/FastAPI, Celery worker, Next.js, and Docker
  Compose.
- Rust migration must be additive and DIFF-governed until Rust replacements
  exist, are verified, and a later active DIFF explicitly allows replacement.

## Allowed Scope

- `docs/diffs/DIFF-085-rust-migration-cutover-plan.md`
- `docs/rust-migration/RUST_MIGRATION_PLAN.md`
- `docs/rust-migration/CUTOVER_MANIFEST.md`
- `docs/rust-migration/CUTOVER_CHECKLIST.md`
- `docs/rust-migration/ARCHIVE_POLICY.md`
- `configs/rust-cutover-manifest.json`
- `scripts/rust-cutover.sh`

## Prohibited Scope

- No locked DIFF edits.
- No backend rewrite.
- No FastAPI removal.
- No worker removal.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No archive execution.
- No file deletion.
- No Rust service replacement.
- No broad refactor.
- No arbitrary shell control.
- No Docker socket mounting.
- No external model calls.
- No runtime/private data moves.

## Required Tags

- Commit message must include `DIFF-085`.
- Final response must identify `DIFF-085`.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/rust-cutover.sh`
- `scripts/rust-cutover.sh --help`
- `scripts/rust-cutover.sh --check`
- `scripts/rust-cutover.sh --plan`
- `scripts/rust-cutover.sh --dry-run`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- If a Cargo workspace exists, run:
  - `cargo test --workspace`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets`

## Completion Criteria

- Rust migration documentation exists under `docs/rust-migration/`.
- `configs/rust-cutover-manifest.json` is valid deterministic JSON and keeps
  `cutover_ready` false.
- `scripts/rust-cutover.sh` defaults to check-only behavior and supports
  `--check`, `--plan`, `--dry-run`, `--execute`, and `--help`.
- `--execute` refuses unless the manifest exists, the worktree is clean,
  `cutover_ready` is true, and required verification passes.
- No archive moves are executed in this DIFF.
- No Python/FastAPI/worker/Docker/web behavior is replaced.
- Verification results are recorded below before locking.

## Verification Result

- `git status --short`: showed only DIFF-085 scoped untracked files before
  staging.
- `git diff --check`: passed.
- `bash -n scripts/rust-cutover.sh`: passed.
- `scripts/rust-cutover.sh --help`: passed and printed supported modes and
  safety notes.
- `scripts/rust-cutover.sh --check`: passed. It warned that `Cargo.toml` does
  not exist yet and that `cutover_ready` is false.
- `scripts/rust-cutover.sh --plan`: passed and printed all required phases as
  pending with empty archive-plan arrays.
- `scripts/rust-cutover.sh --dry-run`: passed. It performed no archive moves
  because the manifest archive plan is empty.
- `scripts/rust-cutover.sh` with no arguments: behaved like `--check`.
- `scripts/rust-cutover.sh --execute`: refused because the git worktree was not
  clean, confirming execution gating during this DIFF.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- Cargo verification was not run because no Cargo workspace exists yet.
