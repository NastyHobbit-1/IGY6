# DIFF-308: Nightly RITR audit 2026-09-01

Status: Locked

## Type

Change-bearing

## Objective

Finish the leftover browser `/api` origin cutover that DIFF-307 documented but did not land on `HomePage`, refresh recorded route-parity counts to live `123`/`81`, correct remaining Redis-as-active and direct-gateway topology wording, and align the Home workflow chip with the Chat tab.

## Baseline Facts

- Active branch: lowercase `grok` at `e5470c8` (DIFF-307 EvidenceFeedbackWorkflow `/api` landing) before this DIFF.
- Live route parity: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- Origin leftovers after DIFF-307: HomePage hypothesis form still compiled `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`; manifest still recorded `118`/`79`; POST_CUTOVER web row still said browser helpers call `http://127.0.0.1:8000`; current-runtime supporting-service lists still named Redis though base Compose does not run it.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.
- `infra/docker-compose.yml` services: postgres, qdrant, neo4j, mlflow, phoenix, api, worker, web. No Redis or Celery.

## Allowed Scope

- `apps/web/src/app/components/HomePage.tsx`
- `configs/rust-cutover-manifest.json` route_parity counts and current-runtime supporting-service list
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` current topology wording only
- `nightly_tasks.md`
- `docs/diffs/DIFF-308-nightly-audit-2026-09-01.md`

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes

## Required Tags

DIFF-308 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check` PASS (`91/123/81/missing 0/fallback 0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `npm --prefix apps/web run typecheck` / `build` not run (no `node_modules` in this sandbox)
- `cargo test` / clippy blocked (sandbox rustc 1.75 / edition2024 lockfile)
- docker/Playwright live smokes not runnable here

## Completion Criteria

- No client component compiles `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- Hypothesis form `data-api-base-url="/api"`.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- Current-runtime docs do not describe Redis as an active Compose service or browser helpers calling `http://127.0.0.1:8000`.
- Home workflow chip says Open Chat.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
