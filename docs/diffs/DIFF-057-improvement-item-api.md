# DIFF-057: Improvement Item API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes to create and inspect self-improvement queue items.

## Baseline Facts

- DIFF-000 through DIFF-056 are locked.
- `ImprovementItem` already exists in the relational model.
- No API routes currently expose improvement items.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-057-improvement-item-api.md`
- `services/api/app/improvements.py`
- `services/api/app/main.py`
- `docs/api.md`

Allowed behavior:

- Add list/create/get routes for improvement items.
- Validate target area and priority against local allowlists.
- Record audit events for created improvement items.
- Document the routes and limits.

## Prohibited Scope

This DIFF does not allow experiment execution, worker dispatch, MLflow/Optuna
calls, method changes, model changes, migrations, dependency changes, or broad
refactors.

## Required Tags

Use `DIFF-057` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate payload construction and allowlists.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
from app.improvements import ImprovementItemCreate, PRIORITIES, TARGET_AREAS

payload = ImprovementItemCreate(target_area="retrieval", objective="Reduce weak retrieval cases")
assert payload.priority == "normal"
assert "safety" in TARGET_AREAS
assert "urgent" in PRIORITIES
PY
```

## Completion Criteria

This DIFF is complete when improvement item routes exist, creation is audited,
routes are documented, and verification passes.
