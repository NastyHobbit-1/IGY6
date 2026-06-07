# DIFF-244 - Data Lifecycle Hardening And Release Readiness

Status: Complete

## Scope

DIFF-244 hardens the existing metadata export, restore dry-run, diagnostics,
normal-user product smoke, and release-readiness lifecycle checks. It does not
promote files, touch `main`, restore records, delete records, create service
backups, or claim production release readiness.

## Current Support Found

- DIFF-229 added metadata-only export under `.igy6-local/exports/`.
- DIFF-230 added restore dry-run validation for `igy6.backup_export.v1`
  bundles.
- DIFF-231 added safe diagnostics bundle generation under
  `.igy6-local/diagnostics/`.
- DIFF-232 added a normal-user product smoke checklist and Codex-safe marker
  check.
- DIFF-233 tightened product claims around backup, restore, delete,
  diagnostics, smoke, graph reasoning, forecasting, and self-improvement.

## Product Behavior Added

- Backup export now validates the sanitized bundle before writing. It fails
  closed if secret-shaped values, non-excluded sensitive/content keys, or raw
  private path hints remain after sanitization.
- Restore dry-run now supports `--strict-safety`. In strict mode, secret-shaped
  fields, raw content fields, or private path hints produce a nonzero exit
  without writing restored records.
- Restore dry-run private path scanning was narrowed to private/local path
  prefixes instead of treating safe API route paths as raw filesystem paths.
- Diagnostics bundle generation now performs a self-redaction check before
  writing or printing a dry-run summary.
- Added a synthetic unsafe bundle fixture to verify strict restore dry-run
  rejection.
- Added `docs/runtime/RELEASE_READINESS_CHECKLIST.md`.
- Added `scripts/normal-user-product-smoke.sh --release-readiness-check` to
  verify release-readiness files and product-smoke UI markers without Docker.
- Updated the UI guide to describe strict export/restore/diagnostics safety
  validation and deferred promotion.

## Safety Behavior

- No runtime database, artifact store, Qdrant, Neo4j, Redis, MLflow, Phoenix,
  Docker volume, or `IGY6_DATA_ROOT` writes are performed.
- Restore remains dry-run only.
- Delete and retention destructive behavior remain unsupported.
- Backup export remains metadata-only and excludes `.env`, secrets, raw
  artifact bytes, raw evidence/document/chunk contents, runtime private data,
  and private path details.
- Diagnostics remain summary-only and do not include raw logs, runtime records,
  or service data dumps.

## Explicit Non-Claims

- This DIFF does not implement destructive restore.
- This DIFF does not implement destructive delete or retention enforcement.
- This DIFF does not create a complete backup/archive system.
- This DIFF does not verify the live stack in Codex.
- This DIFF does not mark IGY6 production ready.
- Promotion can be reconsidered only after explicit owner instruction.

## Files Changed

- `scripts/backup-export-mvp.sh`
- `scripts/restore-dry-run-mvp.sh`
- `scripts/diagnostics-bundle-mvp.sh`
- `scripts/normal-user-product-smoke.sh`
- `tests/fixtures/backup-export-unsafe-bundle-v1.json`
- `docs/runtime/RELEASE_READINESS_CHECKLIST.md`
- `docs/ui/README.md`
- `docs/diffs/DIFF-244-data-lifecycle-hardening-release-readiness.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `bash -n scripts/backup-export-mvp.sh`
- `bash -n scripts/restore-dry-run-mvp.sh`
- `bash -n scripts/diagnostics-bundle-mvp.sh`
- `bash -n scripts/normal-user-product-smoke.sh`
- `scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-safe-bundle-v1.json --strict-safety`
- `if scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-unsafe-bundle-v1.json --strict-safety >/tmp/igy6-unsafe-restore-check.out 2>&1; then printf 'FAIL unsafe fixture was accepted\n'; exit 1; else printf 'PASS strict safety rejected unsafe fixture\n'; fi`
- `scripts/diagnostics-bundle-mvp.sh --dry-run`
- `scripts/normal-user-product-smoke.sh --release-readiness-check`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

Script/lifecycle behavior plus docs. No new API route, persistence schema,
worker behavior, or live-stack verification.

## Scope Confirmation

- No hosted AI call was added.
- No hidden external data transfer was added.
- No browser/account scraping, connector import, credential/cookie/token
  collection, or arbitrary command execution from user text was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete or destructive restore was performed.
- No unsafe backup archive was created.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
