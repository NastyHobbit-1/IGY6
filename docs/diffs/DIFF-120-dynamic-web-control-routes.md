# DIFF-120: Dynamic Web Control Route Parity

Status: Locked

## Type

Change-bearing

## Objective

Resolve the four dynamically referenced `apps/web` control routes that DIFF-119
found outside the route parity extractor so web controls no longer depend on
FastAPI fallback.

## Allowed Scope

- Add Rust-native gateway handlers, validation, DB mutations, and audit events
  for:
  - `POST /analysis/patterns/{pattern_id}/review`
  - `POST /approvals/{approval_id}/decision`
  - `POST /reports/{report_id}/render`
  - `POST /work-items/{work_item_id}/dispatch`
- Add focused Rust route tests for success routing, validation, missing IDs,
  invalid transitions, and unsafe render/dispatch behavior.
- Update `scripts/rust-route-parity.py` and
  `scripts/test-rust-route-parity.py` so dynamic web route references are
  tracked explicitly.
- Update route classification, cutover manifest, and Rust migration docs with
  honest DIFF-120 counts and fallback posture.
- Add snippet-vault JSONL records only for reusable route/write/audit/status
  transition patterns introduced by this DIFF.

## Prohibited Scope

- No unrelated non-web route migration.
- No FastAPI removal, disablement, archival, or Docker topology change.
- No database migrations or dependency changes.
- No arbitrary command execution, user-provided argv execution, approval
  bypass, agent execution broadening, or new action surface.
- No writes outside configured artifact storage.
- No secrets, `.env` commits, runtime/private data commits, or unsafe deletion.
- No locked DIFF edits.

## Required Tags

Use `DIFF-120` in the commit message and final change summary.

## Verification

- `git status --short` checked DIFF-120 scoped files before commit.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=64 web_used=45 missing_from_rust=30
  web_requires_fallback=0`.
- `python3 scripts/test-rust-route-parity.py` passed, 4 tests.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 53 tests.
- `cargo test -p igy6-agent-api` passed, 6 tests.
- `cargo test -p igy6-host-bridge` passed, 7 tests.
- `scripts/rust-cutover.sh --check` passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
  passed.
- Changed snippet-vault JSONL files were validated line-by-line as valid JSON.
- `npm --prefix apps/web run build` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.
- No web test script exists beyond the build script in `apps/web/package.json`.
- `docker run --rm -v
  /home/nasty/projects/IGY6/services/api/tests:/app/tests:ro infra-legacy-api
  python -m unittest discover tests` passed, 8 tests.

## Completion Criteria

- All four target dynamic web routes are Rust-native or any unmigrated route is
  explicitly documented with an honest blocker.
- `scripts/rust-route-parity.py --check` tracks dynamic web references and
  reports no web route requiring fallback.
- Classification counts and manifest counts match route parity.
- FastAPI fallback posture remains honest.
- Required verification passes or blocked checks are reported precisely.
