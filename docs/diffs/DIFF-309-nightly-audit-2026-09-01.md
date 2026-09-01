# DIFF-309: Nightly RITR audit 2026-09-01 (late)

Status: Locked

## Type

Change-bearing

## Objective

Continue the leftover browser `/api` origin cutover and current-runtime accuracy work that DIFF-308 documented as complete but that origin blobs still contained.

## Baseline Facts

- Active branch: lowercase `grok` at `d1ba370` (DIFF-308 docs) before this DIFF.
- Live route parity at start: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- Origin HomePage SHA `fee79a1` still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"` and labeled the Chat tab CTA "Open Results".
- Origin manifest SHA `3dc5d33` still recorded `rust_native_routes=118` and `web_used_routes=79`.
- Origin POST_CUTOVER web row still said browser helpers call `http://127.0.0.1:8000`; current-runtime supporting-service lists still named Redis.
- `infra/docker-compose.yml` has no Redis service.

## Landed On Origin In This DIFF

- `apps/web/src/app/components/helpers.ts` completed-work guidance now says Open Chat.
- `apps/web/src/app/components/SourceDetailPanel.tsx` next action now says Open Chat.
- `nightly_tasks.md` and this record.

## Still On Origin After This DIFF

These are verified leftovers. Local patched copies exist but the GitHub file-update path could not safely replace the large origin blobs in this run without risking truncation:

- `apps/web/src/app/components/HomePage.tsx` hypothesis form still compiles `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`; Home CTA still says Open Results.
- `configs/rust-cutover-manifest.json` still records `118`/`79` and still lists Redis in `remaining_non_rust_components` / `current_runtime_posture`.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` web row still names `http://127.0.0.1:8000` and Redis as a supporting service.
- `LocalProjectPcDiagnosticsHardeningPanel.tsx` and `BrowserWebRouterCollectorMvp.tsx` still say Open Results in next-step copy.

Required origin replacements:

- HomePage form: `data-api-base-url="/api"`
- HomePage chip: `Open Chat`
- Manifest: `rust_native_routes=123`, `web_used_routes=81`; drop Redis from current-runtime supporting-service lists
- POST_CUTOVER web row: same-origin `/api` proxies; Redis retired from active Compose

## Prohibited Scope

- Other branches, promotion to `main`, DIFF-294 PR merges, runtime mutation, feature removal, locked DIFF-308 edits.

## Verification

- `python3 scripts/rust-route-parity.py --check` FAIL on origin (`manifest rust_native_routes is stale`, `manifest web_used_routes is stale`); live inventory is `91/123/81/missing 0/fallback 0`.
- `python3 scripts/test-rust-route-parity.py` FAIL on origin for the same stale counts (3 pass, 1 fail).
- `python3 scripts/post-cutover-runtime-audit.py` PASS.
- `node apps/web/scripts/ui-smoke.mjs` FAIL on origin: `NEXT_PUBLIC_API_BASE_URL`, hardcoded `127.0.0.1:8000`, hypothesis form not `/api`.
- After local HomePage + manifest patches in this sandbox, those four checks passed. cargo/clippy blocked (rustc 1.75 / edition2024). docker/Playwright and npm typecheck/build not runnable here.

## Out Of Scope Follow-Up

- Land the four origin leftovers above as DIFF-310 without reconstructing HomePage from a truncated push.
- Owner-land DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy and live Playwright/docker smokes.
