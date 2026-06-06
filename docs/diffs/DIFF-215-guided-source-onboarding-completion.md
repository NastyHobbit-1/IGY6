# DIFF-215 - Guided Source Onboarding Completion

Status: Complete

## Purpose

Reduce normal-user ID friction in Add Data for supported source onboarding and
manual text collection workflows.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `5b78136a98635291b03c49168e80ce7298dcacb9`
- DIFF-214 was already complete at the start of this turn.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-210-source-trust-sensitivity-management-ux.md`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-213-conversation-history-import-mvp.md`
- `docs/diffs/DIFF-214-user-observation-ingestion-mvp.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`

## Current Guided Onboarding Capability Found

- Add Data already had guided normal-user workflows for:
  - `manual_upload`
  - `conversation_history`
  - `user_observation`
- Each guided workflow could create a source and embedded source permission
  without requiring raw `source_id` or `source_permission_id` entry.
- Each guided workflow could create a pending approval when the selected
  permission required approval.
- Advanced still retained raw ID forms for troubleshooting.

## Product Workflow Gap Found

When a source permission required approval, the guided workflow created the
approval and stopped, but the next-step guidance still pointed users to the
Advanced route console with the approved approval ID. That left normal users
needing raw `approval_id` handling for supported manual, conversation, and user
observation collection flows.

Settings also listed approvals but did not provide a normal source collection
approval decision surface, so collection approval decisions were effectively an
Advanced workflow.

## Product Changes Made

- Passed approval records into all three supported guided Add Data workflows.
- Added matching approved-approval detection for:
  - `manual_upload`
  - `conversation_history`
  - `user_observation`
- When a matching approved source collection approval exists, the guided
  workflow now uses it automatically for `/collection-runs/manual-upload`.
- When a matching pending approval already exists, the guided workflow avoids
  creating duplicate approval requests and shows a pending state with next safe
  action.
- When no matching approval exists and approval is required, the guided workflow
  creates a real pending approval and stops before upload.
- Added a normal Settings source collection approval review panel for pending
  manual/conversation/observation collection approvals.
- The Settings review panel approves or denies through the existing Rust
  approval decision route and refreshes the page so Add Data can see the latest
  approved approval record.
- Added result details showing source status, permission state, approval state,
  upload state, collection run, work item, raw artifact, and next safe action
  where available.
- Kept raw source, permission, approval, collection, and artifact IDs inside
  details/Advanced surfaces for audit and troubleshooting.

## Backend/API Changes

No Rust backend or Next.js proxy changes were required.

The UI uses existing real routes:

- `POST /sources`
- `POST /approvals`
- `POST /approvals/{approval_id}/decision`
- `POST /collection-runs/manual-upload`

## Unsupported States Handled

- Approval is not silently bypassed.
- Approving a source collection request does not upload data by itself.
- Users must return to the same guided Add Data workflow and submit again after
  approval; the guided path then matches the approved approval automatically.
- Unsupported source types were not added.
- Browser scraping, account scraping, connector imports, and external service
  collection remain unsupported.
- Advanced controls remain available for diagnostics.
- Empty source, approval, collection, and work states remain honest empty
  states.

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- Live source onboarding against PostgreSQL was not run because it requires the
  owner's normal WSL stack/database. The UI behavior was covered by the Next.js
  build and uses existing backend routes.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-215-guided-source-onboarding-completion.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-215; DIFF-215 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No unsupported source type was added.
- No fake approval, fake work item, or fake upload state was added.
- No approval requirement was bypassed.
- No browser scraping was added.
- No account scraping was added.
- No connector import was added.
- No external service call was added.
- No hosted AI call was added.
- No arbitrary command execution was added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
