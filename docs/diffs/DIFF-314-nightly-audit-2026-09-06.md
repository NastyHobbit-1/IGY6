# DIFF-314: Nightly RITR audit 2026-09-06

Status: Locked after this correction

## Result

Landed on origin: this record; `nightly_tasks.md`; `scripts/normal-user-product-smoke.sh`
`apps/web/src` scan; `docs/WORKING.md` and `docs/ui/README.md` verifier notes.

Verified locally on a grok worktree but not replaced on origin this run because
GitHub file-update payloads for the remaining product blobs exceed a reliable
PUT size in this agent path (same constraint DIFF-309/313 recorded):

- `HomePage.tsx` `/api` + Open Chat
- Guided / conversation / observation / source-trust / saved-answer Chat wording
- `BrowserWebRouterCollectorMvp.tsx` valid browser JS (no `as any`)
- `configs/rust-cutover-manifest.json` `123`/`81` and Redis drop
- `POST_CUTOVER_ROUTE_AUDIT.md` topology web row

Local verification of those patched copies: rust-route-parity --check PASS,
test-rust-route-parity PASS (4), post-cutover-runtime-audit PASS, ui-smoke PASS
(53 files), check-chat-bounds PASS, validate-chat-script PASS,
validate-media-script PASS, validate-panel-scripts PASS (23),
normal-user-product-smoke --check PASS.

Origin without the leftover blobs still fails parity + ui-smoke.

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-313 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-313.

## Baseline Facts

- Active branch: lowercase `grok` at `e28f5a2` (DIFF-313 docs) before this DIFF.
- DIFF-313 is locked and was not edited.
- Origin after DIFF-313 still had the leftover blobs listed in DIFF-313 Result.
- Live route parity on the local patched tree:
  `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-313 listed when the PUT path can carry them.
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

Recorded in Result. Codex-safe static checks pass on the locally patched tree.
Origin still has stale HomePage/manifest until those blobs land.

Could not run:

- `cargo test` / `cargo clippy` / `cargo fmt`: rustc 1.75 cannot parse
  lockfile version 4 (`-Znext-lockfile-bump`) / edition2024 crates
- `docker compose ... config` and live stack smokes: `docker` not installed
- `npm --prefix apps/web run typecheck|build` and Playwright
  `test:ui-runtime-smoke`: `node_modules` / `playwright` not installed

## Completion Criteria

Origin validators and nightly record are landed. Product leftover blobs remain
local-verified until a tool can PUT the full files.

## Out Of Scope Follow-Up

- PUT HomePage / manifest / POST_CUTOVER / guided panels with a full-file
  capable git push.
- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc
  + running stack.
- Do not merge this work to `main` without an explicit owner promotion request.
