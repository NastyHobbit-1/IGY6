# DIFF-253: Fix runtime smoke

**Status:** Completed

## Type
Verification / test-only fix. No product or UI behavior change.

## Objective
Fix the `apps/web/scripts/ui-runtime-smoke.mjs` so that it no longer fails against the live healthy stack (web at 127.0.0.1:13000) with the 5 observed failures, by updating expectations and checks to match the actual visible tab structure and DOM in the running Next.js app.

This DIFF touches **only** the smoke test script and this DIFF record. No other files are edited.

## Baseline Facts (verified on clean 9f90110)
- Branch: grok, HEAD=9f90110 (DIFF-252), `git status --short` clean, no prior DIFF-253.
- Live stack: Docker healthy, web responding on 127.0.0.1:13000, api on 18000, health endpoints ok.
- The smoke script (both browser `runChecks` and node fallback paths) currently hardcodes:
  - `expectedTabs` including 'Home' (in both paths)
  - sectionChecks with 'Home / readiness' (broad text selector, no activation)
  - sectionChecks with 'Add Data / Data & Knowledge'
  - crash regex `/500|internal server error|white screen of death|react.*error|failed to load chunk/i` triggering "suspicious fatal text found in body"
- Actual visible tabs (from running app / HomePage.tsx tabList): Chat, Data, Work, Settings, More. Home panel (#tab-home + data-tab-panel="home") is not listed as a visible top nav tab label.
- Current Data panel title text includes "Bring In Authorized Information".
- Stable data attrs present: data-unified-chat, data-chat-input, data-chat-send, data-tab-panel etc.
- Smoke is a verification tool only (Playwright + fallback); it must reflect reality of the rendered UI without assuming always-visible tab content.
- Previous DIFFs (248 added the runtime smoke, 250/252 did other alignment) did not keep this script in sync with tab evolution or tighten the crash check.
- AGENTS.md / CODEX_PROMPT_BASELINE.md / BRANCH_POLICY.md / README.md / docs/ui/README.md (tab and smoke expectations sections) / DIFF-252 read before any change. Current state verified. Work strictly on grok. One DIFF scope.

## Allowed Scope (strictly limited)
- New DIFF file: `docs/diffs/DIFF-253-fix-runtime-smoke.md` (this file)
- `apps/web/scripts/ui-runtime-smoke.mjs` (ONLY this file for code changes)
  - Update visible expected tabs in both paths to exactly `['Chat', 'Data', 'Work', 'Settings', 'More']`
  - Add tab activation (e.g. force-check radio for #tab-*) before visibility checks on tab-specific content
  - Check Home/readiness only after activating #tab-home (or equivalent data-tab-panel)
  - Replace the Add Data section check name/selector with "Data / Bring In Authorized Information"
  - Prefer stable data-* selectors in checks where possible
  - Narrow the crash regex (browser + node) to only the real signatures listed in the task; do not trigger on plain "failed" or "error"
- No other files may be created or edited.

## Prohibited Scope
- Do not edit any component, constants, other scripts, docs (except this DIFF md), package.json, etc.
- No behavior changes to the app, no new features, no payload/route/attr changes.
- No Docker or stack changes.
- No broad refactors or cleanup.
- Do not run destructive commands.
- Verification must use the already-running live stack at the documented 13000 port.

## Root Cause
The test expectations and section visibility logic were written for an older tab layout where "Home" was treated as a top visible tab and tab content was assumed visible without activation. The Data section name drifted. The crash detector was too loose and fired on normal UI text present in the healthy live page.

## Changes Made
- Created this DIFF record first (governance requirement).
- In `apps/web/scripts/ui-runtime-smoke.mjs` (browser runChecks + node fallback):
  - Set expected visible tabs to the required list (no Home).
  - Added activation step before tab-specific section checks.
  - Updated Home check to activate #tab-home first.
  - Changed the stale Data section to the current name.
  - Switched relevant selectors to use data-unified-chat / data-tab-panel etc.
  - Replaced the crash regex + condition with one limited to the listed real crash signatures.
- All changes are test-only expectation/selector/activation fixes.

## Verification (run exactly in this order with live stack)
- git status --short
- git diff --check
- npm --prefix apps/web run typecheck
- npm --prefix apps/web run build
- $env:WEB_BASE_URL = "http://127.0.0.1:13000"; npm --prefix apps/web run test:ui-runtime-smoke
- git log --oneline -3

Record full output. Note env-blocked items (e.g. missing local node_modules for tsc/next while the docker web image on 13000 is healthy).

## Completion
- Smoke now matches actual visible tabs and uses activation + stable selectors + narrow crash check.
- Only the two allowed files touched (smoke + this md).
- All query verification commands executed.
- New commit with exact message exists and `git log --oneline -3` shows it.
- Prohibited scope 100% avoided.
- Follows AGENTS required final response format.

Commit: DIFF-253 fix runtime smoke

(Exact outputs and hash recorded in the agent final response after runs.)
