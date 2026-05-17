# DIFF-110: Rust Feedback And Outcome Write Routes

Status: Locked

## Type

Change-bearing

## Objective

Move the next safest web-used write batch to Rust-native gateway handling:
`POST /feedback` and `POST /outcomes`. These routes record review feedback and
outcomes, have explicit validation and audit semantics, and do not execute
agent actions, shell commands, collectors, settings writes, or external
service calls.

## Baseline Facts

- DIFF-109 reports `fastapi=91`, `rust_native=46`, `web_used=41`,
  `missing_from_rust=46`, and `web_requires_fallback=14`.
- `POST /feedback` writes a feedback event, writes a `feedback.created` audit
  event, applies source trust feedback for source targets, and creates
  improvement items for weak non-source feedback.
- `POST /outcomes` writes an outcome, writes an `outcome.created` audit event,
  validates target and evidence references, updates target status/metadata, and
  writes an `outcome.target_updated` audit event.
- The Rust `igy6-write-api` crate already mirrors feedback and outcome
  validation/planning behavior.

## Allowed Scope

- Add Rust-native DB-backed handling for:
  - `POST /feedback`
  - `POST /outcomes`
- Preserve existing request/response shapes where practical.
- Validate request bodies, missing/empty IDs, target types, labels/statuses,
  evidence IDs, metadata object shapes, and target existence where Python does.
- Insert audit events for state-changing behavior.
- Preserve source trust side effects for trusted/noisy/rejected source
  feedback.
- Preserve weak-feedback improvement-item side effects for non-source feedback.
- Preserve outcome target status/metadata side effects.
- Update route-level tests for validation, missing DB behavior, and registry
  coverage.
- Update `configs/rust-cutover-manifest.json` route parity counts and status.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`.
- Add snippet-vault JSONL records for reusable Python-to-Rust feedback/outcome
  write/audit patterns.
- Lock this DIFF after verification passes.

## Prohibited Scope

- No locked DIFF edits.
- No agent execution route migration.
- No settings/env verify or apply route migration.
- No source, collection, report, work-item, approval decision, or analysis
  write route migration.
- No arbitrary shell execution.
- No approval bypass.
- No `.env` content reads or writes.
- No runtime/private data commits.
- No database schema changes or migrations.
- No external service calls.
- No FastAPI removal or disabling.
- No claims that FastAPI is removable.

## Required Tags

Commit messages and final summaries must include `DIFF-110`.

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
- Run existing API/web tests where available.

## Completion Criteria

- `POST /feedback` and `POST /outcomes` are Rust-native and no longer proxied
  to FastAPI.
- Invalid write requests are rejected by Rust without fallback.
- Missing `DATABASE_URL` returns deterministic Rust DB-route errors.
- Audit events are inserted for successful feedback/outcome state changes.
- Route parity counts are updated honestly.
- FastAPI fallback remains required while remaining write/action routes depend
  on it.
- DIFF-110 is locked after verification passes.

## Results

- Migrated `POST /feedback` to Rust-native DB-backed handling.
- Migrated `POST /outcomes` to Rust-native DB-backed handling.
- Added Rust validation for feedback target types, labels, IDs, actor IDs,
  notes, and metadata object shape.
- Added Rust validation for outcome target types, statuses, IDs, evidence IDs,
  target existence, evidence existence, summaries, timestamps, and metadata
  object shape.
- Preserved feedback audit event insertion, source trust feedback side effects,
  and weak non-source feedback improvement-item creation.
- Preserved outcome audit event insertion, outcome target status/metadata
  updates, and target update audit event insertion.
- Route parity changed from `rust_native=46`, `missing_from_rust=46`, and
  `web_requires_fallback=14` to `rust_native=48`, `missing_from_rust=44`, and
  `web_requires_fallback=12`.
- FastAPI fallback remains required for the remaining write/action routes.
- No agent execution routes were migrated.

## Verification Results

- `git status --short` checked DIFF-110 scoped files plus generated `target/`
  before cleanup.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed with
  `fastapi=91 rust_native=48 web_used=41 missing_from_rust=44
  web_requires_fallback=12`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 21 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Changed snippet-vault JSONL files validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- `docker run --rm -v /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api python -m unittest discover tests`
  passed, 8 tests.

## Out Of Scope Follow-Up

- Agent action execution.
- Settings/env verify and apply.
- Approval decisions.
- Source, collection, report, work-item, and analysis writes.
- Full FastAPI retirement.
