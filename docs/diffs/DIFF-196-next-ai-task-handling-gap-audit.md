# DIFF-196 Next AI Task Handling Gap Audit

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- This DIFF does not promote files, merge, cherry-pick, push, touch `main`,
  edit `.env`, remove files, run `sudo`, change user groups, kill processes,
  run destructive commands, implement runtime feature code, or dump
  runtime/private data.

## Purpose

Audit the current AI/task/action/self-improvement surfaces and select the next
highest-value product DIFF.

This DIFF is planning and selection only. It does not implement the selected
runtime feature.

## Baseline

- Branch before work: `dev`.
- HEAD before work:
  `156ea86 Complete DIFF-195 smoke result viewer summary command`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Working tree before work: clean.
- Latest completed dev DIFF before this planning pass was DIFF-195.
- Operator smoke tooling can now check, run, record, list, and summarize the
  verified local manual upload evidence path.

## Allowed Scope

- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md` only.

Optional docs were allowed only if needed for a documentation-link blocker. No
optional documentation edit was needed.

## Prohibited Scope

- Do not remove anything from `dev`.
- Do not remove `.codex`.
- Do not remove `AGENTS.md`.
- Do not remove `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`.
- Do not remove `docs/agents`.
- Do not remove `docs/plans`.
- Do not switch to or touch `main`.
- Do not merge `main` into `dev`.
- Do not merge `dev` into `main`.
- Do not cherry-pick.
- Do not promote files.
- Do not edit runtime app code.
- Do not edit `.env`.
- Do not print secrets.
- Do not dump runtime/private data from `IGY6_DATA_ROOT`.
- Do not run destructive commands.
- Do not kill processes.
- Do not run `sudo`.
- Do not change system group membership.
- Do not implement the selected next feature in this DIFF.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `README.md`
- `docs/ui/README.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-183-dev-next-runtime-work-selection.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`
- `docs/diffs/DIFF-187-basic-report-workflow-ux.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`
- `docs/diffs/DIFF-190-operator-smoke-verification-bundle.md`
- `docs/diffs/DIFF-195-smoke-result-viewer-summary-command.md`
- `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`
- `docs/user-guide.md` from targeted AI/task/action grep output
- `docs/security-policy.md` from targeted AI/task/action grep output
- `services/self_improvement/README.md`
- `services/reports/README.md`
- `crates/igy6-agent-api/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-llm/src/lib.rs`
- `crates/igy6-work-queue-reports/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/agent/intent/route.ts`
- `apps/web/src/app/api/agent/capabilities/route.ts`
- `apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts`
- DIFF inventory under `docs/diffs`
- targeted AI/task/action/self-improvement file and text scans
- tracked private/dev/build instruction file list from
  `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`

The broad required file scan also listed generated `apps/web/.next`, installed
`apps/web/node_modules`, and ignored `.igy6-local/smoke-results` files. Those
generated/local artifacts were not used as source-of-truth for product
capability conclusions.

The broad `find` command reported permission denied for
`./IGY6_Data/postgres`; that runtime/private data directory was not inspected.

## Current Capability Summary

### Request Understanding and `/agent/intent`

- `crates/igy6-agent-api` defines typed request categories:
  `evidence_question`, `add_data`, `check_work_status`, `create_report`,
  `request_action`, `system_changing_action`, `feedback`, `record_outcome`,
  `correction`, `diagnostics`, `project_status`,
  `experiment_or_improvement`, and `unclear`.
- The classifier returns a request summary plus explicit evidence,
  clarification, approval, work-item, unsupported/unsafe, missing-information,
  assumption, and next-step posture.
- Dangerous shell/destructive patterns are rejected and do not become
  executable work.
- The Next proxy `apps/web/src/app/api/agent/intent/route.ts` forwards preview
  requests to the Rust gateway.
- The UI `AgentCommandPanel` previews plain-language requests and shows the
  understood category and posture.
- Gap: `/agent/intent` is still primarily a preview/classification surface. It
  does not guide a normal user into a structured next action such as asking over
  evidence, creating a report draft request, requesting approval, recording
  feedback/outcome, or proposing improvement work without Advanced parameters.

### Task and Work Item Handling

- Work item records exist and are displayed in the Work tab.
- Work item creation stores intent verification and starts in
  `pending_intent_verification`.
- Work item status transitions are typed in `igy6-work-queue-reports`.
- The Rust worker daemon actively processes supported queued pipeline work from
  manual upload: normalization, chunking, evidence creation, and vector upsert.
- The Rust gateway dispatch route for arbitrary work item dispatch remains
  intentionally safe-limited: it records dispatch metadata and audit as
  `queued_without_execution` / `not_executed` rather than invoking arbitrary
  runtime execution.
- Gap: intent classifications that say `work_item_should_be_created` do not
  have a normal-user confirmation path to create the correct bounded work item,
  pick supported parameters, or show what will happen next.

### Approval Handling

- Approval records can be created, decided, listed, and audited.
- Manual upload source permissions can require approval before collection.
- Agent system-changing actions require an approved approval record.
- Agent approval matching checks request type, action name, and parameters.
- The Settings tab shows approval/safety posture and recent approvals.
- Gap: the agent UI can create an approval for a proposed action, but the flow
  still expects raw approval ID handling and raw JSON in an Advanced details
  panel. It does not provide a normal-user approval-to-action review path with
  a clear decision boundary and next-step state.

### Action Execution and Command Plane

- `ACTION_REGISTRY` currently includes fixed allowlisted actions:
  `show_project_health`, `show_git_status`, `show_latest_diff`,
  `show_work_items`, `run_retrieval_preview`, `start_stack`, `stop_stack`, and
  `run_last_healthy_stack`.
- Read-only actions execute through Rust handlers and audit events.
- System-changing stack actions require approval and then use a bounded host
  bridge. The bridge is constrained to `127.0.0.1`, a token, fixed action names,
  and redacted summaries.
- User-provided argv/command/script/shell fields are rejected.
- Gap: the normal-user surface still exposes "Run safe action" and raw
  parameter JSON rather than a guided action planner. The next high-value path
  is not broader execution; it is safer, clearer task/action intake and review.

### Feedback and Outcome Capture

- Results exposes normal-user feedback and outcome capture for visible evidence,
  reports, and work items.
- `POST /feedback` persists review metadata; weak feedback for non-source
  targets can create a proposed improvement item.
- Source feedback can apply limited source trust side effects with audit.
- `POST /outcomes` persists outcome records and updates supported target status
  metadata.
- Gap: feedback and outcomes are captured, but there is no normal-user loop
  that summarizes recurring weak feedback into improvement candidates or guides
  the user to create/review an improvement item from a specific failed answer,
  report, or action.

### Reports

- Reports can be created and rendered through existing Rust routes.
- The Results tab includes a basic report workflow for local metadata markdown
  reports.
- Reports preserve boundaries and do not claim full evidence synthesis or
  external model generation.
- Gap: report creation from an agent request is still a classification and
  manual UI operation, not a guided "plan this report from retrieved evidence"
  flow.

### Improvements and Experiments

- Rust routes exist for improvements and experiments:
  `GET/POST /improvements`, `GET/POST /experiments`, and experiment status
  update.
- Improvement items can be proposed directly or generated from weak feedback.
- Experiment records can store metadata, metrics, artifacts, and status.
- `services/self_improvement/README.md` explicitly states the service is a
  Phase 0 placeholder: no experiments, Optuna studies, DSPy optimization, or
  method promotion logic are implemented there.
- Gap: there is no normal-user improvement review UX, no experiment proposal
  review flow, no experiment runner, no baseline comparison, and no
  approval-gated method promotion loop.

### Local LLM / Ollama / Provider Routing

- `crates/igy6-llm` supports `LLM_PROVIDER=none` and optional local
  `LLM_PROVIDER=ollama`.
- The default is disabled/deterministic evidence fallback.
- Local Ollama URLs are restricted to local/host-docker style origins and
  redact credentials.
- `configs/local-llm-routing.json` is validated by the LLM crate, with
  task-specific routes such as evidence summary, report draft, fast triage, and
  action explanation.
- `scripts/ollama-local-setup.sh` is check-only by default and has explicit
  model pull/configure modes.
- The UI shows local LLM status and answer mode.
- Gap: LLM status exists, but normal task planning is not yet routed through a
  clear local-model/offline decision. LLMs must remain evidence-bounded and must
  not execute actions.

### UI Surfaces

- Home shows readiness, counts, and next cards.
- Add Data supports guided manual UTF-8 text upload.
- Work shows recent work items, status, identifiers, and recovery guidance.
- Results supports evidence retrieval/review, reports, feedback/outcomes, and
  source/evidence history.
- Settings shows safety, approvals, feedback, outcomes, audit, and local model
  status.
- Advanced retains low-level route controls, dispatch, and raw JSON.
- The action preview panel exists in the main page and uses `/agent/intent`,
  `/agent/actions/:action/execute`, and `/approvals`.
- Gap: the action preview panel is still too developer-shaped for the next
  product step. It needs a normal-user guided plan/result path before adding
  more execution capability.

## Product Gap Analysis

IGY6 now has a verified evidence base: a user can add text, see work complete,
retrieve evidence, review support, create a basic report, capture
feedback/outcomes, inspect lineage, and verify the path with operator smoke
tools.

The next product gap is not ingestion reliability. It is the handoff from
"IGY6 understood my request" to "IGY6 can help me safely do the next bounded
thing."

The missing end-product loop is:

1. User makes a plain-language request.
2. IGY6 classifies it and explains evidence, clarification, approval, and work
   posture.
3. IGY6 offers only safe next choices grounded in current capability.
4. User confirms or approves.
5. IGY6 executes only read-only or explicitly approved fixed actions, or creates
   bounded work/improvement records.
6. IGY6 records outcomes and feedback.
7. IGY6 uses repeated feedback/outcome signals to propose improvement work,
   without silently changing production behavior.

Current blockers:

- The agent panel previews intent but does not translate common categories into
  guided product actions.
- It still asks for raw parameter JSON and approval IDs for important paths.
- Work-item creation exists, but there is no normal-user intent-to-work
  confirmation path.
- Improvement records exist, and weak feedback can create them, but normal
  users cannot review the improvement queue in context.
- Experiment metadata exists, but no controlled experiment proposal/review UX
  exists.
- Local LLM routing exists, but it should support evidence-grounded planning or
  explanation only after the deterministic planner UX is clear.

## Ranked Candidate Next DIFFs

### 1. DIFF-197: Guided Agent Task Intake Planner UX

Scope:

- Add a normal-user guided planner around `/agent/intent` in the existing UI.
- Convert intent responses into concrete next-step cards for supported
  categories:
  - ask over evidence;
  - add data;
  - check work status;
  - create report;
  - record feedback;
  - record outcome;
  - propose improvement;
  - request approval for a supported action;
  - run read-only supported action.
- Keep execution bounded to existing routes and actions only.
- Do not add new action execution capability.
- Do not add LLM-driven autonomous planning.

Why ranked first:

- Highest user-visible value: it turns the existing agent classifier into a
  usable product workflow.
- Directly builds on the verified evidence/retrieval/work/report/feedback base.
- Small enough for one UI-focused DIFF if constrained to existing routes.
- Verification can use build checks plus source markers and optional live
  `/agent/intent`/read-only action checks.
- Preserves local-first behavior and approval posture.

Risk:

- Medium UI complexity. Must avoid implying autonomous action execution or
  unsupported work creation.

### 2. Approval-To-Action Execution UX

Scope:

- Improve the agent action approval flow so approval creation, pending status,
  approval decision, and approved execution are visible without raw approval ID
  copying.
- Keep fixed action registry and approval matching unchanged.

Why second:

- Directly improves system-changing action safety and usability.
- Good user value, but narrower than the broader task intake planner.

Risk:

- Medium. Touches sensitive action execution UX and must not bypass approval.

### 3. Feedback/Outcome To Improvement Review UX

Scope:

- Surface proposed improvement items generated by weak feedback.
- Let users review why an improvement exists, what evidence/feedback caused it,
  and what a later experiment would need.
- No experiment runner or method promotion.

Why third:

- Builds the learning loop from verified feedback/outcomes.
- Valuable, but depends on users first having a clearer task/action planner.

Risk:

- Low to medium. Mostly UI/read-route work if existing APIs are enough.

### 4. Safe Task Queue Dispatch Visibility

Scope:

- Improve Work/Advanced visibility for dispatch metadata, intent verification,
  and "queued without execution" semantics.
- Possibly add normal-user warnings for dispatch that does not actually invoke
  runtime execution.

Why fourth:

- Important for avoiding confusion, but less user-visible than making agent
  task intake useful.

Risk:

- Low if UI-only; medium if dispatch contracts are touched.

### 5. Improvement Experiment Proposal / Review UX

Scope:

- Add a user-facing experiment proposal view from improvement items, with
  objective, success criteria, required evidence, metrics, and approval posture.
- No runner, Optuna, MLflow write, Phoenix tracing, or method promotion.

Why fifth:

- Moves toward self-improvement, but should follow the feedback-to-improvement
  review step.

Risk:

- Medium claim risk. Must be explicit that experiment execution is not
  implemented.

### 6. Local LLM/Ollama Provider Routing UX Polish

Scope:

- Improve Settings/Results display for provider, task route, local model
  readiness, deterministic fallback, and evidence-required posture.
- Maybe add a safe check-only local setup link.

Why sixth:

- Useful for optional model users, but not required for the core local-first
  task/action loop.

Risk:

- Medium claim risk around model availability and generation.

### 7. Agent Capabilities Dashboard

Scope:

- Make the fixed action registry easier to understand: read-only vs
  approval-required, parameters, runtime availability, and blocked reasons.

Why seventh:

- Helpful, but largely informational. It should be part of or follow the guided
  planner.

Risk:

- Low.

### 8. "What Can IGY6 Do Next?" Guided Action Planner

Scope:

- Home/Results next-action suggestions based on current records: no sources,
  pending work, available evidence, reports, feedback gaps, pending approvals.

Why eighth:

- Good product polish, but less directly tied to the AI/task/action interface
  than DIFF-197.

Risk:

- Low to medium, mostly UI logic.

## Selected Next DIFF

Recommended next DIFF:

`DIFF-197: Guided Agent Task Intake Planner UX`

Rationale:

- It is the shortest bridge from the verified evidence workflow to the desired
  AI task handling product loop.
- It uses existing `/agent/intent`, evidence retrieval, report, feedback,
  outcome, approval, and action routes instead of inventing new execution
  behavior.
- It can remain UI-focused and bounded.
- It improves normal-user value immediately: the user can type a request and
  see safe next actions instead of raw JSON.
- It preserves local-first behavior, approval requirements, and no-arbitrary
  shell execution.

Out of scope for DIFF-197:

- Creating a new autonomous planner.
- Adding new action types.
- Adding arbitrary shell execution.
- Adding LLM-driven action execution.
- Running experiments.
- Changing work queue execution semantics.
- Changing database schema unless inspection proves a tiny route/UI blocker and
  the DIFF explicitly scopes it.

## Paste-Ready Prompt For Selected Next DIFF

```text
You are working in the IGY6 repo on branch dev.

Branch policy:

* Future IGY6 work happens on dev.
* Do not remove anything from dev.
* Do not remove private/dev/build instruction files from dev.
* main is the public/runtime-clean branch.
* Later, only necessary public/runtime-safe files will be selectively promoted to main.
* Do not merge main into dev unless explicitly instructed.
* Do not cherry-pick main into dev unless explicitly instructed.
* Do not push unless explicitly instructed.
* Do not touch main.

Current known state:

* Latest completed dev DIFF: DIFF-196 next AI task handling gap audit.
* DIFF-196 selected DIFF-197 Guided Agent Task Intake Planner UX as the next highest-value product DIFF.
* The manual upload -> work item -> evidence -> retrieval -> review/report/feedback/history path has been verified.
* Operator smoke tooling exists:
  * scripts/operator-smoke-check.sh --check
  * scripts/operator-smoke-check.sh --run
  * scripts/operator-smoke-check.sh --run --record
  * scripts/operator-smoke-check.sh --run-record
  * scripts/operator-smoke-check.sh --list-results
  * scripts/operator-smoke-check.sh --latest-result
  * scripts/operator-smoke-check.sh --show-result PATH
* Existing agent/request surfaces include:
  * /agent/intent request understanding
  * /agent/capabilities fixed action registry
  * /agent/actions/:action/execute bounded action execution
  * approval records and decisions
  * evidence retrieval preview
  * work item records
  * report, feedback, outcome, improvement, and experiment metadata routes
* The next goal is to turn intent preview into guided, safe next-step UX without adding autonomous execution.
* Private/dev files must remain tracked on dev.

Goal:
Create and complete DIFF-197: Guided Agent Task Intake Planner UX.

Create:

* docs/diffs/DIFF-197-guided-agent-task-intake-planner-ux.md

Purpose:
Add a normal-user planner around the existing /agent/intent response so IGY6 can turn a plain-language request into safe, visible next-step options using current capabilities only.

Required pre-work inspection:

git status --short
git branch --show-current
git log --oneline --decorate -25
git branch -vv
git diff --name-status
git diff --check
sed -n '1,320p' AGENTS.md
sed -n '1,280p' docs/BRANCH_POLICY.md
sed -n '1,420p' docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md
sed -n '1,360p' docs/diffs/DIFF-176-request-understanding-clarification-flow.md
sed -n '1,360p' docs/diffs/DIFF-185-evidence-answer-review-ux.md
sed -n '1,360p' docs/diffs/DIFF-187-basic-report-workflow-ux.md
sed -n '1,360p' docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
sed -n '1,360p' docs/ui/README.md
sed -n '1,760p' crates/igy6-agent-api/src/lib.rs
sed -n '1860,1975p' crates/igy6-gateway/src/lib.rs
sed -n '8660,8865p' crates/igy6-gateway/src/lib.rs
sed -n '1300,1605p' apps/web/src/app/page.tsx
find apps/web/src/app/api/agent -maxdepth 5 -type f | sort
sed -n '1,220p' apps/web/src/app/api/agent/intent/route.ts
sed -n '1,220p' apps/web/src/app/api/agent/capabilities/route.ts
sed -n '1,220p' apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort

Allowed scope:

* docs/diffs/DIFF-197-guided-agent-task-intake-planner-ux.md
* apps/web/src/app/page.tsx
* apps/web/src/app/globals.css only if needed for small planner UX styling
* docs/ui/README.md if user-facing behavior changes
* apps/web/src/app/api/agent/* only if a proxy bug blocks the UX
* crates/igy6-agent-api or crates/igy6-gateway only if inspection finds a tiny response-field blocker and the DIFF explicitly records it

Preferred scope:

* UI-first, using existing /agent/intent, /agent/capabilities, /agent/actions/:action/execute, /approvals, /api/chat/retrieval-preview, report, feedback, outcome, and work/status surfaces.

Prohibited:

* Do not remove anything from dev.
* Do not remove .codex.
* Do not remove AGENTS.md.
* Do not remove Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md.
* Do not remove docs/agents.
* Do not remove docs/plans.
* Do not switch to main.
* Do not touch main.
* Do not merge main into dev.
* Do not merge dev into main.
* Do not cherry-pick.
* Do not promote files.
* Do not edit .env.
* Do not print secrets.
* Do not dump runtime/private data from IGY6_DATA_ROOT.
* Do not add arbitrary shell execution.
* Do not accept user-provided argv/command/script/shell fields.
* Do not add new system-changing actions.
* Do not bypass approval.
* Do not execute unsupported work automatically.
* Do not add LLM-driven autonomous planning or action execution.
* Do not run experiments or promote methods.
* Do not change worker execution semantics.
* Do not run destructive commands.
* Do not kill processes.
* Do not run sudo.
* Do not change system group membership.

Implementation requirements:

1. Keep /agent/intent as the source of request understanding.
2. Add a normal-user planner summary to the existing AgentCommandPanel.
3. Convert request_understanding.category and posture into safe next-step cards:
   * evidence_question: guide to Ask over evidence with the typed request as query.
   * add_data: guide to Add Data, not automatic upload.
   * check_work_status: guide to Work and optionally the read-only show_work_items action.
   * create_report: guide to Results report workflow and require report scope before creation.
   * feedback: guide to Results feedback capture and require target/label.
   * record_outcome: guide to Results outcome capture and require target/status.
   * experiment_or_improvement: guide to proposed improvement review, not experiment execution.
   * request_action/system_changing_action: show approval posture and only fixed supported actions.
   * unclear/unsupported: show clarification guidance and no execution button.
4. Keep raw JSON and raw parameter controls in Advanced only.
5. Do not make the UI claim autonomous execution, complete self-improvement, or unsupported source/media handling.
6. Preserve existing preview, read-only action execution, approval request, and approved action execution behavior unless a narrow bug fix is explicitly scoped.
7. Add DOM markers suitable for verification, such as:
   * data-agent-planner-summary
   * data-agent-planner-card
   * data-agent-planner-next-step
   * data-agent-planner-safety
8. Update docs/ui/README.md with the new guided planner behavior and limits.

Verification requirements:

git status --short
git diff --check
git diff --name-status
npm --prefix apps/web run build
bash -n scripts/operator-smoke-check.sh
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --latest-result || true
grep -n "data-agent-planner-summary\|data-agent-planner-card\|data-agent-planner-next-step\|data-agent-planner-safety" apps/web/src/app/page.tsx
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort

If Rust files change, also run:

cargo fmt --all --check
cargo test -p igy6-agent-api
cargo test -p igy6-gateway

If Docker is available and safe, run:

scripts/operator-smoke-check.sh --check

Do not run full smoke unless needed for a specific verified issue.

DIFF-197 must record:

* Status: Complete
* current branch and HEAD before work
* whether dev was ahead/behind origin/dev before commit
* files inspected
* planner UX behavior implemented
* categories mapped to safe next steps
* safety/approval boundaries preserved
* verification commands and results
* files changed
* confirmation no arbitrary execution, new system action, approval bypass, LLM autonomous planning, experiment execution, runtime/private data dump, .env edit, or main work occurred

Commit:

git add -A
git commit -m "Complete DIFF-197 guided agent task intake planner UX"

Final response must include:

* new DIFF created
* branch and HEAD before work
* whether dev was ahead/behind origin/dev before commit
* whether private/dev files remained tracked
* files inspected
* planner UX changes
* categories mapped
* safety boundaries preserved
* verification commands run and results
* files changed
* commit hash
* confirmation no main work, merge, cherry-pick, push, promotion, sudo, group change, destructive command, .env edit, runtime/private data dump, arbitrary execution, new system action, approval bypass, LLM autonomous planning, or experiment execution was performed
```

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -25
git branch -vv
git diff --name-status
git diff --check
sed -n '1,320p' AGENTS.md
sed -n '1,280p' docs/BRANCH_POLICY.md
sed -n '1,360p' docs/diffs/DIFF-176-request-understanding-clarification-flow.md
sed -n '1,360p' docs/diffs/DIFF-183-dev-next-runtime-work-selection.md
sed -n '1,360p' docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
sed -n '1,360p' docs/diffs/DIFF-185-evidence-answer-review-ux.md
sed -n '1,360p' docs/diffs/DIFF-186-work-status-recovery-ux-polish.md
sed -n '1,360p' docs/diffs/DIFF-187-basic-report-workflow-ux.md
sed -n '1,360p' docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
sed -n '1,360p' docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
sed -n '1,360p' docs/diffs/DIFF-190-operator-smoke-verification-bundle.md
sed -n '1,360p' docs/diffs/DIFF-195-smoke-result-viewer-summary-command.md
sed -n '1,420p' docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md 2>/dev/null || true
sed -n '1,420p' docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md 2>/dev/null || true
sed -n '1,360p' docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md 2>/dev/null || true
sed -n '1,280p' README.md 2>/dev/null || true
sed -n '1,280p' docs/ui/README.md 2>/dev/null || true
find docs/diffs -maxdepth 1 -type f | sort | tail -130
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
find apps/web crates services scripts docs -maxdepth 6 -type f | sort | grep -E "agent|intent|action|task|work|queue|approval|feedback|outcome|improvement|experiment|planner|dispatch|capability|self|report|chat|llm|ollama|model|tool|execute|recovery" || true
grep -R "agent\|intent\|request_action\|system_changing_action\|task\|work item\|approval\|action\|capability\|feedback\|outcome\|improvement\|experiment\|self-improvement\|planner\|dispatch\|LLM\|Ollama\|model\|execute\|tool" apps/web crates services scripts docs -n 2>/dev/null | head -800 || true
rg -n "agent|intent|request_action|system_changing_action|task|work item|approval|action|capability|feedback|outcome|improvement|experiment|self-improvement|planner|dispatch|LLM|Ollama|model|execute|tool" apps/web/src crates scripts docs --glob '!**/.next/**' --glob '!**/node_modules/**' | head -800
sed -n '1,320p' crates/igy6-agent-api/src/lib.rs
sed -n '320,760p' crates/igy6-agent-api/src/lib.rs
sed -n '760,980p' crates/igy6-agent-api/src/lib.rs
find apps/web/src/app/api/agent -maxdepth 5 -type f | sort
sed -n '1,220p' apps/web/src/app/api/agent/intent/route.ts
sed -n '1,220p' apps/web/src/app/api/agent/capabilities/route.ts
sed -n '1,220p' apps/web/src/app/api/agent/actions/[action_name]/execute/route.ts
rg -n "agent/intent|agent/capabilities|agent/actions|execute_agent|ACTION_REGISTRY|classify_agent_intent|work-items|dispatch|improvements|experiments|feedback|outcomes|reports|llm|ollama" crates/igy6-gateway/src/lib.rs apps/web/src/app/page.tsx crates/igy6-llm/src/lib.rs crates/igy6-work-queue-reports/src/lib.rs services/self_improvement/README.md services/reports/README.md docs/llm/LOCAL_LLM_PROVIDER_PLAN.md
sed -n '1,280p' crates/igy6-llm/src/lib.rs
sed -n '1,260p' crates/igy6-work-queue-reports/src/lib.rs
sed -n '1,220p' services/self_improvement/README.md
sed -n '1,220p' services/reports/README.md
sed -n '2450,2665p' crates/igy6-gateway/src/lib.rs
sed -n '4650,4850p' crates/igy6-gateway/src/lib.rs
sed -n '4860,5010p' crates/igy6-gateway/src/lib.rs
sed -n '9950,10140p' crates/igy6-gateway/src/lib.rs
sed -n '10140,10280p' crates/igy6-gateway/src/lib.rs
rg -n "fn execute_agent_action|execute_agent|approved|approval_id|start_stack|run_retrieval_preview|agentRuntime|latestIntent|data-agent|Agent" crates/igy6-gateway/src/lib.rs apps/web/src/app/page.tsx
sed -n '1320,1495p' apps/web/src/app/page.tsx
sed -n '1495,1605p' apps/web/src/app/page.tsx
sed -n '8660,8735p' crates/igy6-gateway/src/lib.rs
sed -n '10300,10535p' crates/igy6-gateway/src/lib.rs
sed -n '1880,1975p' crates/igy6-gateway/src/lib.rs
sed -n '8735,8855p' crates/igy6-gateway/src/lib.rs
sed -n '8855,9130p' crates/igy6-gateway/src/lib.rs
git status --short
git diff --check
git diff --name-status
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

Results:

- Initial `git status --short`: clean.
- Initial branch: `dev`.
- Initial HEAD:
  `156ea86 Complete DIFF-195 smoke result viewer summary command`.
- Initial `git branch -vv`: `dev` synced with `origin/dev`, no ahead/behind
  marker.
- Initial `git diff --name-status`: no output.
- Initial `git diff --check`: passed.
- Final `git diff --check`: passed.
- Final `git diff --name-status`: only this DIFF file added.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope `Status: Draft` strings in
  DIFF-177, DIFF-180, `DIFF_TEMPLATE.md`, and command transcripts in completed
  DIFF records.
- No runtime code was changed.

## Files Changed

- `docs/diffs/DIFF-196-next-ai-task-handling-gap-audit.md`

## Verification Summary

- DIFF-196 is planning-only and DIFF-only.
- Current AI/task/action/self-improvement capability was audited from docs and
  source code.
- At least five candidate next DIFFs were ranked.
- DIFF-197 Guided Agent Task Intake Planner UX was selected as the recommended
  next DIFF.
- A paste-ready future prompt for DIFF-197 is included.
- Private/dev files remained tracked on `dev`.
- No runtime app code, UI code, Rust code, `.env`, Docker volumes, databases,
  Qdrant, Neo4j, local service data, or `IGY6_DATA_ROOT` contents were edited
  or dumped.
- No files were removed.
- No `main` work, merge, cherry-pick, push, or promotion was performed.
- No `sudo`, user group change, process killing, or destructive command was
  performed.

## Final Status

DIFF-196 is complete.
