# DIFF-256: Polished Professional UI Redesign

Status: Active

## Type

Change-bearing

## Objective

Replace the chaotic, redundant, overly technical IGY6 web UI with a polished, professional, modern product interface. The result must feel like a finished 2026 local AI workspace: clean navigation, central chat, one clear add-information flow, readable evidence and activity views, honest system status, and grouped settings. All visible controls must connect to real app state and APIs.

## Baseline Facts

- Branch: `grok`
- Active runtime: Rust gateway, Rust worker, Next.js web UI
- Current UI is implemented in `HomePage.tsx` with 40+ panels, duplicate workflows, advanced surfaces in the main path, and radio-tab navigation
- `UnifiedChatHub`, `ChatRetrievalPreview`, collection proxies, and settings proxies are wired and must keep working
- UI uses plain CSS in `globals.css` (no Tailwind/shadcn on `grok`)
- `apps/web/scripts/ui-smoke.mjs` encodes structural UI expectations and must be updated for the new shell

## Allowed Scope

- `docs/diffs/DIFF-256-polished-professional-ui-redesign.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/layout.tsx` (typography/metadata only if needed)
- `apps/web/src/app/components/ProductApp.tsx`
- `apps/web/src/app/components/product/**`
- `apps/web/src/app/components/UnifiedChatHub.tsx` (navigation integration only)
- `apps/web/src/app/components/helpers.ts` (plain-language copy only)
- `apps/web/scripts/ui-smoke.mjs`
- `docs/ui/README.md` (product navigation and section descriptions)

## Prohibited Scope

- Backend API behavior changes unless required for honest UI wiring
- Database, migrations, Docker, dependency additions
- Runtime/private data mutation
- Removing required API proxy routes
- Fake buttons, placeholder panels, or decorative dead controls
- Promoting dev-only files to `main`
- Broad unrelated cleanup outside UI scope

## Product Sections

1. Command Center — chat, quick actions, runtime summary
2. Add Information — unified add flow (text, website, local folder where supported)
3. Evidence — readable evidence cards, technical details collapsed
4. Activity — work, collections, approvals in plain language
5. System — service health with honest labels
6. Settings — grouped settings, restart, security; advanced diagnostics in drawers only

## Verification

Run:

- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `npm --prefix apps/web run test:ui-runtime-smoke` (if environment allows)
- `git diff --check`
- `git status --short`

Manual: confirm navigation, chat, add-information submit paths, evidence cards, activity cards, system status, and settings render without dead primary buttons.

## Completion Criteria

- Old chaotic main UI replaced by `ProductApp`
- Six coherent product sections with working navigation
- Chat remains central and navigates real sections
- One obvious add-information workflow with only supported options
- Technical data hidden behind Advanced details
- Build and typecheck pass
- UI smoke updated and passing

## Result

- Replaced `HomePage` entry with `ProductApp` — six-section product shell (Command Center, Add Information, Evidence, Activity, System, Settings).
- Central chat (`UnifiedChatHub`) integrated with product navigation via `igy6SwitchSection`.
- Unified add-information flow (`AddInformationFlow`) for text, website, and local folder using real API routes.
- Evidence and activity views use readable cards; technical JSON collapsed under Advanced details.
- System view maps health checks to plain-language service status.
- Settings groups restart, env settings, and user security.
- Updated `ui-smoke.mjs` and `docs/ui/README.md` for the new shell.

## Verification Result

- `npm --prefix apps/web run typecheck` — pass
- `npm --prefix apps/web run build` — pass
- `npm --prefix apps/web run test:ui-smoke` — pass (57 component files)
- `npm --prefix apps/web run test:ui-runtime-smoke` — not run against IGY6 (port 3000 served Open WebUI, not this app)
- `git diff --check` — pass (no whitespace errors)
- `cargo test --workspace` — not required (no backend behavior changes in this DIFF)