# DIFF-207 - Agent Task Plan Live Stack Verification

Status: Complete

## Purpose

Live-stack verify the task-plan path from DIFF-203 through DIFF-206 on a Docker-capable environment and record what is actually verified versus still incomplete.

This DIFF is verification-first. It does not add a new product feature unless a directly verified bug blocks the path.

## Branch And HEAD Before Work

- Branch before work: `dev`
- HEAD before work: `68ba036 Complete DIFF-206 agent task history outcome review`
- `dev` ahead/behind `origin/dev` before work: `dev 68ba036 [origin/dev]`, no ahead/behind marker.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-202-agent-planner-runtime-smoke-gap-followthrough.md`
- `docs/diffs/DIFF-203-persisted-agent-task-plan-records.md`
- `docs/diffs/DIFF-204-approval-gated-plan-to-work-item-flow.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-206-agent-task-history-outcome-review.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/ui/README.md`
- `README.md`
- `scripts/run.sh`
- `scripts/stop.sh`
- `scripts/operator-smoke-check.sh`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/agent/task-plans/route.ts`
- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-item/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `find docs/diffs -maxdepth 1 -type f | sort | tail -170`
- Stale status scan over `docs/diffs`
- AI/task-plan/retrieval implementation scan over `apps/web`, `crates`, `services`, `scripts`, and `docs`

## Live Stack Commands Run

- `npm --prefix apps/web run build`: passed.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in this process.
- `scripts/operator-smoke-check.sh --latest-result || true`: first showed an existing local recorded passing smoke result at repo head `68ba036`.
- `scripts/operator-smoke-check.sh --run --record || true`: failed at Docker socket permission preflight before stack startup and wrote `.igy6-local/smoke-results/operator-smoke-20260605T065530Z.json`.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the new safe failure record.
- `ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true`: no listeners reported before or after the failed preflight.

Docker CLI exists, but this process cannot connect to `/var/run/docker.sock`.
The live stack was not started by this DIFF-207 run.

Existing local smoke result observed before the current failed preflight:

- File: `.igy6-local/smoke-results/operator-smoke-20260605T055444Z.json`
- Repo head: `68ba036`
- Overall status: `passed`
- Steps: `total=35 pass=35 fail=0 other=0`
- API live/ready/retrieval: `200/200/200`
- Web root: `200`
- Counts: `artifacts=11 documents=9 chunks=9 evidence_items=9 retrieval_items=5`
- Stack started/stopped by script: `true/true`

That result is recorded as local operator evidence, but this DIFF does not claim independent live-stack task-plan API verification from the current Codex process.

## Task Plan Create/List/Detail Verification

Not independently live-verified in this process because Docker preflight failed before stack startup.

Source/static verification found the live route implementations:

- `POST /agent/task-plans` is Rust-native and calls `create_agent_task_plan`.
- `GET /agent/task-plans` is Rust-native and calls `list_agent_task_plans`.
- `GET /agent/task-plans/{task_plan_id}` is Rust-native and calls `get_agent_task_plan`.
- `ensure_agent_task_plans_table` creates the `agent_task_plans` table idempotently.
- Gateway tests from the prior implementation cover route registration, missing database posture, parser acceptance, invalid payload rejection, and task-plan path parsing.

## Next Proxy Verification

Not independently live-verified in this process because the web stack could not be started.

Static/build verification found:

- `apps/web/src/app/api/agent/task-plans/route.ts` proxies browser `POST /api/agent/task-plans` to Rust `POST /agent/task-plans`.
- `apps/web/src/app/api/agent/task-plans/[task_plan_id]/work-item/route.ts` proxies browser `POST /api/agent/task-plans/:id/work-item` to Rust `POST /agent/task-plans/{task_plan_id}/work-item`.
- The production web build route table includes:
  - `/api/agent/task-plans`
  - `/api/agent/task-plans/[task_plan_id]/work-item`

## UI Marker Verification

Source marker verification found:

- Task-plan record surface: `data-agent-task-plan-records`
- Task-plan save control: `data-agent-save-plan`
- Evidence-aware planner surface: `data-agent-planner-evidence`
- Task history review surface: `data-agent-task-history-review`
- Conditional work creation control: `data-agent-plan-create-work`

The UI loads task plans from `GET /agent/task-plans` into `AgentCommandPanel` and `AgentTaskHistoryReview`.

## Approval And Work-Item Eligibility Result

Source verification confirms the UI does not show a fake `Create work item` control for normal preview-only saved plans.

The `Create work item` button is rendered only when all of these are true:

- `metadata_json.plan_to_work.work_type` exists.
- `supported_state === "supported"`
- `approval_required === false`
- `status === "proposed"` or `status === "ready"`

Normal UI-saved plans remain `metadata_json.saved_preview_only: true` and do not include `metadata_json.plan_to_work`, so they show honest guidance instead of a create-work button. This is the known limitation that DIFF-208 is intended to close.

## Bugs Found Or Fixed

No product bug was directly verified inside the DIFF-207 scope.

No app, API, Rust, script, or runtime behavior files were changed.

## Remaining Incomplete Items

- Independent live-stack task-plan create/list/detail verification from this process remains incomplete because Docker socket access is unavailable.
- Independent live Next proxy verification remains incomplete for the same reason.
- The existing local recorded smoke at `68ba036` proves the broader operator smoke path passed locally, but it does not include targeted task-plan API assertions.
- Normal UI-saved task plans are still preview-only and usually cannot become eligible for plan-to-work creation until DIFF-208 adds bounded work spec proposal behavior.

## Files Changed

- `docs/diffs/DIFF-207-agent-task-plan-live-stack-verification.md`

## Verification Summary

- `git status --short`: showed only the new DIFF-207 file before commit.
- `git diff --check`: passed.
- `npm --prefix apps/web run build`: passed.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in this process.
- `scripts/operator-smoke-check.sh --run --record || true`: failed at Docker socket permission preflight and wrote a safe failure result record.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the failure record without raw JSON.
- `rg "data-agent-task-history-review|data-agent-planner-evidence|data-agent-task-plan-records|data-agent-plan-create-work|/agent/task-plans|agent_task_plans|plan_to_work|saved_preview_only" apps/web/src/app/page.tsx apps/web/src/app/api/agent/task-plans crates/igy6-gateway/src/lib.rs`: found expected markers.
- `ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true`: no listeners reported.

## Scope Confirmation

- Runtime code changed: no.
- Runtime app behavior changed: no.
- Live stack verification faked: no.
- `.env` edited: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- Raw uploaded text printed: no.
- Main branch work: no.
- Merge/cherry-pick/push/promotion: no.
- Sudo/group/system permission change: no.
- Destructive command: no.
- Arbitrary command execution added: no.
- Private/dev files remain tracked on `dev`.
