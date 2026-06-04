# DIFF-204 - Approval-Gated Plan-To-Work Item Flow

Status: Complete

## Purpose

Connect a persisted task plan to work item creation only when safe and approval requirements are satisfied. This begins the safe handoff from "IGY6 planned the work" to "IGY6 created work to do," without arbitrary execution.

## Files Inspected

- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/agent/task-plans/route.ts`
- `apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts`
- Existing work-item creation, dispatch, status, approval decision, audit, and task-plan route code.

## Plan-To-Work Behavior

Added a bounded Rust-native transition route:

- `POST /agent/task-plans/{task_plan_id}/work-item`

The route creates a work item only when all of these are true:

- The task plan exists.
- The plan status is not `unsupported`, `needs_clarification`, `evidence_needed`, `canceled`, or already `converted_to_work`.
- The plan `supported_state` is exactly `supported`.
- If `approval_required` is true, the request supplies an approved `agent_task_plan` approval whose `request_payload_json.task_plan_id` matches the plan.
- The plan includes `metadata_json.plan_to_work`.
- `metadata_json.plan_to_work.work_type` is one of the existing supported work item types.
- `metadata_json.plan_to_work.payload_json`, when present, is a JSON object.

Created work items use the existing work-item posture:

- `status`: `pending_intent_verification`
- `payload_json.agent_task_plan_id`: source plan id
- `payload_json.intent_verification`: deterministic intent metadata derived from the persisted plan

The route updates the task plan to:

- `status`: `converted_to_work`
- `metadata_json.work_item_id`: created work item id

Audit events written:

- `work_item.created`
- `agent_task_plan.work_item_created`

## Approval Gating Behavior

If a plan requires approval:

- Missing approval returns `403`.
- Nonexistent approval returns `403`.
- Approval with a different `request_type` returns `403`.
- Non-approved approval returns `403`.
- Approval for a different task plan returns `403`.

No work item is created unless the approval is both approved and linked to the exact task plan.

## Unsupported And Risky Behavior

Unsupported, clarification-needed, evidence-needed, canceled, or already converted plans do not create work.

Plans without `metadata_json.plan_to_work` do not create work. The normal UI displays honest guidance rather than a fake button:

- Approval-required plans show approval-needed guidance.
- Unsupported plans show unsupported guidance.
- Plans without a work spec show that no supported work-item specification exists yet.

The UI only renders a `Create work item` button when the plan has:

- `supported_state === "supported"`
- `approval_required === false`
- `status === "proposed"` or `status === "ready"`
- `metadata_json.plan_to_work.work_type`

## UI And Proxy Changes

Added:

- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-item/route.ts`
- A real plan-to-work button handler for eligible plan cards.
- `data-agent-plan-create-work` marker for eligible buttons.
- Route hint including `/agent/task-plans/:id/work-item`.

No arbitrary command execution, shell execution, user-provided argv execution, or direct worker dispatch was added.

## Verification Commands And Results

- `npm --prefix apps/web run build`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test --workspace`: passed.
  - Gateway tests included task-plan transition route missing-DB behavior, invalid transition request validation, route path parsing, and route registry coverage.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in this Codex environment.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the latest recorded Docker-permission failure.
- `rg "data-agent-plan-create-work|/agent/task-plans/.*/work-item|agent_task_plan.work_item_created|plan_to_work|agent_task_plan approval" apps/web/src/app/page.tsx apps/web/src/app/api/agent/task-plans crates/igy6-gateway/src/lib.rs`: found expected markers.

Full smoke run was skipped because Docker socket access is unavailable in this Codex environment:

```text
permission denied connecting to /var/run/docker.sock
```

## Files Changed

- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-item/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-204-approval-gated-plan-to-work-item-flow.md`

## Scope Confirmation

- Worker queue semantics changed: no.
- Arbitrary work types added: no.
- Shell command execution added: no.
- User-provided argv execution added: no.
- Action execution changed: no.
- Autonomous self-improvement added: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- `.env` edited: no.
- Main branch work, merge, cherry-pick, push, or promotion: no.
- Private/dev files remain tracked on `dev`.
