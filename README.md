# IGY6

IGY6 is a private, local-first evidence workspace for collecting information, processing it into searchable evidence, and using that evidence to answer questions, review activity, and produce reports.

It is built for personal/local operation. The application keeps runtime data outside the repository, preserves an audit trail, and favors evidence-backed answers over unsupported claims.

## What IGY6 Does

IGY6 helps you:

- add authorized text-based data and source records;
- track background processing from raw input to documents, chunks, evidence, and memory;
- search and review local evidence;
- ask questions that are grounded in stored evidence;
- inspect approvals, audit events, processing status, and reports;
- keep local runtime data separate from source code.

IGY6 is not a hosted chatbot, a generic RAG demo, or an unrestricted automation agent. Sensitive or system-changing actions are expected to stay explicit, auditable, and approval-aware.

## Current Runtime

The active application runtime is Rust-based for the API and worker path:

- `api` runs the Rust gateway.
- `worker` runs the Rust worker daemon.
- Legacy FastAPI and Python/Celery worker code is archived only for history and rollback.
- Celery beat is inactive.

Supporting services intentionally remain as supporting infrastructure:

- Next.js web UI
- PostgreSQL
- Redis
- Qdrant
- Neo4j
- MLflow
- Phoenix

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

Check service status:

```bash
scripts/status.sh
```

Stop IGY6 safely:

```bash
scripts/stop.sh
```

Restart IGY6:

```bash
scripts/restart.sh
```

The stop script uses a normal Docker Compose shutdown and does not remove volumes. Do not use `docker compose down -v` unless you intentionally want to remove Docker volume data.

## Main User Workflow

1. Start IGY6 with `scripts/run.sh`.
2. Open the web UI at `http://127.0.0.1:3000`.
3. Use **Home** to confirm the system is ready.
4. Use **Add Data** to add supported information.
5. Use **Work** to check processing.
6. Use **Results** to review evidence, answers, and reports.
7. Use **Settings** for safety/configuration review.
8. Use **Advanced** only for diagnostics and technical troubleshooting.

For a full tab-by-tab guide, see [`docs/ui/README.md`](docs/ui/README.md).

## Web UI Tabs

- **Home**: readiness, attention items, and next actions.
- **Add Data**: user-friendly entry point for sources and text-oriented uploads.
- **Work**: processing status and background work.
- **Results**: evidence, answers, reports, and searchable output.
- **Settings**: safety, approvals, local configuration, and policy posture.
- **Advanced**: diagnostics and lower-level tools for troubleshooting.

The main interface is intended for normal use. Technical details belong in **Advanced**, not on the default screen.

## Data and Privacy Rule

Runtime/private data belongs under `IGY6_DATA_ROOT`, not in the repository.

Do not commit:

- `.env`;
- storage folders;
- artifacts;
- exported private data;
- credentials, tokens, cookies, or secrets;
- collected personal data.

The repository should contain source code, documentation, configuration templates, tests, scripts, and archive/history only.

## What Is Supported Now

Current supported/product-facing posture:

- Rust API and worker runtime.
- Docker Compose local stack.
- Normal-user tabbed web interface.
- Runtime status and validation scripts.
- Text-oriented manual upload path.
- Worker processing for normalization, chunking, and vector upsert.
- Evidence-oriented review and answer flows where records exist.
- Archived legacy Python code for history/rollback only.

Current limitations:

- Manual upload is best for UTF-8 text.
- Binary PDF, image, audio, and video parsing are not claimed unless a later change adds and verifies them.
- Some source/collector workflows may be planned, metadata-only, or dependent on current API-backed records.
- Empty UI states are real empty states, not demo data.

## Useful Commands

Start, stop, restart, and inspect the stack:

```bash
scripts/run.sh
scripts/stop.sh
scripts/restart.sh
scripts/status.sh
```

Run non-destructive validation:

```bash
scripts/post-cutover-smoke.sh --check
scripts/fresh-clone-startup-check.sh --check
scripts/runtime-lifecycle-check.sh --check
python3 scripts/post-cutover-runtime-audit.py
```

Build the web UI:

```bash
npm --prefix apps/web run build
```

Run Rust checks when changing Rust code:

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

If the UI does not open:

```bash
scripts/status.sh
```

If the system looks unhealthy:

```bash
scripts/post-cutover-smoke.sh --check
```

If start/stop/restart behavior is unclear:

```bash
scripts/runtime-lifecycle-check.sh --check
```

View logs directly through Docker Compose when needed:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker
```

## Documentation

Primary user/operator docs:

- [`docs/ui/README.md`](docs/ui/README.md): tab-by-tab web UI guide.
- [`docs/runtime/PROCESSING_STATUS.md`](docs/runtime/PROCESSING_STATUS.md): processing and worker status.
- [`docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`](docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md): remaining product-completion plan.
- [`docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`](docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md): runtime cutover audit history.
- [`docs/rust-migration/RUST_CUTOVER_ROLLBACK.md`](docs/rust-migration/RUST_CUTOVER_ROLLBACK.md): rollback posture.

Historical migration files and locked DIFF records may describe older Python/FastAPI/Celery states. Treat those as chronology unless a current-status section says otherwise.

## Development and Governance

The project uses DIFF-governed changes. Work one scoped DIFF at a time, keep the repo runnable, and do not edit locked DIFF records.

Contributor and agent instructions are intentionally kept out of the public-facing README. See `AGENTS.md` only if you are contributing to the repository process.
