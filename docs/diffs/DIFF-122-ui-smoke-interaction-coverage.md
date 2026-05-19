# DIFF-122: UI Smoke Interaction Coverage

Status: Locked

## Type

Change-bearing

## Objective

Add focused UI smoke and interaction coverage for the reorganized workflow sections introduced by DIFF-121, with emphasis on Assistant action gating, Advanced panels, and manual upload guidance states.

## Baseline Facts

- DIFF-121 reorganized the web UI into Home, Assistant, Data & Knowledge, Work & Processing, Reports, Safety & Audit, and Settings.
- DIFF-121 moved raw IDs, raw JSON, route/debug detail, approval IDs, and legacy route forms behind Advanced sections.
- DIFF-121 preserved existing backend behavior and existing API calls.
- `apps/web/package.json` currently has no existing web test script according to the DIFF-121 closeout summary.
- Route parity remains mixed: FastAPI is still required, and Rust-only cannot honestly be claimed until manifest and route parity prove it.

## Allowed Scope

- Add focused web UI smoke/interaction tests or equivalent lightweight coverage for the reorganized workflow sections.
- Add or update the minimal test configuration required to run those web UI checks.
- Cover Assistant action gating states.
- Cover Advanced panel visibility/toggle behavior for raw IDs, raw JSON, route/debug details, approval IDs, and legacy route forms.
- Cover manual upload guidance and disabled/empty/loading/error guidance states where directly relevant to the reorganized UI.
- Update documentation only where needed to describe how to run the new UI smoke/interaction checks.
- Touch only files required for the web UI coverage and its documentation.

## Prohibited Scope

- No backend behavior changes.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim unless manifest and route parity prove it.
- No broad UI redesign.
- No navigation reorganization beyond test-required selectors or accessibility hooks.
- No dependency changes unless required for the selected test runner and explicitly documented in completion notes.
- No data model changes.
- No migration changes.
- No unsafe deletion.
- No secrets or runtime/private data commits.
- No edits to locked DIFF files.

Unless explicitly allowed here, the following are prohibited:

- Renames.
- Refactors.
- Behavior changes.
- Rewiring.
- Redesign.
- Dependency changes.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

## Required Tags

- Use `DIFF-122` in commits, pull requests, and review notes for this work.
- If code comments are needed to explain test-only behavior, tag them with `DIFF-122`.
- Do not add inline tags to unrelated production code unless the code is changed within this DIFF scope.

## Verification

Required checks:

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- Run the new UI smoke/interaction check command added by this DIFF.
- Run existing web tests if a test script is added or already present.
- `python3 scripts/rust-route-parity.py --check` if route usage changes.
- `scripts/rust-cutover.sh --check` if manifest/runtime docs change.
- `cargo fmt --all --check` only if Rust files change.
- `cargo clippy --workspace --all-targets` only if Rust files change.
- `cargo test --workspace` only if Rust files change.

Expected outcomes:

- Build passes.
- New UI smoke/interaction checks pass.
- No route parity regression is introduced.
- FastAPI fallback status remains honestly documented if touched.

## Completion Criteria

This DIFF may be marked complete only when:

- Focused UI smoke/interaction coverage exists for the reorganized workflow sections.
- Assistant action gating behavior is covered.
- Advanced panels for primary hidden/debug fields are covered.
- Manual upload guidance states are covered.
- The new check command is documented if it did not previously exist.
- Required verification commands have been run and results are recorded in this DIFF.
- No out-of-scope files or behavior changes are included.

## Out Of Scope Follow-Up

- Full end-to-end browser automation beyond focused smoke/interaction coverage.
- Backend route migration.
- Rust-only cutover.
- FastAPI removal.
- Large UI redesign or information architecture changes beyond DIFF-121.
- Broad accessibility audit beyond selectors and interaction coverage required for this DIFF.

## Completion Notes

- Added `apps/web/scripts/ui-smoke.mjs`, a dependency-free smoke check for the
  reorganized DIFF-121 UI contract.
- Added `npm --prefix apps/web run test:ui-smoke`.
- Added `npm --prefix apps/web test` as an alias for the smoke check.
- Covered top-level workflow navigation, workflow section anchors, Assistant
  action labels, Assistant gated action buttons, Advanced panel summaries, raw
  parameter/approval controls behind Advanced, the Advanced Route Console,
  manual upload guidance steps/examples, empty/next-step guidance states, and
  visible local-first/evidence-only/approval-gated safety posture text.
- Updated README and user guide with the new UI smoke check command.
- No backend behavior, route usage, dependencies, data models, or migrations
  changed.

## Verification Results

- Passed: `git status --short`
- Passed: `git diff --check`
- Passed: `npm --prefix apps/web run build`
- Passed: `npm --prefix apps/web run test:ui-smoke`
- Passed: `npm --prefix apps/web test`
- Passed: `python3 scripts/rust-route-parity.py --check`
  - `fastapi=91`
  - `rust_native=64`
  - `web_used=45`
  - `missing_from_rust=30`
  - `web_requires_fallback=0`
- Not run: `scripts/rust-cutover.sh --check`; manifest/runtime docs did not
  change.
- Not run: Cargo fmt/clippy/test; no Rust files changed.
