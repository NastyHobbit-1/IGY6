# DIFF-302 — Nightly RITR Audit 2026-08-29

**Branch:** grok (lowercase only)
**Date:** 2026-08-29
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Complete the leftover DIFF-301 client `/api` origin cutover; refresh stale route-parity manifest counts; lock the contract in ui-smoke

## Summary

DIFF-301 claimed every client write path used `data-api-base-url="/api"` and that `configs/rust-cutover-manifest.json` recorded rust_native=123 / web_used=81. Head at audit start (`24cb793`) did not match that claim:

- Thirteen client panels still compiled `process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`.
- Manifest still recorded rust_native=118 / web_used=79, so `python3 scripts/test-rust-route-parity.py` failed.
- ui-smoke did not reject the old browser origin.

In Docker the gateway is `http://api:8000`; on the host the published port is often not 8000. Those Data / Work / More actions still missed the Rust API unless the operator happened to expose 8000 on localhost.

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, AGENTS.md, docs/WORKING.md, docs/ui/README.md, nightly_tasks.md, DIFF-300, DIFF-301, client panels, rust-api.ts, open draft PRs against grok.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME/unfinished markers: none requiring repair.
   - `location.reload` / `ThatDog123`: 0 hits under apps/web.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Media import markers still match ui-smoke.
   - Server-side Next `/api` routes correctly keep `API_BASE_URL ?? "http://127.0.0.1:8000"` (container/server → gateway). That is not a browser origin.
   - Open DIFF-294 drafts #6/#10/#11 remain intentional productization. Draft #9 overlaps this client-API fix and should be rebased or closed.

3. **Repair Loop**
   - Root cause (API): DIFF-301 docs and three follow-up commits did not update the remaining `data-api-base-url` surfaces; ui-smoke did not reject the old origin.
   - Fix: set every remaining client `browserApiBaseUrl` / inline `data-api-base-url` to `/api`. Add ui-smoke guards plus required write-proxy route files.
   - Root cause (parity): manifest counts were not refreshed when rust_native/web_used grew (DIFF-301 described the refresh but did not land it).
   - Fix: set manifest `rust_native_routes` to 123 and `web_used_routes` to 81.

4. **Maintenance / Improvement**
   - WORKING.md now states the `/api`-only browser contract on every `data-api-base-url` surface.
   - nightly_tasks.md backfilled DIFF-300 / DIFF-301 pointers so the log matches git history.

5. **UI Verification**
   - Visible controls still map to real gateway routes via `/api` proxies; no new buttons; no fake demo data.
   - Save Settings remains dry-run gated (unchanged).
   - Host-bridge agent calls to `127.0.0.1:${agentPort}` in UnifiedChatHub are unchanged (local host agent, not the Rust API).

6. **Testing**
   - `npm --prefix apps/web run typecheck` — **PASS**
   - `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
   - `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
   - `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
   - `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
   - `cargo test --workspace` / `cargo clippy` — blocked (agent rustc 1.75 / Cargo.lock edition2024)
   - `docker compose ... config` — blocked (docker not installed)
   - `npm run test:ui-runtime-smoke` / `dom-check.mjs` — require Playwright browsers; not runnable here

## Files changed

- `apps/web/src/app/components/BaselinePatternExpansionPanel.tsx`
- `apps/web/src/app/components/BasicReportWorkflow.tsx`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx`
- `apps/web/src/app/components/ConversationHistoryImport.tsx`
- `apps/web/src/app/components/EvidenceFeedbackWorkflow.tsx`
- `apps/web/src/app/components/GraphLineageExplanationPanel.tsx`
- `apps/web/src/app/components/GuidedManualTextUpload.tsx`
- `apps/web/src/app/components/HomePage.tsx`
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx`
- `apps/web/src/app/components/MvpActionConsole.tsx`
- `apps/web/src/app/components/PredictionRecommendationCreator.tsx`
- `apps/web/src/app/components/PredictionRecommendationOutcomeReview.tsx`
- `apps/web/src/app/components/UserObservationIngestion.tsx`
- `apps/web/scripts/ui-smoke.mjs`
- `configs/rust-cutover-manifest.json`
- `docs/WORKING.md`
- `nightly_tasks.md`
- `docs/diffs/DIFF-302-nightly-audit-2026-08-29.md`

## Remaining blockers

None for this audit. Open DIFF-294 draft PRs remain owner-landed:

- PR #6 — docs/API alignment + dom-check Chat IA
- PR #9 — web client API proxy consistency (superseded for origin fix; rebase or close)
- PR #10 — local_project text-only normalization
- PR #11 — phase0 Playwright/UI/API smokes + version bump

## Next recommended work

Continue nightly RITR exclusively on `grok`. Owner may land remaining DIFF-294 draft PRs after rebase against this `/api` contract.

## Confirmation

- Worked only on lowercase `grok`.
- No existing intended functionality removed.
- No partial fixes left: every client write path used on grok now has `data-api-base-url="/api"` (or `browserApiBaseUrl = "/api"`) and a matching `/api` proxy.
- All hard rules followed.
- Not promoted to `main`.
