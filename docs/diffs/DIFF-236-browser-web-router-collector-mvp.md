# DIFF-236 - Browser / Web / Router Collector MVP

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `b03b5e7 Complete DIFF-235 source expansion connector contract foundation`
- `dev` tracking state before work: ahead of `origin/dev` by 1 commit
- Working tree before work: clean

## Purpose

Add the safest browser/web/router collection MVP surface with explicit user
scope, dry-run preview, read-only posture, approval guidance, sensitivity
warning, and audit expectations without enabling hidden collection.

## Files Inspected

- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- Existing source, permission, approval, dry-run, collection run, artifact,
  document, chunk, and evidence UI flows inspected during the batch pre-work

## Implementation

- Added a Browser / Web / Router Import Dry-Run MVP in Add Data.
- Supported safe preview categories:
  - browser page text export
  - web page text
  - router status/export text
- The preview requires:
  - explicit user-entered scope;
  - manually pasted authorized text;
  - local dry-run action by the user.
- The preview reports:
  - scope entered;
  - read-only manual import posture;
  - what would be collected;
  - what is excluded;
  - approval posture;
  - sensitivity warning;
  - approximate text size;
  - audit expectations for any later real collection.
- Updated the UI guide with the new dry-run behavior and limits.

## Scope Confirmation

This is a UI-only product MVP for dry-run/manual preview. It does not start a
collection run and does not add backend collector behavior.

No HTTP fetch, crawler, browser profile read, cookie/token/session collection,
credential collection, account scraping, router login, router write, router
configuration change, network scan, hidden external request, hosted AI call,
arbitrary filesystem crawl, arbitrary command execution, or runtime/private data
dump was added.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-236-browser-web-router-collector-mvp.md`

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Rust checks were not required because no Rust files changed.
- Script syntax checks were not required because no scripts changed.
- Full Docker smoke was not run from Codex per owner instruction.

## Verification Summary

- Next.js production build passed.
- Working-tree whitespace check passed.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope draft/status strings in
  historical DIFF records and command examples; this DIFF is
  `Status: Complete`.
