# DIFF-001: Phase 1 Evidence Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add the first Phase 1 source and evidence database foundation required by
`Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md` without adding
real ingestion, browser automation, embeddings, graph extraction, chat,
prediction, recommendations, or self-improvement execution.

## Baseline Facts

- DIFF-000 is locked and facts-only.
- Phase 0 skeleton is present.
- Current PostgreSQL foundational tables include sources, source permissions,
  collection runs, raw artifacts, work items, approvals, audit events, and
  reports.
- The build instructions require additional Phase 1 evidence-ledger tables,
  including normalized documents, chunks, evidence items, claims, outcomes,
  feedback events, improvement items, and experiment runs.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-001-phase1-evidence-foundation.md`
- `docs/planning/PHASE-1-PLAN.md`
- `services/api/app/models.py`
- `services/api/migrations/versions/0002_phase1_evidence_foundation.py`

Allowed behavior:

- Add SQLAlchemy model definitions for Phase 1 evidence foundation tables.
- Add a new Alembic migration for those tables.
- Add a Phase 1 planning document that describes this narrow foundation.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Editing the existing Phase 0 migration.
- API endpoint changes.
- UI changes.
- Worker task changes.
- Real source ingestion.
- Browser automation.
- Embeddings.
- Graph extraction.
- Evidence-backed chat.
- Prediction or recommendation execution.
- Outcome workflow UI.
- Self-improvement execution.
- External model calls.
- Renames.
- Broad refactors.
- Dependency changes.
- Docker rewiring.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-001` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall services/api services/worker` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- A Phase 1 planning document exists.
- SQLAlchemy models exist for the narrow Phase 1 evidence foundation tables.
- A new Alembic migration creates the corresponding tables.
- Required verification is run or blocked with a clear reason.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- API endpoints for source registry and evidence review.
- Manual upload connector.
- Local project connector.
- Artifact hashing and storage implementation.
- Normalization workers.
- Source dry-run behavior.
- Evidence inspection UI.
