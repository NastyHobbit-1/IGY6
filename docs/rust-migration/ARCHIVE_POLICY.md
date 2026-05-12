# Rust Cutover Archive Policy

## Core Rule

Deprecated files are moved, not deleted. Final cutover must preserve legacy
context so future maintainers can inspect how behavior changed.

## Archive Timing

Archive moves happen only during the final Rust cutover DIFF after:

- Rust replacements exist.
- Verification has passed.
- The manifest has `cutover_ready: true`.
- The active DIFF explicitly authorizes final cutover.

This migration-control DIFF does not archive anything.

## Archive Layout

Recommended final layout:

```text
archive/
  legacy-python/
  legacy-worker/
  legacy-web/
  legacy-docs/
  legacy-scripts/
  migration-records/
```

The final manifest should define exact moves before `--execute` is allowed.

## Old README And Docs

Old README and legacy docs must be preserved under `archive/legacy-docs/`.

Example:

```text
README.md -> archive/legacy-docs/README.legacy.md
docs/api.md -> archive/legacy-docs/api.md
docs/architecture.md -> archive/legacy-docs/architecture.md
docs/operations.md -> archive/legacy-docs/operations.md
docs/user-guide.md -> archive/legacy-docs/user-guide.md
```

If a source doc does not exist, the script should warn and skip that move
rather than failing unexpectedly.

## Files That Remain Active

These remain active and must not be archived by the Rust cutover script:

- `AGENTS.md`
- `docs/diffs/`
- `docs/agents/`

These preserve repository governance and agent operating instructions.

## Runtime And Private Data

Runtime/private data under `IGY6_DATA_ROOT` must not be archived, moved, copied,
deleted, or modified by the cutover script.

The cutover script must not touch:

- `.env`
- `IGY6_DATA_ROOT`
- `storage/`
- `storage/artifacts/`
- `storage/exports/`
- `storage/env_backups/`
- database, Qdrant, Neo4j, MLflow, or Phoenix runtime data folders

## Move Mechanism

Archive moves must use `git mv` so history remains visible and reviewable.

The script must not use file deletion as part of archive cleanup. If a file is
untracked or missing, the script should report that clearly and skip or refuse
based on the final DIFF's policy.

## Legacy Services

Do not archive `services/api/`, `services/worker/`, legacy Python tests, or
legacy migrations until Rust replacements are complete and the manifest says
final cutover is ready.

## Fresh Rust Docs

Final cutover should create these if missing:

- `README.md`
- `docs/RUST_ARCHITECTURE.md`
- `docs/RUST_OPERATIONS.md`
- `docs/RUST_API.md`
- `docs/RUST_MIGRATION_COMPLETE.md`

Creation must not overwrite existing files.
