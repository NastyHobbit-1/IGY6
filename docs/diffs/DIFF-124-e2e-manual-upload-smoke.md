# DIFF-124: E2E Manual Upload Smoke

Status: Locked

## Type

Change-bearing

## Objective

Add a guided end-to-end smoke plan and safe local script for the core user path:
manual upload -> artifact/work item -> processing status -> evidence/chunks when
available -> retrieval/chat.

## Baseline Facts

- DIFF-123 added a runtime smoke script for checking the local stack.
- `POST /collection-runs/manual-upload` is Rust-native and creates local
  artifact metadata plus queued normalization work metadata.
- Worker processing may be pending or unavailable, so smoke checks must
  distinguish upload success from downstream evidence completion.

## Allowed Scope

- Add `docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md`.
- Add `scripts/e2e-manual-upload-smoke.py`.
- Update README and user guide with the new smoke/checklist references.
- Add completion notes and verification results to this DIFF.

## Prohibited Scope

- No backend behavior changes.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No unsafe deletion.
- No runtime/private data commits.
- No secrets.
- No arbitrary shell/user-provided argv execution.
- No approval bypass.
- No broad worker or route migration.
- No locked DIFF edits.

## Required Behavior

- Script is safe and local-only.
- Script uses harmless payload:
  `IGY6 manual upload test. The secret test keyword is blue-raven-117.`
- Script explains whether it is fully automated or checklist-assisted.
- Script distinguishes upload route success, artifact/work item creation,
  worker processing pending, evidence availability, and retrieval visibility.
- Docs include UI click path and troubleshooting for approval required, queued
  work items, missing evidence, retrieval misses, and worker logs.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m py_compile scripts/e2e-manual-upload-smoke.py`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `npm --prefix apps/web test`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Notes

- Added `docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md` with a UI checklist,
  expected outcomes, and troubleshooting for approval-required, queued work,
  missing evidence, retrieval misses, and worker/API/web logs.
- Added `scripts/e2e-manual-upload-smoke.py`.
- The script is non-mutating by default with `--check`.
- The script only creates local runtime records when `--run` is explicitly
  passed against a running local stack.
- The `--run` path creates a `manual_upload` source, requests and approves a
  matching `manual_upload_collection` approval, uploads the harmless
  `blue-raven-117` payload, and reports artifact/work/evidence/retrieval status
  separately.
- Updated README and user guide with the script and checklist references.
- No backend behavior, route usage, worker architecture, data model, or migration
  changed.

## Verification Results

- Passed: `git status --short`
- Passed: `git diff --check`
- Passed: `python3 -m py_compile scripts/e2e-manual-upload-smoke.py`
- Passed: `npm --prefix apps/web run build`
- Passed: `npm --prefix apps/web run test:ui-smoke`
- Passed: `npm --prefix apps/web test`
- Passed: `python3 scripts/rust-route-parity.py --check`
  - `fastapi=91`
  - `rust_native=64`
  - `web_used=45`
  - `missing_from_rust=30`
  - `web_requires_fallback=0`
- Passed: `scripts/rust-cutover.sh --check`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Not run: `python3 scripts/e2e-manual-upload-smoke.py --run`; that mode creates
  local runtime smoke records and is intended for an already-running local stack.
