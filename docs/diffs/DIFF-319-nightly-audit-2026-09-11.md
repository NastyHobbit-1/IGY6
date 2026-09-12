# DIFF-319: Nightly RITR audit 2026-09-11

Status: Locked after landing leftover product blobs

## Result

Landed on origin `grok`: this record; `nightly_tasks.md`; verifier notes; and
the product leftovers DIFF-309 through DIFF-318 could only patch locally:

- `HomePage.tsx` hypothesis form `data-api-base-url="/api"` and Open Chat CTA
- Guided / conversation / observation Chat next-steps (not Results)
- `BrowserWebRouterCollectorMvp.tsx` valid browser JS (no `as any`)
- `configs/rust-cutover-manifest.json` `rust_native_routes=123`,
  `web_used_routes=81`, Redis removed from current-runtime lists
- `POST_CUTOVER_ROUTE_AUDIT.md` web row uses `/api` proxies; Redis is not an
  active Compose service
- `scripts/normal-user-product-smoke.sh` now requires the Open Chat CTA and
  hypothesis `/api` form because HomePage on origin satisfies them

Verification on this tree: rust-route-parity --check PASS; test-rust-route-parity
PASS (4); post-cutover-runtime-audit PASS; ui-smoke PASS (53 files);
check-chat-bounds PASS; validate-chat-script PASS; validate-media-script PASS;
validate-panel-scripts PASS (23); normal-user-product-smoke --check PASS.

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-318 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-318.

## Baseline Facts

- Active branch: lowercase `grok` at `14a2071` (DIFF-318 docs) before this DIFF.
- DIFF-318 is locked and was not edited.
- Origin after DIFF-318 still had HomePage `NEXT_PUBLIC_API_BASE_URL ??
  "http://127.0.0.1:8000"`, Open Results CTA, collector `as any`, manifest
  `118`/`79` plus Redis in current-runtime lists, POST_CUTOVER browser
  `http://127.0.0.1:8000` plus Redis-as-active wording, and guided-panel
  next-steps that said Results for evidence.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.
- Route parity guard counts on this tree: fastapi=91 rust_native=123 web_used=81
  missing_from_rust=0 web_requires_fallback=0.

## Allowed Scope

- Land the leftover origin blobs DIFF-318 listed.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Restore product-smoke guards that HomePage can now satisfy.
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
- Editing locked DIFF-308 through DIFF-318
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-319 on commits and this file.

## Verification

Recorded in Result. Codex-safe static checks pass on this tree after the
leftover blobs landed.

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- Playwright `test:ui-runtime-smoke`: requires a running stack and Playwright
  browsers
- `npm --prefix apps/web run typecheck` / `build`: `node_modules` not installed
  in this agent environment

## Completion Criteria

Origin validators and leftover product blobs are landed. Nightly record matches
the verified tree.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
