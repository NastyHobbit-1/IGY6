# Nightly Tasks Log

## Format for Entries
- Date: YYYY-MM-DD
- Branch: grok
- Summary of checks/repairs/improvements
- Files changed
- New DIFF reference if applicable

---

## 2026-08-30
- Branch: grok
- Full sync/inspection of grok at d4a8901 (DIFF-304). Worked only on lowercase grok.
- Full Functionality Audit: leftover DIFF-304 origin list confirmed on origin. No unfinished product-code TODO/FIXME requiring repair. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional. `location.reload` / `ThatDog123` under apps/web: 0 hits.
- Repair Loop:
  - Root cause: DIFF-304 locked ui-smoke against compiled localhost:8000 origins but seven panels plus the HomePage hypothesis form still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`. Manifest counts were still 118/79. POST_CUTOVER topology still said browser helpers call `http://127.0.0.1:8000`.
  - Local one-line replacements applied and verified: `browserApiBaseUrl = "/api"` on BaselinePatternExpansionPanel, BrowserWebRouterCollectorMvp, ConversationHistoryImport, EvidenceFeedbackWorkflow, LocalProjectPcDiagnosticsHardeningPanel, PredictionRecommendationOutcomeReview; HomePage hypothesis form `data-api-base-url="/api"`; manifest 123/81; POST_CUTOVER topology row corrected.
  - Origin landed this run: DIFF-305, WORKING.md write-proxy table rows, this log. Remaining complete-file origin pushes: the seven panels + HomePage + rust-cutover-manifest.json + POST_CUTOVER_ROUTE_AUDIT.md.
- Testing (local checkout with replacements applied): typecheck PASS; ui-smoke PASS (53 files); next build PASS; test-rust-route-parity PASS (4); rust-route-parity --check PASS (91/123/81/missing 0/fallback 0); post-cutover-runtime-audit PASS. cargo/clippy blocked (rustc 1.75 / edition2024 lockfile). docker/Playwright runtime smokes not runnable here.
- Documentation: this entry; DIFF-305-nightly-audit-2026-08-30.md; docs/WORKING.md.
- Remaining blockers: complete-file origin pushes listed above; open DIFF-294 draft PRs #6/#9/#10/#11 remain owner-landed. Full cargo + live stack smokes need a newer rustc and docker/Playwright.
- Next: Push remaining complete files on grok only; owner may land remaining DIFF-294 draft PRs; rebase or close PR #9 after origin `/api` cutover completes.

**All hard rules followed strictly: only grok, no functionality removed, no partials left locally, never assumed works — always verified via tools.**

## 2026-08-29 (earlier runs)
- Branch: grok
- DIFF-302, DIFF-303, DIFF-304 continued the leftover client `/api` origin cutover. See those DIFF files and git history for full detail.
- Origin at DIFF-304 still had seven client panels plus the HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`.

## 2026-08-24 through 2026-08-28
- See DIFF-297 through DIFF-302 and git history.

## 2026-07-13 through 2026-08-23
- See prior entries in git history and DIFF-257 through DIFF-296.
