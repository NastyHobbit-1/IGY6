# DIFF-084: Agent Command UI Capabilities

Status: Locked

## Type

Change-bearing

## Objective

Expose the existing local agent command plane in the IGY6 web UI/chat area and
add an honest runtime capability layer so users can preview typed actions,
execute read-only actions, and see approval/runtime blockers for stack-control
actions without implying autonomous or arbitrary shell control.

## Baseline Facts

- DIFF-083 is locked and added `/agent/intent` and
  `/agent/actions/{action_name}/execute`.
- DIFF-083 added a typed action registry in `services/api/app/agent_actions.py`
  but did not change frontend files.
- DIFF-082 added host-side operator scripts for run, stop, and last-healthy
  stack recovery.
- The API container may not include Docker CLI, Docker Compose, or Docker socket
  access, so stack-control actions must report runtime capability honestly.
- Existing approvals support `agent_action` records with payload JSON that can
  match an action name and parameters.
- The current web UI already contains the chat retrieval preview and uses small
  inline client scripts for existing local controls.

## Allowed Scope

- `docs/diffs/DIFF-084-agent-command-ui-capabilities.md`
- `services/api/app/agent.py`
- `services/api/app/agent_actions.py`
- Narrow backend tests under `services/api/tests/`
- Minimal frontend files under `apps/web`
- README documentation for the agent command UI

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose changes.
- No `.env` changes.
- No Docker socket mounting.
- No API container Docker installation.
- No external model calls.
- No browser, router, or account automation.
- No arbitrary shell execution.
- No autonomous background loop.
- No ingestion changes.
- No database migrations.
- No broad UI redesign.

## Required Tags

- Commit message must include `DIFF-084`.
- Final response must identify `DIFF-084`.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m compileall services/api services/worker`
- Backend agent tests
- `npm --prefix apps/web run build`
- Direct API:
  - `GET /agent/capabilities`
  - `POST /agent/intent` with `show project health`
  - `POST /agent/intent` with `start the stack`
  - `POST /agent/intent` with `run rm -rf /`
  - `POST /agent/actions/show_project_health/execute`
  - `POST /agent/actions/start_stack/execute` without approval must return 403
- UI smoke check through the local web app if the stack is running.

## Completion Criteria

- `GET /agent/capabilities` returns deterministic action and runtime capability
  data.
- UI can call `/agent/intent` and display the intent packet.
- UI can execute read-only actions through `/agent/actions/{action_name}/execute`.
- UI blocks approval-required actions unless approval is explicitly satisfied.
- UI displays arbitrary shell rejection from the typed intent classifier.
- Runtime capability reporting does not falsely claim Docker stack control when
  Docker is unavailable inside the API runtime.
- No secrets are displayed.
- Existing chat retrieval preview still works.
- Existing DIFF-082 scripts still work from the host.
- Verification results are recorded below before locking.

## Verification Result

- `git status --short`: showed only DIFF-084 scoped files before staging.
- `git diff --check`: passed.
- `python3 -m compileall services/api services/worker`: passed.
- `.venv/bin/python services/api/tests/test_agent_actions.py`: passed
  (`Ran 6 tests`).
- `npm --prefix apps/web run build`: passed.
- `scripts/run.sh --detached`: passed; local stack rebuilt and health checks
  passed.
- `GET /agent/capabilities`: returned all typed actions. Read-only actions
  were executable. Stack-control actions reported required scripts present but
  `executable_in_api_runtime: false` because Docker CLI and Docker socket/control
  path are unavailable in the API runtime.
- `POST /agent/intent` with `show project health`: returned
  `show_project_health`, `read_only`, `approval_required: false`, and
  `executable_now: true`.
- `POST /agent/intent` with `start the stack`: returned `start_stack`,
  `system_changing`, `approval_required: true`, and `executable_now: false`.
- `POST /agent/intent` with `run rm -rf /`: returned no proposed action,
  `unknown`, high risk, and a reason that arbitrary shell/destructive commands
  are not allowed.
- `POST /agent/actions/show_project_health/execute`: returned `completed` with
  local readiness checks.
- `POST /agent/actions/start_stack/execute` without approval: returned
  `403 Forbidden` with `Agent action requires approval`.
- Same-origin web proxy checks:
  - `POST /api/agent/intent` with `show git status`: returned the expected
    read-only action.
  - `POST /api/agent/actions/show_git_status/execute`: returned `completed`
    git metadata from mounted git files.
- `POST /chat/retrieval-preview`: returned HTTP 200 and preserved deterministic
  not-generated retrieval behavior.
- UI smoke check: host curl to `127.0.0.1:3000` was not reachable from this
  sandbox after Compose startup, but the web container itself returned HTTP 200
  for `/` and the rendered HTML contained `Agent Command` and
  `Typed local command plane`.
- `scripts/stop.sh`: passed and stopped the stack without deleting data.
