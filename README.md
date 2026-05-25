# IGY6

IGY6 is a private, local-first adaptive intelligence workspace. It registers
authorized sources, stores raw artifacts, normalizes them into evidence,
searches vector and graph memory, tracks work and audit events, and exposes a
review UI for evidence-backed questions, approvals, outcomes, reports, and
controlled self-improvement work.

IGY6 is not a generic chatbot, hosted RAG demo, external-model workflow, or
autonomous system-changing agent. The default posture is local, evidence-first,
read-only where possible, and approval-gated before sensitive or
system-changing actions.

## Current Runtime Status

The active application API and worker runtime is Rust-only:

- `api` is the Rust gateway built from `crates/igy6-gateway/Dockerfile`.
- `worker` is the Rust worker daemon built from
  `crates/igy6-worker/Dockerfile`.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.
- Route parity reports zero FastAPI routes missing from Rust and zero web-used
  routes requiring fallback.

The Rust-only claim applies to the application API and worker runtime. These
supporting components intentionally remain non-Rust:

- Next.js web
- PostgreSQL
- Redis
- Qdrant
- Neo4j
- MLflow
- Phoenix

Archived legacy Python source is history and rollback material only:

- FastAPI API archive: `archive/legacy-python/services-api`
- Python/Celery worker archive: `archive/legacy-python/services-worker`

Do not treat archive paths as active services. Base Docker Compose must not
define `legacy-api`, a Python/Celery worker, Celery beat, or legacy worker
source-tree runtime references.

## Runtime Data Rule

Runtime/private data does not belong in the repository. Keep local runtime data
under `IGY6_DATA_ROOT`.

Repository contents are source code, docs, configs, scripts, tests, and
archive/history. Do not commit `.env`, storage roots, Docker volume data,
private exports, artifacts, credentials, tokens, cookies, or collected personal
data.

## Quickstart

Create `.env` from `.env.example` before normal local runs.

Start the stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

Stop the stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env down
```

Show services:

```bash
docker compose -f infra/docker-compose.yml --env-file .env ps
```

Follow logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200
```

Do not use `down -v` as a normal stop command. It can delete Docker volume
data.

## Local URLs

| Service | URL |
| --- | --- |
| Web UI | `http://127.0.0.1:3000` |
| Rust API gateway | `http://127.0.0.1:8000` |
| API readiness | `http://127.0.0.1:8000/health/ready` |

## Validation Commands

Post-cutover runtime audit:

```bash
python3 scripts/post-cutover-runtime-audit.py
```

Post-cutover smoke suite:

```bash
scripts/post-cutover-smoke.sh --check
```

Fresh-clone startup readiness:

```bash
scripts/fresh-clone-startup-check.sh --check
```

Startup/shutdown/restart command-shape validation:

```bash
scripts/runtime-lifecycle-check.sh --check
```

Rust cutover guard:

```bash
scripts/rust-cutover.sh --check
```

Web build:

```bash
npm --prefix apps/web run build
```

Useful supporting checks:

```bash
git status --short
git diff --check
python3 -m json.tool configs/rust-cutover-manifest.json
python3 -m json.tool configs/legacy-fastapi-route-classification.json
python3 scripts/rust-route-parity.py --check
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

For Rust code changes, also run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

## What The Validation Scripts Do

`scripts/post-cutover-smoke.sh --check` validates static runtime ownership,
Compose config, route parity, the cutover audit, and Rust worker help/check. It
does not start services or touch runtime data. If a local Rust API stack is
already running, it also probes live health endpoints unless those probes are
allowed to skip.

`scripts/fresh-clone-startup-check.sh --check` validates required tools,
tracked files, `.env.example` coverage for Compose, Rust-only runtime posture,
Compose config, route parity, Rust worker help/check, and the post-cutover
smoke path. It does not create `.env`, create data-root folders, start
services, install dependencies, pull images, or process queues.

`scripts/runtime-lifecycle-check.sh --check` validates Compose config, expected
service names, Rust API/worker ownership, absence of `legacy-api` and `beat`,
documented start/shutdown/restart command shapes, non-volume-removing shutdown
posture, rollback posture, the post-cutover audit, and the post-cutover smoke
suite. It does not start, stop, restart, or mutate Compose services.

## Active Services

Base Docker Compose defines the active local stack:

| Service | Role |
| --- | --- |
| `web` | Next.js UI |
| `api` | Rust gateway API |
| `worker` | Rust worker daemon for supported work-item processing |
| `postgres` | Relational state and audit store |
| `redis` | Queue/supporting runtime service |
| `qdrant` | Vector memory |
| `neo4j` | Graph memory |
| `mlflow` | Experiment tracking support |
| `phoenix` | Trace/evaluation support |

The Rust worker daemon covers the post-cutover worker runtime for
`collection_normalization`, `document_chunking`, and `chunk_vector_upsert`.
Graph sync, reports, ingestion workflows, and product flows should still be
verified by their own DIFF-scoped checks before being considered complete
end-to-end product workflows.

## Using The Product

The web UI is organized around these workflows:

- Home: runtime status, service readiness, recent data, recent work, audit, and
  next action.
- Assistant: evidence questions, retrieval preview, safe actions, approvals,
  and action results.
- Data & Knowledge: sources, uploads, collection runs, raw artifacts,
  documents, chunks, evidence, memory, analysis, and search.
- Work & Processing: queue status, work item detail, dispatch status, worker
  readiness, and processing pipeline.
- Reports: report list, detail, render controls, and status.
- Safety & Audit: approvals, audit log, safety rules, local-first state, and
  external-model policy.
- Settings: redacted environment status, runtime status, storage paths, and
  diagnostics.

Manual upload currently works best with UTF-8 text. Binary PDF/image/audio
parsing is not claimed unless a later DIFF adds it.

## Optional Local LLM

IGY6 does not call an external model by default. Optional local Ollama support
exists behind local-only configuration and evidence-required checks. With
`LLM_PROVIDER=none`, deterministic evidence fallback is active. If a local
provider is enabled, answers must still cite retrieved evidence or report
insufficient evidence.

See `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`.

## Docs Map

Start here:

- `AGENTS.md`: repository rules and DIFF-governed workflow.
- `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`: current post-cutover
  completion plan.
- `docs/runtime/PROCESSING_STATUS.md`: worker processing status and diagnostics.
- `docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md`: guided manual upload smoke path.
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`: route and runtime cutover
  audit history.
- `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`: rollback posture.
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`: final FastAPI
  route classification result.
- `configs/rust-cutover-manifest.json`: machine-readable cutover manifest.
- `configs/legacy-fastapi-route-classification.json`: machine-readable legacy
  route classification.
- `docs/diffs/`: locked DIFF history and active DIFF records.

Historical migration docs and locked DIFF records may describe earlier
FastAPI/Python/Celery states. Treat those as chronology unless a current-status
section says otherwise.

## Troubleshooting

- Web UI unavailable: run `docker compose -f infra/docker-compose.yml --env-file .env ps`
  and check the `web` logs.
- API unavailable: open `http://127.0.0.1:8000/health/ready` and check `api`
  logs.
- Empty `ps` output: the stack is probably not running for the selected Compose
  project/env file.
- No evidence after upload: check Work & Processing, worker logs, and
  `docs/runtime/PROCESSING_STATUS.md`.
- Qdrant/vector errors: run the post-cutover smoke suite and check `qdrant`
  logs.
- Settings save blocked: run verify first and read the redacted validation
  warnings.
- Phoenix `GET / 200 OK` log lines are normal local health/readiness probes.

Useful log commands:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
```

## Rollback Posture

Rollback is intentional and DIFF-scoped. To restore a legacy Python API or
worker topology for diagnosis, use git history or the archives under
`archive/legacy-python/`, restore the needed Compose service definitions,
validate Compose, and restart intentionally. Do not move runtime/private data
into the repository during rollback.

```bash
docker compose -f infra/docker-compose.yml --env-file .env config
```

See `docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`.

## Development Rules

- Work one DIFF at a time.
- Do not edit locked DIFFs.
- Do not start the next DIFF without explicit instruction.
- Keep changes scoped to the active DIFF.
- Do not mutate `.env` unless the active DIFF explicitly allows it.
- Do not touch runtime/private data unless explicitly scoped.
- Do not remove archive/history files unless explicitly scoped.
- Do not claim non-Rust infrastructure has been rewritten in Rust.
- Preserve approval gates, auditability, evidence lineage, and read-only
  defaults.

Before editing, read `AGENTS.md`, inspect `git status --short`, inspect the
current diff, and read the relevant active DIFF and agent prompt.
