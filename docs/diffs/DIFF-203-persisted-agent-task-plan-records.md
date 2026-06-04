# DIFF-203 - Persisted Agent Task Plan Records

Status: Complete

## Purpose

Add the first persisted task-plan record layer so IGY6 can remember a proposed task plan after `/agent/intent` task planning instead of only showing transient UI state.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `docs/diffs/DIFF-197-guided-agent-task-intake-planner-ux.md`
- `docs/diffs/DIFF-198-approval-to-action-execution-ux.md`
- `docs/diffs/DIFF-199-feedback-outcome-to-improvement-review-ux.md`
- `docs/diffs/DIFF-200-safe-task-queue-dispatch-visibility.md`
- `docs/diffs/DIFF-201-improvement-experiment-proposal-review-ux.md`
- `docs/diffs/DIFF-202-agent-planner-runtime-smoke-gap-followthrough.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`
- `README.md`
- `docs/ui/README.md`
- `infra/migrations/README.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/agent/intent/route.ts`
- `apps/web/src/app/api/approvals/route.ts`

## Chosen Persistence Approach

Added a dedicated Rust-gateway owned `agent_task_plans` table instead of reusing `work_items`.

Reason:

- Task plans are proposed planning records, not dispatchable work.
- Reusing `work_items` would blur queue semantics and imply worker execution.
- A dedicated table lets DIFF-204 connect plans to work items explicitly and approval-gated later.

The active migration directory currently records that Alembic is archived and active schema governance remains a follow-up item. To keep this route usable on a fresh local DB, the new task-plan handlers run an idempotent `CREATE TABLE IF NOT EXISTS agent_task_plans` before list/detail/create operations.

## Schema, API, And UI Changes

Rust gateway:

- Added Rust-native routes:
  - `GET /agent/task-plans`
  - `GET /agent/task-plans/{task_plan_id}`
  - `POST /agent/task-plans`
- Added safe parser validation for task-plan creation.
- Added bounded status values:
  - `proposed`
  - `needs_clarification`
  - `approval_required`
  - `evidence_needed`
  - `ready`
  - `unsupported`
  - `converted_to_work`
  - `canceled`
- Added bounded supported-state values:
  - `supported`
  - `unsupported`
  - `clarification_needed`
  - `approval_required`
  - `evidence_needed`
- Added audit event on creation:
  - `event_type`: `agent_task_plan.created`
  - `resource_type`: `agent_task_plan`

Persisted safe fields:

- `id`
- `user_request_summary`
- `intent_category`
- `status`
- `proposed_steps`
- `required_evidence`
- `approval_required`
- `supported_state`
- `next_safe_action`
- `requested_by_actor_id`
- `metadata_json`
- `created_at`
- `updated_at`

Web:

- Added `apps/web/src/app/api/agent/task-plans/route.ts` as a same-origin proxy for browser `POST /api/agent/task-plans`.
- Loaded recent plans from `GET /agent/task-plans`.
- Added a real `Save task plan` control to the existing agent planner UI.
- Added a recent persisted task-plan card list with `data-agent-task-plan-records`.

## Safety Limits

- The save flow records safe summary metadata from the current `/agent/intent` preview.
- It does not execute actions.
- It does not create work items.
- It does not accept shell commands or argv.
- It does not store `.env` contents, secrets, raw runtime data, `IGY6_DATA_ROOT` contents, Docker credentials, auth tokens, or full logs.
- The UI marks saved plans as preview-only metadata in `metadata_json.saved_preview_only`.

## Verification Commands And Results

- `npm --prefix apps/web run build`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test --workspace`: passed.
  - Gateway unit tests included `65 passed`.
  - New tests covered task-plan route registration, missing `DATABASE_URL`, parser acceptance, and invalid payload rejection.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in the Codex environment.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the latest recorded failure result.
- `rg "data-agent-task-plan-records|data-agent-save-plan|/agent/task-plans|agent_task_plans|agent_task_plan.created" apps/web/src/app/page.tsx apps/web/src/app/api/agent/task-plans/route.ts crates/igy6-gateway/src/lib.rs`: found the expected UI/API/backend markers.
- `git diff --check`: passed.

Full smoke run was skipped because Docker socket access is unavailable in this Codex environment:

```text
permission denied connecting to /var/run/docker.sock
```

## Files Changed

- `apps/web/src/app/api/agent/task-plans/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`

## Scope Confirmation

- Runtime code changed: yes, scoped to persisted task-plan records and UI display/save.
- Worker semantics changed: no.
- Work item creation changed: no.
- Action execution changed: no.
- Arbitrary command execution added: no.
- Autonomous self-improvement added: no.
- `.env` edited: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- Main branch work, merge, cherry-pick, push, or promotion: no.
- Private/dev files remain tracked on `dev`.
