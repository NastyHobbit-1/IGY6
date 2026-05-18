# DIFF-116: Rust Work Item Create Route

Status: Locked

## Type

Change-bearing

## Objective

Migrate `POST /work-items/` to the Rust gateway so web work-item creation no
longer requires FastAPI fallback, while preserving safe creation-only semantics,
intent verification payload storage, deterministic initial status, and audit
logging.

## Baseline Facts

- DIFF-115 is locked and leaves 4 web-used routes requiring FastAPI fallback.
- FastAPI `POST /work-items` creates a work item with
  `pending_intent_verification` status, stores the provided intent verification
  context inside `payload_json.intent_verification`, inserts a
  `work_item.created` audit event, commits both records together, and returns
  the created work item.
- FastAPI `POST /work-items` does not synchronously dispatch work or execute
  agent actions.
- Remaining web-used fallback routes are:
  - `POST /agent/actions/`
  - `POST /agent/actions/{action_name}/execute`
  - `POST /collection-runs/manual-upload`
  - `POST /work-items/`
- FastAPI remains required until all active web-used fallback routes are either
  Rust-native or consciously retained with an explicit retirement plan.

## Allowed Scope

- Add Rust-native handling for `POST /work-items` and `POST /work-items/` in
  `crates/igy6-gateway/`.
- Add request validation, DB insertion, response construction, and
  `work_item.created` audit insertion for work-item creation.
- Add route-level and validation tests for the migrated route.
- Update `scripts/rust-route-parity.py` only if needed to count the migrated
  route accurately.
- Update `configs/rust-cutover-manifest.json` route parity fields honestly.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` counts and route
  matrix entries.
- Add snippet-vault Rust equivalent JSONL records for reusable work-item
  create/audit behavior.
- Update this DIFF document with verification results and lock it after all
  required checks pass.

## Prohibited Scope

- No agent execution route migration.
- No manual upload route migration.
- No work-item dispatch or status route migration.
- No FastAPI removal or archival.
- No Docker Compose rewiring.
- No database migrations.
- No runtime/private data reads or writes beyond the intended work-item and
  audit DB inserts.
- No `.env` changes or secret handling changes.
- No arbitrary shell execution.
- No approval bypass.
- No unrelated refactor, broad cleanup, renames, or redesign.

## Required Tags

Use `DIFF-116` in the commit message and final change summary.

## Verification

- `git status --short`
- `git diff --check`
- `python3 scripts/rust-route-parity.py --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-gateway`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Run the established legacy API tests if the repo still supports the DIFF-114
  and DIFF-115 command.

## Completion Criteria

- `POST /work-items/` is Rust-native and no longer proxied to FastAPI.
- Work-item creation validates required fields and unsupported work types.
- Work-item creation stores deterministic `pending_intent_verification` status.
- Work-item creation records `payload_json.intent_verification`.
- Work-item creation writes `work_item.created` audit event in the same DB
  transaction.
- No dispatch, agent execution, manual upload, or artifact write behavior is
  added.
- Route parity count is reduced from 4 web-used fallback routes to 3.
- Manifest and route audit remain honest that FastAPI fallback is still
  required.
- Required verification passes.
- This DIFF is locked only after verification passes.

## Verification Results

- `git status --short` inspected before edits and after verification.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=57 web_used=41 missing_from_rust=36
  web_requires_fallback=3`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed with 42 gateway tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- Established legacy API tests passed:
  `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`.

## Completion Summary

- Migrated `POST /work-items` and `POST /work-items/` to Rust-native gateway
  handling.
- Added validation for required work item fields, intent verification context,
  supported work types, actor ID, and payload object shape.
- Preserved deterministic `pending_intent_verification` initial status.
- Inserted `work_item.created` audit event in the same DB transaction as the
  work item.
- Intentionally did not dispatch work, execute agents, migrate manual upload,
  or alter work-item status routes.
- Reduced web-used FastAPI fallback count from 4 to 3.

## Out Of Scope Follow-Up

- `POST /collection-runs/manual-upload`
- `POST /agent/actions/`
- `POST /agent/actions/{action_name}/execute`
- `POST /work-items/{work_item_id}/dispatch`
- `POST /work-items/{work_item_id}/status`
