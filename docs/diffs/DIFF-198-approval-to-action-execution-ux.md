# DIFF-198 Approval-To-Action Execution UX

Status: Complete

## Purpose

Connect approved action posture to honest, bounded action execution UX. Users
can see fixed actions, whether approval is required, available approved agent
approvals, and whether execution is supported by the existing API runtime.

## Branch And Baseline

- Branch before work: `dev`.
- HEAD before work:
  `16d3cad Complete DIFF-197 guided agent task intake planner UX`.
- Working tree before work: clean.
- `dev` ahead/behind `origin/dev` before work: ahead by 2 from DIFF-196 and
  DIFF-197.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`
- `docs/diffs/DIFF-197-guided-agent-task-intake-planner-ux.md`
- `crates/igy6-agent-api/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts`
- `apps/web/src/app/api/agent/capabilities/route.ts`
- `apps/web/src/app/api/approvals/route.ts`

## Current Action/Approval Behavior

- `ACTION_REGISTRY` contains fixed allowlisted actions only.
- Read-only actions can execute through existing Rust gateway handlers.
- System-changing stack actions require a matching approved approval record.
- The Rust gateway rejects arbitrary action names, dangerous patterns, raw argv,
  shell, command, and script fields.
- Approval records are listed by `/approvals` and include
  `request_payload_json`.
- The prior UI could request approval and run approved actions, but the visible
  flow still relied on raw approval ID entry in Advanced.

## UX/API Changes Made

- Added a visible approval-to-action bridge in `AgentCommandPanel`.
- The bridge lists existing fixed action capabilities and labels each as
  read-only or approval-required.
- The bridge lists approved `agent_action` approvals when present.
- The bridge summarizes:
  - selected/proposed action;
  - action class;
  - approval state;
  - runtime execution support.
- When a matching approved approval exists for the proposed action and current
  parameters, the UI fills the existing approval ID field for the bounded
  approved execution route.
- Existing execution buttons remain bound to existing `/agent/actions/:action`
  execution behavior. No new execution route or arbitrary command path was
  added.

## Execution Behavior Verified Or Unsupported Finding

- Read-only action execution remains the existing supported path.
- Approval-required execution remains supported only after a matching approved
  approval is selected or found.
- Stack-control actions remain runtime-blocked when Docker control is not
  available to the API runtime.
- Unsupported actions remain rejected by the typed registry.

## Safety Notes

- No arbitrary command execution was added.
- No user-provided argv, shell, command, or script execution was added.
- Dangerous-pattern rejection remains backend-owned.
- The UI does not bypass approval matching.
- Raw approval ID entry remains available only inside Advanced.

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
- `git diff --name-status`: showed only DIFF-198 scoped files.
- `npm --prefix apps/web run build`: passed.
- Private/dev files remained tracked.
- Stale status scan continued to report older out-of-scope draft/status strings
  and command transcripts already known from prior DIFFs.
- Full operator smoke was skipped because this DIFF changes visible UI guidance
  and selection around existing routes, not runtime/API/operator smoke script
  behavior.

## Files Changed

- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/diffs/DIFF-198-approval-to-action-execution-ux.md`
