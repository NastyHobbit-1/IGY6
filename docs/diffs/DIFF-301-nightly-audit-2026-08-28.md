# DIFF-301 — Nightly RITR Audit 2026-08-28

**Branch:** grok (lowercase only)
**Date:** 2026-08-28
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Complete the incomplete DIFF-300 client `/api` origin cutover; refresh stale route-parity manifest counts; lock the contract in ui-smoke

## Summary

DIFF-300 added Next `/api` write proxies and switched a few panels (`MediaImportMvp`, `SourceCollectionApprovalReview`, `PipelineOperationsPanel`) to `browserApiBaseUrl = "/api"`. Fourteen other client panels still compiled `process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`. In Docker the gateway is `http://api:8000`; on the host the published port is often not 8000. Those Data / Work / More actions still missed the Rust API.

The route-parity guard also failed: `configs/rust-cutover-manifest.json` still recorded rust_native=118 and web_used=79. Live inventory is 123/81 (`configs/legacy-fastapi-route-classification.json` was already correct from DIFF-299).

Head SHA at audit start: `b81b7ff76235a438f53556889d37777130495e24` (DIFF-300).

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, AGENTS.md, docs/WORKING.md, docs/ui/README.md, nightly_tasks.md, DIFF-300, package.json, client panels, rust-api.ts, docker-compose web env, open draft PRs against grok.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME/unfinished markers: none requiring repair.
   - `location.reload` / `ThatDog123`: 0 hits under apps/web.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Media import markers still match ui-smoke.
   - Open DIFF-294 drafts #6/#10/#11 remain intentional productization. Draft #9 overlaps this client-API fix and should be rebased or closed.

3. **Repair Loop**
   - Root cause (API): DIFF-300 landed proxies and three panels; the remaining `data-api-base-url` surfaces were not updated, and ui-smoke did not reject the old origin.
   - Fix: set every remaining client `browserApiBaseUrl` / inline `data-api-base-url` to `/api`. Add ui-smoke guards plus required write-proxy route files.
   - Root cause (parity): manifest counts were not refreshed when rust_native/web_used grew.
   - Fix: set manifest `rust_native_routes` to 123 and `web_used_routes` to 81.

4. **Maintenance / Improvement**
   - Removed unused `NEXT_PUBLIC_API_BASE_URL` from the Compose web service so a future client change cannot bake a host origin at image build.
   - WORKING.md now states the `/api`-only browser contract explicitly.

5. **UI Verification**
   - Visible controls still map to real gateway routes via `/api` proxies; no new buttons; no fake demo data.
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

- Fourteen client panels that still used NEXT_PUBLIC_API_BASE_URL
- `apps/web/scripts/ui-smoke.mjs`
- `configs/rust-cutover-manifest.json`
- `infra/docker-compose.yml`
- `docs/WORKING.md`
- `nightly_tasks.md`
- `docs/diffs/DIFF-301-nightly-audit-2026-08-28.md`

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
