# DIFF-229 - Backup Export MVP

Status: Complete

## Scope

DIFF-229 creates the first safe local backup/export MVP for IGY6 records. The
MVP is a metadata-only local script export. It does not restore, delete, mutate
runtime records, copy raw artifact bytes, dump database files, include `.env`,
or include runtime/private data.

## Current Support Found

- DIFF-228 mapped the data lifecycle classes and explicitly left archive
  creation, restore, and delete unimplemented.
- The Rust gateway already exposes read routes for the metadata classes needed
  by a first export pass.
- Report rendering can create markdown artifacts, but raw artifact bytes and
  rendered contents are sensitive and are excluded from this MVP export.

## Product Behavior Added

- Added `scripts/backup-export-mvp.sh`.
- The script reads allowlisted local API GET routes and writes a JSON bundle to
  `.igy6-local/exports/`.
- The bundle includes:
  - `schema_version`;
  - `created_at_utc`;
  - current repo branch and HEAD when available;
  - included record classes;
  - endpoint paths;
  - record counts;
  - sanitized metadata records;
  - unavailable-route warnings.
- Export classes include sources, source permissions, approvals, audit events,
  collection run metadata, artifact metadata, document/chunk/evidence metadata,
  claims, answer records, feedback, outcomes, work items, task plans, reports,
  patterns, hypotheses, predictions, recommendations, improvements, and
  experiments where the local API route is available.
- Added `.igy6-local/exports/` to `.gitignore` so generated local bundles are
  not committed.

## Safety Behavior

- Secret-shaped fields are excluded.
- Content/body/text-shaped fields are excluded rather than exported as raw
  source text.
- Path/location/url/uri-shaped string fields are redacted.
- Raw artifact bytes, raw document text, raw chunk text, raw evidence text, raw
  answer text, and raw report markdown are excluded.
- `.env`, `.env` backups, Docker volumes, database dumps, Qdrant, Neo4j,
  MLflow, Phoenix, and `IGY6_DATA_ROOT` content are excluded.

## Explicit Non-Claims

- This is not a complete backup system.
- This is not a destructive restore path.
- This is not a database dump or service-level backup.
- This does not prove a bundle can be restored.
- Full operator live-stack export verification is pending owner WSL smoke or
  owner-run export against a running local stack.

## Files Changed

- `.gitignore`
- `scripts/backup-export-mvp.sh`
- `docs/diffs/DIFF-229-backup-export-mvp.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `bash -n scripts/backup-export-mvp.sh`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

Product/lifecycle script plus docs. No new API route, schema, persistence,
worker behavior, or UI behavior.

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
