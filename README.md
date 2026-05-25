# IGY6

IGY6 is a private, local-first evidence and decision-support workspace.

It is designed to help a user collect authorized information, turn that information into traceable evidence, search and reason over that evidence, review system activity, and produce decision-ready outputs without sending private runtime data to a hosted service by default.

IGY6 is not just a chatbot and not just a RAG demo. The goal is a local intelligence layer over approved information: sources, artifacts, documents, chunks, evidence items, claims, patterns, reports, approvals, audit events, and outcomes.

## What IGY6 Is For

IGY6 is built for questions like:

- What information has been collected?
- What evidence supports this answer?
- What changed recently?
- What work is still processing?
- What sources, artifacts, documents, and chunks exist?
- What answers can be generated from stored evidence?
- What reports or review items are available?
- What actions need approval?
- What is known, what is uncertain, and what is unsupported?

The core design principle is evidence-first operation. Answers, reports, and review surfaces should connect back to stored records whenever possible. Unsupported statements should be treated as assumptions, estimates, or insufficient-evidence results rather than hidden facts.

## Current Product Status

IGY6 currently runs as a Rust-only application API and worker runtime with a Next.js web interface and local supporting services.

Active runtime ownership:

- Rust API gateway: active.
- Rust worker daemon: active.
- Next.js web UI: active.
- Legacy Python/FastAPI API: archived, inactive.
- Legacy Python/Celery worker: archived, inactive.
- Celery beat: inactive.

Supporting infrastructure:

- PostgreSQL for relational state, evidence metadata, work items, approvals, reports, and audit records.
- Qdrant for vector memory.
- Neo4j for graph/relationship memory surfaces.
- Redis as supporting infrastructure.
- MLflow and Phoenix as supporting observability/experiment infrastructure.

Archived legacy Python code remains in the repository only for history and rollback review. It is not the active runtime path on `main`.

Rollback review material includes `archive/legacy-python/services-api` and
`archive/legacy-python/services-worker`. Restoring the prior Python/Celery
worker would require an explicit later rollback procedure and Docker Compose
validation; it is not part of the active runtime.

## What IGY6 Can Do Now

Current verified/product-facing capabilities include:

- Start, stop, restart, and inspect the local stack with simple scripts.
- Run a Rust API gateway and Rust worker daemon through Docker Compose.
- Use a tabbed normal-user web UI instead of a developer-heavy dashboard.
- Add source records and supported text-oriented data through the UI/API surfaces.
- Process supported text input through the worker pipeline.
- Normalize text into documents.
- Split documents into chunks.
- Create evidence-oriented records.
- Upsert chunk vectors into Qdrant.
- Track work items and processing state.
- Preview plain-language requests with an explicit category, request summary,
  clarification posture, approval posture, and work-item posture before taking
  action.
- Inspect runtime status, route parity, and post-cutover validation results.
- Use local LLM routing configuration where enabled, with evidence-oriented fallback behavior.
- Review approvals, audit events, reports, evidence records, and runtime diagnostics where records exist.

The current system is strongest for UTF-8 text-oriented workflows and repository-visible local development/runtime validation.

## Important Current Limits

IGY6 is still under active development.

Current limits:

- Manual upload is best for UTF-8 text.
- Binary PDF, image, audio, and video parsing are not claimed as complete unless a later scoped change adds and verifies them.
- Some source types may be planned, metadata-only, or partially wired.
- Empty UI states are real empty states, not demo data.
- Graph reasoning, forecasting, self-improvement experiments, and advanced reporting depend on the records and routes currently present.
- Sensitive or system-changing actions must remain explicit, auditable, and approval-aware.

The README should not imply that every planned intelligence feature is fully complete. The project goal is broader than the current implementation, and the documentation separates those two things.

## Architecture Overview

High-level runtime shape:

```text
User
  |
  v
Next.js web UI
  |
  v
Rust API gateway
  |
  +--> PostgreSQL control/evidence/audit store
  +--> Rust worker daemon
  +--> Qdrant vector memory
  +--> Neo4j graph memory
  +--> Redis / MLflow / Phoenix supporting services
```

Core Rust crates include:

- `crates/igy6-gateway/`: Rust HTTP gateway and route handling.
- `crates/igy6-worker/`: Rust worker runtime and queue processing logic.
- `crates/igy6-agent-api/`: typed local agent command-plane classification and capability logic.
- `crates/igy6-llm/`: local LLM provider and routing support.
- `crates/igy6-evidence-answer/`: evidence-grounded answer packet construction and fallback behavior.
- `crates/igy6-artifacts/`: content-addressed artifact handling.
- `crates/igy6-normalization/`: text normalization.
- `crates/igy6-chunking/`: deterministic chunking.
- `crates/igy6-vector-memory/`: vector generation and Qdrant request logic.
- `crates/igy6-write-api/`: write API planning and validation logic.
- `crates/igy6-work-queue-reports/`: work queue and report contract logic.

## Web Interface

The web UI is organized for normal use first, with technical detail moved out of the default path.

Tabs:

- **Home**: readiness, attention items, and next actions.
- **Add Data**: source and upload entry points.
- **Work**: processing status and background work.
- **Results**: evidence, answers, reports, and searchable output.
- **Settings**: safety, approvals, and local configuration posture.
- **Advanced**: diagnostics and lower-level troubleshooting tools.

See [`docs/ui/README.md`](docs/ui/README.md) for the tab-by-tab user guide and workflow examples.

## Main Workflow

A typical local workflow:

1. Start IGY6.
2. Open the web UI.
3. Confirm readiness on **Home**.
4. Add supported data in **Add Data**.
5. Watch processing in **Work**.
6. Review evidence and outputs in **Results**.
7. Use **Settings** for safety and approval posture.
8. Use **Advanced** only when diagnostics are needed.

## Quickstart

From the repository root:

```bash
cp .env.example .env
scripts/run.sh
```

Open the web UI:

```text
http://127.0.0.1:3000
```

Check status:

```bash
scripts/status.sh
```

Stop safely:

```bash
scripts/stop.sh
```

Restart:

```bash
scripts/restart.sh
```

The stop script uses normal Docker Compose shutdown and does not remove volumes. Do not use `docker compose down -v` unless you intentionally want to remove Docker volume data.

## Runtime Data Rule

Runtime and private data belongs outside the repository under `IGY6_DATA_ROOT`.

Do not commit:

- `.env`;
- storage directories;
- runtime artifacts;
- private exports;
- credentials;
- tokens;
- cookies;
- collected personal data;
- Docker volume data.

The repository should contain source code, tests, documentation, scripts, configuration templates, and historical archive material only.

## Useful Commands

Start, stop, restart, and inspect the stack:

```bash
scripts/run.sh
scripts/stop.sh
scripts/restart.sh
scripts/status.sh
```

The wrapper scripts use these Docker Compose lifecycle command shapes:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
docker compose -f infra/docker-compose.yml --env-file .env down
docker compose -f infra/docker-compose.yml --env-file .env config
```

Run non-destructive runtime validation:

```bash
scripts/post-cutover-smoke.sh --check
scripts/fresh-clone-startup-check.sh --check
scripts/runtime-lifecycle-check.sh --check
python3 scripts/post-cutover-runtime-audit.py
scripts/rust-cutover.sh --check
```

Build the web UI:

```bash
npm --prefix apps/web run build
```

Run Rust checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

## Local URLs

| Area | URL |
| --- | --- |
| Web UI | `http://127.0.0.1:3000` |
| Rust API gateway | `http://127.0.0.1:8000` |
| API readiness | `http://127.0.0.1:8000/health/ready` |

## Troubleshooting

Check services:

```bash
scripts/status.sh
```

Run the post-cutover smoke suite:

```bash
scripts/post-cutover-smoke.sh --check
```

Validate startup/shutdown/restart command posture:

```bash
scripts/runtime-lifecycle-check.sh --check
```

View logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker
```

## Documentation Map

Product and operator docs:

- [`docs/ui/README.md`](docs/ui/README.md): web UI guide and workflow examples.
- [`docs/runtime/PROCESSING_STATUS.md`](docs/runtime/PROCESSING_STATUS.md): current processing/runtime posture.
- [`docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`](docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md): full project completion plan.
- [`docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`](docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md): Rust cutover and route audit history.
- [`docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`](docs/rust-migration/RUST_CUTOVER_ROLLBACK.md): rollback posture.

Historical DIFF records may mention earlier Python/FastAPI/Celery states, build instructions, or migration steps. Treat locked DIFFs as chronology, not as the current runtime description.

## Branch and Repository Policy

The public `main` branch is product/runtime-facing. It should not contain private build prompts, local Codex instructions, or personal coordination notes.

Private build-agent instructions belong only on a local development branch, not on `main`.

## Development Notes

Use scoped changes. Keep the repository runnable after each change. Do not edit locked historical DIFF records. Do not commit runtime/private data.

For product work, prefer small changes with explicit verification:

```bash
git diff --check
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
npm --prefix apps/web run build
```
