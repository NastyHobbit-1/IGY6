# DIFF-290: Ops logs, live Qdrant status, in-place UI updates

Status: Complete

## Type

Change-bearing

## Branch

- Branch: `grok` only (lowercase)
- Do not touch `main`, `dev`, or `Grok`

## Objective

Complete three leftover operator/UX fixes from the abandoned June 10 local `grok` commits, implemented against the current `HomePage` product (not ProductApp):

1. Persistent startup/error troubleshooting logs under `IGY6_DATA_ROOT/ops/` with a Settings UI.
2. Honest live `GET /memory/vector/chunks` Qdrant collection existence (no DIFF-108 `exists: false` stub).
3. Stop remaining automatic `window.location.reload()` calls so actions update in place.

## Baseline Facts

- Current entrypoint is `apps/web/src/app/page.tsx` → `HomePage.tsx` (Chat / Data / Work / Settings / More).
- `igy6 start` does not write `ops/startup.log` or `ops/error.log`.
- `GET /memory/vector/chunks` still returns `"exists": false` with a DIFF-108 read-only note, even though POST ensure/search/upsert already talk to Qdrant.
- Automatic full-page reloads remain in Pipeline operations dispatch, collection approval decisions, experiment status updates, and hypothesis create.
- Simple mode already exists in `HomePage`; this DIFF does not rebuild the shell.

## Allowed Scope

- `docs/diffs/DIFF-290-ops-logs-live-qdrant-status-in-place-ui.md`
- `crates/igy6-cli/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-gateway/src/main.rs`
- `apps/web/src/app/api/ops/**`
- `apps/web/src/app/components/TroubleshootingLogsPanel.tsx`
- `apps/web/src/app/components/HomePage.tsx`
- `apps/web/src/app/components/SettingsHubNav.tsx`
- `apps/web/src/app/components/types.ts`
- `apps/web/src/app/components/PipelineOperationsPanel.tsx`
- `apps/web/src/app/components/SourceCollectionApprovalReview.tsx`
- `apps/web/src/app/components/ImprovementExperimentReview.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/scripts/ui-smoke.mjs`
- `docs/ui/README.md`

## Prohibited Scope

- ProductApp / ModeToggle / Simple-Full shell rewrite
- Framework, Tailwind, shadcn, dependency, schema, or Docker changes
- Secret exposure in logs
- Editing locked DIFFs
- Automatic page reloads in the patched flows
- `main`, `dev`, or `Grok`

## Verification

- `cargo test -p igy6-cli`
- `cargo test -p igy6-gateway`
- `npm --prefix apps/web run test:ui-smoke`
- `git diff --check`

## Completion Criteria

- `igy6 start` writes redacted timestamped lines to `ops/startup.log`; failures write to `ops/error.log`.
- Gateway records listen/startup and request failures to the same files.
- `GET /ops/runtime-logs` (proxied at `/api/ops/runtime-logs`) returns tailed startup/error lines.
- Settings shows a Refresh logs panel; no fake data.
- `GET /memory/vector/chunks` inspects Qdrant (no collection create on GET). `exists` is true only when verified.
- Pipeline dispatch, approval decision, experiment status, and hypothesis create update in place with no `window.location.reload()`.

## Result

Implemented against current `HomePage` (not ProductApp).

1. **Ops logs.** CLI writes redacted timestamped lines to `IGY6_DATA_ROOT/ops/startup.log` on `igy6 start` and `ops/error.log` on start failures. Gateway appends listen/startup and request-failure lines. `GET /ops/runtime-logs` and `POST /ops/runtime-logs/append` are Rust-native. Next.js proxies them at `/api/ops/runtime-logs`. Settings → Troubleshooting shows tailed startup/error logs with Refresh.
2. **Live Qdrant GET.** `GET /memory/vector/chunks` now inspects Qdrant (TCP + collection GET). It does not create the collection. `exists` is true only when Qdrant confirms the collection. Unreachable Qdrant returns `exists: false` with `collection_existence_verified: false`. The DIFF-108 `read_only_status` stub is gone from this route.
3. **In-place UI.** Removed automatic `window.location.reload()` from pipeline dispatch, collection/agent approval decisions, experiment status updates, and hypothesis create. Those flows now update status text / item state in place.

## Verification Result

- `cargo test -p igy6-cli --offline`: 13 passed
- `cargo test -p igy6-gateway --offline --lib` filters `vector_collection_status_live runtime_logs_route rust_native_route_registry status_config_routes`: 4 passed
- DIFF-290 UI smoke strings present (`TroubleshootingLogsPanel`, `/api/ops/runtime-logs`, `data-hypothesis-create-result`)
- `git grep location.reload -- apps/web`: no matches
- `git diff --check`: clean
- Full `npm --prefix apps/web run test:ui-smoke` still fails two **pre-existing** origin/grok checks (`data-media-collect-text`, `Collect extracted text`) that are not in this DIFF and are absent from current Media Import UI (`Upload media file`). Not changed here.
