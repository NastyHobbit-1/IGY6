# DIFF-060: Phase Metadata Reconciliation

Status: Locked

## Type

Change-bearing.

## Objective

Update API and README phase metadata so local service descriptions match the
implemented foundation through DIFF-059.

## Baseline Facts

- DIFF-000 through DIFF-059 are locked.
- README still describes only the Phase 0 skeleton.
- API metadata still describes a narrow Phase 1 foundation even though source,
  evidence, collection, vector, graph, review, worker, improvement, and
  experiment metadata foundations exist.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-060-phase-metadata-reconciliation.md`
- `services/api/app/main.py`
- `README.md`
- `docs/api.md`

Allowed behavior:

- Update descriptive phase/status metadata.
- Update README service capability and boundary wording.
- Update API introduction wording.

## Prohibited Scope

This DIFF does not allow endpoint behavior changes, new routes, worker changes,
database model changes, migrations, dependency changes, Docker changes, broad
refactors, or claims that unimplemented execution features exist.

## Required Tags

Use `DIFF-060` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate root API metadata.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python -c "import sys; sys.path.insert(0, 'services/api'); from app.main import app, root; assert app.version == '0.1.0-memory-review-foundation'; assert root()['phase'] == 'memory-review-foundation'"
```

## Completion Criteria

This DIFF is complete when metadata and docs accurately describe the current
foundation without overstating unimplemented execution behavior, and
verification passes.
