# DIFF-253 fix runtime smoke

## Objective

Fix the stale UI runtime smoke expectations so the test matches the current IGY6 web UI.

## Scope

Changed only:

- apps/web/scripts/ui-runtime-smoke.mjs

## Root Cause

The runtime smoke test still expected the old visible Home tab and old Add Data / Data & Knowledge wording. It also used a broad fatal-text regex that matched normal UI text.

## Changes

- Removed Home from required visible tab labels.
- Updated Add Data / Data & Knowledge expectation to current Data / Bring In Authorized Information wording.
- Narrowed crash signature regex to real crash text only.
- Left application behavior unchanged.

## Verification

Run:

- git diff --check
- npm --prefix apps/web run typecheck
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-runtime-smoke
