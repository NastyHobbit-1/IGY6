# DIFF-103: Final Rust Cutover

Status: Locked

## Type

Change-bearing

## Objective

Execute the final Rust cutover workflow after DIFF-102 made the Rust gateway the
primary `api` service and marked `cutover_ready` true. This DIFF authorizes an
explicit archive/keep plan, final dry-run and execute checks, and rollback
documentation.

## Baseline Facts

- DIFF-102 is locked and committed.
- `configs/rust-cutover-manifest.json` has all required phases complete and
  `cutover_ready` set to true.
- `infra/docker-compose.yml` builds the Rust gateway as `api` and keeps FastAPI
  as `legacy-api` fallback.
- The Rust gateway still delegates unsupported routes to FastAPI fallback, so
  Python API files are not verified as deprecated for archival.
- `scripts/rust-cutover.sh --execute` requires a clean git worktree before it
  applies the manifest archive/create plan.

## Allowed Scope

- Update this DIFF file from Active to Locked after verification.
- Update `configs/rust-cutover-manifest.json` archive plan only.
- Add or update final cutover/rollback documentation under
  `docs/rust-migration/`.
- Run `scripts/rust-cutover.sh --plan`, `--dry-run`, and `--execute`.
- Commit DIFF-103 plan and completion changes with DIFF-103 in commit messages.

## Prohibited Scope

- No locked DIFF edits.
- No runtime/private data reads or writes.
- No `.env` content reads or writes.
- No Docker Compose rewrite beyond the already committed DIFF-102 gateway
  configuration.
- No database migrations.
- No dependency additions.
- No deletion.
- No archiving of FastAPI, workers, governance files, docs/diffs, docs/agents,
  runtime storage, or private data.
- No disabling FastAPI fallback while unsupported routes still depend on it.
- No marking unimplemented replacement behavior complete.

## Required Tags

Commit messages and final summaries must include `DIFF-103`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `scripts/rust-cutover.sh --check`
- `scripts/rust-cutover.sh --plan`
- `scripts/rust-cutover.sh --dry-run`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line if any are changed.
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `scripts/rust-cutover.sh --execute` from a clean worktree.

## Completion Criteria

- Final cutover archive plan is explicit.
- Rollback expectations are documented.
- Rust checks, web build, and Docker Compose config validation pass.
- `scripts/rust-cutover.sh --dry-run` passes.
- `scripts/rust-cutover.sh --execute` runs from a clean worktree.
- Runtime/private data, `.env`, governance files, FastAPI fallback, and worker
  files are not archived.
- DIFF-103 is locked after verification.

## Completion Notes

- `scripts/rust-cutover.sh --execute` completed successfully from a clean
  worktree.
- The manifest archive plan had no move or create actions.
- FastAPI remains active as `legacy-api` fallback because unsupported routes
  still depend on it.

## Out Of Scope Follow-Up

- Full Rust implementation of every remaining FastAPI fallback route.
- Archiving Python API or worker files.
- Removing the legacy FastAPI service from Docker Compose.
