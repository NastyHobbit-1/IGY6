# DIFF-308: Nightly RITR audit 2026-08-31 (late)

Status: Locked

## Type

Change-bearing

## Objective

Land the remaining DIFF-307 origin leftovers on lowercase `grok`: HomePage hypothesis form same-origin `/api`, live route-parity counts `123`/`81`, and POST_CUTOVER topology text (browser `/api` + no Redis as an active Compose service). Re-verify smoke/typecheck/build/parity/post-cutover-audit.

## Baseline Facts

- Active branch: lowercase `grok` at `e5470c8` (EvidenceFeedbackWorkflow `/api` + DIFF-307 log) before this DIFF.
- DIFF-307 is locked. It landed EvidenceFeedbackWorkflow, the `web_used_routes` guard, WORKING.md repo-name correction, and the DIFF-307 record. Origin still had:
  - HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`
  - `configs/rust-cutover-manifest.json` route_parity `118`/`79` vs live `123`/`81`
  - POST_CUTOVER web row still saying browser helpers call `http://127.0.0.1:8000`
  - POST_CUTOVER supporting-services sentence still listing Redis
- Live route parity: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `/api/analysis/hypotheses` Next.js write proxy already exists.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. Host-bridge `127.0.0.1:${agentPort}` calls remain intentional.
- `infra/docker-compose.yml` does not run Redis or Celery.

## Allowed Scope

- `apps/web/src/app/components/HomePage.tsx` hypothesis form `data-api-base-url` only
- `apps/web/src/app/components/LifecycleAuditStatusPanel.tsx` runtime-databases exclusion text only
- `configs/rust-cutover-manifest.json` route_parity counts only
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` runtime topology web row and supporting-services sentence
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md` supporting-services sentence
- `docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md` queued-work troubleshooting line
- `nightly_tasks.md`
- `docs/diffs/DIFF-308-nightly-audit-2026-08-31.md`

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-307

## Required Tags

DIFF-308 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check` PASS (`91/123/81/missing 0/fallback 0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `npm --prefix apps/web run test:ui-smoke` PASS (53 files)
- `npm --prefix apps/web run typecheck` PASS
- `npm --prefix apps/web run build` PASS
- `cargo test` / clippy blocked (sandbox rustc 1.75 / edition2024)
- docker/Playwright live smokes not runnable here

## Completion Criteria

- No client component compiles `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- Hypothesis form `data-api-base-url="/api"`.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- POST_CUTOVER topology matches Compose (browser `/api`; Redis not listed as active).

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
- Historical "Grok6 clone" wording in older capability-table / plan docs.
