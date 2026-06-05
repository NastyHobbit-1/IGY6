# DIFF-209 - Persist Evidence Check Summary On Task Plans

Status: Complete

## Purpose

Make evidence-aware task planning durable. When the planner checks evidence for a request, persist a safe summary on the related task plan so the plan's evidence readiness survives reloads and can be reviewed later.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-206-agent-task-history-outcome-review.md`
- `docs/diffs/DIFF-208-task-plan-work-spec-proposal-ux.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `apps/web/src/app/api/agent/task-plans/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-retrieval-preview/src/lib.rs`

## Evidence Summary Persistence Behavior

- Added a bounded Rust gateway route:
  - `POST /agent/task-plans/{task_plan_id}/evidence-summary`
- Added a matching Next.js proxy route:
  - `POST /api/agent/task-plans/[task_plan_id]/evidence-summary`
- The route updates the existing persisted `agent_task_plans.metadata_json` object with a safe `evidence_summary` object.
- The route writes an audit event with event type `agent_task_plan.evidence_summary_recorded`.
- The planner UI can save a summary after running the existing retrieval-preview evidence check for the latest saved task plan.
- Existing task plan cards can run `Check and save evidence` against a persisted task plan.
- The task history/review surface reads the persisted `metadata_json.evidence_summary` and displays evidence readiness after reload.
- This DIFF does not store raw evidence text, does not create AI conclusions, and does not change retrieval engine behavior.

## Fields Stored

- `evidence_checked_at`
- `answer_status`
- `retrieved_count`
- `safe_labels`
- `missing_evidence`
- `missing_evidence_guidance`

The stored values are intentionally summary-only. Labels are bounded to 10 items with bounded length, and counts are bounded to `0..1000`.

## Fields Intentionally Not Stored

- Raw uploaded text.
- Raw evidence/chunk text.
- Raw source content.
- Secrets, tokens, credentials, or `.env` contents.
- Runtime/private data from `IGY6_DATA_ROOT`.
- Raw database rows.
- Full retrieval responses or logs.
- AI conclusions or unsupported answer claims.

## UI And History Changes

- The task planner now tracks the latest saved task plan ID and can persist the evidence check summary after retrieval-preview runs.
- Persisted task plan cards now show:
  - evidence readiness status
  - retrieved hit count
  - safe evidence labels when present
  - a real `Check and save evidence` control wired to the new proxy route
- The task history/review surface now shows persisted evidence readiness for each task plan.
- The route hint was updated to include `/agent/task-plans/:id/evidence-summary`.

## Verification Commands And Results

- `git status --short`
  - Showed only expected DIFF-209 files before commit.
- `git diff --check`
  - Passed.
- `git diff --name-status`
  - Showed expected modified runtime/UI files; new files were visible in `git status --short` before staging.
- `npm --prefix apps/web run build`
  - Passed.
- `cargo fmt --all --check`
  - Initially reported formatting changes needed after Rust edits.
- `cargo fmt --all`
  - Applied formatting only.
- `cargo fmt --all --check`
  - Passed after formatting.
- `CARGO_TARGET_DIR=/tmp/igy6-diff209-target cargo test --workspace`
  - Passed.
- `scripts/operator-smoke-check.sh --check`
  - Failed in this Codex environment because Docker socket access is denied for the current user.
- `scripts/operator-smoke-check.sh --latest-result || true`
  - Ran and reported the latest local smoke-result summary.
- Full smoke with `scripts/operator-smoke-check.sh --run --record`
  - Skipped for this DIFF because Docker preflight fails in this environment with Docker socket permission denied. This was recorded instead of faking live verification.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
  - Confirmed private/dev files remain tracked on `dev`.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`
  - Returned only pre-existing out-of-scope draft/template references.

## Files Changed

- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/evidence-summary/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-209-persist-evidence-check-summary-on-task-plans.md`

## Scope Confirmation

- Runtime code changed only for the bounded evidence-summary persistence route, its Next.js proxy, and the UI display/save path.
- No arbitrary command execution was added.
- No user-provided shell commands or argv execution was added.
- No work item execution behavior was added.
- No autonomous self-improvement behavior was added.
- No `.env` file was edited.
- No secrets or runtime/private data were printed or stored.
- No main-branch work, merge, cherry-pick, promotion, push, sudo, group change, destructive command, or private/dev file removal was performed.
