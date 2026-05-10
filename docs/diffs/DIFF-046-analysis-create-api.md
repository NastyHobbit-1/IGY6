# DIFF-046: Analysis Create API

Status: Locked

## Type

Change-bearing.

## Objective

Add explicit create APIs for patterns, hypotheses, predictions, and
recommendations so analysis records can be entered with supporting evidence and
auditable provenance.

This DIFF only allows human/API-created analysis records. It does not authorize
automatic generation, scoring, pattern detection, prediction execution, outcome
evaluation, worker jobs, or self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-045 are locked.
- Existing analysis endpoints are read-only list/get routes.
- Existing database models already include `Pattern`, `Hypothesis`,
  `Prediction`, and `Recommendation`.
- Existing evidence records are stored as immutable `EvidenceItem` rows.
- Existing audit events can record important API writes.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-046-analysis-create-api.md`
- `services/api/app/analysis.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /analysis/patterns`.
- Add `POST /analysis/hypotheses`.
- Add `POST /analysis/predictions`.
- Add `POST /analysis/recommendations`.
- Validate referenced evidence IDs before creating records.
- Record audit events for created analysis records.
- Document the new endpoints and their limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Worker scheduling.
- Pattern detection or automatic analysis generation.
- Chat answer generation.
- Prediction execution.
- Recommendation execution.
- Outcome evaluation.
- Feedback side effects.
- Graph or vector upserts.
- Report generation.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-046` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Targeted API smoke checks should validate:

- Valid analysis create payloads construct successfully.
- Missing evidence IDs are rejected before record creation.
- Created records produce audit events.

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Blocked targeted local API smoke: `PYTHONPATH=services/api python3 -c "from app.analysis import PatternCreate, HypothesisCreate, PredictionCreate, RecommendationCreate; ..."` because the host Python environment does not have `fastapi` installed.
- Blocked targeted runtime evidence-validation and audit-event smoke for the same missing host `fastapi` dependency. The implemented create routes validate evidence IDs with `_validated_evidence_ids` before insertion and add `AuditEvent` rows in each create route.

## Completion Criteria

This DIFF is complete when:

- Pattern, hypothesis, prediction, and recommendation create endpoints exist.
- Create endpoints validate referenced evidence IDs.
- Create endpoints write audit events.
- New endpoints are documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Pattern review status transitions.
- Outcome target validation.
- Feedback side effects for source trust.
- Worker-backed normalization, chunking, embedding, and graph sync.
- Generated evidence-backed chat answers.
- Automatic pattern, prediction, or recommendation generation.
