# DIFF-263: Next.js Polished UI with Simple and Full Modes

Status: Complete

## Type

Change-bearing.

## Objective

Rebuild the current IGY6 web UI into a polished Next.js application with a professional app layout, reusable components, and a real Simple Mode / Full Mode experience. The result must feel like a finished local AI application—not a haphazard technical dashboard.

Keep the UI on Next.js with plain CSS (`globals.css`). Do not migrate to another platform, framework, or component library in this DIFF.

## Baseline Facts

- Branch: `grok`
- Active entrypoint: `apps/web/src/app/page.tsx` → `ProductApp.tsx`
- Prior shell work: **DIFF-256** (six-section product shell), **DIFF-257** (evidence cards, image gallery, search auto-prepare, attention panel, overflow utilities, Simple/Full toggle)
- Runtime unchanged: Rust gateway + Rust worker + Next.js web UI
- Styling: plain CSS in `apps/web/src/app/globals.css` — **no** Tailwind or shadcn on `grok`
- `HomePage.tsx` and ~40 legacy panel components remain in the repo but are **not** the active entrypoint; this DIFF must not reintroduce them into the default user path
- Hidden legacy radio tabs (`tab-results`, `tab-add-data`, etc.) still exist for `nav-script.ts` compatibility — remove or replace without breaking navigation
- Current gallery nav label is **Images**; this DIFF renames the top-level section to **Gallery** per product spec
- `ui-smoke.mjs` and `ui-runtime-smoke.mjs` encode structural expectations and must be updated with the new shell
- Locked DIFFs must not be edited

## Main Problem

Despite DIFF-256/257, the UI still feels cluttered and technical in places:

- Monolithic `ProductApp.tsx` mixes layout, data loading, and section markup
- Reusable primitives exist partially (`ui/`, `product/`) but lack a cohesive component system (shell, headers, mode toggle, drawers)
- Duplicate status surfaces (sidebar, hero counters, pipeline panel, system grid)
- Simple Mode hiding relies on ad hoc `.productAdvanced` / `[data-ui-full-only]` without a single language policy module
- Legacy debug/MVP panels remain importable and some technical strings still surface in Simple Mode
- Evidence and gallery flows work but need standardization pass, full-list behavior, and stronger empty/loading/error states
- Attention items, search preparation, and add-information flows need consolidation so every visible control is real

## Target Architecture

Keep:

```text
Next.js web UI + Rust backend
```

Improve:

```text
Next.js + polished app shell + reusable components + Simple Mode + Full Mode
```

## Required Top-Level Layout

One app shell with exactly these seven sections—no redundant top-level tabs, no fake buttons, no dead controls, no debug panels on the default path:

| # | Section | Purpose |
|---|---------|---------|
| 1 | Command Center | Chat, quick actions, honest runtime snapshot, recent activity |
| 2 | Add Information | Single guided add flow (supported source types only) |
| 3 | Evidence | Standardized evidence cards, all non-image evidence |
| 4 | Gallery | Dedicated image evidence grid (not mixed into Evidence) |
| 5 | Activity | Processing, collections, approvals, actionable work |
| 6 | System | Service health, search status, troubleshooting logs (operator details in Full Mode) |
| 7 | Settings | Ports, storage, model, security, stack restart |

## Simple Mode / Full Mode

### Toggle

Visible control:

```text
Simple | Full
```

- Default: **Simple**
- Persist selection in `localStorage` (key: `igy6-ui-mode`, values: `simple` | `full`)
- Apply via `data-ui-mode` on the app root and `body` classes (`igy6-simple-mode` / `igy6-full-mode`)

### Simple Mode (normal users)

Show:

- Plain-language labels
- Clean summaries
- Best next actions
- Standardized cards
- Minimal technical detail
- Useful status messages

Hide by default:

- Raw JSON
- Internal IDs
- Route names
- Vector/chunk terminology
- Database terminology
- Debug counters
- Raw service/env values

### Full Mode (operators)

May show:

- IDs
- Raw metadata
- API route names
- Backend service details
- Diagnostics
- Advanced actions

Full Mode must remain organized—use expanders/drawers/secondary panels, not wall-of-text dumps.

## Component System

Refactor into reusable components under `apps/web/src/app/components/`. Create or consolidate:

| Component | Responsibility |
|-----------|----------------|
| `shell/AppShell` | Root layout wrapper, sidebar/top nav slot, main content |
| `shell/ProductNav` | Seven-section navigation |
| `shell/SectionHeader` | Eyebrow, title, lead, optional status |
| `shell/ModeToggle` | Simple / Full control |
| `ui/StatusBadge` | Extend or alias `StatusPill` for consistent status display |
| `ui/ActionButton` | Primary/ghost/warn button variants |
| `ui/EmptyState` | Honest empty states (existing; extend) |
| `ui/LoadingState` | Skeleton-based loading (existing `Skeleton` / `SkeletonBlock`) |
| `ui/ErrorState` | Actionable error display with retry when wired |
| `ui/DetailDrawer` | `<details>` or drawer pattern for technical expanders |
| `product/EvidenceCard` | Standardized evidence card (extend existing) |
| `product/SourceCard` | Source summary card where needed |
| `product/ImageCard` | Gallery thumbnail card |
| `product/AttentionItem` | Single attention row with real action |
| `product/SystemServiceCard` | One service health card |
| `product/ChatCommandPanel` | Thin wrapper around `UnifiedChatHub` integration |

`ProductApp.tsx` should compose these components and load data; it should not contain large inline section markup.

Use consistent spacing, typography, button styles, card styling, and responsive behavior via existing semantic classes (`panel`, `productCard`, `productNav`, etc.) before adding new global rules.

## UI-Wide Overflow Fix

Apply globally in `globals.css` and reuse everywhere:

- `uiOverflowSafe` — container overflow containment
- `uiBreakLong` — break long unbroken strings (URLs, hashes, paths)
- `uiClampTitle` — single- or multi-line title clamp
- `uiClampSummary` — summary clamp
- Responsive grids: `productCardGrid`, `responsivePanelGrid`, `responsiveToolbar`

Audit all cards/panels for filenames, URLs, hashes, IDs, MIME strings, paths, and summaries. No text may spill outside card bounds.

## Evidence Page

Every evidence card must show:

1. Content-type badge at top (normalized label)
2. Clean title
3. Source
4. Short summary
5. Status
6. Date/time when available
7. Open/View action (only if a real route/viewer exists)
8. Technical details only inside expander/drawer

Normalized content-type labels (do not show raw MIME/source_type as the main label):

- Image
- PDF
- Text
- Website
- Video
- Audio
- Folder
- Document
- Unknown

Centralize mapping in `product/evidence-display.ts` (extend existing `buildEvidenceCardModel`).

## Gallery Page

Dedicated **Gallery** section (top-level nav label **Gallery**, not Images).

Required:

- Grid of image cards/thumbnails
- Placeholder when thumbnail unavailable
- Sort: newest, oldest, source, file type, status, size (when data available)
- Filter: source, status, file type (when data available)
- Search: filename/title/source
- Image detail/viewer (modal or dedicated panel)
- Loading, empty, and error states

Reuse/extend `product/ImageGallery.tsx` and `product/ImageCard` (extract if inline).

## Add Information Flow

Replace confusing collection-level choices with one guided flow:

1. Choose what to add (text, website, local folder—only if backend supports)
2. Enter/select information
3. IGY6 picks the best supported method automatically
4. Show progress
5. Show result
6. Offer next action (e.g. open Activity, browse Evidence)

Do not expose technical collection mode names in Simple Mode. Extend `product/AddInformationFlow.tsx`; remove dead submit paths.

## Search Index / Preparation Behavior

User-facing rules:

- Never tell the user to manually create a search index in Simple Mode
- Not ready → **Search is being prepared.**
- Auto-trigger preparation on load when Qdrant is reachable (`POST /api/memory/vector/chunks/ensure`)
- Failure → plain-language cause + real retry/repair action (link to System or inline repair)
- Hide “index not created” and similar jargon in Simple Mode
- Full Mode may show collection name, reachability, and technical index metadata

Consolidate status in one primary location per section; remove duplicate search status blocks where possible.

## Attention Items

Requirements:

- Count in sidebar must equal visible, actionable items in the drawer
- Each item: clear reason + real next action (navigate, retry, approve, repair search)
- Deduplicate failed work by type/error signature (extend `product/attention.ts`)
- Simple Mode: hide technical JSON/IDs
- Full Mode: technical details in expander

## Language Cleanup (Simple Mode)

Centralize replacements in `product/plain-language.ts` (new) and apply across shell:

| Technical term | Simple Mode label |
|----------------|-------------------|
| retrieval preview | What IGY6 found |
| connector registry | Ways to add information |
| task plan | Planned work |
| work item | Work |
| vector chunk | Evidence detail |
| Qdrant | Search memory |
| Neo4j | Relationship memory |
| schema | database setup |
| route | hidden unless Full Mode |

Service health labels in Simple Mode should use plain names (API, Search memory, Relationship memory, Background worker)—not raw check keys.

## Visual Quality

- Modern app shell with clear hierarchy
- Clean sidebar navigation (current pattern; polish spacing/typography)
- Polished cards, consistent buttons, useful empty/loading/error states
- No wall-of-text panels, no raw debug dashboard feel, no chaotic grids
- No duplicated status blocks for the same fact
- No fake or dead primary buttons

## Allowed Scope

- `docs/diffs/DIFF-263-nextjs-polished-ui-simple-full-mode.md`
- `docs/ui/README.md` (section names, Simple/Full behavior, Gallery rename)
- `docs/ui/UI_PLATFORM_RECOMMENDATION.md` (status note only if needed)
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/layout.tsx` (metadata/typography only)
- `apps/web/src/app/globals.css`
- `apps/web/src/app/components/ProductApp.tsx`
- `apps/web/src/app/components/product/**`
- `apps/web/src/app/components/shell/**` (new)
- `apps/web/src/app/components/ui/**`
- `apps/web/src/app/components/UnifiedChatHub.tsx` (integration/copy only)
- `apps/web/src/app/components/PipelineOperationsPanel.tsx` (activity section only)
- `apps/web/src/app/components/SettingsPanel.tsx` (settings section wiring/copy only)
- `apps/web/src/app/components/StackRestartPanel.tsx` (settings wiring only)
- `apps/web/src/app/components/UserSecurityPanel.tsx` (settings wiring only)
- `apps/web/src/app/components/helpers.ts` (plain-language copy re-exports only)
- `apps/web/scripts/ui-smoke.mjs`
- `apps/web/scripts/ui-runtime-smoke.mjs`
- `README.md` (UI instructions only if navigation/section names change)

Allowed behavior:

- Extract shell components from `ProductApp.tsx`
- Rename nav label Images → Gallery (section id may remain `gallery`)
- Remove hidden legacy radio tabs when `nav-script.ts` no longer depends on them
- Hide or relocate technical panels from default path (do not delete legacy files unless unused and explicitly listed)
- Wire only real API actions; remove or hide controls that do not call a backend route
- Update smoke scripts for new structure

## Prohibited Scope

- Platform migration (Electron, Tauri, native app, etc.)
- Tailwind, shadcn, or new CSS framework introduction
- Backend API behavior changes unless required for honest UI wiring (prefer existing routes)
- Database migrations, schema changes, Docker/Compose changes
- `.env` or runtime data mutation
- Fake buttons, placeholder panels, demo data, or decorative dead controls
- Removing required API proxy routes under `apps/web/src/app/api/`
- Editing locked DIFFs
- Broad unrelated refactors outside UI scope
- Merging `dev` or `grok` into `main`
- Deleting `HomePage.tsx` or archive panels unless a later DIFF explicitly authorizes cleanup

## Verification

Required commands:

```bash
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run test:ui-smoke
npm --prefix apps/web run test:ui-runtime-smoke
git diff --check
git status --short
```

Manual verification:

- [ ] All seven sections navigate correctly
- [ ] Simple/Full toggle works and persists across refresh
- [ ] Evidence text does not overflow cards
- [ ] Evidence cards show normalized type badges
- [ ] Gallery is its own top-level section labeled **Gallery**
- [ ] Gallery sort/filter/search works where data supports it
- [ ] Technical details hidden in Simple Mode
- [ ] Full Mode shows technical details in expanders cleanly
- [ ] Search preparation auto-runs; plain-language status shown
- [ ] Attention count matches drawer items; each action works
- [ ] Add Information flow completes without mode jargon
- [ ] No fake buttons; no dead clicks on primary controls

## Acceptance Criteria

Complete only when:

- UI remains Next.js with plain CSS
- Layout feels like a polished app, not a technical dashboard
- Simple Mode is default; Full Mode exists and persists
- Reusable shell and UI components are extracted and used consistently
- Evidence cards are standardized with normalized type badges
- Gallery is a dedicated section separate from Evidence
- Text overflow is fixed globally
- Technical clutter is hidden from normal users
- All visible primary actions are real and wired
- `typecheck`, `build`, and `ui-smoke` pass
- `ui-runtime-smoke` passes when environment allows (record skip reason if not)

## Required Tags

Use `DIFF-263` in change summaries, commits, and review notes.

## Commit Message

```text
DIFF-263 Next.js polished UI with Simple and Full modes
```

## Result

- Extracted reusable shell (`AppShell`, `ProductNav`, `SectionHeader`, `ModeToggle`) and UI primitives (`StatusBadge`, `ActionButton`, `ErrorState`, `DetailDrawer`, `LoadingState`).
- Slimmed `ProductApp.tsx` to data loading + section composition via `product/sections/*`.
- Renamed top-level nav **Images** → **Gallery**; evidence list excludes image-type cards (gallery owns images).
- Centralized Simple Mode language in `product/plain-language.ts`; service labels use Search memory / Relationship memory.
- Consolidated duplicate search status in Activity pipeline; authoritative search repair remains in System.
- Updated `ui-smoke.mjs` and `ui-runtime-smoke.mjs` for ProductApp contracts.

Verification: `npm --prefix apps/web run typecheck`, `build`, `test:ui-smoke` passed. `test:ui-runtime-smoke` requires IGY6 at `WEB_BASE_URL` (port 3000 was occupied by another app during verification).