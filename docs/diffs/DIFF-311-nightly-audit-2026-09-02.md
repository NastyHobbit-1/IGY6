# DIFF-311: Nightly RITR audit 2026-09-02

Status: Locked

## Type

Change-bearing

## Objective

Land the origin leftovers DIFF-310 documented but could not safely replace on lowercase `grok`: HomePage same-origin `/api` + Open Chat chip, live route-parity counts `123`/`81`, POST_CUTOVER topology text, and remaining Results-tab wording.

## Baseline Facts

- Active branch: lowercase `grok` at `9042b8e` (DIFF-310 docs) before this DIFF.
- DIFF-310 is locked and was not edited.
- Live route parity: `fastapi=91 rust_native=123 web_used=81 missing_from_rust=0 web_requires_fallback=0`.
- `infra/docker-compose.yml` has no Redis service.
- `/api/analysis/hypotheses` Next.js write proxy already exists.
- Server-side Next proxies and `getJson` still use container/server `API_BASE_URL`. That is correct.

## Allowed Scope

- `apps/web/src/app/components/HomePage.tsx` hypothesis form `data-api-base-url` and start-here Open Chat chip
- `apps/web/src/app/components/MissingEvidencePromptPanel.tsx` Return to Chat label
- `configs/rust-cutover-manifest.json` route_parity counts and current-runtime Redis wording
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` runtime topology web row and supporting-services sentence
- `docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md` Open Chat wording
- `nightly_tasks.md`
- `docs/diffs/DIFF-311-nightly-audit-2026-09-02.md`

## Prohibited Scope

- Other branches
- Promotion to `main`
- Merging open DIFF-294 draft PRs
- Runtime/secret/volume mutation
- Tailwind/shadcn
- Feature removal
- Gateway/worker behavior changes
- Editing locked DIFF-308/DIFF-309/DIFF-310

## Required Tags

DIFF-311 on commits and this file.

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
- HomePage start-here chip says Open Chat.
- Manifest `rust_native_routes=123` and `web_used_routes=81`.
- Current-runtime supporting-service lists do not treat Redis as active Compose.
- POST_CUTOVER topology matches Compose (browser `/api`; Redis not listed as active).

## Out Of Scope Follow-Up

- Owner-land remaining DIFF-294 draft PRs #6/#9/#10/#11.
- Full cargo/clippy matrix and live Playwright/docker smokes on a newer rustc + running stack.
