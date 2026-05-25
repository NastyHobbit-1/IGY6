# Rust Cutover Manifest

## Purpose

`configs/rust-cutover-manifest.json` is the source of truth for the Rust
migration cutover state. It records which Rust phases are required, which phases
are pending or complete, and what archive/rewrite/create actions the final
cutover script may perform.

The manifest is deliberately conservative. It starts with `cutover_ready: false`
and all phases pending.

## `cutover_ready`

`cutover_ready` must remain `false` until all required Rust phases are complete
and verified.

When `cutover_ready` is false:

- `scripts/rust-cutover.sh --check` warns that final cutover is blocked.
- `scripts/rust-cutover.sh --plan` can still print the current manifest plan.
- `scripts/rust-cutover.sh --dry-run` can show what would happen, without
  changing files.
- `scripts/rust-cutover.sh --execute` must refuse to run.

Only a later final cutover DIFF may set `cutover_ready` to true.

## Phase Status

Allowed phase status values are:

- `pending`: no verified Rust replacement exists yet.
- `partial`: a Rust component exists but does not fully replace the legacy
  behavior.
- `complete`: Rust behavior exists, parity is documented, and verification has
  passed.

Do not mark a phase complete because work is planned. Complete means tested
code is present.

## Adding Completed Rust Phases

When a DIFF completes a Rust phase, update the phase entry with the DIFF number,
crate or component path, and verification commands.

Example:

```json
{
  "status": "complete",
  "diff": "DIFF-086",
  "crate": "crates/igy6-host-bridge",
  "verification": [
    "cargo test -p igy6-host-bridge",
    "curl http://127.0.0.1:8765/health"
  ]
}
```

If parity is incomplete, use `partial` and document the missing behavior.

## Archive Plan

The `archive_plan` object has four arrays:

- `move`: tracked files or directories to move into `archive/` during final
  cutover.
- `keep`: active files or directories that must not be archived.
- `rewrite`: files that final cutover or later DIFFs must rewrite.
- `create_if_missing`: fresh Rust-facing docs or files to create if absent.

The initial archive plan is empty. Future DIFFs should add entries only when
the corresponding Rust replacement is verified.

## Move Entries

Move entries should use object form:

```json
{
  "from": "services/api",
  "to": "archive/legacy-python/services-api",
  "after_phase": "rust_gateway",
  "reason": "FastAPI archived after Rust gateway parity"
}
```

Final cutover must use `git mv` for moves. Files are moved, not deleted.

## Keep Entries

Keep entries should document active governance and active Rust files:

```json
{
  "path": "docs/diffs",
  "reason": "DIFF history remains active after Rust cutover"
}
```

`docs/diffs/` remains active as locked project history. Build-agent instructions are not active on `main` and belong only on local `dev` unless a future
DIFF explicitly replaces the governance system.

## Rewrite Entries

Rewrite entries describe files that must be changed after Rust parity, such as
`infra/docker-compose.yml` when the Rust API gateway becomes primary.

Example:

```json
{
  "path": "infra/docker-compose.yml",
  "reason": "Switch API service from FastAPI to Rust Axum",
  "after_phase": "rust_gateway"
}
```

This DIFF does not rewrite Docker Compose.

## Create If Missing Entries

Final cutover should create fresh Rust-facing docs if missing:

- `README.md`
- `docs/RUST_ARCHITECTURE.md`
- `docs/RUST_OPERATIONS.md`
- `docs/RUST_API.md`
- `docs/RUST_MIGRATION_COMPLETE.md`

Creation entries should include a path and title. They should not overwrite
existing files.

## How The Cutover Script Reads The Manifest

`scripts/rust-cutover.sh` reads `configs/rust-cutover-manifest.json` in every
mode. It validates required phase metadata and checks `cutover_ready`.

The script is manifest-driven:

- `--check` validates readiness and warns on incomplete Rust workspace.
- `--plan` prints archive plan entries.
- `--dry-run` prints planned moves and creations without changing files.
- `--execute` refuses unless the worktree is clean and `cutover_ready` is true.

Future DIFFs may extend the script to process richer manifest entries, but must
preserve check-only default behavior.
