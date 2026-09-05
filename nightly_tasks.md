# Nightly Tasks Log

## Format for Entries
- Date: YYYY-MM-DD
- Branch: grok
- Summary of checks/repairs/improvements
- Files changed
- New DIFF reference if applicable

---

## 2026-09-04 (DIFF-313)
- Branch: grok
- Continuation after DIFF-312. Worked only on lowercase grok. DIFF-312 is locked and was not edited.
- Landed origin leftovers DIFF-312 could not PUT: HomePage `/api` + Open Chat; guided/conversation/observation Open Chat next-step and remaining Results-as-tab copy; manifest `123`/`81` + Redis drop; POST_CUTOVER browser `/api` and Redis retired.
- Additional repairs: product-smoke `--check` scans `apps/web/src` after page split; chat/media/panel validators read component files; collector media-library script dropped TypeScript `as any`.
- Testing: rust-route-parity --check PASS; test-rust-route-parity PASS (4); post-cutover-runtime-audit PASS; ui-smoke PASS (53 files); check-chat-bounds PASS; validate-chat-script PASS; validate-media-script PASS; validate-panel-scripts PASS (23); normal-user-product-smoke --check PASS. cargo/clippy blocked on rustc 1.75 / lockfile v4 / edition2024. docker/Playwright live smokes not runnable here (`docker` missing). npm typecheck/build blocked (`tsc` / node_modules not installed).
- Next: owner-land DIFF-294 draft PRs #6/#9/#10/#11; full cargo + live stack smokes on newer rustc + docker.
- See `docs/diffs/DIFF-313-nightly-audit-2026-09-04.md`.

## 2026-09-03 (DIFF-312)
- Branch: grok
- Continuation after DIFF-311. Worked only on lowercase grok. DIFF-311 is locked and was not edited.
- Origin at `d7b487c` still had the leftovers DIFF-311 claimed: HomePage hypothesis `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`; start-here chip Open Results; sidebar Open chat; manifest `118`/`79` plus Redis in current-runtime lists; POST_CUTOVER browser `http://127.0.0.1:8000` and Redis-as-active; guided upload / conversation / observation next-step `open Results`.
- Landed on origin: DIFF-312 record; CODEX baseline Redis retirement; product smoke Chat history/report wording.
- Verified locally but not replaced on origin (large-blob update payload limit): HomePage `/api` + Open Chat; guided/conversation/observation Open Chat next-step; manifest `123`/`81` + Redis drop; POST_CUTOVER topology.
- Testing on local patched copies: rust-route-parity --check PASS; test-rust-route-parity PASS (4); post-cutover-runtime-audit PASS; ui-smoke PASS (53 files). Origin without those blobs still fails parity + ui-smoke. cargo/clippy blocked on rustc 1.75 / lockfile v4 / edition2024. docker/Playwright live smokes not runnable here. npm typecheck/build blocked (registry 502 installing typescript).
- Next: land HomePage/manifest/POST_CUTOVER blobs with a tool that can PUT the full files; owner-land DIFF-294 draft PRs #6/#9/#10/#11; full cargo + live stack smokes on newer rustc + docker.
- See `docs/diffs/DIFF-312-nightly-audit-2026-09-03.md`.

## 2026-09-02 (DIFF-311)
- Branch: grok
- Continuation after DIFF-310. Worked only on lowercase grok. DIFF-310 is locked and was not edited.
- Landed origin leftovers DIFF-310 could not replace: HomePage hypothesis `data-api-base-url="/api"`; start-here chip Open Chat; MissingEvidencePromptPanel Return to Chat; manifest `rust_native_routes=123` / `web_used_routes=81`; Redis dropped from current-runtime supporting-service lists; POST_CUTOVER web row same-origin `/api` and Redis retired; NORMAL_USER_PRODUCT_SMOKE Open Chat.
- Testing: rust-route-parity --check, test-rust-route-parity, post-cutover-runtime-audit, ui-smoke recorded in DIFF-311. cargo/clippy blocked on rustc 1.75 / edition2024 unless a newer toolchain is present. docker/Playwright live smokes not runnable here.
- Next: owner-land DIFF-294 draft PRs #6/#9/#10/#11; full cargo + live stack smokes.
- See `docs/diffs/DIFF-311-nightly-audit-2026-09-02.md`.

## 2026-09-01 late-2 (DIFF-310)
- Branch: grok
- Continuation after DIFF-309. Worked only on lowercase grok. DIFF-309 is locked and was not edited.
- Landed: collector next-step Open Chat on BrowserWebRouterCollectorMvp and LocalProjectPcDiagnosticsHardeningPanel; DIFF-310 record.
- Incident: `7cb757f` briefly wrote PLACEHOLDER into those two panels; restored in `6a6fe75` and `2cd40a8`.
- Not landed (large-blob update risk): HomePage hypothesis `/api` + Open Chat chip; manifest `123`/`81` and Redis drop; POST_CUTOVER web row.
- Testing: panel restore verified by reading origin blobs. rust-route-parity --check and ui-smoke still fail on stale HomePage/manifest origin. cargo/clippy blocked. docker/Playwright and npm typecheck/build not runnable here.
- Next: DIFF-311 land HomePage `/api`, manifest 123/81, POST_CUTOVER Redis/origin wording.
- See `docs/diffs/DIFF-310-nightly-audit-2026-09-01.md`.

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
