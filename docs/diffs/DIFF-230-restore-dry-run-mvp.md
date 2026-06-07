# DIFF-230 - Restore Dry-Run MVP

Status: Complete

## Scope

DIFF-230 validates a DIFF-229 backup/export bundle without restoring it. This
is a read-only dry-run validator. It does not write restored records into
PostgreSQL, artifact storage, Qdrant, Neo4j, Redis, MLflow, Phoenix, or any
runtime data directory.

## Current Support Found

- DIFF-229 introduced `igy6.backup_export.v1` metadata-only JSON bundles under
  `.igy6-local/exports/`.
- The export format has top-level schema, timestamp, repo, export metadata,
  classes, and warning fields.
- Exported class records are sanitized metadata, not raw artifact bytes or
  service-level database dumps.

## Product Behavior Added

- Added `scripts/restore-dry-run-mvp.sh`.
- The script accepts `--bundle PATH` or `--latest` for the newest bundle under
  `.igy6-local/exports/`.
- The dry-run validator:
  - validates `schema_version`;
  - validates required top-level fields;
  - reports repo branch/head from the bundle;
  - reports record classes and counts;
  - reports what would be restored;
  - reports unsupported classes;
  - reports absent supported classes;
  - reports declared-count mismatches;
  - scans for non-excluded secret-shaped fields;
  - scans for content/body/text-shaped fields that were not excluded;
  - scans for raw local absolute path hints;
  - exits nonzero on malformed or incompatible bundles.
- Added a synthetic safe fixture at
  `tests/fixtures/backup-export-safe-bundle-v1.json`.

## Safety Behavior

- The script reads one JSON bundle and prints a summary.
- It never opens `.env`.
- It never connects to PostgreSQL, Qdrant, Neo4j, Redis, MLflow, Phoenix, or
  Docker.
- It never writes runtime data or restored records.
- It does not create hidden import behavior.

## Explicit Non-Claims

- Restore is not implemented.
- Restore conflict handling, dependency ordering, artifact reconciliation,
  vector restore, graph restore, and rollback are not implemented.
- A passing dry-run means the bundle shape is compatible with this MVP
  validator; it does not prove destructive restore readiness.

## Files Changed

- `scripts/restore-dry-run-mvp.sh`
- `tests/fixtures/backup-export-safe-bundle-v1.json`
- `docs/diffs/DIFF-230-restore-dry-run-mvp.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `bash -n scripts/restore-dry-run-mvp.sh`
- `scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-safe-bundle-v1.json`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

Product/lifecycle script plus synthetic fixture and docs. No new API route,
schema, persistence, worker behavior, or UI behavior.

## Scope Confirmation

- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No arbitrary command execution from user text was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete or destructive restore was performed.
- No unsafe backup archive was created.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
