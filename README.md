# IGY6

IGY6 ("I Got Your Six") is a local evidence workspace. You add source material, the Rust services turn it into artifacts, documents, chunks, and evidence records, and the web UI lets you ask questions that stay tied to those records.

The repository is one Cargo workspace plus a Next.js app:

- `apps/web` — Next.js UI (Chat, Data, Work, Settings, More)
- `crates/` — Rust workspace (gateway, CLI, worker, retrieval, vector memory, policy, and the rest)
- `infra/docker-compose.yml` — Postgres, Qdrant, Neo4j, MLflow, Phoenix, gateway, worker, and web
- `configs/` — cutover manifest, local LLM routing, guardrail notes
- `scripts/` — start, stop, health wrappers, and offline checks
- `.env.example` — runtime configuration template (copy to `.env`; do not commit `.env`)

`rust-toolchain.toml` pins Rust **1.99.1**. With rustup, that file selects the compiler.

This tree was checked with `rustc 1.99.1`, `cargo 1.99.1`, Node `v22.14.0`, and npm `10.9.7`. Docker was not installed here, so the compose stack was not started.

## Install

1. Install Rust (rustup will follow `rust-toolchain.toml`) and Node.js 22.
2. From the repository root, build the workspace:

```bash
cargo build --workspace
```

3. Install the web app from the lockfile:

```bash
npm ci --prefix apps/web
```

4. For a full local stack, Docker is required. `./install.sh` builds the `igy6` CLI and expects Docker. That installer was not run in this check. The CLI help, from `cargo run -q -p igy6-cli -- --help`, lists `igy6` and `igy6 start` as the stack entry points.

Create a private `.env` from `.env.example` before starting the stack, and set `IGY6_DATA_ROOT` to a directory outside the repo. Leave secrets out of git. This check left `.env` absent. `cargo run -q -p igy6-cli -- config check` treats a missing `.env` as allowed and validates `.env.example`.

The single-user UI default password is `ThatDog123` (see Settings → User & Security). Change it after you unlock the app.

## Run

Developer checks that do not need Docker:

```bash
cargo test --workspace
npm --prefix apps/web run typecheck
npm --prefix apps/web test
npm --prefix apps/web run build
npm --prefix apps/web run test:ui-runtime-smoke
cargo run -q -p igy6-cli -- health
cargo run -q -p igy6-cli -- config check
bash scripts/normal-user-product-smoke.sh --check
bash scripts/normal-user-product-smoke.sh --release-readiness-check
bash scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-safe-bundle-v1.json
```

`npm --prefix apps/web run test:ui-runtime-smoke` starts `npm run start` in `apps/web` when nothing is already listening, checks the HTML contract, and stops the server. On this machine Playwright's Chromium binary was not installed, so the script used its node fetch fallback and still passed.

There is no separate ESLint script. `npm run build` runs Next.js type checking, and `npm run typecheck` runs `tsc --noEmit`.

The product entry point, once Docker is available, is the `igy6` CLI (`igy6` or `igy6 start`). `scripts/run.sh`, `scripts/stop.sh`, and `scripts/status.sh` are the fixed wrappers that CLI commands call. They were syntax-checked with `bash -n` and were not executed, because they need Docker.

## Use

Open the web UI (the runtime smoke served it at `http://127.0.0.1:3000`). The main tabs are **Chat**, **Data**, **Work**, **Settings**, and **More**. **Simple mode** (also labeled "Full workspace" when it is on) hides the extra panels and leaves the short chat layout.

Without the API (`API_BASE_URL`, default `http://api:8000`), pages still render and show the fetch error on each panel. Live collection, search, and saved evidence need the gateway and its databases.

## Capabilities

### Chat

Ask a question over material that is already in the workspace. The chat hub (`UnifiedChatHub`) previews retrieval and can save an evidence answer record. Use it when you want a cited answer rather than a raw document dump.

### Simple mode and the full workspace

Simple mode is the shorter layout. The full workspace shows Data, Work, Settings, and More. Use Simple mode when you only want to talk to the workspace. Use the full workspace when you are loading data or checking jobs.

### Add data

The Data tab is where new material enters: guided text upload, conversation history, observations, local project notes, and media files (PDF, image, audio, video). Use it when the chat does not yet have the source you need. Media text extraction (`igy6-media-extract`) calls `pdftotext`, `tesseract`, or `ffmpeg` plus a local whisper binary when those tools exist. Image-only PDFs can come back with an empty text layer.

### Work and approvals

The Work tab shows queued and running items, approvals, and processing status. Use it when something you added should be normalized, chunked, or reviewed before you trust an answer.

### Settings, password, and local models

Settings holds User & Security (password and optional TOTP), environment checks, and troubleshooting logs. Local answers can use Ollama when `LLM_PROVIDER=ollama` and Ollama is reachable. The template default is `LLM_PROVIDER=none`, which keeps model calls off. Use Settings when you are changing the password, pointing at a local model, or reading startup logs.

### Vector search and graph lineage

Qdrant stores chunk vectors. Neo4j is the graph store declared in `infra/docker-compose.yml`. The UI shows collection and lineage status when those services answer the gateway. Use them when you want similarity search or a trail between sources and answers. They were not started in this check.

### Backup export and restore dry-run

`scripts/backup-export-mvp.sh` reads the API and writes a sanitized metadata bundle. It was not run here because the API was down. `scripts/restore-dry-run-mvp.sh` only validates a bundle. It does not write to Postgres or the artifact store. Use the dry-run before any real restore. The safe fixture command above passed. The unsafe fixture with `--strict-safety` exits 1 on purpose when it finds secret-like fields.

### Diagnostics and operator checks

`scripts/diagnostics-bundle-mvp.sh` builds a redacted diagnostics bundle and was not run (it calls the API). `scripts/normal-user-product-smoke.sh --check` and `--release-readiness-check` only read the repo. `cargo run -q -p igy6-cli -- health` checks that the compose file, scripts, and manifest are present. Use these before you look for a product bug in a live stack.

### Deep collection and the host bridge

The gateway can collect from allowed local paths, URLs, and a host bridge (`crates/igy6-host-bridge`, port 8765 in `.env.example`). Use that path only for sources you are allowed to read, with approval still on. It was not executed in this check.

## Layout

| Path | Role |
| --- | --- |
| `Cargo.toml` | Workspace of 19 crates |
| `apps/web` | Next.js UI and its API proxies |
| `crates/igy6-gateway` | HTTP API |
| `crates/igy6-cli` | `igy6` command |
| `crates/igy6-worker` | Background processing |
| `crates/igy6-llm` | Local model routing (`none` or `ollama`) |
| `crates/igy6-retrieval-preview` | Retrieval preview |
| `crates/igy6-vector-memory` | Qdrant client |
| `crates/igy6-policy` | Approval and external-model policy |
| `infra/` | Compose file, SQL bootstrap, Qdrant and Neo4j notes |
| `configs/` | Manifest and routing files |
| `docs/runtime/` | Operator checklists and the capability truth table |
| `packages/schemas` | Placeholder note. Live contracts are the Rust gateway and the TypeScript types in `apps/web` |

`archive/legacy-python/` is historical and is not the running server.

## Verification from this cleanup

Passed:

- `cargo build --workspace`
- `cargo test --workspace` — 280 tests passed, 0 failed
- `npm ci --prefix apps/web` — `npm audit` reported 0 vulnerabilities
- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web test` (`scripts/ui-smoke.mjs`, 53 component files)
- `npm --prefix apps/web run build` — Next.js 15.5.28, 38 pages
- `npm --prefix apps/web run test:ui-runtime-smoke` — pass on the HTML fallback; Playwright Chromium was not installed; the server process was gone afterward
- `cargo run -q -p igy6-cli -- --help`
- `cargo run -q -p igy6-cli -- health` — status ok; Docker not found
- `cargo run -q -p igy6-cli -- config check` — `.env.example` valid, 49 keys, values redacted
- `bash scripts/normal-user-product-smoke.sh --check`
- `bash scripts/normal-user-product-smoke.sh --release-readiness-check`
- `bash scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-safe-bundle-v1.json`
- `bash -n` on `install.sh` and `scripts/*.sh`

Not run, because Docker or a live API was not available:

- `./install.sh`, `igy6 start`, `igy6 stop`, `scripts/run.sh`
- `scripts/backup-export-mvp.sh` and `scripts/diagnostics-bundle-mvp.sh`
- Compose health URLs and a browser click-through of Chat → Data → Work

`scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-unsafe-bundle-v1.json --strict-safety` exits 1. That is the fail-closed result for a bundle that contains a synthetic token.
