# DIFF-054: Expanded Graph Analysis Lineage

Status: Locked

## Type

Change-bearing.

## Objective

Expand deterministic Neo4j lineage sync beyond source/artifact/document/chunk
and evidence nodes to include claims, patterns, hypotheses, predictions,
recommendations, outcomes, and reports.

## Baseline Facts

- DIFF-000 through DIFF-053 are locked.
- Existing graph sync handles source, artifact, document, chunk, and evidence
  provenance only.
- Relational tables already store claims, analysis records, outcomes, reports,
  and evidence ID lists.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-054-expanded-graph-analysis-lineage.md`
- `services/api/app/graph_memory.py`
- `docs/api.md`

Allowed behavior:

- Add graph constraints and allowed labels for claims, patterns, hypotheses,
  predictions, recommendations, outcomes, and reports.
- Upsert deterministic graph nodes for those records.
- Upsert deterministic evidence/support/outcome/report relationships from
  existing relational IDs.
- Document the expanded graph lineage behavior.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- Worker changes.
- Graph analytics.
- Inferred relationships beyond existing relational IDs.
- API route changes beyond existing graph sync behavior.
- Dependency changes.
- Docker rewiring.
- Renames or broad refactors.

## Required Tags

Use `DIFF-054` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate:

- Expanded allowed graph labels include analysis/outcome/report labels.
- Expanded relationship type list includes evidence and outcome relationships.
- Graph constraints include the new node labels.

Results:

- Passed: `.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `git diff --check`
- Passed targeted venv smoke: expanded graph labels include `Prediction`.
- Passed targeted venv smoke: relationship types include `EVIDENCE_SUPPORTS_PREDICTION`.
- Passed targeted venv smoke: graph constraints include `Outcome`.

## Completion Criteria

This DIFF is complete when:

- Expanded graph lineage sync code exists.
- Relationship inspection permits the expanded labels.
- New behavior is documented.
- Verification passes or any blockage is recorded.

## Out Of Scope Follow-Up

Future DIFFs must cover worker-backed graph sync, graph analytics, and inferred
relationship extraction.
