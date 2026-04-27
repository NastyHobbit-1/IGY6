# DIFF-004: Policy Foundation

Status: Locked

## Type

Change-bearing.

## Objective

Add shared policy constants and pure validation helpers for source permissions,
sensitivity labels, approval defaults, and external model policy decisions.

This DIFF does not authorize runtime rewiring or enforcement changes.

## Baseline Facts

- DIFF-000 through DIFF-003 are locked.
- `packages/policy` currently contains a placeholder README.
- The security policy requires registered sources, explicit permission scope,
  sensitivity labels, allowed operations, audit events, and approval for
  sensitive or system-changing actions.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-004-policy-foundation.md`
- `packages/policy/README.md`
- `packages/policy/app/__init__.py`
- `packages/policy/app/rules.py`

Allowed behavior:

- Add constants for source types, allowed operations, sensitivity labels, and
  external model policies.
- Add pure helper functions that classify whether approval is required.
- Document that these helpers are not wired into runtime enforcement yet.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- UI changes.
- Worker task changes.
- Database model changes.
- Migration changes.
- Runtime policy rewiring.
- Real source access.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-004` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall packages/policy services/api services/worker services/collectors
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- `python3 -m compileall packages/policy services/api services/worker
  services/collectors` passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
  passed.

## Completion Criteria

This DIFF is complete when:

- Policy constants and pure helpers exist.
- Policy README describes the new foundation and runtime boundary.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Wiring policy validation into source registry endpoints.
- Approval request APIs.
- Worker-side approval enforcement.
- Audit-backed policy decisions.
