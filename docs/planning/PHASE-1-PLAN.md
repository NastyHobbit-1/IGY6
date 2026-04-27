# Phase 1 Implementation Plan

DIFF: `DIFF-001`

## Narrow Scope

This phase starts the source and evidence core by adding the database foundation
for normalized evidence records. It does not implement collection, ingestion,
normalization execution, embeddings, graph extraction, chat, prediction,
recommendation execution, or self-improvement execution.

## Tables Added

- `normalized_documents`
- `chunks`
- `evidence_items`
- `claims`
- `patterns`
- `hypotheses`
- `predictions`
- `recommendations`
- `outcomes`
- `feedback_events`
- `improvement_items`
- `experiment_runs`

## Out Of Scope

- API endpoints.
- UI changes.
- Worker tasks.
- Manual upload connector.
- Local project connector.
- Browser automation.
- External model calls.
- Qdrant writes.
- Neo4j writes.
- MLflow or Optuna execution.

## Verification

Required by DIFF-001:

```bash
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
```
