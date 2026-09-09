# DIFF-316: Nightly RITR audit 2026-09-08

Status: Locked after this correction

## Result

Landed on origin: this record; `nightly_tasks.md`; `docs/WORKING.md` and
`docs/ui/README.md` verifier notes; `SourceTrustSensitivityManagement.tsx`
Chat wording.

Verified locally on a grok worktree but not replaced on origin this run because
GitHub file-update payloads for the remaining product blobs exceed a reliable
PUT size in this agent path (same constraint DIFF-309/315 recorded):

- `HomePage.tsx` `/api` + Open Chat
- Guided / conversation / observation Chat wording
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

Complete the leftover origin landings DIFF-315 recorded, on lowercase `grok`
only, without editing locked DIFF-308 through DIFF-315.

## Baseline Facts

- Active branch: lowercase `grok` at `8958c65` (DIFF-315 docs) before this DIFF.
- DIFF-315 is locked and was not edited.
- Origin after DIFF-315 still had HomePage `NEXT_PUBLIC_API_BASE_URL ??
  "http://127.0.0.1:8000"`, Open Results CTA, collector `as any`, manifest
  `118`/`79` plus Redis in current-runtime lists, and POST_CUTOVER browser
  `http://127.0.0.1:8000` plus Redis-as-active wording.
- Live route parity on the local patched tree:
  `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `apps/web/src/app/page.tsx` remains a two-line HomePage re-export.
- Server-side Next proxies and `getJson` still use container/server
  `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- Land the leftover origin blobs DIFF-315 listed when the PUT path can carry them.
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
