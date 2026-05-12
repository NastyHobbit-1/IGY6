# Rust Cutover Checklist

This checklist is for the final Rust cutover DIFF. It is not authorization to
run cutover early.

## Pre-Cutover Checks

- Confirm the active DIFF explicitly authorizes final cutover.
- Confirm all locked DIFFs remain untouched.
- Confirm `configs/rust-cutover-manifest.json` has `cutover_ready: true`.
- Confirm every required phase is `complete`.
- Confirm every complete phase lists verification commands.
- Confirm runtime/private data under `IGY6_DATA_ROOT` is not inside the archive
  plan.
- Confirm `.env` is not modified by the cutover plan.
- Confirm `AGENTS.md`, `docs/diffs/`, and `docs/agents/` remain active.

## Required Rust Verification

When a Cargo workspace exists, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

If the final Rust stack uses package-specific checks, record those commands in
the final DIFF and manifest.

## Required Frontend Verification

Run the active web verification command:

```bash
npm --prefix apps/web run build
```

If the web UI is replaced or archived in a later DIFF, record the replacement
verification command in that DIFF before changing the checklist.

## Required API Verification

Verify the Rust API preserves required routes and safety behavior:

- `/health/live`
- `/health/ready`
- `/agent/capabilities`
- `/chat/retrieval-preview`
- `/chat/evidence-answer`
- Source, approval, audit, feedback, outcome, work-item, report, evidence, and
  memory routes that are active at cutover.

Verify approval-required actions still require approval and arbitrary shell
requests are still rejected.

## Required Docker/Compose Verification

Before final execution:

```bash
docker compose -f infra/docker-compose.yml --env-file .env config
```

After cutover rewrites Compose:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

Then check service readiness and stop the stack safely. Do not run destructive
Docker cleanup, volume deletion, or prune commands.

## Git Cleanliness Requirement

Final execution requires a clean worktree before `--execute`:

```bash
git status --short
```

The script must refuse execution if the worktree is dirty.

## Dry-Run Process

Run:

```bash
scripts/rust-cutover.sh --check
scripts/rust-cutover.sh --plan
scripts/rust-cutover.sh --dry-run
```

Review the planned archive moves, rewrites, and created docs. Confirm no runtime
data, `.env`, locked DIFF files, or active governance files are moved.

## Execute Process

Only inside the final cutover DIFF:

```bash
scripts/rust-cutover.sh --execute
```

Then inspect:

```bash
git status --short
git diff --stat
git diff --check
```

Run all final Rust, frontend, API, and Docker checks again.

## Post-Cutover Checks

- Confirm fresh Rust docs exist:
  - `README.md`
  - `docs/RUST_ARCHITECTURE.md`
  - `docs/RUST_OPERATIONS.md`
  - `docs/RUST_API.md`
  - `docs/RUST_MIGRATION_COMPLETE.md`
- Confirm old README/docs are preserved under `archive/legacy-docs/`.
- Confirm deprecated Python files were moved, not deleted.
- Confirm `IGY6_DATA_ROOT` data is untouched.
- Confirm the web UI still reaches the Rust API.
- Confirm audit, approval, source-permission, and evidence behavior remain
  permissioned and local-first.

## Commit And Push

After final verification:

```bash
git add .
git commit -m "DIFF-101 final Rust cutover"
git push origin main
```

Use the actual active DIFF number if final cutover happens under a different
DIFF.
