# DIFF-064: Worker Compose Config Reconciliation

Status: Locked

## Type

Change-bearing.

## Objective

Pass the same local runtime settings that worker tasks use through Docker
Compose so worker execution does not rely on divergent defaults.

## Baseline Facts

- DIFF-000 through DIFF-063 are locked.
- Worker tasks use database, artifact store, Qdrant URL, Qdrant collection, and
  Qdrant vector size settings.
- Docker Compose currently passes only Celery settings to `worker` and `beat`.
- `.env.example` does not expose Qdrant chunk collection or vector size.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-064-worker-compose-config-reconciliation.md`
- `infra/docker-compose.yml`
- `.env.example`
- `docs/api.md`
- `README.md`

Allowed behavior:

- Add worker and beat environment variables needed by existing worker settings.
- Add Qdrant chunk collection/vector size defaults to `.env.example`.
- Document that worker runtime config is supplied by Compose.

## Prohibited Scope

This DIFF does not allow worker behavior changes, API behavior changes,
dependency changes, Docker image changes, service topology changes, migrations,
or broad refactors.

## Required Tags

Use `DIFF-064` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate the rendered worker environment includes
database, artifact, and Qdrant settings.

Completed verification:

```bash
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
docker compose -f infra/docker-compose.yml --env-file .env.example config | rg -n "DATABASE_URL|ARTIFACT_STORE_PATH|QDRANT_URL|QDRANT_CHUNK_COLLECTION|QDRANT_CHUNK_VECTOR_SIZE"
```

## Completion Criteria

This DIFF is complete when Compose passes worker runtime settings explicitly,
the environment template documents them, docs are updated, and verification
passes.
