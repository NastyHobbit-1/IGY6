# DIFF-047: Pattern Review API

Status: Locked

## Type

Change-bearing.

## Objective

Add an explicit pattern review API so a candidate pattern can be marked
verified or rejected with an auditable reviewer decision.

This DIFF only allows review status transitions for existing pattern records.
It does not authorize automatic pattern detection, pattern generation, outcome
evaluation, feedback side effects, graph or vector updates, worker jobs, or
self-improvement behavior.

## Baseline Facts

- DIFF-000 through DIFF-046 are locked.
- DIFF-046 added explicit create APIs for analysis records.
- Existing `Pattern` rows include a `status` field.
- Existing audit events can record important API writes.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-047-pattern-review-api.md`
- `services/api/app/analysis.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /analysis/patterns/{pattern_id}/review`.
- Allow review decisions that set a pattern status to `verified` or `rejected`.
- Require the current pattern status to be `candidate`.
- Accept a reviewer actor ID and optional review note.
- Record an audit event for the review decision.
- Document the new endpoint and its limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Worker scheduling.
- Pattern detection or automatic analysis generation.
- Hypothesis, prediction, or recommendation review endpoints.
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

Use `DIFF-047` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted API smoke checks should validate:

- Pattern review payloads construct successfully.
- Candidate patterns can transition to `verified` or `rejected`.
- Non-candidate patterns are rejected.
- Review decisions add audit events.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: candidate pattern transitioned to `verified` and produced `analysis.pattern.reviewed` audit event data.
- Passed targeted venv smoke: non-candidate pattern review returned `409`.
- Passed targeted venv smoke: invalid review status returned `422`.
- Passed targeted venv smoke: unknown pattern returned `404`.

## Completion Criteria

This DIFF is complete when:

- Pattern review endpoint exists.
- Review endpoint only permits `candidate` to `verified` or `rejected`.
- Review endpoint writes an audit event.
- New endpoint is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Outcome target validation.
- Feedback side effects for source trust.
- Worker-backed normalization, chunking, embedding, and graph sync.
- Review endpoints for other analysis record types if needed.
- Generated evidence-backed chat answers.
- Automatic pattern, prediction, or recommendation generation.
