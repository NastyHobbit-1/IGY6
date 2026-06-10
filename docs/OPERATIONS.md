# Operations

This document covers normal local runtime operation for IGY6.

## Prerequisites

Install these before running the stack:

- Git
- Docker with Compose
- Rust toolchain with `cargo`
- Node.js and npm
- PowerShell 7 on Windows

## Configuration

Create a local runtime environment file from the template:

```powershell
Copy-Item .env.example .env
```

For verification workflows, `.env.test` may also be used.

Set `IGY6_DATA_ROOT` to an absolute path outside the repository. Runtime data should not live inside the Git working tree.

## Build and start

```powershell
docker compose -f infra/docker-compose.yml --env-file .env.test build
docker compose -f infra/docker-compose.yml --env-file .env.test up -d
docker compose -f infra/docker-compose.yml --env-file .env.test ps
```

Open the web UI:

```text
http://127.0.0.1:13000
```

Verify the API:

```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/live"
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/ready"
```

## Full runtime verification

Run:

```powershell
pwsh -NoProfile -File scripts/verify-runtime.ps1
```

The script runs Rust tests, web install/audit/typecheck/build, Docker web build/start, container status, API health checks, and a web HTTP status check.

## Stop

```powershell
docker compose -f infra/docker-compose.yml --env-file .env.test down
```

## Do not commit

Do not commit local runtime or generated material:

- `.env`
- `.env.test`
- runtime data under `IGY6_DATA_ROOT`
- logs
- database files
- exports
- generated artifacts
- caches
- local build output
