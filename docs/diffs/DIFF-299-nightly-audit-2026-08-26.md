# DIFF-299 — Nightly RITR Audit 2026-08-26

**Branch:** grok (lowercase only)
**Date:** 2026-08-26
**Type:** Nightly Repair-Improve-Test-Repeat (RITR) audit
**Status:** Locked
**Scope:** Full inspection of IGY6 on `grok`; repair stale route-parity guard matching for query-string web URLs

## Summary

Nightly audit found a failing route-parity guard: `GET /ops/runtime-logs?limit=120` from the Next.js ops-logs proxy was treated as requiring FastAPI fallback even though the Rust gateway implements `GET /ops/runtime-logs`. The scanner now strips query and hash fragments from discovered web paths. Recorded route counts in the classification file and cutover manifest were updated to the live inventory (rust_native=123, web_used=81).

No product UI or gateway behavior changed. Known intentional DIFF-294 draft PRs (#6, #9, #10, #11) remain out of nightly scope.

Head SHA at audit start: `59d2fb6a7e8c180a982ed90b1625f3da0428c6a1` (DIFF-298).

## Inspection performed

1. **Sync & Inspect**
   - Confirmed active branch is exactly `grok`.
   - Never touched `main`, `dev`, `Grok`, or any other branch.
   - Read README.md, AGENTS.md, docs/WORKING.md, docs/ui/README.md, docs/BRANCH_POLICY.md, nightly_tasks.md, DIFF-294, DIFF-298, package.json, ui-smoke.mjs, MediaImportMvp, route-parity scripts, open draft PRs against grok.
   - Shallow clone of `grok` for local verification.

2. **Full Functionality Audit**
   - Product-code TODO/FIXME/unfinished markers: none requiring repair.
   - `location.reload` / `ThatDog123`: 0 hits under apps/web.
   - Tabs remain Chat / Data / Work / Settings / More.
   - Media import markers still match ui-smoke.
   - `NEXT_PUBLIC_API_BASE_URL` still present in several client panels — covered by open draft PR #9.
   - package.json version still `0.0.0-phase0` — covered by open draft PR #11.

3. **Repair Loop**
   - Root cause: `web_used_routes()` kept `?limit=120` on the path; `route_matches` compared path segments and failed against `GET /ops/runtime-logs`. Classification/manifest counts lagged later native routes.
   - Fix: `normalize_web_path()` in `scripts/rust-route-parity.py`; refresh counts in `configs/legacy-fastapi-route-classification.json` and `configs/rust-cutover-manifest.json`.
   - Verification below.

4. **Maintenance / Improvement**
   - No product UI/API behavior changes from this nightly.

5. **UI Verification**
   - Visible controls purposeful; media import and collection panels match docs.
   - No unfinished or non-functional controls introduced by this audit.

6. **Testing**
   - `python3 scripts/test-rust-route-parity.py` — **PASS** (4 tests)
   - `python3 scripts/rust-route-parity.py --check` — **PASS** (91 / 123 / 81 / missing 0 / fallback 0)
   - `python3 scripts/post-cutover-runtime-audit.py` — **PASS**
   - `npm --prefix apps/web run test:ui-smoke` — **PASS** (53 component files)
   - `npm --prefix apps/web run typecheck` — **PASS**
   - `cargo test --workspace` / `cargo clippy` — blocked (agent rustc 1.75 / Cargo.lock edition2024)
   - `docker compose ... config` — blocked (docker not installed)
   - `npm run test:ui-runtime-smoke` / `dom-check.mjs` — require Playwright browsers; not runnable here

## Files changed

- `scripts/rust-route-parity.py` — strip query/hash from web-discovered paths
- `configs/legacy-fastapi-route-classification.json` — live route_parity counts
- `configs/rust-cutover-manifest.json` — live route_parity counts
- `nightly_tasks.md` — 2026-08-26 entry
- `docs/diffs/DIFF-299-nightly-audit-2026-08-26.md` — this record

## Remaining blockers

None for this audit. Open DIFF-294 draft PRs remain intentional owner-landed productization work:
- PR #6 — docs/API alignment + dom-check Chat IA
- PR #9 — web client API proxy consistency (replace remaining NEXT_PUBLIC_API_BASE_URL direct calls)
- PR #10 — local_project text-only normalization
- PR #11 — phase0 Playwright/UI/API smokes + version bump

## Next recommended work

Continue nightly RITR exclusively on `grok`. Owner may land remaining DIFF-294 draft PRs when ready.

## Confirmation

- Worked only on lowercase `grok`.
- No existing intended functionality removed.
- No partial fixes left.
- All hard rules followed.
