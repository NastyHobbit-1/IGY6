# DIFF-259: Fix Qdrant Search Status Polling Loop

Status: Complete

## Objective

Stop the ~1Hz Qdrant `GET /collections/igy6_chunks` spam caused by stale vector status (`exists: false`) and the web UI auto-ensure reload loop.

## Allowed Scope

- `docs/diffs/DIFF-259-fix-qdrant-search-status-polling-loop.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/components/product/nav-script.ts`
- `apps/web/scripts/ui-smoke.mjs` (if needed)

## Verification

- `cargo test -p igy6-gateway`
- `npm --prefix apps/web run test:ui-smoke`

## Result

Root cause: `GET /memory/vector/chunks` always returned `exists: false` (DIFF-108 read-only stub). DIFF-257 auto-ensure saw "Search is being prepared" on every page load, called ensure (Qdrant GET), reloaded, and repeated ~1Hz.

Fix: live Qdrant collection check on GET; auto-ensure runs once per browser session without reload loop; added GET `/api/memory/vector/chunks` proxy.

Verification passed.