# DIFF-197 Guided Agent Task Intake Planner UX

Status: Complete

## Purpose

Bridge `/agent/intent` request understanding into a normal-user task planning
surface. After a user previews a request, the UI shows what IGY6 understood,
whether the request is supported, whether evidence or approval is needed, and
the next safe step.

## Branch And Baseline

- Branch before work: `dev`.
- HEAD before work:
  `72a1da8 Complete DIFF-196 next AI task handling gap audit`.
- `dev` ahead/behind `origin/dev` before work: ahead by 1 from DIFF-196.
- Working tree before work: clean.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `README.md`
- `docs/ui/README.md`
- `crates/igy6-agent-api/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/api/agent/intent/route.ts`
- `apps/web/src/app/api/agent/capabilities/route.ts`
- `apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts`
- targeted AI/task/action grep and file scans required by the batch prompt
- private/dev tracked-file list

## Current Behavior

- The Results tab already includes `AgentCommandPanel`.
- The panel posts a plain-language request to `/api/agent/intent`.
- The Rust gateway forwards to `crates/igy6-agent-api`, which returns:
  request category, wants, evidence posture, clarification posture, approval
  posture, work-item posture, unsupported/unsafe posture, missing parameters,
  proposed action, risk, safety notes, and next step.
- The previous UI showed a short understanding summary plus raw JSON in
  Advanced. It did not turn the intent response into a normal-user next-step
  planner.

## UX/API Changes Made

- Added a read-only `agentPlanner` section to `AgentCommandPanel`.
- The planner renders existing `/agent/intent` response fields into cards:
  - status;
  - request category;
  - evidence needed/not-needed;
  - approval required/not-required;
  - next safe step.
- Unsupported/unsafe requests are shown as unsupported guidance.
- Clarification-needed requests are shown with missing-information guidance.
- Approval-required or system-changing requests are shown as approval-gated
  guidance.
- Evidence-required requests direct the user to Ask over evidence or add/process
  more local data.
- No action execution behavior changed.
- No backend fields, routes, or runtime code changed.

## Safety Notes

- The planner does not execute actions.
- The planner does not create work items.
- The planner does not add autonomous planning.
- Raw JSON and raw parameters remain in Advanced.
- Unsupported requests remain unsupported guidance.

## Verification

Commands run:

```bash
git status --short
git diff --check
git diff --name-status
npm --prefix apps/web run build
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

Results:

- `git diff --check`: passed.
- `git diff --name-status`: showed only DIFF-197 scoped files.
- `npm --prefix apps/web run build`: passed.
- Private/dev files remained tracked.
- Stale status scan continued to report older out-of-scope draft/status strings
  and command transcripts already known from prior DIFFs.
- Full operator smoke was skipped for this DIFF because the change is UI-only
  planner rendering and does not affect runtime/API/operator smoke script
  behavior.

## Files Changed

- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/diffs/DIFF-197-guided-agent-task-intake-planner-ux.md`
