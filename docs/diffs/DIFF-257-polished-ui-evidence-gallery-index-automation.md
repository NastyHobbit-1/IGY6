# DIFF-257: Polished UI Evidence Gallery and Automatic Search Preparation

Status: Complete

## Type

Change-bearing

## Objective

Fix the IGY6 web UI so it behaves like a polished end-user application: standardized evidence cards with overflow-safe layout, a dedicated Image Gallery section with sort/filter, automatic search index preparation, actionable attention items, improved failed-work handling, Simple/Full mode, and reduced technical language in the default path.

## Baseline Facts

- Branch: `grok`
- Active shell: `ProductApp.tsx` (DIFF-256)
- Runtime: Rust gateway, Rust worker, Next.js web UI, plain CSS in `globals.css`
- Vector collection ensure route exists: `POST /memory/vector/chunks/ensure` (proxied at `/api/memory/vector/chunks/ensure`)
- DIFF-256 remains the prior shell; DIFF-257 completes polish gaps called out in user acceptance testing

## Allowed Scope

- `docs/diffs/DIFF-257-polished-ui-evidence-gallery-index-automation.md`
- `docs/diffs/DIFF-256-polished-professional-ui-redesign.md` (status only: mark Complete)
- `docs/ui/README.md`
- `docs/ui/UI_PLATFORM_RECOMMENDATION.md`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/components/ProductApp.tsx`
- `apps/web/src/app/components/api.ts`
- `apps/web/src/app/components/types.ts` (display-only field additions)
- `apps/web/src/app/components/product/**`
- `apps/web/src/app/components/PipelineOperationsPanel.tsx` (activity wiring only)
- `apps/web/scripts/ui-smoke.mjs`
- `crates/igy6-cli/src/lib.rs` (optional startup vector ensure after schema)
- `configs/rust-cutover-manifest.json` (verification notes only if touched)

## Prohibited Scope

- Framework migration (Next.js stays)
- Tailwind/shadcn introduction
- Database migrations or schema changes
- `.env` / runtime data mutation
- Fake controls or demo data
- Removing real backend capabilities
- Locked DIFF edits
- Merge `main` into `grok`

## Verification

- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `cargo test -p igy6-cli` if CLI touched
- `git diff --check`
- Manual browser checks per acceptance criteria in user request

## Completion Criteria

- Global text overflow utilities applied
- Standardized evidence cards with content-type badge
- Image Gallery top-level section with sort/filter
- Automatic search index preparation on load
- Attention panel with traceable actionable items
- Simple Mode (default) and Full Mode toggle persisted in localStorage
- Failed preparation work items actionable with retry
- UI platform recommendation document added
- Verification passing

## Result

Delivered polished product UI on Next.js + plain CSS:

- Global overflow-safe utilities (`uiOverflowSafe`, `uiClampTitle`, `uiClampSummary`, `uiBreakLong`) applied across product cards, panels, and metadata.
- Standardized `EvidenceCard` with normalized content-type badge, readable fields, and technical details expander.
- Dedicated **Images** section with `ImageGallery` (search, sort, filter, placeholders for broken thumbnails).
- Automatic search preparation on load via client ensure call when Qdrant is reachable; plain-language search status labels.
- `AttentionPanel` with traceable items from `buildAttentionItems` (approvals, deduped failed work, search repair).
- Simple/Full mode toggle persisted in `localStorage` (`igy6-ui-mode`); Simple hides `.productAdvanced` and `[data-ui-full-only]`.
- Actionable failed-work retry on Activity cards and pipeline panel; improved plain-language failure copy.
- `docs/ui/UI_PLATFORM_RECOMMENDATION.md` recommends keeping Next.js.

Verification: `npm --prefix apps/web run typecheck`, `build`, `test:ui-smoke`, `git diff --check` — all passed. Backend/runtime unchanged (no `cargo test` required).