# DIFF-049: Feedback Source Trust Side Effects

Status: Locked

## Type

Change-bearing.

## Objective

Apply explicit source trust feedback to source records while preserving the
feedback event and audit trail.

This DIFF only allows side effects for source-target feedback labels
`trusted`, `noisy`, and `rejected`. It does not authorize feedback-driven
ranking changes, retrieval changes, outcome evaluation, graph or vector updates,
worker jobs, or self-improvement handoffs.

## Baseline Facts

- DIFF-000 through DIFF-048 are locked.
- Feedback events can be created with target type `source`.
- Source records already include `trust_level` and `enabled` fields.
- Existing audit events can record feedback creation and source changes.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-049-feedback-source-trust-side-effects.md`
- `services/api/app/feedback.py`
- `docs/api.md`

Allowed behavior:

- For `target_type=source` and label `trusted`, set source `trust_level` to
  `trusted` and keep the source enabled.
- For `target_type=source` and label `noisy`, set source `trust_level` to
  `noisy` and keep the source enabled.
- For `target_type=source` and label `rejected`, set source `trust_level` to
  `rejected` and disable the source.
- Validate source existence before applying source trust side effects.
- Record an audit event for source trust side effects.
- Document the behavior and limits.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Worker scheduling.
- Feedback side effects for non-source targets.
- Feedback-driven ranking changes.
- Outcome evaluation.
- Graph or vector upserts.
- Report generation.
- Self-improvement queue creation.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-049` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted API smoke checks should validate:

- Source `trusted` feedback updates source trust level and audit data.
- Source `noisy` feedback updates source trust level and audit data.
- Source `rejected` feedback disables the source and audit data.
- Non-source feedback has no source side effect.
- Missing source target is rejected for source trust labels.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: `trusted` source feedback set trust level to `trusted`, kept source enabled, and produced source audit data.
- Passed targeted venv smoke: `noisy` source feedback set trust level to `noisy`, kept source enabled, and produced source audit data.
- Passed targeted venv smoke: `rejected` source feedback set trust level to `rejected`, disabled the source, and produced source audit data.
- Passed targeted venv smoke: non-source feedback produced no source side effect.
- Passed targeted venv smoke: missing source trust feedback returned `404` and added no source audit data.

## Completion Criteria

This DIFF is complete when:

- Source trust feedback labels update source records as scoped.
- Source trust side effects write audit events.
- Non-source feedback remains record-only.
- New behavior is documented.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Feedback target validation beyond source trust side effects.
- Outcome-driven status updates.
- Feedback-driven self-improvement queue entries.
- Feedback-driven retrieval ranking changes.
