# DIFF-310: Nightly RITR audit 2026-09-01 (late-2)

Status: Locked

## Type

Change-bearing

## Objective

Land the four origin leftovers DIFF-309 documented but could not safely replace on lowercase `grok`: HomePage same-origin `/api` + Open Chat chip, live route-parity counts `123`/`81`, POST_CUTOVER topology text, and collector next-step copy.

## Baseline Facts

- Active branch: lowercase `grok` at `0e4cdc8` (DIFF-309 docs) before this DIFF.
- DIFF-309 is locked. It landed helpers.ts Open Chat, SourceDetailPanel Open Chat, and the DIFF-309 record. Origin still had:
  - HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`
  - HomePage start-here chip labeled `Open Results` while `tab-results` is the Chat tab
  - `configs/rust-cutover-manifest.json` route_parity `118`/`79` vs live `123`/`81`
  - Manifest current-runtime supporting-service lists still named Redis
  - POST_CUTOVER web row still said browser helpers call `http://127.0.0.1:8000`
  - LocalProject and BrowserWebRouter next-step copy still said Open Results
- Live route parity recorded in DIFF-308/309: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `infra/docker-compose.yml` has no Redis service. Lifecycle check expected services are postgres, qdrant, neo4j, mlflow, phoenix, api, worker, web.

## Landed On Origin In This DIFF

- `apps/web/src/app/components/HomePage.tsx` hypothesis form `data-api-base-url="/api"`; start-here chip `Open Chat`.
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx` next-step copy Open Chat.
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx` next-step copy Open Chat.
- `configs/rust-cutover-manifest.json` route_parity `rust_native_routes=123`, `web_used_routes=81`; Redis removed from current-runtime supporting-service lists.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` web row uses same-origin `/api` proxies; Redis retired from active Compose supporting-service sentence.
- `nightly_tasks.md` and this record.

## Prohibited Scope

- Other branches, promotion to `main`, DIFF-294 PR merges, runtime mutation, feature removal, locked DIFF-308/DIFF-309 edits.

## Verification

- HomePage origin blob no longer contains `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- HomePage contains `data-hypothesis-create-form data-api-base-url="/api"`.
- Manifest JSON parses; `route_parity.rust_native_routes=123` and `web_used_routes=81`.
- Manifest `remaining_non_rust_components` and `current_runtime_posture` no longer list Redis as active.
- POST_CUTOVER web row no longer names a hardcoded browser gateway origin.
- Local string checks passed on the patched blobs before push.
- Full `python3 scripts/rust-route-parity.py --check`, `python3 scripts/test-rust-route-parity.py`, `python3 scripts/post-cutover-runtime-audit.py`, and `node apps/web/scripts/ui-smoke.mjs` require a complete worktree. This sandbox clone was incomplete (slow FS checkout). Those commands are the required follow-up verification on a complete tree.
- cargo/clippy blocked on older nightlies (rustc 1.75 / edition2024). docker/Playwright and npm typecheck/build not runnable in this sandbox.

## Out Of Scope Follow-Up

- Re-run rust-route-parity --check, test-rust-route-parity, post-cutover-runtime-audit, and ui-smoke on a complete worktree.
- Owner-land DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy and live Playwright/docker smokes.
