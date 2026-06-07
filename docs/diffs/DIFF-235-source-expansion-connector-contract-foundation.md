# DIFF-235 - Source Expansion And Connector Contract Foundation

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `d78f564 Complete DIFF-234 defer promotion consolidate next build phase`
- `dev` tracking state before work: aligned with `origin/dev`
- Working tree before work: clean

## Purpose

Define a source and connector contract foundation for future source expansion
without enabling live browser scraping, account scraping, router access,
external service collection, credential capture, or hidden collection.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-234-defer-promotion-consolidate-next-build-phase.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- Relevant source/connector/search results under `apps/web`, `crates`, `docs`,
  `scripts`, `infra`, and `tests`
- Private/dev tracked file list from `git ls-files`

## Implementation

- Added a product-facing Source And Connector Contract panel in Add Data.
- Defined required collector behavior:
  - `validate_scope`
  - `dry_run`
  - `collect`
  - `normalize`
  - `classify_sensitivity`
  - `extract_metadata`
  - `cleanup`
  - `audit`
- Added an honest source-type posture matrix:
  - `manual_upload`: implemented
  - `conversation_history`: implemented
  - `user_observation`: implemented
  - `local_project`: partial
  - `browser_export`: planned-disabled
  - `web_public`: planned-disabled
  - `router_network`: planned-disabled
  - `local_pc_diagnostics`: planned-disabled
  - `media_import`: planned-disabled
- Updated source help copy so planned connector-backed types are not described
  as active collectors.
- Updated the UI guide to explain the contract/status surface and its limits.

## Scope Confirmation

This is a UI and UI documentation product-building DIFF. It establishes the
contract and normal-user visibility required before later collectors become
active.

No backend behavior, persistence schema, worker runtime, source collection
route, browser collector, web fetcher, router collector, media parser, PC
diagnostics collector, external service call, hosted AI call, account scraping,
credential/cookie/token collection, router write, arbitrary filesystem crawl,
or arbitrary command execution was added.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`

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
