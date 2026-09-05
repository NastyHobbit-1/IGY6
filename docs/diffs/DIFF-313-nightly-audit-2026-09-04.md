# DIFF-313: Nightly RITR audit 2026-09-04

Status: Locked

## Result

Landed on origin: this record; `nightly_tasks.md`; `apps/web/package.json` `test:panel-scripts`; `.gitignore` generated extract ignores; chat/media/panel validators pointed at split component files.

Verified locally on a grok worktree but not replaced on origin this run because GitHub file-update payloads truncate the large blobs (same constraint DIFF-309/312 recorded):

- `HomePage.tsx` `/api` + Open Chat
- Guided / conversation / observation / source-trust Chat wording
- `BrowserWebRouterCollectorMvp.tsx` valid browser JS (no `as any`)
- `configs/rust-cutover-manifest.json` `123`/`81` and Redis drop
- `POST_CUTOVER_ROUTE_AUDIT.md` topology web row
- `scripts/normal-user-product-smoke.sh` `apps/web/src` scan
- `docs/WORKING.md` and `docs/ui/README.md` verifier notes

Local verification of those patched copies: rust-route-parity --check PASS, test-rust-route-parity PASS (4), post-cutover-runtime-audit PASS, ui-smoke PASS (53 files), check-chat-bounds PASS, validate-chat-script PASS, validate-media-script PASS, validate-panel-scripts PASS (23), normal-user-product-smoke --check PASS.

## Type

Change-bearing

## Objective

Complete the leftover origin landings DIFF-312 recorded, then repair stale
checks and one broken browser script discovered while verifying those
landings on lowercase `grok`.

## Baseline Facts

- Active branch: lowercase `grok` at `3262170` (DIFF-312 docs) before this DIFF.
- DIFF-312 is locked and was not edited.
- Origin after DIFF-312 still had:
  - HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`
  - HomePage start-here chip `Open Results` and sidebar `Open chat`
  - Guided / conversation / observation next-step `open Results`
  - Manifest route_parity `118`/`79` vs live `123`/`81`
  - Manifest current-runtime lists still including Redis
  - POST_CUTOVER web row still saying browser helpers call `http://127.0.0.1:8000`
  - POST_CUTOVER supporting-services sentence still listing Redis as active
- Live route parity: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `/api/analysis/hypotheses` Next.js write proxy already exists.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.
- `apps/web/src/app/page.tsx` is a two-line HomePage re-export. Product markers
  and inline scripts live under `apps/web/src/app/components/`.
- BrowserWebRouterCollectorMvp media-tools `ClientScript` contained TypeScript
  `as any`, which is invalid in the browser.

## Allowed Scope

Same as DIFF-313 body previously recorded. Locked after this correction.

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308 through DIFF-312
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-313 on commits and this file.

## Verification

Recorded in Result.

## Completion Criteria

Origin validators and nightly record are landed. Product leftover blobs remain local-verified until a tool can PUT the full files.

## Out Of Scope Follow-Up

- PUT HomePage / manifest / POST_CUTOVER / guided panels with a full-file capable git push.
- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
