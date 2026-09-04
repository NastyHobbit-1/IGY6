# DIFF-312: Nightly RITR audit 2026-09-03

Status: Locked

## Type

Change-bearing

## Objective

Land the origin leftovers DIFF-311 recorded as complete but that were still
stale on lowercase `grok` at `d7b487c`: HomePage same-origin `/api` + Open Chat
chip, live route-parity counts `123`/`81`, POST_CUTOVER topology text, remaining
user-facing "open Results" next-step copy, and current-runtime Redis wording.

## Baseline Facts

- Active branch: lowercase `grok` at `d7b487c` (DIFF-311 docs) before this DIFF.
- DIFF-311 is locked and was not edited.
- Origin after DIFF-311 still had:
  - HomePage hypothesis form compiling `NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000"`
  - HomePage start-here chip `Open Results` and sidebar `Open chat`
  - `configs/rust-cutover-manifest.json` route_parity `118`/`79` vs live `123`/`81`
  - Manifest current-runtime lists still including Redis
  - POST_CUTOVER web row still saying browser helpers call `http://127.0.0.1:8000`
  - POST_CUTOVER supporting-services sentence still listing Redis as active
  - Guided upload / conversation import / observation next-step copy saying `open Results`
- Live route parity: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `/api/analysis/hypotheses` Next.js write proxy already exists.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. That is correct.
- `infra/docker-compose.yml` has no Redis service.

## Allowed Scope

- `apps/web/src/app/components/HomePage.tsx` hypothesis form `data-api-base-url`, start-here Open Chat chip, sidebar Open Chat label
- `apps/web/src/app/components/GuidedManualTextUpload.tsx` next-step Open Chat wording
- `apps/web/src/app/components/ConversationHistoryImport.tsx` next-step Open Chat wording
- `apps/web/src/app/components/UserObservationIngestion.tsx` next-step Open Chat wording
- `configs/rust-cutover-manifest.json` route_parity counts and current-runtime Redis wording
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` runtime topology web row and supporting-services sentence
- `docs/agents/CODEX_PROMPT_BASELINE.md` current-runtime supporting-service sentence
- `docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md` Chat history / report wording
- `nightly_tasks.md`
- `docs/diffs/DIFF-312-nightly-audit-2026-09-03.md`

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308/DIFF-309/DIFF-310/DIFF-311
- Cargo.lock / edition downgrade to paper over rustc 1.75

## Required Tags

DIFF-312 on commits and this file.

## Verification

- `python3 scripts/rust-route-parity.py --check`
- `python3 scripts/test-rust-route-parity.py`
- `python3 scripts/post-cutover-runtime-audit.py`
- `node apps/web/scripts/ui-smoke.mjs`
- `npm --prefix apps/web run typecheck` when node_modules is available
- cargo/clippy recorded if blocked

## Completion Criteria

- No client component compiles `NEXT_PUBLIC_API_BASE_URL` or `http://127.0.0.1:8000`.
- Hypothesis form `data-api-base-url="/api"`.
- HomePage start-here chip and sidebar primary control say Open Chat.
- User-facing next-step copy no longer says open Results.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- Current-runtime supporting-service lists do not treat Redis as active Compose.
- POST_CUTOVER topology matches Compose (browser `/api`; Redis not listed as active).

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
- Historical "Grok6 clone" wording in older capability-table / plan docs.
