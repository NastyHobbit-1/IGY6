# DIFF-067: Report Render Artifact API

Status: Locked

## Type

Change-bearing.

## Objective

Add a bounded report render endpoint that writes a Markdown report artifact and
marks the report ready.

## Baseline Facts

- DIFF-000 through DIFF-066 are locked.
- Report metadata and report work item scaffold routes exist.
- Raw artifact storage exists and is content-addressed.
- Reports currently do not render files or create exportable artifacts.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-067-report-render-artifact-api.md`
- `services/api/app/reports.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /reports/{report_id}/render`.
- Render deterministic Markdown from existing local metadata records.
- Store the Markdown in the local content-addressed artifact store.
- Create a raw artifact metadata record for the generated report.
- Set report status to `ready`, attach the artifact path, and audit the render.
- Document the route and limits.

## Prohibited Scope

This DIFF does not allow external model calls, artifact content reads, export
store changes, worker changes, report template engines, migrations, dependency
changes, UI changes, or broad refactors.

## Required Tags

Use `DIFF-067` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate Markdown construction without writing
files.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
from types import SimpleNamespace

sys.path.insert(0, 'services/api')
from app.reports import build_report_markdown

report = SimpleNamespace(id='r1', title='MVP report', report_type='summary', requested_by_actor_id='local-owner', status='requested')
content = build_report_markdown(report, {'sources': 1, 'evidence_items': 2}, ['- source: test'], 'note')
assert content.startswith('# MVP report')
assert '- sources: 1' in content
assert 'does not call external models' in content
assert 'note' in content
PY
```

## Completion Criteria

This DIFF is complete when reports can render a local Markdown artifact, report
metadata points at the artifact, render actions are audited, docs are updated,
and verification passes.
