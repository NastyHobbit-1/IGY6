# DIFF-318: Nightly RITR audit 2026-09-10

Status: Locked after this landing

## Result

Landed on origin (lowercase `grok` only): the leftover product blobs DIFF-317
could not PUT, plus this record, nightly log, and verifier notes.

Landed product repairs:

- `HomePage.tsx` Ask-with-evidence CTA is **Open Chat**; hypothesis form
  `data-api-base-url="/api"` (no `NEXT_PUBLIC_API_BASE_URL`, no
  `http://127.0.0.1:8000` in browser surfaces).
- Guided upload / conversation import / observation next-steps point at Chat,
  not a visible Results tab.
- Work queue hint points at Chat → Search Memory And Findings.
- `BrowserWebRouterCollectorMvp.tsx` media/deep-scan script is valid browser
  JS (no `as any`).
- `configs/rust-cutover-manifest.json` `rust_native_routes=123`,
  `web_used_routes=81`; Redis removed from current-runtime supporting lists
  and Compose service-name claim.
- `POST_CUTOVER_ROUTE_AUDIT.md` web row uses same-origin `/api`; Redis retired
  from current supporting-service list.
- `scripts/normal-user-product-smoke.sh --check` now requires the Open Chat CTA
  and hypothesis `/api` form that origin HomePage satisfies.

Verification on this tree:

- `python3 scripts/rust-route-parity.py --check` PASS
  (`fastapi=91 rust_native=123 web_used=81 missing_from_rust=0
  web_routes_requiring_fallback=0`)
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

Complete the leftover origin landings DIFF-317 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-317.

## Baseline Facts

- Active branch: lowercase `grok` at `c76b4b6` (DIFF-317 docs) before this DIFF.
- DIFF-317 is locked and was not edited.
- Origin after DIFF-317 still had HomePage
  `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`, Open Results CTA,
  collector `as any`, manifest `118`/`79` plus Redis in current-runtime lists,
  POST_CUTOVER browser `http://127.0.0.1:8000` plus Redis-as-active wording,
  and guided-panel next-steps that said Results for evidence.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service
  (`postgres`, `qdrant`, `neo4j`, `mlflow`, `phoenix`, `api`, `worker`, `web`).

## Allowed Scope

- Land the leftover origin blobs DIFF-317 listed.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Restore product-smoke Open Chat and hypothesis `/api` markers now that
  HomePage satisfies them.
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
- Editing locked DIFF-308 through DIFF-317
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-318 on commits and this file.

## Verification

Recorded in Result. Codex-safe static checks pass on the landed tree.

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- Playwright `test:ui-runtime-smoke`: requires a running stack and Playwright
  browsers

## Completion Criteria

Origin validators and the leftover product blobs are landed on `grok`.
Nightly record matches tested origin behavior.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
