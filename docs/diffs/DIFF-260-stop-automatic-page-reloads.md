# DIFF-260: Stop Automatic Page Reloads

Status: Complete

## Objective

Stop the UI from blinking via automatic full-page reloads during data add/process flows. Updates should happen in place; full reload only when the user refreshes the browser or navigates away.

## Allowed Scope

- `docs/diffs/DIFF-260-stop-automatic-page-reloads.md`
- `apps/web/src/app/components/product/nav-script.ts`
- `apps/web/src/app/components/PipelineOperationsPanel.tsx`
- `apps/web/src/app/components/StackRestartPanel.tsx`
- `apps/web/scripts/ui-smoke.mjs`

## Verification

- `npm --prefix apps/web run test:ui-smoke`
- `npm --prefix apps/web run typecheck`

## Result

Removed all automatic `window.location.reload()` calls from the ProductApp path (nav-script, PipelineOperationsPanel, StackRestartPanel). Background actions now update status text in place; legacy auto-queue runs once per session via `sessionStorage`. Sidebar status banner shows non-blocking messages. Full page refresh is user-initiated only.