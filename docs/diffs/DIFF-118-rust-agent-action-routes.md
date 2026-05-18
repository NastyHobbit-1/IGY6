# DIFF-118: Rust Agent Action Routes

Status: Locked

## Type

Change-bearing

## Objective

Migrate the final web-used agent action fallback routes to the Rust gateway:

- `POST /agent/actions/`
- `POST /agent/actions/{action_name}/execute`

The migration must preserve the existing fixed action allowlist, approval gates,
audit events, redaction, local-only host bridge assumptions, and timeout-bound
execution behavior without accepting arbitrary shell commands or user-provided
argv.

## Baseline Facts

- DIFF-117 is locked and leaves 2 web-used routes requiring FastAPI fallback.
- Existing Python action names are:
  - `show_project_health`
  - `show_git_status`
  - `show_latest_diff`
  - `show_work_items`
  - `run_retrieval_preview`
  - `start_stack`
  - `stop_stack`
  - `run_last_healthy_stack`
- `start_stack`, `stop_stack`, and `run_last_healthy_stack` are
  system-changing and require approved `agent_action` approvals.
- Existing Rust `igy6-agent-api` already contains the fixed registry and
  classifier.
- Existing Rust `igy6-host-bridge` already exposes a local-only fixed action
  surface for the script-backed stack actions.
- FastAPI is still required only for the final web-used agent action routes
  before this DIFF.

## Allowed Scope

- Add Rust-native handling for `POST /agent/actions/` and
  `POST /agent/actions/{action_name}/execute` in `crates/igy6-gateway/`.
- Reuse `crates/igy6-agent-api/` and `crates/igy6-host-bridge/` where useful.
- Add deterministic request validation, allowlist checks, approval checks,
  host-bridge request planning/calling, result redaction, timeout behavior, and
  audit insertion for agent actions.
- Add route-level and unit tests for safe success, rejection, approval, audit,
  redaction, fixed argv, no arbitrary command, and timeout/host-bridge
  behavior.
- Update `configs/rust-cutover-manifest.json` honestly based on route parity.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` counts and route
  matrix entries.
- Add snippet-vault Rust equivalent JSONL records for reusable agent
  action/approval/audit/allowlist behavior.
- Update this DIFF document with verification results and lock it after checks
  pass.

## Prohibited Scope

- No new action names.
- No action surface expansion beyond the Python/Rust fixed registries.
- No arbitrary shell execution.
- No user-provided argv or command execution.
- No raw user text execution.
- No approval bypass.
- No unaudited action execution attempt.
- No non-local or unauthenticated host bridge calls.
- No `.env` reads or writes.
- No secrets, tokens, private keys, runtime/private data commits, or raw secret
  output exposure.
- No FastAPI removal or archival unless route parity proves no fallback remains
  and this DIFF explicitly records that result.
- No Docker Compose rewiring.
- No database migrations.
- No unrelated refactor, broad cleanup, renames, or redesign.

## Required Tags

Use `DIFF-118` in the commit message and final change summary.

## Verification

- `git status --short` passed with DIFF-118 scoped files before commit.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=60 web_used=41 missing_from_rust=34
  web_requires_fallback=0`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 49 gateway tests.
- `cargo test -p igy6-agent-api` passed, 6 tests.
- `cargo test -p igy6-host-bridge` passed, 7 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v
  /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api
  python -m unittest discover tests` passed, 8 tests.

## Completion Criteria

- `POST /agent/actions/` is Rust-native and returns the fixed allowlisted action
  inventory without fallback.
- `POST /agent/actions/{action_name}/execute` is Rust-native and handles only
  fixed allowlisted actions without fallback.
- Unknown and malformed action names are rejected.
- Missing required parameters are rejected before execution.
- System-changing actions require an approved matching `agent_action` approval.
- Every execution attempt writes deterministic audit events.
- Script-backed actions use only fixed argv through the local host bridge path
  or return an honest blocked/unavailable response without arbitrary command
  execution.
- Output summaries are redacted and bounded.
- Route parity is updated honestly. `web_routes_requiring_fallback` is set to
  `0` only if both target routes are migrated.
- Required verification passes.
- This DIFF is locked only after verification passes.

## Out Of Scope Follow-Up

- Removing or archiving FastAPI.
- Replacing Python workers.
- Expanding agent action names.
- Migrating non-web FastAPI fallback routes.
