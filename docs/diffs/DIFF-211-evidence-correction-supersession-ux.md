# DIFF-211 - Evidence Correction And Supersession UX

Status: Complete

## Purpose

Add a safe normal-user way to mark evidence as corrected, superseded, disputed,
verified, or needing correction while preserving original evidence and lineage.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `47fcd7accdaebe7f9f7fd3016ffe3ce9f73cbfb7`
- `dev` ahead/behind `origin/dev` before commit: aligned with `origin/dev`
  according to `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-210-source-trust-sensitivity-management-ux.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-209-persist-evidence-check-summary-on-task-plans.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-write-api/src/lib.rs`
- `crates/igy6-retrieval-preview/src/lib.rs`

## Current Evidence Correction/Supersession Capability Found

- Evidence item records already include immutable lineage fields:
  `source_id`, `document_id`, `chunk_id`, `statement`, and `metadata_json`.
- Evidence read routes already return `metadata_json` for evidence items.
- Feedback can target `evidence_item` and persist labels such as `verified`,
  `incomplete`, and `wrong`.
- Existing feedback does not provide a direct evidence correction state, does
  not link a superseding evidence item, and does not show correction state in
  the Results evidence list.
- No existing correction/supersession route was found before this DIFF.

## Product Workflow Gap Found

Users could review whether an evidence item was useful or wrong, but they could
not mark an evidence item as corrected, superseded, disputed, verified, or
needing correction from a normal Results workflow without deleting or rewriting
history.

## UX/API/Backend Changes Made

- Added a Results-tab evidence correction and supersession workflow.
- The workflow lists recent evidence items with current correction review state
  from `metadata_json.review_state`.
- The workflow lets a user save one of the supported review states with a short
  correction note and an optional superseding evidence item link.
- Existing evidence item cards now show correction/supersession review state.
- Added a bounded Rust gateway route:
  - `POST /evidence/items/{evidence_item_id}/review-state`
- Added a matching Next.js proxy route:
  - `POST /api/evidence/items/[evidence_item_id]/review-state`
- The backend route updates only `evidence_items.metadata_json` and
  `evidence_items.updated_at`.
- The backend route writes an audit event:
  - `evidence_item.review_state_updated`
- The backend validates:
  - allowed review state;
  - bounded correction note;
  - bounded actor id;
  - evidence item existence;
  - optional superseding evidence item existence;
  - superseding evidence item cannot be the same evidence item.

## Correction/Supersession States Supported

- `needs_correction`
- `corrected`
- `superseded`
- `disputed`
- `verified`

## Immutable-History Behavior

- Original evidence item text is not changed.
- Raw artifacts are not changed.
- Normalized documents are not changed.
- Chunks are not changed.
- Sources are not changed.
- Review state is additive metadata plus audit history.

## Unsupported States Handled

- Superseded evidence is not silently hidden from retrieval.
- Retrieval ranking, retrieval filtering, and policy enforcement are not
  claimed to change in this DIFF.
- Empty evidence state remains an honest empty state with disabled controls.
- Missing or invalid superseding evidence IDs are rejected by backend
  validation.

## Verification Commands And Results

Passed:

- `git status --short`
  - Clean before work.
  - Showed only expected DIFF-211 files before commit after generated
    `target/` cleanup.
- `git branch --show-current`
  - Returned `dev`.
- `git log --oneline --decorate -35`
  - Confirmed latest completed commit before work was DIFF-210.
- `git branch -vv`
  - Confirmed `dev` was aligned with `origin/dev` before DIFF-211 work.
- `git diff --name-status`
  - Showed expected modified tracked files before staging; new files were
    visible in `git status --short`.
- `git diff --check`
  - Passed.
- `npm --prefix apps/web run build`
  - Passed.
- `cargo fmt --all --check`
  - Passed.
- `cargo test --workspace`
  - Passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
  - Confirmed private/dev files remain tracked on `dev`.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`
  - Returned only pre-existing out-of-scope draft/template/status-command
    references; DIFF-211 is `Status: Complete`.

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- A live synthetic source/evidence update against PostgreSQL was not run
  because it would require a running local stack/database; backend behavior was
  covered by Rust route and validation tests.

## Full Docker Smoke

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`. The owner should run full operator smoke locally in normal
WSL.

Owner-run local WSL verification commands:

```bash
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result
```

## Files Changed

- `apps/web/src/app/api/evidence/items/[evidence_item_id]/review-state/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `docs/ui/README.md`

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No evidence deletion behavior was added.
- No raw artifact mutation was added.
- No document or chunk rewrite was added.
- No silent evidence hiding was added.
- No arbitrary command execution was added.
- No fake controls were added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
