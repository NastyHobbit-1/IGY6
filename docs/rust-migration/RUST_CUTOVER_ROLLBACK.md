# Rust Cutover Rollback

DIFF-138 keeps the Rust gateway as the `api` service and removes the FastAPI
`legacy-api` fallback wiring after route parity reaches zero missing FastAPI
routes. DIFF-139 archives the tracked legacy FastAPI API source under
`archive/legacy-python/services-api`. DIFF-140 records the final Rust API
cutover audit from that point: the active API path became Rust-native, FastAPI
fallback was removed, and Python/Celery `worker` and `beat` still remained
active until later worker cutover DIFFs. The cutover checks remain
non-destructive: they do not delete files, do not touch runtime/private data,
and do not move `.env` files.
DIFF-162 reviewed production worker replacement and decided not to replace the
Python/Celery worker yet because the Rust worker live loop is still canary-only
and intentionally bounded. The production worker rollback posture is therefore
unchanged.
DIFF-163 adds a production-capable Rust worker `--daemon` mode, but it does not
change production Docker Compose worker ownership. Rollback remains stopping any
manually run Rust daemon and keeping the existing Python/Celery worker and beat
services active.
DIFF-164 changes production Docker Compose worker ownership to the Rust worker
daemon and removes the empty Celery `beat` service after verifying that
the now-archived worker source defines no repo beat schedule. Rollback for
worker ownership is to restore the prior Python/Celery `worker` and `beat`
service definitions from git, validate Compose, and restart.
DIFF-165 archives the inactive Python/Celery worker source under
`archive/legacy-python/services-worker` and records the final Rust-only
application runtime audit. Rollback now uses that archive path or git history
when restoring a Python/Celery worker topology.
DIFF-168, DIFF-169, and DIFF-170 add non-destructive post-cutover validation
commands for smoke checks, fresh-clone startup readiness, and lifecycle command
shape checks. DIFF-171 updates documentation only and does not change runtime
ownership or rollback mechanics.

## Current Cutover State

- Rust gateway service: `api`
- Web API target inside Compose: `http://api:8000`
- Unsupported gateway routes return deterministic Rust 404 responses.
- Legacy FastAPI API source is archived at
  `archive/legacy-python/services-api`.
- Production `worker` service runs the Rust worker daemon from
  `crates/igy6-worker/Dockerfile`.
- Python/Celery `worker` and `beat` services are not active in base Compose
  after DIFF-164.
- Legacy Python/Celery worker source is archived at
  `archive/legacy-python/services-worker`.
- The DIFF-161 Rust worker Compose override is canary-only.
- Rust-only application runtime is claimed for the active API and worker path.
  Next.js web and infrastructure services remain intentionally non-Rust
  supporting components.

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

To roll back the DIFF-164/DIFF-165 production worker cutover, restore the
previous Python/Celery worker source from
`archive/legacy-python/services-worker` or git history, restore the previous
`worker` service definition, restore the previous `beat` service definition if
scheduled-work rollback is needed, validate Compose, and only then restart the
stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env config
docker compose -f infra/docker-compose.yml --env-file .env up --build worker
```

## Manual Verification Points

- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`
- `scripts/rust-cutover.sh --check`
- `python3 scripts/post-cutover-runtime-audit.py`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Local API health after services are started
