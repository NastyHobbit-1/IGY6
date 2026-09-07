# DIFF-314: Nightly RITR audit 2026-09-06

Status: Locked after this landing

## Result

Landed on origin the leftover product blobs DIFF-313 verified locally but could
not PUT:

- `HomePage.tsx` hypothesis form same-origin `/api`; sidebar and start-here
  chip say Open Chat
- Guided / conversation / observation / source-trust / saved-answer wording
  points at Chat, not Results
- `BrowserWebRouterCollectorMvp.tsx` media-tools script is valid browser JS
  (no TypeScript `as any`)
- `configs/rust-cutover-manifest.json` route_parity `123` / `81`; Redis dropped
  from current-runtime supporting-service lists
- `POST_CUTOVER_ROUTE_AUDIT.md` web row uses same-origin `/api`; Redis retired
  from current supporting-services sentence
- `scripts/normal-user-product-smoke.sh --check` scans `apps/web/src`
- `docs/WORKING.md` and `docs/ui/README.md` verifier notes

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-313 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-313.

## Baseline Facts

- Active branch: lowercase `grok` at `e28f5a2` (DIFF-313 docs) before this DIFF.
- DIFF-313 is locked and was not edited.
- Origin after DIFF-313 still had the leftover blobs listed in DIFF-313 Result.
- Live route parity after this landing:
  `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-313 listed.
- Align user-facing next-step / CTA copy that still said Results with visible
  Chat tab labels.
- Update product-smoke source scan and verifier docs.
- Record this nightly DIFF and `nightly_tasks.md` entry.

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308 through DIFF-313
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-314 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check` PASS
  (`fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`)
- `python3 scripts/test-rust-route-parity.py` PASS (4)
- `python3 scripts/post-cutover-runtime-audit.py` PASS
- `node apps/web/scripts/ui-smoke.mjs` PASS (53 component files)
- `node apps/web/scripts/validate-panel-scripts.mjs` PASS (23)
- `node apps/web/scripts/validate-chat-script.mjs` PASS
- `node apps/web/scripts/validate-media-script.mjs` PASS
- `node apps/web/scripts/check-chat-bounds.mjs` PASS
- `scripts/normal-user-product-smoke.sh --check` PASS

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- `npm --prefix apps/web run typecheck|build` and Playwright
  `test:ui-runtime-smoke`: `node_modules` / `playwright` not installed

## Completion Criteria

Origin product leftovers from DIFF-313 are replaced and the static Codex-safe
checks above pass on the landed tree.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
