# DIFF-206 - Agent Task History And Outcome Review

Status: Complete

## Purpose

Give users a clear review surface for prior task plans, created work items, approvals, outcomes, and improvement links so task handling does not become opaque.

## Files Inspected

- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`
- `docs/diffs/DIFF-204-approval-gated-plan-to-work-item-flow.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `apps/web/src/app/page.tsx`
- Existing task plan, work item, approval, feedback, outcome, and improvement record shapes in the dashboard.

## Task History Behavior

Added a read-only task history review surface in the Results workflow:

- `data-agent-task-history-review`
- `data-agent-task-history-item`

The surface shows recent persisted task plans and, where real persisted links exist:

- plan id/status/category/created date
- linked work item id/status
- matching `agent_task_plan` approval id/status
- linked work-item feedback id/label
- linked work-item outcome id/status
- linked improvement item id/status
- safe next action guidance

Links are derived only from existing persisted IDs:

- `task_plan.metadata_json.work_item_id`
- `approval.request_payload_json.task_plan_id`
- feedback/outcome records targeting the linked `work_item`
- improvement metadata with `agent_task_plan_id` or `work_item_id`

## Outcome And Improvement Review Behavior

The UI shows `not linked` when no persisted feedback, outcome, approval, work item, or improvement relationship exists.

It does not add fake links, fake persisted state, new creation controls, autonomous outcome scoring, autonomous improvement behavior, or experiment execution.

Safe next action guidance is derived from existing state:

- converted plans point users to review linked work item status
- approval-required plans without approved approval point users to review/create matching approval
- other plans show the plan's existing `next_safe_action`

## Verification Commands And Results

- `npm --prefix apps/web run build`: passed.
- `git diff --check`: passed.
- `rg "data-agent-task-history-review|data-agent-task-history-item|Agent Task History And Outcomes|not linked|approval required, not linked" apps/web/src/app/page.tsx`: found expected markers.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in this Codex environment.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the latest recorded Docker-permission failure.

No Rust files changed in DIFF-206, so Rust formatting/tests were not required for this DIFF.

Full smoke run was skipped because Docker socket access is unavailable in this Codex environment:

```text
permission denied connecting to /var/run/docker.sock
```

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-206-agent-task-history-outcome-review.md`

## Scope Confirmation

- New backend route added: no.
- New persistence added: no.
- Work item creation changed: no.
- Action execution changed: no.
- Fake links or fake persisted state added: no.
- Autonomous self-improvement added: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- `.env` edited: no.
- Main branch work, merge, cherry-pick, push, or promotion: no.
- Private/dev files remain tracked on `dev`.
