# DIFF-063: Collection Permission Approval Gates

Status: Locked

## Type

Change-bearing.

## Objective

Tighten non-dry-run collection safety so empty operation lists are not
permissive and approval-required permissions must provide an approved approval
record before collection.

## Baseline Facts

- DIFF-000 through DIFF-062 are locked.
- Source permissions default to `approval_required: true`.
- Collection routes currently allow collection when `allowed_operations` is
  empty.
- Collection routes do not require an approved approval record when
  `approval_required` is true.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-063-collection-permission-approval-gates.md`
- `services/api/app/collection_runs.py`
- `docs/api.md`

Allowed behavior:

- Require explicit allowed operations for dry-run and collection routes.
- Add optional approval IDs to non-dry-run collection payloads.
- Require an approved approval record when the source permission has
  `approval_required: true`.
- Validate that approval payload metadata matches the requested source,
  permission, and operation when present.
- Document the safety gates.

## Prohibited Scope

This DIFF does not allow worker dispatch, new approval endpoints, source model
changes, migrations, UI changes, dependency changes, connector rewrites, or
broad refactors.

## Required Tags

Use `DIFF-063` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate operation and approval matching helpers.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python - <<'PY'
import sys
from types import SimpleNamespace

sys.path.insert(0, 'services/api')
from fastapi import HTTPException
from app.collection_runs import COLLECTION_APPROVAL_REQUEST_TYPES, _require_permission_operation

_require_permission_operation(SimpleNamespace(allowed_operations=['read']), {'read'}, 'read')
assert 'manual_upload_collection' in COLLECTION_APPROVAL_REQUEST_TYPES
raised = False
try:
    _require_permission_operation(SimpleNamespace(allowed_operations=[]), {'read'}, 'read')
except HTTPException:
    raised = True
assert raised
PY
```

## Completion Criteria

This DIFF is complete when collection permission checks are explicit,
approval-required collection requires an approved matching approval, docs are
updated, and verification passes.
