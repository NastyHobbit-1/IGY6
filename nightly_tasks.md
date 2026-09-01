# Nightly Tasks Log

## Format for Entries
- Date: YYYY-MM-DD
- Branch: grok
- Summary of checks/repairs/improvements
- Files changed
- New DIFF reference if applicable

---

## 2026-08-31 late (DIFF-308)
- Branch: grok
- Continuation after DIFF-307. Worked only on lowercase grok. DIFF-307 is locked and was not edited.
- Origin leftovers after DIFF-307 land: HomePage hypothesis form still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`; manifest still recorded `118`/`79`; POST_CUTOVER still said browser helpers call `http://127.0.0.1:8000` and listed Redis as an active supporting service.
- Repairs: HomePage hypothesis form `data-api-base-url="/api"`; rust-cutover-manifest route_parity `123`/`81`; POST_CUTOVER web row + Redis sentence; LifecycleAuditStatusPanel / E2E smoke / non-web classification no longer treat Redis as an active runtime service.
- Testing: typecheck PASS; ui-smoke PASS (53 files); next build PASS (Next.js 15.5.15); test-rust-route-parity PASS (4); rust-route-parity --check PASS (91/123/81/missing 0/fallback 0); post-cutover-runtime-audit PASS. cargo/clippy blocked (rustc 1.75 / edition2024). docker/Playwright not runnable here.
- Remaining blockers: owner-land DIFF-294 draft PRs #6/#9/#10/#11; full cargo + live stack smokes need newer rustc and docker/Playwright.
- See `docs/diffs/DIFF-308-nightly-audit-2026-08-31.md`.

## 2026-08-31 (DIFF-307)
- Branch: grok
- Continuation after DIFF-306 leftover `/api` origin list. Worked only on lowercase grok.
- Repairs landed on origin in DIFF-307: EvidenceFeedbackWorkflow same-origin `/api`; rust-route-parity `web_used_routes` staleness guard; WORKING.md "Grok6 repo" typo corrected to IGY6; DIFF-307 record.
- DIFF-307 completion criteria for HomePage / manifest 123/81 / POST_CUTOVER were written ahead of those origin blobs. DIFF-308 lands those leftovers.
- Improvement: `scripts/rust-route-parity.py --check` now fails when `web_used_routes` is stale (same as rust_native_routes).
- Testing: typecheck PASS; ui-smoke PASS (53 files); next build PASS; test-rust-route-parity PASS (4); rust-route-parity --check PASS after DIFF-308 manifest refresh; post-cutover-runtime-audit PASS. cargo/clippy blocked (rustc 1.75 / edition2024). docker/Playwright not runnable here.
- See `docs/diffs/DIFF-307-nightly-audit-2026-08-31.md`.

## 2026-08-30 (late / DIFF-306)
- Branch: grok
- Continuation after DIFF-305 panel landings. Worked only on lowercase grok.
- Origin this window: README Compose lifecycle/rollback strings; DIFF-306 record; ConversationHistoryImport / BrowserWebRouterCollectorMvp / BaselinePatternExpansionPanel `/api` complete blobs (plus restore after one truncated placeholder).
- Remaining origin leftovers: EvidenceFeedbackWorkflow, HomePage hypothesis form, manifest 118/79, POST_CUTOVER web-row `127.0.0.1:8000` text, WORKING.md repo-name typo.
- Local replacements + docs prepared and verified: typecheck / ui-smoke / next build / route-parity / post-cutover-runtime-audit PASS. cargo/clippy blocked (rustc 1.75 / edition2024). docker/Playwright not runnable here.
- See `docs/diffs/DIFF-306-nightly-audit-2026-08-30.md`.

## 2026-08-30
- Branch: grok
- Full sync/inspection of grok at d4a8901 (DIFF-304). Worked only on lowercase grok. Full Functionality Audit: leftover DIFF-304 origin list confirmed on origin. No unfinished product-code TODO/FIXME requiring repair. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional. `location.reload` / `ThatDog123` under apps/web: 0 hits.
- See prior DIFF-305/306 entries and git history for the rest of this window.

## 2026-08-29 (earlier runs)
- Branch: grok
- DIFF-302, DIFF-303, DIFF-304 continued the leftover client `/api` origin cutover. See those DIFF files and git history for full detail.
- Origin at DIFF-304 still had seven client panels plus the HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`.

## 2026-08-24 through 2026-08-28
- See DIFF-297 through DIFF-302 and git history.

## 2026-07-13 through 2026-08-23
- See prior entries in git history and DIFF-257 through DIFF-296.
