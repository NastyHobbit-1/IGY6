# UI Platform Recommendation (DIFF-257)

## Current state

IGY6 ships a Next.js App Router UI with server-rendered sections (`ProductApp.tsx`), plain CSS in `globals.css`, and small client scripts for navigation, mode toggles, gallery filtering, and repair actions. There is no Tailwind or component library on the `grok` branch.

## Limitations observed

- **Mixed rendering model:** Most UI is SSR; interactive pieces (gallery sort/filter, mode toggle, retries) use inline `ClientScript` blocks instead of a cohesive client component layer. This works but spreads behavior across TSX and string scripts.
- **No design system package:** Visual polish depends on hand-maintained CSS classes. Scaling new card types and overflow-safe layouts is manual but manageable.
- **Single-page shell:** The product shell is one large server component with section toggling. This is fine for a local-first tool but will grow harder to test in isolation without extracting more presentational components.
- **Plain CSS maintenance:** Global utilities (`uiOverflowSafe`, `uiClampTitle`, etc.) help, yet complex widgets (image viewer, drag/drop uploads) would benefit from focused client components.

## Recommendation

**Keep Next.js** for the active UI. The current architecture can support a polished end-user experience without a framework change.

### Near-term next steps

1. Continue extracting product components under `apps/web/src/app/components/product/` (evidence cards, gallery, attention panel, mode helpers).
2. Keep plain CSS with semantic product classes; add utilities only where patterns repeat.
3. Prefer small `"use client"` modules only when SSR + `ClientScript` becomes awkward (e.g., a true lightbox image viewer).
4. Expand `ui-smoke.mjs` checks as new sections and mode behavior land.

### Component library

Do **not** introduce Tailwind/shadcn in this branch unless a future DIFF explicitly authorizes it. If visual velocity becomes a bottleneck, evaluate a **small internal token layer** (spacing, radii, typography variables) in `globals.css` before adopting an external library.

### Tauri / native shell

A Tauri or native desktop shell is **not required** for the current local-first browser workflow. Docker Compose + browser UI matches the product goal today. Revisit native packaging only if the owner wants offline installer branding, OS tray integration, or filesystem APIs beyond the host bridge.

## Migration risks if leaving Next.js

- Loss of existing API proxy routes under `apps/web/src/app/api/**`.
- Rewrite cost for chat, settings verification, and pipeline repair flows.
- Duplicate auth/session handling for a new shell.

## Conclusion

Stay on **Next.js + plain CSS + incremental product components**. DIFF-257 demonstrates that polish, Simple/Full mode, gallery, and automatic search preparation can be delivered without platform churn.