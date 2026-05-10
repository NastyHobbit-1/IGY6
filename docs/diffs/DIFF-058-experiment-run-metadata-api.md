# DIFF-058: Experiment Run Metadata API

Status: Locked

## Type

Change-bearing.

## Objective

Add API routes to create, inspect, and status-update self-improvement
experiment run metadata.

## Baseline Facts

- DIFF-000 through DIFF-057 are locked.
- `ExperimentRun` already exists in the relational model.
- DIFF-057 exposes improvement item metadata.
- No API routes currently expose experiment runs.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-058-experiment-run-metadata-api.md`
- `services/api/app/experiments.py`
- `services/api/app/main.py`
- `docs/api.md`

Allowed behavior:

- Add list/create/get routes for experiment run metadata.
- Add an explicit experiment run status update route.
- Validate optional improvement item references.
- Validate experiment statuses against a local allowlist.
- Record audit events for created and status-updated experiment runs.
- Document the routes and limits.

## Prohibited Scope

This DIFF does not allow experiment execution, worker dispatch, MLflow/Optuna
calls, method changes, model changes, migrations, dependency changes, or broad
refactors.

## Required Tags

Use `DIFF-058` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate payload construction and status allowlists.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python -c "import sys; sys.path.insert(0, 'services/api'); from app.experiments import ExperimentRunCreate, ExperimentRunStatusUpdate, EXPERIMENT_STATUSES; assert ExperimentRunCreate().status == 'planned'; assert ExperimentRunStatusUpdate(status='completed').status == 'completed'; assert 'abandoned' in EXPERIMENT_STATUSES"
```

## Completion Criteria

This DIFF is complete when experiment run metadata routes exist, creation and
status updates are audited, routes are documented, and verification passes.
