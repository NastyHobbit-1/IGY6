# DIFF-078: UI Technical Term Help Bubbles

Status: Locked

## Type

Change-bearing UI-only

## Objective

Add UI-only contextual help bubbles for technical IGY6 terms, preserving
existing labels and behavior.

## Baseline Facts

- The worktree was clean before this DIFF started.
- No active or in-progress DIFF existed before this DIFF.
- The web UI is a single Next.js page using plain CSS and small inline scripts
  for existing interactive controls.
- There is no web lint script; the web package defines `dev`, `build`, and
  `start`.
- Existing labels include technical terms such as Source, Collection Run,
  Vector Memory, Qdrant, Graph Memory, Work Item, Dispatch, Evidence Answer,
  Approval, Audit Event, and settings `.env` keys.

## Allowed Scope

- `docs/diffs/DIFF-078-ui-technical-term-help-bubbles.md`
- `README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/layout.tsx` only if needed
- Small UI-only helper/component files under `apps/web/src/app/components/` or
  `apps/web/src/app/lib/` only if needed

## Prohibited Scope

- No backend code changes.
- No API route changes.
- No database changes.
- No migrations.
- No Docker changes.
- No dependency changes.
- No settings `.env` behavior changes.
- No source, approval, or work-item behavior changes.
- No model calls.
- No ComfyUI or AI-stack features.
- No renaming API fields.
- No renaming database fields.
- No renaming files.
- No broad refactor.
- No unrelated cleanup.

## Required Tags

- Commit message must include `DIFF-078`.
- Final response must identify `DIFF-078`.

## Verification

- `git diff --check`
- `npm --prefix apps/web run build`
- `python3 -m compileall services/api services/worker`
- `npm --prefix apps/web run lint` is optional if a lint script exists.

## Completion Criteria

- Existing labels are preserved unless a small help marker is added.
- Technical/confusing terms have specific help bubbles.
- Help bubble text explains normal meaning, UI location, project purpose, and
  limitation or safety rule.
- Common terms are not cluttered with unnecessary bubbles.
- No backend, API, database, or settings behavior changed.
- No dependencies were added.
- README documents help bubble behavior.
- Prohibited scope was avoided.
- Verification results are recorded below.

## Verification Result

- `git diff --check` passed.
- `npm --prefix apps/web run build` passed.
- `python3 -m compileall services/api services/worker` passed.
- `npm --prefix apps/web run lint` was not run because `apps/web/package.json`
  has no `lint` script.
- Docker was not started because this DIFF is UI-help text only.
