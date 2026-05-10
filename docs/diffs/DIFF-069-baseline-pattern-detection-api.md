# DIFF-069: Baseline Pattern Detection API

Status: Locked

## Type

Change-bearing.

## Objective

Add a conservative baseline pattern detector that creates candidate pattern
records from existing local evidence without calling models.

## Baseline Facts

- DIFF-000 through DIFF-068 are locked.
- Pattern records can be created manually and reviewed.
- No route detects recurrence, source conflict, or missing-information gap
  candidates.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-069-baseline-pattern-detection-api.md`
- `services/api/app/analysis.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /analysis/patterns/detect-baseline`.
- Detect simple recurrence candidates from repeated evidence types.
- Detect simple cross-source conflict candidates from repeated statements with
  different source IDs.
- Detect a missing-information gap when no evidence exists.
- Avoid creating duplicate detector candidates with the same detector key.
- Audit created pattern candidates.
- Document the route and limits.

## Prohibited Scope

This DIFF does not allow external model calls, advanced ML, worker dispatch,
graph/vector writes, prediction/recommendation generation, migrations,
dependency changes, UI changes, or broad refactors.

## Required Tags

Use `DIFF-069` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate baseline detector candidates without a
database.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
from types import SimpleNamespace

sys.path.insert(0, 'services/api')
from app.analysis import baseline_pattern_candidates

empty = baseline_pattern_candidates([], recurrence_threshold=3)
assert empty[0]['pattern_type'] == 'missing_information_gap'
items = [SimpleNamespace(id=f'e{i}', evidence_type='document_chunk', statement='Same Statement', source_id='s1' if i < 2 else 's2') for i in range(3)]
candidates = baseline_pattern_candidates(items, recurrence_threshold=3)
types = {candidate['pattern_type'] for candidate in candidates}
assert 'recurrence' in types
assert 'cross_source_conflict' in types
PY
```

## Completion Criteria

This DIFF is complete when baseline pattern detection can create candidate
patterns from local evidence, duplicates are avoided, docs are updated, and
verification passes.
