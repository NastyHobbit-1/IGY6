# DIFF-313: Nightly RITR audit 2026-09-04

Status: Locked

## Result

Landed on origin the leftovers DIFF-312 verified locally but could not replace
because of GitHub file-update payload limits, plus follow-on repairs found in
this run:

- HomePage hypothesis form `data-api-base-url="/api"`
- HomePage start-here chip and sidebar primary control say Open Chat
- Guided upload / conversation import / observation next-step and approval
  copy point at Chat, not a Results tab
- Source trust copy and Work pipeline hint no longer name a Results tab
- Manifest `rust_native_routes=123` / `web_used_routes=81`
- Redis dropped from current-runtime supporting-service lists
- POST_CUTOVER topology: browser same-origin `/api`; Redis retired
- Product-smoke marker check searches `apps/web/src` after the page split
- Chat/media/panel script validators read component files
- Browser collector media-library script no longer embeds TypeScript `as any`

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

- `apps/web/src/app/components/HomePage.tsx`
- `apps/web/src/app/components/GuidedManualTextUpload.tsx`
- `apps/web/src/app/components/ConversationHistoryImport.tsx`
- `apps/web/src/app/components/UserObservationIngestion.tsx`
- `apps/web/src/app/components/SourceTrustSensitivityManagement.tsx`
- `apps/web/src/app/components/BrowserWebRouterCollectorMvp.tsx`
- `apps/web/scripts/check-chat-bounds.mjs`
- `apps/web/scripts/validate-chat-script.mjs`
- `apps/web/scripts/validate-media-script.mjs`
- `apps/web/scripts/validate-panel-scripts.mjs`
- `apps/web/package.json`
- `scripts/normal-user-product-smoke.sh`
- `.gitignore`
- `configs/rust-cutover-manifest.json` route_parity counts and current-runtime Redis wording
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` runtime topology web row and supporting-services sentence
- `docs/WORKING.md`
- `docs/ui/README.md`
- `nightly_tasks.md`
- `docs/diffs/DIFF-313-nightly-audit-2026-09-04.md`

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

- `python3 scripts/rust-route-parity.py --check`
- `python3 scripts/test-rust-route-parity.py`
- `python3 scripts/post-cutover-runtime-audit.py`
- `node apps/web/scripts/ui-smoke.mjs`
- `node apps/web/scripts/check-chat-bounds.mjs`
- `node apps/web/scripts/validate-chat-script.mjs`
- `node apps/web/scripts/validate-media-script.mjs`
- `node apps/web/scripts/validate-panel-scripts.mjs`
- `scripts/normal-user-product-smoke.sh --check`
- `npm --prefix apps/web run typecheck` when node_modules is available
- cargo/clippy recorded if blocked

## Completion Criteria

- No client component compiles `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- Hypothesis form `data-api-base-url="/api"`.
- HomePage start-here chip and sidebar primary control say Open Chat.
- User-facing next-step copy no longer says open Results or treats Results as a tab.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- Current-runtime supporting-service lists do not treat Redis as active Compose.
- POST_CUTOVER topology matches Compose (browser `/api`; Redis not listed as active).
- Product-smoke `--check` finds markers in the split component tree.
- Chat/media/panel script validators pass against component files.
- Collector media-library script is valid browser JavaScript.

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
- Historical "Grok6 clone" wording in older capability-table / plan docs.
- Internal panel eyebrows that still say Results/Add Data as section headings inside Chat/Data.
