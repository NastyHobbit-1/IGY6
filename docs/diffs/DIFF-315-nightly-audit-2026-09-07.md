# DIFF-315: Nightly RITR audit 2026-09-07

Status: Locked after this landing

## Result

Landed on origin the leftover product blobs DIFF-314 could only verify locally:

- `apps/web/src/app/components/HomePage.tsx` hypothesis form `data-api-base-url="/api"`; start-here and sidebar CTAs say Open Chat; Chat-tab eyebrows no longer say Results.
- Guided upload, conversation import, and observation next-step copy now point at Chat, not Results.
- Source-trust and saved-answer copy no longer refer to a Results tab.
- `BrowserWebRouterCollectorMvp.tsx` media-tool ClientScript is valid browser JS (no `as any`).
- `configs/rust-cutover-manifest.json` `rust_native_routes=123`, `web_used_routes=81`; Redis removed from current-runtime supporting-service lists.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` web row uses same-origin `/api`; Redis retired from active supporting services.

Verification on this worktree (equivalent to origin after this DIFF):

- `python3 scripts/rust-route-parity.py --check` PASS (`fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 files)
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `scripts/normal-user-product-smoke.sh --check` PASS

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- `npm --prefix apps/web run typecheck|build` and Playwright `test:ui-runtime-smoke`: `node_modules` / `playwright` not installed

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-314 recorded, on lowercase `grok` only, without editing locked DIFF-308 through DIFF-314.

## Baseline Facts

- Active branch: lowercase `grok` at `4b73a8a` (DIFF-314 docs) before this DIFF.
- DIFF-314 is locked and was not edited.
- Origin after DIFF-314 still had the leftover blobs listed in DIFF-314 Result.
- Live route parity after this landing: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-314 listed.
- Align user-facing next-step / CTA copy that still said Results with visible Chat tab labels.
- Update nightly record, WORKING.md / ui README verifier notes.
- Record this nightly DIFF.

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308 through DIFF-314
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-315 on commits and this file.

## Verification

Recorded in Result. Codex-safe static checks pass on the landed tree.

## Completion Criteria

Origin validators and leftover product blobs are landed. Nightly record exists.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
