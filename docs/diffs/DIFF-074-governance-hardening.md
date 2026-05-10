# DIFF-074: Governance Hardening

Status: Locked

## Type

Change-bearing

## Objective

Harden the current IGY6 foundation so approval matching, work-item status
changes, dispatch checks, collection summaries, retrieval hydration, UTF-8
normalization expectations, and README boundaries accurately match the
local-first, permissioned, auditable design.

## Baseline Facts

- The worktree was clean before this DIFF was created.
- No active DIFF existed before this DIFF was created.
- Collection approval validation currently rejects mismatched payload keys but
  accepts missing `source_id`, `source_permission_id`, or `operation` keys.
- `POST /work-items/{id}/status` currently accepts any known status from any
  current status.
- `POST /work-items/{id}/dispatch` currently checks only that the work item is
  `queued`.
- Collection routes create queued normalization work items while collection
  summaries still include `would_enqueue_worker: False`.
- Retrieval hydration currently returns hits from disabled sources.
- Worker normalization decodes raw artifacts as UTF-8 text and fails non-UTF-8
  artifacts at worker execution time.
- README says there are no generated evidence-backed answers, while
  deterministic evidence-summary packets now exist without LLM generation.
- No Python test files or pytest configuration were found during pre-work
  inspection.

## Allowed Scope

- `docs/diffs/DIFF-074-governance-hardening.md`
- `README.md`
- `services/api/app/collection_runs.py`
- `services/api/app/work_items.py`
- `services/api/app/retrieval.py`
- `services/api/app/chat.py` only if necessary
- `services/worker/app/tasks.py` only if necessary
- Existing tests only if they exist and can be updated without adding new
  frameworks

Allowed behavior changes:

- Require exact collection approval payload matches for `source_id`,
  `source_permission_id`, and `operation`.
- Enforce minimal safe work-item status transitions.
- Require intent-verification metadata before queued work items can dispatch.
- Make collection summary metadata describe queued normalization work items
  accurately.
- Exclude retrieval/chat hits from disabled sources.
- Add a clearly named future policy extension point for source retrieval
  filtering.
- Clarify UTF-8 text-only normalization expectations in API metadata, worker
  error messages, and README.

## Prohibited Scope

- No new dependencies.
- No schema migrations unless absolutely required; prefer existing JSON payload
  fields.
- No database model rewrites.
- No auth system.
- No frontend redesign.
- No Docker changes.
- No Qdrant, Neo4j, MLflow, or Phoenix changes.
- No browser automation.
- No LLM/model generation.
- No broad refactor.
- No unrelated cleanup.
- No file renames.

## Required Tags

Use `DIFF-074` in change summaries, commits, pull requests, and review notes for
this work.

## Verification

Run:

```bash
python3 -m compileall services/api services/worker
```

If relevant tests exist, run only the relevant tests. If no tests exist, record
that no test files or test configuration were present.

Do not start the full Docker stack for this DIFF unless a narrow verification
failure requires it and that reason is recorded.

## Completion Criteria

- Collection approvals fail when any required payload key is missing or
  mismatched.
- Work-item status transitions are constrained and invalid transitions return
  clear `409 Conflict` responses.
- Dispatch rejects queued work items without intent-verification metadata.
- Collection summaries no longer claim worker enqueue would not happen when a
  normalization work item is created.
- Retrieval/chat hydration excludes disabled sources.
- README distinguishes no LLM-generated answers from deterministic
  evidence-summary packets.
- UTF-8 text-only normalization expectations are visible.
- Verification is recorded below before the DIFF is locked.

## Verification Result

- Passed: `python3 -m compileall services/api services/worker`.
- Passed: `git diff --check`.
- Passed with project venv: work-item governance smoke verified that
  `pending_intent_verification -> queued` requires intent verification and that
  `completed -> queued` is rejected with `409`.
- Passed with project venv: retrieval policy smoke verified that `None` and
  enabled sources are allowed while disabled sources are filtered.
- Passed with project venv: manual-upload UTF-8 smoke verified UTF-8 bytes are
  accepted and non-UTF-8 bytes are rejected with `422`.
- Passed with project venv: approval exact-match smoke verified a complete
  collection approval payload is accepted and a missing required key is
  rejected with `409`.
- No relevant test files or pytest configuration were present to run.
- Blocked: equivalent import-level smokes with system `python3` could not run
  because FastAPI is not installed in the system interpreter. The project
  `.venv/bin/python` was used for dependency-backed smokes instead.
- Not run: Docker/full-stack verification, because this DIFF did not require
  starting services and the narrow compile/smoke checks covered the touched
  behavior.

## Out Of Scope Follow-Up

- Full authorization/authentication.
- Full workflow engine.
- Sensitivity, trust, or external-model retrieval policies beyond the disabled
  source guard and named extension point.
- Binary artifact normalization.
- UI changes for governance controls.
