# Rust Cutover Rollback

DIFF-103 keeps the Rust gateway as the primary `api` service while preserving
the FastAPI `legacy-api` fallback. The final cutover script is non-destructive:
it does not delete files, does not touch runtime/private data, and does not move
`.env` files.

## Current Cutover State

- Rust gateway service: `api`
- FastAPI fallback service: `legacy-api`
- Web API target inside Compose: `http://api:8000`
- Unsupported gateway routes are proxied to `http://legacy-api:8000`.
- No Python API or worker files are archived in DIFF-103 because fallback
  behavior remains required.

## Rollback Expectations

If the Rust gateway fails in local deployment, rollback is a Compose-level
configuration rollback to the DIFF-101 state or to the last known good commit
before DIFF-102. The expected rollback path is:

1. Stop local services with the existing operator script or Docker Compose.
2. Revert the DIFF-102 and DIFF-103 commits in a normal git rollback.
3. Re-run Docker Compose config validation before starting services.
4. Start services and verify the Python/FastAPI `api` health endpoint.

Do not move runtime data into the repository during rollback. Do not archive or
delete governance files, `docs/diffs/`, `docs/agents/`, `.env`, storage roots,
or external data roots.

## Manual Verification Points

- `scripts/rust-cutover.sh --check`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Local API health after services are started
