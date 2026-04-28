# DIFF-024: Evidence Item Create API

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal evidence-item creation endpoint so the existing evidence ledger
can record immutable evidence entries with source, document, and chunk links.

This DIFF does not authorize claim generation, pattern detection, chat
integration, worker scheduling, normalization changes, or artifact writes.

## Baseline Facts

- DIFF-000 through DIFF-023 are locked.
- Evidence, claim, chunk, and normalized-document models already exist.
- Evidence read endpoints already exist.
- No evidence-item create endpoint exists yet.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-024-evidence-item-create-api.md`
- `services/api/app/evidence.py`

Allowed behavior:

- Add an evidence-item create request model.
- Validate source, document, and chunk link consistency when IDs are provided.
- Create immutable evidence-item rows.
- Record an audit event for evidence creation.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API route changes outside `services/api/app/evidence.py`.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Claim creation.
- Pattern creation.
- Hypothesis creation.
- Prediction creation.
- Recommendation creation.
- Report generation.
- Chat integration.
- Normalization logic changes.
- Artifact writes.
- Filesystem traversal.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-024` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api python3 -c "from app.evidence import EvidenceItemCreate; payload = EvidenceItemCreate(source_id='src', document_id='doc', chunk_id='chunk', evidence_type='note', statement='example'); print(payload.evidence_type, payload.statement)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Blocked: `PYTHONPATH=services/api python3 -c "from app.evidence import EvidenceItemCreate; payload = EvidenceItemCreate(source_id='src', document_id='doc', chunk_id='chunk', evidence_type='note', statement='example'); print(payload.evidence_type, payload.statement)"` because the base Python environment does not have the project runtime dependencies installed

## Completion Criteria

This DIFF is complete when:

- An evidence-item create endpoint exists.
- Evidence links are validated where applicable.
- Evidence creation is audited.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Claim generation APIs.
- Pattern write flows.
- Prediction and recommendation creation flows.
- Evidence synthesis from normalization and collection runs.
