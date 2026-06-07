# IGY6 (grok branch)

**Operating guide for the running program on the grok branch only.** This document (and all other docs in the tree) now focuses exclusively on setup, how to run and use the program, and its features. No build, compile, development, or cargo/npm instructions.

On this branch the program is a local-only evidence + aggressive collection workspace. It can deeply and thoroughly collect from any target it can reach (local files/directories, web URLs/pages, system state, WiFi, etc.), stores **everything only inside itself** (no content exfil), preserves full provenance, and provides an easy Image & Video Library for viewing collected media at full/original resolution directly from the source.

Default password: "ThatDog123". The program is password-protected. Optional TOTP authenticator support is **off by default** until you explicitly link it (works with any standard authenticator app).

## Setup (Running the Program)

1. Docker + docker compose recommended for the full stack (or use the supporting scripts for direct runs if your environment is prepared).
2. Have a `.env` (copy `.env.example` and set `IGY6_DATA_ROOT` to a writable host directory — this is where everything local lives: DBs, full-res media artifacts, evidence, graph, etc.).
3. Start it:
   ```bash
   scripts/run.sh
   ```
   - Starts the complete local stack (gateway, worker, web UI, Postgres, Neo4j, Qdrant, etc.).
   - The `igy6` CLI and scripts automatically pick **clear unused local ports** when 3000/8000 are busy (`WEB_PORT` / `APP_PORT` in `.env`) and print the exact `http://127.0.0.1:PORT` to open.
4. Open the printed URL in your browser.
5. Unlock with the current password ("ThatDog123" by default). If you have linked TOTP, also provide a current code from your authenticator app when performing protected actions.

## The Compiled `igy6` Executable (Recommended for Daily Use)

For the easiest experience, use the compiled `igy6` binary instead of calling `scripts/run.sh` directly:

- It starts the full stack detached.
- Waits for the UI to be ready.
- Automatically opens your browser to the local UI URL.
- Handles first-run .env bootstrap (grok defaults, password, data dir, telemetry off).
- Cross-platform (Linux, macOS, Windows with Docker Desktop).

### Easy Install

From the Grok6 root:

- Linux/macOS: `./install.sh`
- Windows (PowerShell): `.\install.ps1`

Then (restart shell if needed):

```bash
igy6
# or
igy6 start
```

Other commands: `igy6 stop`, `igy6 health`, `igy6 --help`, `igy6 run` (for foreground logs if desired).

The binary finds the repo root automatically (or respects `IGY6_REPO` env if installed globally).

Stop with `igy6 stop` or the scripts.

Stop with Ctrl+C or the stop/restart scripts in `scripts/`.

**Dynamic URL**: The start logic (web dynamic starter + support in operator scripts) detects port conflicts and switches to a free one, always telling you the usable local address. No manual port editing required.

## Operating the Program (UI & Daily Use)

The UI is tabbed and intentionally easy to understand. Key areas on this branch:

- **Home / overview** — stack readiness, recent collections, quick status.
- **Add Data / Collector** — the deep collection surface. Use the **Deep Thorough Scan** buttons/controls. Give it URLs, local paths, or "everything". It runs deep/thorough on the target(s):
  - Recursively follows and extracts from pages.
  - For every image and video it finds: fetches **directly from the original source at full/original resolution** (handles data-*, fullsrc, strips resize params, etc.).
  - Captures complete local files, system info, WiFi, etc.
  - Stores as first-class artifacts (with real mime/kind detection) + evidence + graph entries. All local only.
  - Protected calls require the current password (and totp_code if you enabled authenticator).
- **Media Library** (grok branch feature) — easy grid of every collected image and video. Click a card for the full-resolution viewer (real bytes via the content endpoint, native <img> or <video> at original res from the source). Refresh after scans. Simple, visual, no guessing.
- **Work / Results / Evidence** — processing state, evidence items, answers, reports, relationships.
- **Settings / User & Security** (the user section):
  - **Change password**: Enter current password + new password (≥4 chars). Submit. Persisted immediately. Default is "ThatDog123".
  - **Authenticator (TOTP)**: Off by default. Use the generate/link flow (provide current password). You receive a secret (base32 text) and otpauth URL. Enter the secret (or use the URL for QR) in **any standard authenticator program** (Google Authenticator, Authy, Microsoft Authenticator, etc.). Then submit a current 6-digit code from the app to enable. Once on, protected actions (deep collector, etc.) will also require a current TOTP code.
  - Check status button shows whether TOTP is enabled.
- **Advanced** — diagnostics and raw views (only when you need them).

**Typical flow**:
1. scripts/run.sh → note the clear UI URL.
2. Open URL → unlock with password (and TOTP code if enabled).
3. Collector area → run deep scan on desired targets (include password, and totp_code if 2FA is on, in direct calls).
4. Media Library → refresh → browse and view full-res images/videos pulled from their sources.
5. Results/Evidence/Graph for the complete extracted info and relationships.
6. User & Security anytime to change password or manage authenticator linking.

Stop/restart via the scripts.

Everything (full-res media artifacts, evidence, graph in Neo4j, audit, etc.) stays **only inside the local instance** under your data root.

## Features

- Deep & thorough collection on any reachable target with original/full-res images & videos fetched directly from source. Real deep PDF text extraction (plus rich metadata for images etc.) turns collected content into proper evidence/claims/graph instead of placeholders.
- Easy Image & Video Library with full-res modal viewer for all collected media.
- Password protection with easy changing in the User section.
- Optional TOTP authenticator (off by default, link with any authenticator app via secret/otpauth — standard TOTP).
- All data tied real: collector → artifacts (mime/kind/full bytes) → evidence → worker pipelines (normalization/chunking/vectors/evidence-answer) → graph (Neo4j sync) → library + results + audit.
- Dynamic clear local URLs (web UI and noted services auto-switch and report the usable address if a port is busy).
- Local-only by design; simple focused UI for collection, media viewing, evidence, and security.

Supporting scripts (backup, diagnostics, smoke checks, etc.) are optional helpers that print clear status — not required for normal operation.

This branch (grok) gives you a powerful, private, local full-access collector + media workspace. All documents (this README + docs/*.md, user-guide, security-policy, operations, ui guide, truth table notes, etc.) have been updated to reflect the program on this branch and contain only setup/operating/feature instructions.

Start with `scripts/run.sh`, unlock with the password, use the collector and Media Library. Change password or link an authenticator in User & Security whenever you want. Everything just works locally.

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
- MLflow and Phoenix as supporting observability/experiment infrastructure.

Archived legacy Python code remains in the repository only for history and rollback review. It is not the active runtime path on `grok`.

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
