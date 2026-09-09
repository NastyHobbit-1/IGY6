# DIFF-316: Nightly RITR audit 2026-09-08

Status: Locked after this landing

## Result

Landed on origin `grok` the leftover product blobs DIFF-309 through DIFF-315
verified locally but could not PUT:

- `apps/web/src/app/components/HomePage.tsx` hypothesis form
  `data-api-base-url="/api"` and start-here chip label Open Chat
- Guided upload / conversation import / observation next-step copy uses
  open Chat
- Source-trust copy no longer points users at a Results tab
- `BrowserWebRouterCollectorMvp.tsx` unlock and deep-scan bodies are valid
  browser JS (no `as any`)
- `configs/rust-cutover-manifest.json` `rust_native_routes=123`,
  `web_used_routes=81`; Redis removed from current-runtime supporting-service
  lists and Compose service-name claims
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` web row uses same-origin
  `/api`; Redis retired from active Compose supporting-service wording

Origin verification after landing:

- `python3 scripts/rust-route-parity.py --check` PASS
  (`fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `scripts/normal-user-product-smoke.sh --check` PASS

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-315 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-315.

## Baseline Facts

- Active branch: lowercase `grok` at `8958c65` (DIFF-315 docs) before this DIFF.
- DIFF-315 is locked and was not edited.
- Origin after DIFF-315 still had HomePage `NEXT_PUBLIC_API_BASE_URL ??
  "http://127.0.0.1:8000"`, Open Results CTA, collector `as any`, manifest
  `118`/`79` plus Redis in current-runtime lists, and POST_CUTOVER browser
  `http://127.0.0.1:8000` plus Redis-as-active wording.
- Live route parity after this landing:
  `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-315 listed.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Update nightly record and verifier docs.
- Record this nightly DIFF.

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308 through DIFF-315
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-316 on commits and this file.

## Verification

Recorded in Result. Codex-safe static checks pass on origin after landing.

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- `npm --prefix apps/web run typecheck|build` and Playwright
  `test:ui-runtime-smoke`: `node_modules` / `playwright` not installed

## Completion Criteria

Origin validators and the leftover product blobs are landed on `grok`.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
