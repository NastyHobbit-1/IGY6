# DIFF-083: Local Agent Command Plane

Status: Locked

## Type

Change-bearing

## Objective

Add an MVP local agent command plane that accepts chat/API-style requests,
classifies them into fixed project actions, returns intent verification packets,
and executes only typed allowed actions with approval gates and audit logging.

## Baseline Facts

- DIFF-080 fixed `/chat/retrieval-preview` missing Qdrant collection handling.
- DIFF-081 exists and is locked.
- DIFF-082 added `scripts/run.sh`, `scripts/stop.sh`,
  `scripts/run-last-healthy-config.sh`, and `scripts/lib/igy6-ops.sh`.
- README says IGY6 does not generate LLM answers, does not call external
  models, and does not implement autonomous system-changing actions.
- Existing concepts include work items, approvals, audit events, source
  registry, retrieval preview, and deterministic evidence answer.
- Existing approval records can store request payload JSON and approved/denied
  status without a schema change.
- No arbitrary shell command plane exists before this DIFF.

## Allowed Scope

- `docs/diffs/DIFF-083-local-agent-command-plane.md`
- API agent/action modules and routes under `services/api/app/`
- `services/api/app/main.py` to register the new route
- Narrow API tests under `services/api/tests/`
- README documentation for the command plane

## Prohibited Scope

- No locked DIFF edits.
- No Docker Compose changes.
- No `.env` changes.
- No database migrations unless existing audit/approval schema is insufficient.
- No ingestion behavior changes.
- No external model calls.
- No unrestricted shell execution.
- No autonomous background agent loop.
- No browser, router, or account control.
- No broad refactor.
- No frontend changes in this DIFF.

## Required Tags

- Commit message must include `DIFF-083`.
- Final response must identify `DIFF-083`.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m compileall services/api services/worker`
- Relevant API tests
- Direct API call for `/agent/intent` with `show project health`
- Direct API call for `/agent/intent` with `start the stack`
- Direct API call for `/agent/intent` with `run rm -rf /`
- Direct API call proving approval-required actions do not execute without
  approval
- If UI changed, `npm --prefix apps/web run build`

## Completion Criteria

- Unknown action requests are rejected.
- Arbitrary shell requests are rejected.
- Read-only actions can return project health, git status, latest DIFF,
  work-item, and retrieval-preview status.
- `start_stack`, `stop_stack`, and `run_last_healthy_stack` require approval.
- Approved stack actions call the DIFF-082 scripts safely by argv array with
  timeout and bounded output.
- No secrets are returned.
- Tests cover known read-only classification, unknown action rejection,
  arbitrary shell rejection, and approval-required action blocked without
  approval.
- Existing retrieval preview still works.
- Existing run/stop scripts still work.
- Verification results are recorded below before locking.

## Verification Result

- `git status --short`: showed only DIFF-083 scoped files before staging.
- `git diff --check`: passed.
- `python3 -m compileall services/api services/worker`: passed.
- `.venv/bin/python services/api/tests/test_agent_actions.py`: passed
  (`Ran 4 tests`).
- `.venv/bin/python services/api/tests/test_vector_memory_missing_collection.py`:
  passed (`Ran 2 tests`) to confirm retrieval-preview behavior stayed intact.
- `scripts/run.sh --detached`: passed; rebuilt/recreated the local stack,
  all health checks passed, and a last-healthy snapshot was written under
  `IGY6_DATA_ROOT`.
- `POST /agent/intent` with `show project health`: returned
  `show_project_health`, `read_only`, `approval_required: false`, and
  `executable_now: true`.
- `POST /agent/intent` with `start the stack`: returned `start_stack`,
  `system_changing`, `approval_required: true`, and `executable_now: false`.
- `POST /agent/intent` with `run rm -rf /`: returned no proposed action,
  `unknown`, high risk, and a reason that arbitrary shell/destructive commands
  are not allowed.
- `POST /agent/actions/start_stack/execute` without approval: returned
  `403 Forbidden` with `Agent action requires approval`.
- `POST /agent/actions/show_latest_diff/execute`: returned the active
  DIFF-083 document and status.
- `POST /agent/actions/show_git_status/execute`: returned branch and commit
  from mounted git metadata. The API container does not include the `git`
  executable, so dirty state is reported as unavailable rather than guessed.
- `POST /agent/actions/show_work_items/execute`: returned a structured empty
  work-item list.
- `POST /agent/actions/run_retrieval_preview/execute`: returned HTTP 200
  retrieval preview data with `answer_status: not_generated`.
- `POST /chat/retrieval-preview`: returned HTTP 200 and preserved the
  DIFF-080 missing-vector-collection behavior.
- `scripts/stop.sh`: passed and stopped the stack without deleting data.
- `npm --prefix apps/web run build`: not run because this DIFF did not change
  frontend files.
