# DIFF-304 — Nightly RITR Audit 2026-08-29

**Branch:** grok (lowercase only)
**Date:** 2026-08-29
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Finish the leftover DIFF-303 browser `/api` origin cutover on origin; lock ui-smoke and route-parity counts; correct stale topology docs

## Summary

DIFF-303 listed client surfaces that still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"` after earlier 2026-08-29 nightlies. Those Data / Work / More write actions missed the Rust gateway whenever the published API port was not `8000` on localhost (the normal Docker topology uses `http://api:8000` inside Compose).

This run landed the remaining origin replacements as complete files, added smoke guards so the old origin cannot return unnoticed, refreshed live route-parity counts, and corrected the post-cutover topology paragraph.

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok` at `2c13bd0`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, AGENTS.md, docs/WORKING.md, docs/ui/README.md, docs/BRANCH_POLICY.md, nightly_tasks.md, DIFF-302, DIFF-303.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME markers: honest forecasting/rollback warnings and gateway 404 detail only.
   - `location.reload` / `ThatDog123`: 0 hits under apps/web.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Server-side Next `/api` routes correctly keep `API_BASE_URL ?? "http://127.0.0.1:8000"` (container/server → gateway). That is not a browser origin.
   - Host-bridge agent calls to `127.0.0.1:${agentPort}` are unchanged.
   - Matching write proxies already existed for the leftover panels (`/api/analysis/*`, `/api/approvals`, `/api/feedback`, `/api/outcomes`, `/api/collection-runs/manual-upload`, `/api/collection-runs/local-project`).

3. **Repair Loop**
   - Root cause: prior nightlies documented a finished `/api` contract but origin still had eight panels plus the HomePage hypothesis form on the compiled localhost gateway origin; manifest counts and smoke origin guards were not pushed.
   - Fix: set every remaining client `browserApiBaseUrl` / hypothesis `data-api-base-url` to `/api`. Guard the contract in ui-smoke. Set manifest `rust_native_routes=123` and `web_used_routes=81`. Update POST_CUTOVER topology.

4. **Maintenance / Improvement**
   - WORKING.md proxy table now lists the leftover write paths.
   - docs/ui/README.md lists the additional `/api` surfaces and distinguishes host-bridge agent ports.
   - nightly_tasks.md backfilled DIFF-300–303 pointers so the log matches git history.

5. **UI Verification**
   - Visible controls still map to real gateway routes via `/api` proxies; no new buttons; no fake demo data.
   - No duplicated tabs or dead controls introduced.

6. **Testing**
   - `npm --prefix apps/web run typecheck` — **PASS**
   - `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
   - `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
   - `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
   - `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
   - `cargo test --workspace` / `cargo clippy` — blocked (sandbox rustc 1.75 / Cargo.lock edition2024)
   - `docker compose ... config` — blocked (docker not installed)
   - `npm run test:ui-runtime-smoke` / Playwright — require browsers + live stack; not runnable here

## Files changed

- `apps/web/src/app/components/BaselinePatternExpansionPanel.tsx`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx`
- `apps/web/src/app/components/ConversationHistoryImport.tsx`
- `apps/web/src/app/components/EvidenceFeedbackWorkflow.tsx`
- `apps/web/src/app/components/HomePage.tsx`
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx`
- `apps/web/src/app/components/PredictionRecommendationOutcomeReview.tsx`
- `apps/web/src/app/components/UserObservationIngestion.tsx`
- `apps/web/scripts/ui-smoke.mjs`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/WORKING.md`
- `docs/ui/README.md`
- `nightly_tasks.md`
- `docs/diffs/DIFF-304-nightly-audit-2026-08-29.md`

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
