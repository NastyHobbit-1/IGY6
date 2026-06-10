# IGY6 - Local Evidence & Intelligence Workspace

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white) ![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white) ![Next.js](https://img.shields.io/badge/Next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white)

**IGY6 ("I Got Your Six")** is a private, local-first evidence collection, analysis, and decision-support platform. Built for thorough data ingestion, provenance tracking, and evidence-grounded reasoning — all running entirely on your hardware with zero external data exfiltration.

## Runtime Architecture

IGY6 runs as a containerized stack:

- **Rust gateway / API** (`igy6-gateway`): primary API surface
- **Rust worker daemon** (`igy6-worker`): background processing
- **Next.js web UI**: tabbed dashboard (Home, Add Data, Work, Results, Settings, Advanced)
- **PostgreSQL**: metadata, audit, evidence records
- **Qdrant**: vector memory / similarity search
- **Neo4j**: graph relationships and lineage
- **MLflow + Phoenix**: observability

Data flows through artifacts → normalized documents → chunks → vectors/graph → evidence answers with full provenance.

## Prerequisites

- Git
- Docker Desktop (Windows/macOS) or Docker Engine + Compose (Linux)
- Rust toolchain (cargo)
- Node.js + npm (for web UI development and build checks)

## Quick Start (Runtime)

1. Clone the repository and enter the directory:

   ```bash
   git clone https://github.com/NastyHobbit-1/IGY6.git
   cd IGY6
   ```

2. Create your local runtime configuration from the template:

   ```bash
   # Bash / macOS / Linux
   cp .env.example .env

   # PowerShell (Windows)
   Copy-Item .env.example .env
   ```

   Edit `.env` and **set `IGY6_DATA_ROOT`** to an absolute path outside this repository (example: `C:/Users/you/IGY6_Data` or `/home/you/IGY6_Data`). This is where all runtime data (databases, artifacts, vectors, etc.) will live.

   Review other values (ports, local service credentials) as needed for your machine. `APP_PORT` (default 18000) and `WEB_PORT` (default 13000) control the exposed endpoints.

3. Start the stack:

   ```bash
   # Full stack (recommended)
   docker compose -f infra/docker-compose.yml --env-file .env up -d

   # Or start the web service (brings in API + dependencies via Compose)
   docker compose -f infra/docker-compose.yml --env-file .env up -d web
   ```

   This builds the Rust gateway (api), Rust worker, and Next.js web images and starts all services.

4. Verify the runtime:

   - API live: `http://127.0.0.1:18000/health/live`
   - API ready: `http://127.0.0.1:18000/health/ready`
   - Web UI: `http://127.0.0.1:13000`

   (Use your configured `APP_PORT` / `WEB_PORT` if you changed them.)

## Running Tests and Checks

```bash
# Rust workspace
cargo test --workspace

# Web frontend (from repository root)
npm --prefix apps/web install
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
```

## Runtime Data and Configuration

- All persistent runtime data is stored under the directory you set in `IGY6_DATA_ROOT` (PostgreSQL data, Qdrant storage, Neo4j data/logs, MLflow artifacts, Phoenix data, collected artifacts, etc.). Keep this path outside the source tree.
- `.env` holds your local runtime configuration (ports, data root, service credentials, policy defaults such as `EXTERNAL_MODEL_POLICY_DEFAULT=blocked`, `APPROVAL_REQUIRED_DEFAULT=true`). It is ignored by Git and **must not be committed**.
- `.env.example` is the non-secret, reusable template. Always start from it when setting up a new environment.

---

**IGY6: I've Got Your Six.** Private. Local. Evidence-driven.