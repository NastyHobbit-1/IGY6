# DIFF-013: Analysis Read API

Status: Locked

## Type

Change-bearing.

## Objective

Add read-only API routes for inspecting existing analysis records: patterns,
hypotheses, predictions, and recommendations.

This DIFF does not authorize creating analysis records, scoring confidence,
generating predictions, producing recommendations, or updating outcomes.

## Baseline Facts

- DIFF-000 through DIFF-012 are locked.
- Pattern, hypothesis, prediction, and recommendation tables exist from
  DIFF-001.
- No API currently exposes these records.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-013-analysis-read-api.md`
- `docs/api.md`
- `services/api/app/analysis.py`
- `services/api/app/main.py`

Allowed behavior:

- Add read-only list and retrieve routes for patterns.
- Add read-only list and retrieve routes for hypotheses.
- Add read-only list and retrieve routes for predictions.
- Add read-only list and retrieve routes for recommendations.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Record creation or mutation.
- Pattern detection.
- Hypothesis generation.
- Prediction generation.
- Recommendation generation.
- Outcome updates.
- Self-improvement queueing.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-013` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker services/collectors
  packages/policy` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- Read-only routes exist for patterns, hypotheses, predictions, and
  recommendations.
- API docs list the read-only analysis endpoints.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Pattern detection.
- Hypothesis generation.
- Prediction lifecycle updates.
- Recommendation generation and approval handling.
