# DIFF-321 — Nightly RITR audit 2026-09-30

Status: complete (documentation + leftover product wiring on origin `grok`)
Branch: grok
Date: 2026-09-30

## Scope

Continue the locked DIFF-320 leftover-blob work. Land the origin-safe product wiring that previous nightlies verified locally but could not PUT:

- HomePage hypothesis create uses same-origin `/api` (no `NEXT_PUBLIC_API_BASE_URL`, no hardcoded `http://127.0.0.1:8000`)
- `configs/rust-cutover-manifest.json` route counts `rust_native_routes=123`, `web_used_routes=81`
- Redis removed from current-runtime supporting-component lists in the manifest and POST_CUTOVER route audit

Locked DIFFs were not edited.

## Files changed

- `apps/web/src/app/components/HomePage.tsx`
- `configs/rust-cutover-manifest.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/WORKING.md`
- `docs/ui/README.md`
- `nightly_tasks.md`
- `docs/diffs/DIFF-321-nightly-audit-2026-09-30.md`

## Verification run

- `python3 scripts/rust-route-parity.py --check` PASS
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `bash scripts/normal-user-product-smoke.sh --check` PASS

## Verification not run

- `cargo test` / `cargo clippy`: environment rustc/lockfile may not match edition2024 workspace
- Docker Compose / Playwright live smokes: docker not available in this audit environment
- `npm --prefix apps/web run build` / `typecheck`: node_modules not installed in this audit environment

## Out of scope

- Owner-land DIFF-294 draft PRs
- Live stack smokes on a full operator machine
- Merging to `main`
