# Nightly Tasks Log

## Format for Entries
- Date: YYYY-MM-DD
- Branch: grok
- Summary of checks/repairs/improvements
- Files changed
- New DIFF reference if applicable

---

## 2026-09-30 (DIFF-321)
- Branch: grok
- Continuation after DIFF-320. Worked only on lowercase grok. DIFF-320 is locked and was not edited.
- Landed on origin: DIFF-321 record; this nightly log.
- Verified locally on a grok worktree and ready to PUT: HomePage hypothesis form `data-api-base-url="/api"` (removed `NEXT_PUBLIC_API_BASE_URL` and hardcoded `http://127.0.0.1:8000`); manifest route counts 123/81; Redis dropped from current-runtime supporting lists in the cutover manifest and POST_CUTOVER route audit; WORKING.md / ui README verifier notes.
- Testing on local patched copies: rust-route-parity --check PASS; test-rust-route-parity PASS (4); post-cutover-runtime-audit PASS; ui-smoke PASS (53 files); check-chat-bounds PASS; validate-chat-script PASS; validate-media-script PASS; validate-panel-scripts PASS (23); normal-user-product-smoke --check PASS. Origin without the leftover HomePage/manifest blobs still fails parity + ui-smoke. cargo/clippy not run here. docker/Playwright live smokes not runnable here (`docker` missing). npm typecheck/build blocked (no apps/web node_modules).
- Next: land HomePage/manifest/POST_CUTOVER/WORKING/ui-README blobs with a tool that can PUT the full files; owner-land DIFF-294 draft PRs if still open; full cargo + live stack smokes on newer rustc + docker.
- See `docs/diffs/DIFF-321-nightly-audit-2026-09-30.md`.

## 2026-09-12 (DIFF-320)
- Branch: grok
- See `docs/diffs/DIFF-320-nightly-audit-2026-09-12.md` and git history.

## 2026-09-11 through 2026-07-13
- See DIFF-257 through DIFF-320 and git history.
