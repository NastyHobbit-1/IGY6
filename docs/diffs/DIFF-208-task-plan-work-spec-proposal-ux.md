# DIFF-208 - Task Plan Work Spec Proposal UX

Status: Complete

## Purpose

Close the DIFF-204 limitation where normal UI-saved plans are `saved_preview_only` and usually do not include `metadata_json.plan_to_work`.

Add a safe, bounded way for supported task plans to propose an eligible work spec without arbitrary execution.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`
- `docs/diffs/DIFF-204-approval-gated-plan-to-work-item-flow.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-206-agent-task-history-outcome-review.md`
- `docs/diffs/DIFF-207-agent-task-plan-live-stack-verification.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/agent/task-plans/route.ts`
- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-item/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-agent-api/src/lib.rs`

## Work Spec Proposal Behavior

Added a bounded work-spec proposal path for agent task plans:

- Existing and newly saved `create_report` task plans can receive a `metadata_json.plan_to_work` specification.
- The only work type proposed in this DIFF is `report_generation`.
- The route updates the task plan to:
  - `status = ready` when approval is not required;
  - `status = approval_required` when approval is required.
- The route sets `metadata_json.saved_preview_only = false`.
- The route writes an audit event:
  - `event_type`: `agent_task_plan.work_spec_proposed`
  - `resource_type`: `agent_task_plan`

Newly saved report-category plans from the agent planner include the bounded `report_generation` spec at creation time. Existing preview-only report plans can use the new `Propose report work spec` UI action.

The work spec does not create or execute a work item. DIFF-204 remains the only plan-to-work creation path, and created work items still enter the existing `pending_intent_verification` posture.

## Safety Constraints

- No shell command field is accepted.
- No user-provided argv is accepted.
- The backend validates `work_type` against the existing supported work item types.
- The backend additionally restricts this DIFF's proposal route to `work_type = report_generation` and `intent_category = create_report`.
- The backend derives `payload_json` from the persisted task plan instead of accepting arbitrary work payload JSON from the browser.
- Unsupported, canceled, converted, or already work-specified plans are blocked.
- Approval requirements are preserved; approval-required plans are not made directly createable.
- No action execution, worker dispatch, or arbitrary command execution was added.

## API And UI Changes

Rust gateway:

- Added `POST /agent/task-plans/{task_plan_id}/work-spec`.
- Added `parse_agent_task_plan_work_spec`.
- Added `propose_agent_task_plan_work_spec`.
- Added route registry/help text coverage.
- Added tests for route registration, missing database posture, invalid payload rejection, parser acceptance, and dynamic path parsing.

Next/web:

- Added `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-spec/route.ts`.
- Added bounded report work spec generation during task-plan save for `create_report` plans.
- Added `data-agent-plan-propose-work-spec` for existing preview-only report plans.
- Updated task-plan cards to show:
  - `preview only` versus `eligible spec`;
  - supported work type;
  - approval-required state;
  - next safe action;
  - real create-work button only when DIFF-204 eligibility rules are met.

## Verification Commands And Results

- `npm --prefix apps/web run build`: passed. The route table includes `/api/agent/task-plans/[task_plan_id]/work-spec`.
- `cargo fmt --all --check`: initially failed on one formatting wrap; `cargo fmt --all` applied mechanical formatting.
- `cargo fmt --all --check`: passed after formatting.
- `CARGO_TARGET_DIR=/tmp/igy6-diff208-target cargo test --workspace`: passed.
- `git diff --check`: passed.
- `rg "work-spec|data-agent-plan-propose-work-spec|eligible spec|preview only|boundedWorkSpecFor|agent_task_plan.work_spec_proposed|Only create_report" apps/web/src/app/page.tsx apps/web/src/app/api/agent/task-plans crates/igy6-gateway/src/lib.rs`: found expected markers.

Full live smoke was not run because this process cannot access `/var/run/docker.sock`; DIFF-207 and the current `scripts/operator-smoke-check.sh --check` run hit Docker permission preflight before stack startup.

## Files Changed

- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-spec/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-208-task-plan-work-spec-proposal-ux.md`

## Scope Confirmation

- Runtime code changed: yes, scoped to bounded task-plan work spec proposal.
- Work item creation behavior changed: only eligibility metadata was added; DIFF-204 remains the creation route.
- Worker semantics changed: no.
- Arbitrary command execution added: no.
- User-provided shell commands or argv accepted: no.
- Fake/dead controls added: no; the new button calls a real same-origin proxy backed by a Rust route.
- `.env` edited: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- Main branch work, merge, cherry-pick, push, or promotion: no.
- Private/dev files remain tracked on `dev`.
