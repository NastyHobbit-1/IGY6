# DIFF-317: Nightly RITR audit 2026-09-09

Status: Locked after this landing

## Result

Landed on origin `grok` the leftover product blobs DIFF-316 recorded as
local-only, plus a product-smoke guard so they cannot silently regress:

- `HomePage.tsx`: Ask-with-evidence CTA is Open Chat; hypothesis form
  `data-api-base-url="/api"`; pipeline hint says Chat → Memory.
- Guided upload / conversation import / observation next-steps say Chat,
  not Results, for evidence inspection.
- `BrowserWebRouterCollectorMvp.tsx` media-tool script is valid browser JS
  (`as any` removed).
- `configs/rust-cutover-manifest.json` route counts `123` / `81`; Redis
  dropped from current-runtime supporting-service lists.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` topology web row uses
  same-origin `/api`; Redis marked retired from active Compose.
- `scripts/normal-user-product-smoke.sh --check` requires the Open Chat CTA
  and hypothesis `/api` form strings.

Verification on this tree:

- `python3 scripts/rust-route-parity.py --check` PASS
  (`fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `scripts/normal-user-product-smoke.sh --check` PASS

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-316 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-316, and add a smoke
guard against the HomePage regressions.

## Baseline Facts

- Active branch: lowercase `grok` at `c64ef5e` (DIFF-316 docs) before this DIFF.
- DIFF-316 is locked and was not edited.
- Origin after DIFF-316 still had HomePage `NEXT_PUBLIC_API_BASE_URL ??
  "http://127.0.0.1:8000"`, Open Results CTA, collector `as any`, manifest
  `118`/`79` plus Redis in current-runtime lists, POST_CUTOVER browser
  `http://127.0.0.1:8000` plus Redis-as-active wording, and guided-panel
  next-steps that said Results for evidence.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-316 listed.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Add product-smoke markers for the Open Chat CTA and hypothesis `/api` form.
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
- Editing locked DIFF-308 through DIFF-316
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-317 on commits and this file.

## Verification

Recorded in Result. Codex-safe static checks pass on this tree after the
product blobs landed.

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- `npm --prefix apps/web run typecheck|build` and Playwright
  `test:ui-runtime-smoke`: `node_modules` / `playwright` not installed

## Completion Criteria

Origin validators and the leftover product blobs are landed. Nightly record
matches the tested tree.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
