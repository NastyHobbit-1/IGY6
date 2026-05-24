# IGY6 Full Project Completion Plan

## Status

IGY6 has completed the Rust-only application API and worker runtime cutover.

Current runtime posture:

- Rust gateway API is active.
- Rust worker daemon is active.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.
- Runtime/private data remains outside the repo under `IGY6_DATA_ROOT`.
- Remaining non-Rust components are expected supporting/product components:
  - Next.js web
  - PostgreSQL
  - Redis
  - Qdrant
  - Neo4j
  - MLflow
  - Phoenix

This plan covers post-cutover project completion, not Rust migration.

## Build Rule From This Point Forward

One DIFF must produce one clear product outcome.

Every DIFF must:

- keep the repo runnable;
- keep exactly one active DIFF;
- avoid editing locked DIFFs;
- avoid touching runtime/private data unless explicitly scoped;
- avoid mixing unrelated backend, UI, docs, and cleanup work;
- end with verification results recorded in the DIFF document.

## Completion Tracks

### Track 1 — Runtime Reliability

Purpose: prove the Rust-only runtime is stable enough for normal operation.

Planned DIFFs:

- Runtime smoke suite
- Fresh clone startup validation
- Startup/shutdown/restart validation
- Worker retry/failure/recovery hardening
- Runtime logging and diagnostics
- Backup/restore and data-root validation

Done when:

- `docker compose up --build` works from clean checkout.
- Rust API health checks pass.
- Rust worker starts and exits/restarts cleanly.
- Route parity remains complete.
- Runtime data stays under `IGY6_DATA_ROOT`.
- Failure modes are visible and recoverable.

### Track 2 — End-to-End Product Workflows

Purpose: prove the application works as a product, not just as services.

Planned DIFFs:

- Source/artifact ingestion workflow
- Collection run workflow
- Normalization/chunk/vector worker pipeline
- Evidence/retrieval workflow
- Report generation workflow
- Full user journey smoke test

Done when:

- User can add data.
- Artifact records are created.
- Worker normalizes, chunks, and vectorizes.
- Evidence can be searched/retrieved.
- Reports can be generated or rendered.
- Errors are clear and recoverable.

### Track 3 — UI Completion

Purpose: make the current UI accurate, usable, and aligned with the Rust backend.

Planned DIFFs:

- UI inventory and broken-flow audit
- Runtime status/dashboard cleanup
- Source/artifact upload flow polish
- Work item and processing monitor
- Evidence browser/search UX
- Report workflow UX
- Settings/runtime capability page

Done when:

- No dead primary buttons.
- No misleading Python/Celery/FastAPI status text.
- No hidden backend assumptions.
- Every visible action maps to a verified Rust API path.
- UI build passes.

### Track 4 — Operations and Packaging

Purpose: make IGY6 easy to run, recover, and diagnose.

Planned DIFFs:

- `.env.example` and config validation hardening
- Run/stop/restart command normalization
- Logs and diagnostics bundle
- Rollback/recovery dry run
- Release checklist
- Fresh install operator workflow

Done when:

- Fresh clone can start.
- Required folders are created safely.
- Config errors are readable.
- Operator can collect diagnostics without guessing.
- Rollback/recovery docs match actual runtime posture.

### Track 5 — Documentation Lock

Purpose: make docs match the actual post-cutover product.

Planned DIFFs:

- User quickstart
- Operator guide
- Runtime architecture guide
- Troubleshooting guide
- Final project completion audit

Done when:

- README is current.
- Runtime docs are current.
- Rollback docs are current.
- Architecture docs are current.
- No stale migration-era claims remain.

## Recommended Critical Path

The shortest safe path to “done enough to use confidently” is:

1. Runtime smoke suite.
2. Fresh clone/startup validation.
3. End-to-end ingestion pipeline.
4. End-to-end worker pipeline.
5. Evidence/retrieval workflow.
6. Report workflow.
7. UI broken-flow audit.
8. Fix top UI blockers.
9. Runtime recovery/logging pass.
10. Quickstart/operator docs.
11. Release checklist.
12. Final project completion audit.

## Standard DIFF Modes

Each future DIFF must declare one mode:

- `Mode: audit only`
- `Mode: implementation`
- `Mode: verification only`
- `Mode: documentation only`
- `Mode: UI only`
- `Mode: runtime only`

Do not mix modes unless explicitly allowed in the active DIFF.

## Standard Preflight

Every DIFF must begin with:

```bash
git status --short
```

Then inspect:

- active DIFF state;
- locked DIFF status;
- intended file list;
- relevant runtime/docs/config files.

No code changes should start until the file scope is known.

## Standard Verification

For runtime/backend DIFFs:

```bash
git status --short
git diff --check
python3 -m json.tool configs/rust-cutover-manifest.json
python3 -m json.tool configs/legacy-fastapi-route-classification.json
python3 scripts/post-cutover-runtime-audit.py
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
python3 scripts/rust-route-parity.py --check
scripts/rust-cutover.sh --check
docker compose -f infra/docker-compose.yml --env-file .env.example config
npm --prefix apps/web run build
```

For docs-only DIFFs:

```bash
git status --short
git diff --check
python3 -m json.tool configs/rust-cutover-manifest.json
python3 scripts/post-cutover-runtime-audit.py
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

For UI DIFFs:

```bash
git status --short
git diff --check
npm --prefix apps/web run build
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

## Runtime Data Rule

Runtime/private data must not be committed.

Repository contents are:

- source code;
- docs;
- configs;
- tests;
- scripts;
- archive/history.

Runtime/private data belongs under:

```text
IGY6_DATA_ROOT
```

## Project Completion Definition

The whole project is complete when:

- Rust API runtime is active and verified.
- Rust worker runtime is active and verified.
- No active Python/FastAPI/Celery runtime path remains.
- Fresh clone starts successfully.
- Docker Compose config validates.
- Runtime smoke suite passes.
- Ingestion works.
- Worker pipeline works.
- Retrieval/evidence workflow works.
- Report workflow works.
- UI has no known broken primary flows.
- Docs match actual runtime behavior.
- Rollback/recovery posture is documented.
- Final completion audit is locked.

## Estimated Remaining Work

Minimum completion path:

```text
10–12 DIFFs
```

Realistic completion path:

```text
18–24 DIFFs
```

Polished/shareable release path:

```text
25–35 DIFFs
```

## Next DIFF After This Plan

Recommended next DIFF:

```text
DIFF-168 post-cutover runtime smoke suite
```
