# DIFF-305 — Nightly RITR Audit 2026-08-30

**Branch:** grok (lowercase only)
**Date:** 2026-08-30
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Open (local cutover verified; remaining complete-file origin pushes listed below)
**Scope:** Complete the leftover DIFF-303/304 browser `/api` origin cutover on origin; refresh route-parity recorded counts; correct stale topology/proxy docs

## Summary

DIFF-304 landed ui-smoke origin guards and `UserObservationIngestion.tsx` on `/api`, but seven client panels plus the HomePage hypothesis form still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`.

This run verified and applied those one-line `/api` replacements locally, refreshed cutover-manifest counts to live 123/81, expanded the WORKING.md write-proxy table, and prepared the POST_CUTOVER topology correction. Local verification passed. Origin received DIFF-305 plus the WORKING.md table expansion. The leftover complete TSX/manifest/topology/nightly-log files must still be pushed as complete blobs (GitHub file-update payloads truncate if incomplete content is sent).

Server-side Next proxies and `getJson` still use container/server `API_BASE_URL` (default `http://api:8000` or `http://127.0.0.1:8000`). That is intentional and is not a browser origin.

## Landed on origin this run

- `docs/diffs/DIFF-305-nightly-audit-2026-08-30.md`
- `docs/WORKING.md` — write-proxy rows for collection/feedback/outcomes/patterns/hypotheses

## Verified locally; still must be pushed as complete files

- `apps/web/src/app/components/BaselinePatternExpansionPanel.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/ConversationHistoryImport.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/EvidenceFeedbackWorkflow.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/PredictionRecommendationOutcomeReview.tsx` — `browserApiBaseUrl = "/api"`
- `apps/web/src/app/components/HomePage.tsx` — hypothesis form `data-api-base-url="/api"`
- `configs/rust-cutover-manifest.json` — `rust_native_routes` 123, `web_used_routes` 81
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` — web service row: browser `/api` proxies; Redis/Celery retired from active Compose
- `nightly_tasks.md` — 2026-08-30 entry

Do not push truncated payloads. After each file, confirm GitHub `size` matches the local complete file.

## Inspection performed

1. Sync & Inspect on lowercase `grok` at start SHA `d4a89017ff0ca1d82e7b88f7d086e37545136358`. Never touched other branches.
2. Full functionality audit: leftover DIFF-304 origin list confirmed. No unfinished product-code TODO/FIXME requiring repair. Honest forecasting/rollback warnings and gateway 404 detail string remain intended. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.
3. `location.reload` / `ThatDog123` under `apps/web`: 0 hits.
4. Visible tabs remain Chat / Data / Work / Settings / More.
5. Matching write proxies already exist for the leftover panels.

## Testing (local checkout with remaining one-line replacements applied)

- `npm --prefix apps/web run typecheck` — **PASS**
- `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
- `npm --prefix apps/web run build` — **PASS** (Next.js 15.5.15)
- `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
- `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
- `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
- `cargo test --workspace` / clippy — blocked (sandbox rustc 1.75 / edition2024 lockfile)
- docker compose / Playwright runtime smokes — not runnable here (docker CLI not installed; Playwright browsers not provisioned)

Note: origin ui-smoke will fail until the remaining client files lose `NEXT_PUBLIC_API_BASE_URL`. That guard is intentional.

## Remaining blockers

- Complete-file GitHub updates for the seven leftover panels + HomePage + manifest + POST_CUTOVER + nightly_tasks.
- Open DIFF-294 draft PRs #6/#9/#10/#11 remain owner-landed productization. PR #9 overlaps this origin work and should be rebased or closed after the cutover lands.
- Full cargo/clippy matrix and live Playwright/docker smokes still require a newer Rust toolchain and a running stack.

## Next recommended work

Push each remaining file as a **complete** blob, verify size, then re-run `npm --prefix apps/web run test:ui-smoke` against origin sources. Do not promote to `main`.

## Confirmation

- Worked only on lowercase `grok`.
- No intended functionality removed.
- Local one-line origin replacements are complete and tested.
- Not promoted to `main`.
