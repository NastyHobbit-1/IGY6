# DIFF-117: Rust Manual Upload Route

Status: Locked

## Type

Change-bearing

## Objective

Migrate `POST /collection-runs/manual-upload` to the Rust gateway so the web
manual upload collection workflow no longer requires FastAPI fallback, while
preserving source permission checks, approval checks, content-addressed artifact
storage, collection-run/raw-artifact metadata, and audit events.

## Baseline Facts

- DIFF-116 is locked and leaves 3 web-used routes requiring FastAPI fallback.
- FastAPI `POST /collection-runs/manual-upload` validates a manual upload
  source, source permission, optional approval, text MIME/content, stores bytes
  under the configured artifact store, creates a completed collection run,
  creates raw artifact metadata, writes audit events, and creates a queued
  normalization work item.
- FastAPI `POST /collection-runs/manual-upload` does not execute the queued
  work item synchronously.
- Existing Rust crate `crates/igy6-artifacts` provides content-addressed
  artifact storage bounded under a configured data root.
- Remaining web-used fallback routes are:
  - `POST /agent/actions/`
  - `POST /agent/actions/{action_name}/execute`
  - `POST /collection-runs/manual-upload`

## Allowed Scope

- Add Rust-native handling for `POST /collection-runs/manual-upload` in
  `crates/igy6-gateway/`.
- Use `crates/igy6-artifacts` from the gateway for bounded artifact writes.
- Add request validation, source/permission/approval checks, artifact writes,
  collection run insertion, raw artifact insertion, queued normalization work
  item insertion, response construction, and audit insertion for manual upload.
- Add route-level and validation tests for the migrated route.
- Update `configs/rust-cutover-manifest.json` route parity fields honestly.
- Update `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md` counts and route
  matrix entries.
- Add snippet-vault Rust equivalent JSONL records for reusable manual
  upload/artifact/audit behavior.
- Update this DIFF document with verification results and lock it after all
  required checks pass.

## Prohibited Scope

- No `POST /agent/actions/` migration.
- No `POST /agent/actions/{action_name}/execute` migration.
- No `/collection-runs/manual-upload/ingest` migration.
- No work-item dispatch route migration.
- No synchronous worker execution, ingestion, vector upsert, graph mutation, or
  agent execution.
- No FastAPI removal or archival.
- No Docker Compose rewiring.
- No database migrations.
- No `.env` reads or writes.
- No runtime/private data commits.
- No artifact writes outside the configured safe data root.
- No arbitrary shell execution.
- No approval bypass.
- No unrelated refactor, broad cleanup, renames, or redesign.

## Required Tags

Use `DIFF-117` in the commit message and final change summary.

## Verification

- `git status --short` passed with DIFF-117 scoped files before commit.
- `git diff --check` passed.
- `python3 scripts/rust-route-parity.py --check` passed:
  `fastapi=91 rust_native=58 web_used=41 missing_from_rust=35
  web_requires_fallback=2`.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-gateway` passed, 45 gateway tests.
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

- `POST /collection-runs/manual-upload` is Rust-native and no longer proxied to
  FastAPI.
- Manual upload validates required request fields, base64 content, UTF-8 text
  content, MIME type, source, source permission, and approval requirements.
- Artifact bytes are stored content-addressed under the configured safe artifact
  root and path traversal is prevented.
- Collection run, raw artifact, queued normalization work item, and audit
  metadata are written consistently with Python behavior.
- No synchronous downstream dispatch, ingestion, vector upsert, graph mutation,
  or agent execution is added.
- Route parity count is reduced from 3 web-used fallback routes to 2.
- Manifest and route audit remain honest that FastAPI fallback is still
  required for agent routes.
- Required verification passes.
- This DIFF is locked only after verification passes.

## Out Of Scope Follow-Up

- `POST /agent/actions/`
- `POST /agent/actions/{action_name}/execute`
- `POST /collection-runs/manual-upload/ingest`
- `POST /work-items/{work_item_id}/dispatch`
