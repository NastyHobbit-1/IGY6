# DIFF-310: Nightly RITR audit 2026-09-01 (late-2)

Status: Locked

## Type

Change-bearing

## Objective

Land the four origin leftovers DIFF-309 documented but could not safely replace on lowercase `grok`: HomePage same-origin `/api` + Open Chat chip, live route-parity counts `123`/`81`, POST_CUTOVER topology text, and collector next-step copy.

## Baseline Facts

- Active branch: lowercase `grok` at `0e4cdc8` (DIFF-309 docs) before this DIFF.
- DIFF-309 is locked.
- Live route parity recorded in DIFF-308/309: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `infra/docker-compose.yml` has no Redis service.

## Landed On Origin In This DIFF

- `docs/diffs/DIFF-310-nightly-audit-2026-09-01.md` and `nightly_tasks.md`.
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx` next-step copy Open Chat (full panel restored after a truncated placeholder push was reverted).
- `apps/web/src/app/components/LocalProjectPcDiagnosticsHardeningPanel.tsx` next-step copy Open Chat (same restore).

## Still On Origin After This DIFF

Large-blob updates were not landed for:

- `apps/web/src/app/components/HomePage.tsx` hypothesis form still compiles `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`; start-here chip still says Open Results.
- `configs/rust-cutover-manifest.json` still records `118`/`79` and still lists Redis in current-runtime supporting-service lists.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` web row still names `http://127.0.0.1:8000` and Redis as a supporting service.

Required origin replacements remain:

- HomePage form: `data-api-base-url="/api"`
- HomePage chip: `Open Chat`
- Manifest: `rust_native_routes=123`, `web_used_routes=81`; drop Redis from current-runtime supporting-service lists
- POST_CUTOVER web row: same-origin `/api` proxies; Redis retired from active Compose

## Incident

Commit `7cb757f` briefly replaced the two collector panels with the literal text `PLACEHOLDER`. That was reverted by restoring the full panel sources in `6a6fe75` and `2cd40a8`.

## Prohibited Scope

- Other branches, promotion to `main`, DIFF-294 PR merges, runtime mutation, feature removal, locked DIFF-308/DIFF-309 edits.

## Verification

- Collector panels on origin export their components and use Open Chat in post-collect next steps.
- HomePage origin blob still contains `NEXT_PUBLIC_API_BASE_URL` and `http://127.0.0.1:8000`.
- Manifest origin blob still has `118`/`79`.
- Full `python3 scripts/rust-route-parity.py --check` and `node apps/web/scripts/ui-smoke.mjs` still fail on those stale origin blobs until HomePage + manifest land.
- cargo/clippy blocked on older nightlies (rustc 1.75 / edition2024). docker/Playwright and npm typecheck/build not runnable in this sandbox.

## Out Of Scope Follow-Up

- DIFF-311: land HomePage `/api` + Open Chat, manifest `123`/`81`, POST_CUTOVER Redis/origin wording without truncating those blobs.
- Owner-land DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy and live Playwright/docker smokes.
