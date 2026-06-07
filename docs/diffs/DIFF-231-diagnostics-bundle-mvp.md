# DIFF-231 - Diagnostics Bundle MVP

Status: Complete

## Scope

DIFF-231 creates a safe diagnostics bundle for support/troubleshooting. It
contains repo and local runtime posture summaries only. It excludes `.env`,
secrets, raw runtime data, raw artifact/document/chunk/evidence contents,
service database dumps, Docker volume data, full logs, and raw smoke result
JSON.

## Current Support Found

- Existing operator smoke tooling records safe summaries under
  `.igy6-local/smoke-results/` when the owner runs it with `--record`.
- Existing status and smoke scripts already avoid printing `.env` contents and
  runtime/private data.
- The Rust gateway exposes safe status routes such as health, migration status,
  vector status, and graph schema status when the local stack is running.

## Product Behavior Added

- Added `scripts/diagnostics-bundle-mvp.sh`.
- The script writes a JSON diagnostics bundle under
  `.igy6-local/diagnostics/` by default.
- The bundle includes:
  - schema version;
  - creation time;
  - repo branch/head;
  - safe git dirty summary with counts/prefixes only;
  - active runtime posture labels;
  - localhost route health summaries when the API is running;
  - local dependency/tool presence booleans;
  - latest recorded operator smoke summary if present.
- Added `.igy6-local/diagnostics/` to `.gitignore`.

## Safety Behavior

- The script never opens `.env`.
- The script never prints or records absolute tool paths.
- Route checks use localhost GET requests only.
- Smoke result handling records selected scalar summary fields only, not raw
  JSON, logs, uploaded text, or private data.
- Missing local services are recorded as unavailable rather than treated as
  product failure in Codex.

## Explicit Non-Claims

- This is not full live-stack verification.
- This does not replace owner-run WSL smoke.
- This does not collect runtime/private evidence, artifacts, logs, or database
  dumps.
- Route health summaries are best-effort and depend on whether the local stack
  is already running.

## Files Changed

- `.gitignore`
- `scripts/diagnostics-bundle-mvp.sh`
- `docs/diffs/DIFF-231-diagnostics-bundle-mvp.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `bash -n scripts/diagnostics-bundle-mvp.sh`
- `scripts/diagnostics-bundle-mvp.sh --dry-run`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

Product/lifecycle diagnostics script plus docs. No new API route, schema,
persistence, worker behavior, or UI behavior.

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
