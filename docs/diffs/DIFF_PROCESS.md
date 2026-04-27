# DIFF Process

This repository uses a strict DIFF-governed workflow. A DIFF is the written
scope contract for a unit of work. Agents and humans must follow the active DIFF
exactly.

## Required Pre-Work

Before editing, inspect:

- Current git status.
- Current git diff.
- Root `AGENTS.md`.
- `docs/diffs`.
- `docs/agents`.

Do not start implementation from memory, product ambition, or inferred intent.

## DIFF Numbering

- `DIFF-000` is baseline/facts-only. It records known repository facts,
  constraints, decisions, inventory, or observations. It must not include code
  changes.
- `DIFF-001` and later are change-bearing. They may authorize changes only
  within their explicit scope.

## Active DIFF

There may be only one active DIFF at a time.

The active DIFF must define:

- DIFF ID.
- Status.
- Objective.
- Allowed files or areas.
- Explicitly prohibited changes.
- Verification steps.
- Completion criteria.

If no active DIFF exists, agents must stop before code edits and ask for or
propose the smallest appropriate DIFF.

## Locked DIFFs

Locked DIFFs are historical records and must never be edited. If a correction is
needed, create a new DIFF that references the locked DIFF and explains the
change.

## Scope Rules

No code change is valid unless it is inside the active DIFF scope.

Do not perform any of the following unless the active DIFF explicitly allows it:

- Renames.
- Refactors.
- Behavior changes.
- Rewiring.
- Redesign.
- Dependency changes.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

Do not make unrelated cleanup changes.

## DIFF Tagging

When a change-bearing DIFF applies, code changes must be tagged with the DIFF ID
where the project workflow can see it. Acceptable tags include commit messages,
pull request descriptions, change summaries, review notes, and narrowly scoped
inline comments when an inline note is useful. Do not add noisy comments solely
to satisfy tagging.

## Verification

Run only the verification required by the active DIFF unless broader checks are
explicitly allowed or necessary to validate the touched scope. If verification
cannot be run, record why.

## Completion

A DIFF is complete only when:

- All requested changes are inside scope.
- Prohibited changes were avoided.
- Required verification was run or blocked with a clear reason.
- The final status and summary identify the DIFF ID.
