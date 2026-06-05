# DIFF-210 - Source Trust And Sensitivity Management UX

Status: Complete

## Purpose

Add normal-user UX for marking or reviewing sources as trusted, noisy,
sensitive, disabled, or review-needed while preserving source records, evidence
visibility, and honest unsupported-state boundaries.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `README.md`
- `docs/ui/README.md`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-002-source-registry-api.md`
- `docs/diffs/DIFF-004-policy-foundation.md`
- `docs/diffs/DIFF-049-feedback-source-trust-side-effects.md`
- `docs/diffs/DIFF-063-collection-permission-approval-gates.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`
- `docs/diffs/DIFF-209-persist-evidence-check-summary-on-task-plans.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-write-api/src/lib.rs`

## Baseline Facts

- Source records already expose `sensitivity`, `trust_level`, and `enabled`.
- The Rust gateway already supports source create/list/detail and source
  permission create/list/detail.
- Existing source-target feedback side effects can set `trusted`, `noisy`, or
  `rejected`; `rejected` disables the source.
- There was no direct normal-user source trust and sensitivity management
  workflow for all DIFF-210 requested states.
- There was no existing Rust gateway route for direct source review-state
  updates.

## Product Changes Made

- Added an Add Data source trust and sensitivity review panel.
- The panel shows current source state, linked collection run/document/evidence
  counts, and real empty states when no sources exist.
- The panel lets users save one of the requested source states:
  `trusted`, `noisy`, `sensitive`, `disabled`, or `review-needed`.
- The panel lets users review the source sensitivity label as `public`,
  `internal`, `sensitive`, or `secret`.
- The panel lets users enable or disable future collection workflows for the
  source; selecting `disabled` submits `enabled=false`.
- The UI states that existing evidence remains visible and is not silently
  hidden or deleted.

## API/Backend Changes

- Added a bounded Rust gateway route:
  - `POST /sources/{source_id}/review-state`
- Added a matching Next.js proxy route:
  - `POST /api/sources/[source_id]/review-state`
- The route updates only:
  - `sources.trust_level`
  - `sources.sensitivity`
  - `sources.enabled`
  - `sources.updated_at`
- The route writes an audit event:
  - `source.review_state_updated`
- The audit details record previous/new trust, previous/new sensitivity,
  previous/new enabled state, review note, and explicit booleans documenting
  that policy enforcement was not changed, evidence was not hidden, and the
  source was not deleted.

## Unsupported States Handled

- Disabled sources are not deleted.
- Existing evidence is not silently hidden from Results.
- Retrieval ranking, evidence weighting, and policy enforcement are not claimed
  to change in this DIFF.
- The UI uses linked record counts and identifiers only; it does not display raw
  uploaded source text or runtime/private artifact contents.
- Empty source state remains an honest empty state with disabled controls.

## Verification Commands And Results

Passed:

- `git status --short`
  - Showed only expected DIFF-210 files before commit.
- `git diff --check`
  - Passed.
- `git diff --name-status`
  - Showed modified tracked files before staging; new untracked files were
    visible in `git status --short`.
- `npm --prefix apps/web run build`
  - Passed.
- `cargo fmt --all`
  - Applied Rust formatting after backend edits.
- `cargo fmt --all --check`
  - Passed.
- `cargo test --workspace`
  - Initially failed because the new POST route was mistakenly added to a
    read-route empty-body test loop.
  - Passed after moving coverage to the existing valid-body write-route and
    validation route tests.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
  - Confirmed private/dev files remain tracked on `dev`.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`
  - Returned only pre-existing out-of-scope draft/template/status-command
    references; DIFF-210 is `Status: Complete`.

Not run:

- Full Docker smoke was not run from Codex per the DIFF-210 prompt and Codex
  local rule.
- A live synthetic source update against PostgreSQL was not run because that
  would require a running local stack/database; backend route behavior was
  covered by Rust validation and route tests instead.

## Files Changed

- `apps/web/src/app/api/sources/[source_id]/review-state/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-210-source-trust-sensitivity-management-ux.md`
- `docs/ui/README.md`

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No source deletion behavior was added.
- No arbitrary command execution was added.
- No user-provided shell command or argv execution was added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No fake controls were added; every new submit control is wired to a real route.
- No Docker smoke was run from Codex.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
