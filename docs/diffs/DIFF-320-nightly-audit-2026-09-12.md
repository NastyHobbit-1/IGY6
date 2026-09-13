# DIFF-320: Nightly RITR audit 2026-09-12

Status: Locked after this landing

## Result

Landed on origin `grok` the leftover product blobs DIFF-319 recorded as
local-only:

- `HomePage.tsx` hypothesis form uses same-origin `/api` (no
  `NEXT_PUBLIC_API_BASE_URL`, no `http://127.0.0.1:8000` in browser markup)
- Home readiness CTA label is **Open Chat** (internal radio id remains
  `tab-results`)
- Guided manual upload, conversation import, and observation ingestion
  next-steps point at Chat, not Results
- `configs/rust-cutover-manifest.json` route counts `123` / `81`; Redis
  removed from current-runtime supporting-component lists; lifecycle service
  list matches Compose (no Redis)
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` documents browser
  helpers as same-origin `/api` and Redis as retired from active Compose
- ui-smoke and product-smoke now guard the Open Chat CTA, `/api` hypothesis
  form, and "Results for evidence" regression
- WORKING.md / ui README verifier notes updated to origin-landed truth

DIFF-319 is locked and was not edited.

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-319 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-319.

## Baseline Facts

- Active branch: lowercase `grok` at `c50e1a8` (DIFF-319 docs) before this DIFF.
- DIFF-319 is locked and was not edited.
- Origin after DIFF-319 still had HomePage
  `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`, Open Results CTA,
  manifest `118`/`79` plus Redis in current-runtime lists, POST_CUTOVER
  browser `http://127.0.0.1:8000` plus Redis-as-active wording, and guided
  next-steps that said Results for evidence.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.
- Route parity guard counts: fastapi=91 rust_native=123 web_used=81
  missing_from_rust=0 web_requires_fallback=0.

## Allowed Scope

- Land leftover origin product blobs.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Add regression guards to static smokes now that origin can satisfy them.
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
- Editing locked DIFF-308 through DIFF-319
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-320 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check` PASS
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `bash scripts/normal-user-product-smoke.sh --check` PASS

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- Playwright `test:ui-runtime-smoke`: requires a running stack and Playwright
  browsers
- `npm --prefix apps/web run typecheck` / `build`: `node_modules` not installed
  in this agent environment

## Completion Criteria

Origin HomePage `/api` + Open Chat, guided Chat next-steps, manifest 123/81,
POST_CUTOVER topology wording, smoke guards, and nightly record are landed
and the listed Codex-safe static checks pass on that tree.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
