# DIFF-304 — Nightly RITR Audit 2026-08-29

**Branch:** grok (lowercase only)
**Date:** 2026-08-29
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Open (origin cutover incomplete; continue on grok only)
**Scope:** Finish the leftover DIFF-303 browser `/api` origin cutover on origin; lock ui-smoke and route-parity counts; correct stale topology docs

## Summary

DIFF-303 listed client surfaces that still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`. This run verified the defect, implemented the remaining one-line `/api` replacements locally, locked ui-smoke origin guards on origin, restored `UserObservationIngestion.tsx` after a truncated first push, and documented the leftover complete-file pushes.

GitHub content updates truncate when the payload is not the full file. That is why DIFF-302/303 left leftovers, and why this run restores files one at a time and records exact origin state.

## Landed on origin this run

- `apps/web/src/app/components/UserObservationIngestion.tsx` — complete file, `browserApiBaseUrl = "/api"` (size 19525 after restore)
- `apps/web/scripts/ui-smoke.mjs` — rejects `NEXT_PUBLIC_API_BASE_URL` and `http://127.0.0.1:8000`; requires hypothesis `data-api-base-url="/api"`; expanded write-proxy list
- `nightly_tasks.md` — DIFF-300–304 pointers
- `docs/diffs/DIFF-304-nightly-audit-2026-08-29.md`

## Still must be pushed as complete files (local one-line `/api` replacements already verified)

- `apps/web/src/app/components/BaselinePatternExpansionPanel.tsx`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx`
- `apps/web/src/app/components/ConversationHistoryImport.tsx`
- `apps/web/src/app/components/EvidenceFeedbackWorkflow.tsx`
- `apps/web/src/app/components/HomePage.tsx` (hypothesis form `data-api-base-url="/api"`)
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx`
- `apps/web/src/app/components/PredictionRecommendationOutcomeReview.tsx`
- `configs/rust-cutover-manifest.json` (`rust_native_routes` 123, `web_used_routes` 81)
- `docs/WORKING.md` write-proxy table rows
- `docs/ui/README.md` browser API paragraph
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` topology row (browser `/api` proxies; Redis/Celery retired from active Compose)

Do not push truncated payloads. After each file, confirm GitHub `size` matches the local complete file.

## Inspection performed

1. Sync & Inspect on lowercase `grok` at start SHA `2c13bd0`. Never touched other branches.
2. Full functionality audit: leftover DIFF-303 origin list confirmed. No unfinished product-code TODO/FIXME requiring repair. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.
3. Matching write proxies already exist for leftover panels.

## Testing (local checkout with remaining one-line replacements applied)

- `npm --prefix apps/web run typecheck` — **PASS**
- `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
- `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
- `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
- `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
- `cargo test --workspace` / clippy — blocked (sandbox rustc 1.75 / edition2024 lockfile)
- docker / Playwright runtime smokes — not runnable here

Note: origin ui-smoke will fail until the remaining client files lose `NEXT_PUBLIC_API_BASE_URL`. That guard is intentional.

## Remaining blockers

- Complete-file GitHub updates for the seven leftover panels + HomePage + manifest + three docs listed above.
- Open DIFF-294 draft PRs #6/#9/#10/#11 remain owner-landed productization. PR #9 overlaps this origin work and should be rebased or closed after the cutover lands.

## Next recommended work

Push each remaining file as a **complete** blob, verify size, then re-run `npm --prefix apps/web run test:ui-smoke` against origin sources. Do not promote to `main`.

## Confirmation

- Worked only on lowercase `grok`.
- No intended functionality removed.
- Truncated `UserObservationIngestion.tsx` was restored the same run.
- Not promoted to `main`.
