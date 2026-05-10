# DIFF-068: Feedback Outcome Learning Side Effects

Status: Locked

## Type

Change-bearing.

## Objective

Make feedback and outcome records update local review state and create
self-improvement candidates for weak results without changing production
methods automatically.

## Baseline Facts

- DIFF-000 through DIFF-067 are locked.
- Outcomes validate targets but do not update target records.
- Feedback applies source trust side effects only.
- Improvement items exist but weak feedback does not create them.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-068-feedback-outcome-learning-side-effects.md`
- `services/api/app/outcomes.py`
- `services/api/app/feedback.py`
- `docs/api.md`

Allowed behavior:

- When an outcome is recorded for an analysis/report/work item target, update
  the target status and metadata with the latest outcome.
- Audit target status updates caused by outcomes.
- When weak feedback is recorded, create a proposed improvement item.
- Audit improvement item creation from feedback.
- Document the side effects.

## Prohibited Scope

This DIFF does not allow production method changes, experiment execution,
worker dispatch, vector/graph updates, model calls, migrations, UI changes, or
broad refactors.

## Required Tags

Use `DIFF-068` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate status and improvement target mappings.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
sys.path.insert(0, 'services/api')
from app.outcomes import outcome_target_status
from app.feedback import improvement_target_area, WEAK_FEEDBACK_LABELS

assert outcome_target_status('correct') == 'correct'
assert outcome_target_status('not_useful') == 'not_useful'
assert improvement_target_area('prediction') == 'prediction'
assert improvement_target_area('report') == 'reporting'
assert 'wrong' in WEAK_FEEDBACK_LABELS
PY
```

## Completion Criteria

This DIFF is complete when outcomes update target review state, weak feedback
creates proposed improvement items, all side effects are audited, docs are
updated, and verification passes.
