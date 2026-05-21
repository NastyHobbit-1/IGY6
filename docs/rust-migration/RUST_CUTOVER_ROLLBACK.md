# Rust Cutover Rollback

DIFF-138 keeps the Rust gateway as the `api` service and removes the FastAPI
`legacy-api` fallback wiring after route parity reaches zero missing FastAPI
routes. DIFF-139 archives the tracked legacy FastAPI API source under
`archive/legacy-python/services-api`. The cutover checks remain
non-destructive: they do not delete files, do not touch runtime/private data,
and do not move `.env` files.

## Current Cutover State

- Rust gateway service: `api`
- Web API target inside Compose: `http://api:8000`
- Unsupported gateway routes return deterministic Rust 404 responses.
- Legacy FastAPI API source is archived at
  `archive/legacy-python/services-api`.
- Python/Celery worker services remain active from `services/worker`.

## Rollback Expectations

If the Rust gateway fails in local deployment, rollback is a normal git rollback
to the last known good commit before DIFF-139 or DIFF-138, depending on whether
the archived FastAPI source or the earlier fallback topology must be restored.
The expected rollback path is:

1. Stop local services with the existing operator script or Docker Compose.
2. Revert DIFF-139 to restore the legacy FastAPI source path. Revert DIFF-138
   only if the old Rust-primary-with-FastAPI-fallback topology is specifically
   needed for diagnosis.
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
