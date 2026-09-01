# Nightly Tasks Log

## Format for Entries
- Date: YYYY-MM-DD
- Branch: grok
- Summary of checks/repairs/improvements
- Files changed
- New DIFF reference if applicable

---

## 2026-09-01 late (DIFF-309)
- Branch: grok
- Continuation after DIFF-308 docs. Worked only on lowercase grok. DIFF-308 records were not edited.
- Origin after DIFF-308 docs still had HomePage `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`, Open Results CTA, manifest `118`/`79`, POST_CUTOVER direct-gateway + Redis-as-active wording.
- Landed on origin: helpers.ts completed-work guidance Open Chat; SourceDetailPanel next action Open Chat; DIFF-309 record.
- Not landed on origin this run (large-blob update risk): HomePage hypothesis `/api` + Open Chat chip; manifest `123`/`81` and Redis drop; POST_CUTOVER web row; LocalProject / BrowserWebRouter Open Results copy.
- Testing on origin: rust-route-parity --check FAIL (stale 118/79 vs live 123/81); test-rust-route-parity 3 pass / 1 fail; post-cutover-runtime-audit PASS; ui-smoke FAIL (HomePage origin). Local patched copies of HomePage+manifest made those four checks pass. cargo/clippy blocked. docker/Playwright and npm typecheck/build not runnable here.
- Next: DIFF-310 land HomePage `/api`, manifest 123/81, POST_CUTOVER Redis/origin wording.
- See `docs/diffs/DIFF-309-nightly-audit-2026-09-01.md`.

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
- See `docs/diffs/DIFF-307-nightly-audit-2026-08-31.md`.

## 2026-08-30 (late / DIFF-306)
- See DIFF-306 record and git history.

## 2026-08-30
- See prior DIFF-305/306 entries and git history.

## 2026-08-29 through 2026-07-13
- See DIFF-257 through DIFF-304 and git history.
