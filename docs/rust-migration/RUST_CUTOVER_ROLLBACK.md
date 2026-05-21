# Rust Cutover Rollback

DIFF-138 keeps the Rust gateway as the `api` service and removes the FastAPI
`legacy-api` fallback wiring after route parity reaches zero missing FastAPI
routes. The cutover checks remain non-destructive: they do not delete files, do
not touch runtime/private data, and do not move `.env` files.

## Current Cutover State

- Rust gateway service: `api`
- Web API target inside Compose: `http://api:8000`
- Unsupported gateway routes return deterministic Rust 404 responses.
- No Python API or worker files are archived in DIFF-138. `services/api/`
  remains present for the DIFF-139 archive or preservation decision, and
  Python/Celery worker services remain active.

## Rollback Expectations

If the Rust gateway fails in local deployment, rollback is a normal git rollback
to the last known good commit before DIFF-138, or to the earlier DIFF-103
Rust-primary-with-fallback topology if FastAPI fallback must be restored for
diagnosis. The expected rollback path is:

1. Stop local services with the existing operator script or Docker Compose.
2. Revert DIFF-138 in a normal git rollback, or restore the earlier
   Rust-primary-with-fallback commits if that specific topology is needed.
3. Re-run Docker Compose config validation before starting services.
4. Start services and verify the expected API health endpoint for the restored
   topology.

Do not move runtime data into the repository during rollback. Do not archive or
delete governance files, `docs/diffs/`, `docs/agents/`, `.env`, storage roots,
or external data roots.

## Manual Verification Points

- `scripts/rust-cutover.sh --check`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Local API health after services are started
