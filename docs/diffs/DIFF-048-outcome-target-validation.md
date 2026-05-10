# DIFF-048: Outcome Target Validation

Status: Locked

## Type

Change-bearing.

## Objective

Strengthen outcome creation so outcomes can only be recorded for existing
target records and existing evidence items.

This DIFF only validates references before creating outcome records. It does
not authorize prediction/recommendation status updates, feedback side effects,
graph or vector updates, worker jobs, report generation, or self-improvement
handoffs.

## Baseline Facts

- DIFF-000 through DIFF-047 are locked.
- Existing outcome endpoints validate target type and outcome status allowlists.
- Existing outcome creation does not validate target record existence.
- Existing outcome creation does not validate referenced evidence IDs.
- Existing audit events can record outcome creation.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-048-outcome-target-validation.md`
- `services/api/app/outcomes.py`
- `docs/api.md`

Allowed behavior:

- Validate outcome target existence for `prediction`, `recommendation`,
  `work_item`, `hypothesis`, `pattern`, and `report`.
- Validate referenced evidence IDs before creating an outcome.
- Deduplicate outcome evidence IDs while preserving request order.
- Document the validation behavior and limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Worker scheduling.
- Prediction or recommendation status updates.
- Pattern, hypothesis, prediction, recommendation, or report creation.
- Outcome evaluation logic.
- Feedback side effects.
- Graph or vector upserts.
- Report generation.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-048` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted API smoke checks should validate:

- Valid outcome references pass validation.
- Missing target records are rejected.
- Missing evidence IDs are rejected.
- Evidence IDs are deduplicated while preserving order.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: valid prediction target reference passed validation.
- Passed targeted venv smoke: duplicate evidence IDs were deduplicated in first-seen order.
- Passed targeted venv smoke: missing target returned `422`.
- Passed targeted venv smoke: missing evidence ID returned `422`.

## Completion Criteria

This DIFF is complete when:

- Outcome creation validates supported target record existence.
- Outcome creation validates referenced evidence IDs.
- New validation behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Feedback side effects for source trust.
- Outcome-driven prediction or recommendation status updates.
- Outcome-driven self-improvement queue entries.
- Worker-backed normalization, chunking, embedding, and graph sync.
