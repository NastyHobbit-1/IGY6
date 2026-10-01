# DIFF-321 — Nightly RITR audit 2026-09-30

Status: complete for this audit environment; leftover product blobs still need a full-file PUT
Branch: grok
Date: 2026-09-30

## Scope

Continue the locked DIFF-320 leftover-blob work. Worked only on lowercase `grok`. Locked DIFFs were not edited.

## Landed on origin

- `docs/diffs/DIFF-321-nightly-audit-2026-09-30.md`
- `nightly_tasks.md`

## Verified locally, not replaced on origin

GitHub file-update payload size blocked replacing these product blobs in this environment:

- `apps/web/src/app/components/HomePage.tsx` — hypothesis form `data-api-base-url="/api"` (remove `NEXT_PUBLIC_API_BASE_URL` and `http://127.0.0.1:8000`)
- `configs/rust-cutover-manifest.json` — `rust_native_routes=123`, `web_used_routes=81`; drop Redis from current-runtime supporting lists
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` — Redis removed from current-runtime supporting-service wording
- `docs/WORKING.md` / `docs/ui/README.md` — verifier notes pointing at DIFF-321

Local patched copies pass the static suite below. Origin without those blobs still fails `rust-route-parity --check` (stale 118/79) and `ui-smoke` (hypothesis form still compiles `NEXT_PUBLIC_API_BASE_URL` / `127.0.0.1:8000`).

## Verification run (local patched worktree)

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

- `cargo test` / `cargo clippy`
- Docker Compose / Playwright live smokes (`docker` missing)
- `npm --prefix apps/web run build` / `typecheck` (no `apps/web/node_modules`)

## Next

Land the leftover blobs with a tool that can PUT the full files. Then owner-land DIFF-294 draft PRs if still open; full cargo + live stack smokes on newer rustc + docker.
