# Operations

## Prerequisites
Docker, Rust toolchain, Node.js/npm.

## Configuration
Copy `.env.example` to `.env` or `.env.test` and configure `IGY6_DATA_ROOT`.

## Docker Compose
Build and start with:
docker compose -f infra/docker-compose.yml --env-file .env.test up -d

## Verification Commands
Use the `scripts/verify-runtime.ps1` script or run manually:
- cargo test --workspace
- npm --prefix apps/web install
- npm --prefix apps/web audit
- npm --prefix apps/web run typecheck
- npm --prefix apps/web run build
- docker compose ... (as specified)

## Stop
 docker compose -f infra/docker-compose.yml --env-file .env.test down

Runtime data lives under IGY6_DATA_ROOT. Do not commit it.