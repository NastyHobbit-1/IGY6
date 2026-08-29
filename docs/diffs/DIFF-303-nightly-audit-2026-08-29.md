# DIFF-303 — Nightly RITR Audit 2026-08-29

**Branch:** grok (lowercase only)
**Date:** 2026-08-29
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Continue leftover DIFF-301/302 client `/api` origin cutover; restore files truncated by a partial push; document remaining origin gaps

## Summary

DIFF-302 documented a complete leftover `/api` cutover, but origin still had client panels compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`. This run landed additional `/api` origin fixes and had to restore two files after a truncated GitHub push.

## Landed on origin this run

- Restored full `BasicReportWorkflow.tsx` and `PredictionRecommendationCreator.tsx` after commit `07cee83` truncated them.
- `MvpActionConsole.tsx` now uses `browserApiBaseUrl = "/api"`.
- `docs/WORKING.md` states the `/api`-only browser contract.
- Parallel/prior commits already pointed GraphLineage, GuidedManualTextUpload, BasicReport, and PredictionRecommendationCreator at `/api`.

## Still on origin at lock time (must finish next)

These client surfaces still compile `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`:

- `BaselinePatternExpansionPanel.tsx`
- `BrowserWebRouterCollectorMvp.tsx`
- `ConversationHistoryImport.tsx`
- `EvidenceFeedbackWorkflow.tsx`
- `HomePage.tsx` (hypothesis form `data-api-base-url`)
- `LocalProjectPcDiagnosticsHardeningPanel.tsx`
- `PredictionRecommendationOutcomeReview.tsx`
- `UserObservationIngestion.tsx`

Also not yet on origin:

- ui-smoke origin guards + expanded write-proxy list
- cutover-manifest counts 123/81 (file may still say 118/79)
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` topology row still mentions browser helpers calling `http://127.0.0.1:8000`

Local copies of those one-line origin replacements were verified before push. They must be pushed as **complete files**, not truncated payloads.

## Testing

Ran on a local checkout with the remaining one-line `/api` replacements applied:

- `npm --prefix apps/web run typecheck` — **PASS**
- `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files) after local smoke guard update
- `python3 scripts/test-rust-route-parity.py` — **PASS** after local manifest 123/81
- `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
- `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
- `cargo test --workspace` / clippy — blocked (sandbox rustc 1.75 / lockfile edition2024)
- docker compose / Playwright runtime smokes — not runnable here

## Confirmation

- Worked only on lowercase `grok`.
- No intended functionality removed.
- Truncated-file incident was repaired on origin.
- Remaining `/api` origin cutover is incomplete on origin and is the next required work.
- Not promoted to `main`.
