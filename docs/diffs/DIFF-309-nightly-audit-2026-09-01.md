# DIFF-309: Nightly RITR audit 2026-09-01 (late)

Status: Locked

## Type

Change-bearing

## Objective

Land the leftover browser `/api` origin cutover and current-runtime accuracy fixes that DIFF-308 documented as complete but that origin blobs still contained: HomePage hypothesis form still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`, Home workflow chip still said Open Results, manifest still recorded `118`/`79`, and POST_CUTOVER/current-runtime supporting-service lists still named Redis and a direct gateway origin.

## Baseline Facts

- Active branch: lowercase `grok` at `d1ba370` (DIFF-308 docs) before this DIFF.
- Live route parity at start: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- Origin HomePage SHA `fee79a1` still compiled the direct gateway origin and labeled the Chat tab CTA "Open Results".
- Origin manifest SHA `3dc5d33` still recorded `rust_native_routes=118` and `web_used_routes=79`, so `scripts/rust-route-parity.py --check` and `scripts/test-rust-route-parity.py` failed on stale counts.
- Origin POST_CUTOVER web row still said browser helpers call `http://127.0.0.1:8000`; `remaining_non_rust_components` and `current_runtime_posture` still listed Redis.
- `infra/docker-compose.yml` services: postgres, qdrant, neo4j, mlflow, phoenix, api, worker, web. No Redis or Celery.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.
- `node apps/web/scripts/ui-smoke.mjs` failed on the three HomePage origin checks until this DIFF.

## Allowed Scope

- `apps/web/src/app/components/HomePage.tsx`
- `apps/web/src/app/components/helpers.ts`
- `apps/web/src/app/components/SourceDetailPanel.tsx`
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx`
- `configs/rust-cutover-manifest.json` route_parity counts and current-runtime supporting-service list
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` current topology wording only
- `nightly_tasks.md`
- `docs/diffs/DIFF-309-nightly-audit-2026-09-01.md`

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308 records

## Required Tags

DIFF-309 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check` PASS (`91/123/81/missing 0/fallback 0`) after manifest refresh
- `python3 scripts/test-rust-route-parity.py` PASS (4) after manifest refresh
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files) after HomePage origin cutover
- `npm --prefix apps/web run typecheck` / `build` not run (no `node_modules` in this sandbox)
- `cargo test` / clippy blocked (sandbox rustc 1.75 / edition2024 lockfile)
- docker/Playwright live smokes not runnable here

## Completion Criteria

- No client component compiles `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- Hypothesis form `data-api-base-url="/api"`.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- Current-runtime docs do not describe Redis as an active Compose service or browser helpers calling `http://127.0.0.1:8000`.
- Home workflow chip says Open Chat, not Open Results.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
- Remaining "Open Results" next-step copy in SourceDetailPanel / LocalProjectPcDiagnosticsHardeningPanel / BrowserWebRouterCollectorMvp.
- Historical worker-parity manifest sections still describe earlier Celery/Redis posture by chronology; they are not current-runtime claims.
