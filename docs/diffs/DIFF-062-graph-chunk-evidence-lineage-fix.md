# DIFF-062: Graph Chunk Evidence Lineage Fix

Status: Locked

## Type

Change-bearing.

## Objective

Fix deterministic graph lineage sync so `CHUNK_HAS_EVIDENCE` relationships are
created from each evidence item that references a chunk, not from the outcome
loop.

## Baseline Facts

- DIFF-000 through DIFF-061 are locked.
- Graph lineage sync creates document-evidence relationships in the evidence
  item loop.
- `CHUNK_HAS_EVIDENCE` creation currently appears in the outcome loop while
  referencing `evidence_item` from a prior loop.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-062-graph-chunk-evidence-lineage-fix.md`
- `services/api/app/graph_memory.py`
- `docs/api.md`

Allowed behavior:

- Move `CHUNK_HAS_EVIDENCE` creation into the evidence item loop.
- Remove the stale outcome-loop reference to `evidence_item`.
- Add a small deterministic helper for relationship planning if useful for
  smoke verification.
- Document the fix.

## Prohibited Scope

This DIFF does not allow graph schema redesign, new graph node labels, database
model changes, migrations, vector behavior changes, worker changes, UI changes,
dependency changes, or broad refactors.

## Required Tags

Use `DIFF-062` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate that chunk-evidence relationship planning
uses evidence item chunk IDs directly.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python -c "import sys; sys.path.insert(0, 'services/api'); from types import SimpleNamespace; from app.graph_memory import chunk_evidence_relationship_parameters; assert chunk_evidence_relationship_parameters(SimpleNamespace(id='e1', chunk_id='c1')) == {'chunk_id': 'c1', 'evidence_id': 'e1'}; assert chunk_evidence_relationship_parameters(SimpleNamespace(id='e2', chunk_id=None)) is None"
```

## Completion Criteria

This DIFF is complete when `CHUNK_HAS_EVIDENCE` sync is scoped to evidence
items, outcome sync no longer references stale evidence variables, docs are
updated, and verification passes.
