# DIFF-297 — Nightly RITR Audit 2026-08-24

**Branch:** grok (lowercase only)
**Date:** 2026-08-24
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Scope:** Full inspection of IGY6 on `grok`; one smoke-expectation repair for media import markers; no product behavior change

## Summary

One defect found and fully repaired: static UI smoke still expected the pre–binary-media-import collection markers. After DIFF-268 / PR #7, MediaImportMvp uses binary upload (`data-media-upload-binary` / "Upload media file"). Smoke was updated and verified green.

Head SHA before this audit: `eb6890430fa1931c7441f33f9c4bc3a3ec222e63` (DIFF-296).

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, docs/WORKING.md, AGENTS.md, docs/ui/README.md, docs/BRANCH_POLICY.md, nightly_tasks.md, DIFF-294, DIFF-296, package.json, HomePage imports, MediaImportMvp, collection panels, API proxy tree, open draft PRs against grok.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME/unfinished markers: none requiring repair (only HTML placeholders, enum values, historical cutover-manifest notes, bounded gateway status string).
   - `location.reload` / `ThatDog123`: 0 hits.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Known intentional gaps under active DIFF-294 (open draft PRs #6, #9, #10, #11): client-side `NEXT_PUBLIC_API_BASE_URL` still used by several panels (PR #9), package version still `0.0.0-phase0` (PR #11), local_project text-only normalization (PR #10), docs/API alignment (PR #6). Documented as productization scope, not nightly defects.

3. **Repair Loop**
   - **Issue:** `npm --prefix apps/web run test:ui-smoke` failed:
     - `implemented collection panels: data-media-collect-text`
     - `implemented collection panels: Collect extracted text`
   - **Root cause:** Smoke expectations not updated when MediaImportMvp switched to binary upload + worker extraction.
   - **Fix:** In `apps/web/scripts/ui-smoke.mjs`, replace those two expectations with current markers:
     - `data-media-import-mvp`
     - `data-media-upload-binary`
     - `Upload media file`
   - **Verify:** `npm run test:ui-smoke` → PASS (53 component files). `npm run typecheck` → PASS.

4. **Maintenance / Improvement**
   - No product UI/API behavior changes from this nightly.

5. **UI Verification**
   - Media import control and attributes match current MediaImportMvp and docs/ui/README.md.
   - No unfinished or non-functional controls introduced.

6. **Testing**
   - `npm --prefix apps/web run typecheck` — PASS
   - `npm --prefix apps/web run test:ui-smoke` — PASS (after fix)
   - `cargo test --workspace` — blocked in agent env (Cargo.lock v4 / edition2024 toolchain mismatch)
   - `npm run test:ui-runtime-smoke` — requires live stack + Playwright browsers; not runnable here
   - Recommended local matrix:
     ```bash
     git checkout grok
     cp .env.example .env
     ./install.sh
     igy6 start
     npm --prefix apps/web run check
     cargo test --workspace
     scripts/post-cutover-smoke.sh --check
     ```

## Files changed

- `apps/web/scripts/ui-smoke.mjs` — align media collection panel smoke markers with binary import UI
- `nightly_tasks.md` — 2026-08-24 entry
- `docs/diffs/DIFF-297-nightly-audit-2026-08-24.md` — this record

## Remaining blockers

None for this audit. Open DIFF-294 draft PRs remain intentional owner-landed productization work.

## Next recommended work

Continue nightly RITR exclusively on `grok`. Owner may land remaining DIFF-294 draft PRs when ready:
- PR #6 — docs/API alignment + dom-check Chat IA
- PR #9 — web client API proxy consistency (replace remaining NEXT_PUBLIC_API_BASE_URL direct calls)
- PR #10 — local_project text-only normalization
- PR #11 — phase0 Playwright/UI/API smokes + version bump

## Confirmation

- Worked only on lowercase `grok`.
- No existing intended functionality removed.
- No partial fixes left; the discovered smoke failure was fully repaired and verified.
- All hard rules followed.
