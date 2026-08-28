# DIFF-300 — Nightly RITR Audit 2026-08-27

**Branch:** grok (lowercase only)
**Date:** 2026-08-27
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Full inspection of IGY6 on `grok`; repair browser API origin mismatch and stale route-parity manifest counts

## Summary

Nightly audit found two defects on `grok`:

1. Seventeen client panels still posted to `process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"` while Chat already used same-origin `/api` proxies. In Docker the gateway is `http://api:8000`; on the host the published port is often not 8000. Those Data/Work/More actions missed the Rust API.
2. `configs/rust-cutover-manifest.json` `route_parity` still recorded rust_native=118 and web_used=79. Live inventory is 123/81 (classification file was already updated in DIFF-299). `python3 scripts/test-rust-route-parity.py` failed until the manifest matched.

Missing Next `/api` POST proxies were added for every client-used write path. All `data-api-base-url` attributes now use `/api`. Docs and ui-smoke were updated to lock the contract.

Head SHA at audit start: `a21bb1066ba54cdd7a522df01f784ab7b91e19df` (DIFF-299).

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, AGENTS.md, docs/WORKING.md, docs/ui/README.md, docs/BRANCH_POLICY.md, nightly_tasks.md, DIFF-294, DIFF-299, package.json, UI components, gateway route registry, open draft PRs against grok.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME/unfinished markers: none requiring repair.
   - `location.reload` / `ThatDog123`: 0 hits under apps/web.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Media import markers still match ui-smoke.
   - Open DIFF-294 drafts #6/#10/#11 remain intentional productization. Draft #9 overlapped this client-API fix and should be rebased or closed after this lands.

3. **Repair Loop**
   - Root cause (API): browser fetch used a compile-time public origin instead of the Next proxy layer already used by Chat.
   - Fix: `browserApiBaseUrl = "/api"` plus new proxy route files forwarding JSON POSTs through `proxyJsonPost` → `API_BASE_URL`.
   - Root cause (parity): manifest counts were not refreshed when rust_native/web_used grew.
   - Fix: set manifest `rust_native_routes` to 123 and `web_used_routes` to 81.

4. **Maintenance / Improvement**
   - WORKING.md proxy table lists the new routes.
   - docs/ui/README.md and POST_CUTOVER_ROUTE_AUDIT.md describe `/api` browser calls.
   - ui-smoke requires the new route files and fails if client corpus contains `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.

5. **UI Verification**
   - Visible controls still map to real gateway routes; no new buttons; no fake demo data.
   - Save Settings remains dry-run gated (unchanged).

6. **Testing**
   - `npm --prefix apps/web run typecheck` — **PASS**
   - `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
   - `npm --prefix apps/web run build` — **PASS** (Next.js 15.5.15)
   - `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
   - `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
   - `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
   - `cargo test --workspace` / `cargo clippy` — blocked (agent rustc 1.75 / Cargo.lock edition2024)
   - `docker compose ... config` — blocked (docker not installed)
   - `npm run test:ui-runtime-smoke` / `dom-check.mjs` — require Playwright browsers; not runnable here

## Files changed

- `apps/web/src/lib/rust-api.ts` — `browserApiBaseUrl` and `proxyJsonPost`
- `apps/web/src/app/api/sources/route.ts`
- `apps/web/src/app/api/approvals/[approval_id]/decision/route.ts`
- `apps/web/src/app/api/collection-runs/dry-run/route.ts`
- `apps/web/src/app/api/collection-runs/manual-upload/route.ts`
- `apps/web/src/app/api/collection-runs/local-project/route.ts`
- `apps/web/src/app/api/work-items/[work_item_id]/dispatch/route.ts`
- `apps/web/src/app/api/analysis/hypotheses/route.ts`
- `apps/web/src/app/api/analysis/patterns/route.ts`
- `apps/web/src/app/api/analysis/patterns/detect-baseline/route.ts`
- `apps/web/src/app/api/analysis/patterns/[pattern_id]/review/route.ts`
- `apps/web/src/app/api/analysis/predictions/route.ts`
- `apps/web/src/app/api/analysis/recommendations/route.ts`
- `apps/web/src/app/api/experiments/propose-from-improvement/route.ts`
- `apps/web/src/app/api/experiments/[experiment_id]/status/route.ts`
- `apps/web/src/app/api/memory/graph/schema/ensure/route.ts`
- `apps/web/src/app/api/memory/graph/lineage/sync/route.ts`
- `apps/web/src/app/api/memory/vector/chunks/ensure/route.ts`
- `apps/web/src/app/api/feedback/route.ts`
- `apps/web/src/app/api/outcomes/route.ts`
- `apps/web/src/app/api/improvements/route.ts`
- `apps/web/src/app/api/reports/route.ts`
- `apps/web/src/app/api/reports/[report_id]/render/route.ts`
- Client panels: MediaImportMvp, GuidedManualTextUpload, UserObservationIngestion, ConversationHistoryImport, SourceCollectionApprovalReview, BrowserWebRouterCollectorMvp, LocalProjectPcDiagnosticsHardeningPanel, PipelineOperationsPanel, EvidenceFeedbackWorkflow, BasicReportWorkflow, MvpActionConsole, ImprovementExperimentReview, BaselinePatternExpansionPanel, PredictionRecommendationCreator, PredictionRecommendationOutcomeReview, GraphLineageExplanationPanel, HomePage
- `apps/web/scripts/ui-smoke.mjs`
- `configs/rust-cutover-manifest.json`
- `docs/WORKING.md`, `docs/ui/README.md`, `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `nightly_tasks.md`, `docs/diffs/DIFF-300-nightly-audit-2026-08-27.md`

## Remaining blockers

None for this audit. Open DIFF-294 draft PRs remain owner-landed:

- PR #6 — docs/API alignment + dom-check Chat IA
- PR #9 — web client API proxy consistency (partially superseded by this DIFF; rebase or close)
- PR #10 — local_project text-only normalization
- PR #11 — phase0 Playwright/UI/API smokes + version bump

## Next recommended work

Continue nightly RITR exclusively on `grok`. Owner may land remaining DIFF-294 draft PRs after rebase against this client-API change.

## Confirmation

- Worked only on lowercase `grok`.
- No existing intended functionality removed.
- No partial fixes left: every client write path used on grok now has a matching `/api` proxy.
- All hard rules followed.
- Not promoted to `main`.
