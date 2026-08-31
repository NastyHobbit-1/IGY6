# DIFF-306 — Nightly RITR Audit 2026-08-30 (late)

**Branch:** grok (lowercase only)
**Date:** 2026-08-30
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Open until remaining complete-file origin blobs are size-verified
**Scope:** Finish leftover browser `/api` origin cutover; refresh recorded route-parity counts; correct stale topology/README lifecycle text

## Summary

After DIFF-305, origin still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"` on leftover client surfaces and recorded stale route-parity counts (`118`/`79` vs live `123`/`81`).

This run continued the complete-blob `/api` cutover on `grok` only. Concurrent complete-file landings during the same window covered ConversationHistoryImport, BrowserWebRouterCollectorMvp, and BaselinePatternExpansionPanel (after a truncated placeholder push was restored). README Compose lifecycle/rollback strings landed on origin in this DIFF.

Remaining origin leftovers after this document: EvidenceFeedbackWorkflow, HomePage hypothesis form, rust-cutover-manifest counts, POST_CUTOVER web-row text, WORKING.md repo-name typo.

Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.

## Inspection performed

1. Sync & Inspect on lowercase `grok`. Never touched other branches.
2. No unfinished product-code TODO/FIXME requiring repair.
3. `location.reload` / `ThatDog123` under `apps/web`: 0 hits.
4. Visible tabs remain Chat / Data / Work / Settings / More.
5. Matching `/api/*` write proxies already exist for the leftover panels.

## Testing (local checkout with remaining one-line replacements applied)

- `npm --prefix apps/web run typecheck` — **PASS**
- `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
- `npm --prefix apps/web run build` — **PASS** (Next.js 15.5.15)
- `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
- `python3 scripts/rust-route-parity.py --check` — **PASS** after local manifest 123/81
- `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
- `cargo test --workspace` / clippy — blocked (sandbox rustc 1.75 / edition2024 lockfile)
- docker compose / Playwright runtime smokes — not runnable here

## Remaining blockers

- Complete-file origin updates for EvidenceFeedbackWorkflow, HomePage, rust-cutover-manifest.json, POST_CUTOVER web row.
- Open DIFF-294 draft PRs #6/#9/#10/#11 remain owner-landed.
- Full cargo/clippy matrix and live Playwright/docker smokes still require a newer Rust toolchain and a running stack.

## Confirmation

- Worked only on lowercase `grok`.
- No intended functionality removed.
- Not promoted to `main`.
