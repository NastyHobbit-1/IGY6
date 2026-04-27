# DIFF Template

Use this template for new DIFF-governed work.

```md
# DIFF-XXX: Short Title

Status: Draft | Active | Locked

## Type

Baseline/facts-only | Change-bearing

## Objective

State the exact outcome this DIFF authorizes.

## Baseline Facts

Record relevant current-state facts. For DIFF-000, this section is the primary
content and must not authorize code changes.

## Allowed Scope

List files, directories, commands, or behavior areas that may be changed.

## Prohibited Scope

List files, directories, commands, behavior areas, and change types that must
not be touched.

Unless explicitly allowed here, the following are prohibited:

- Renames.
- Refactors.
- Behavior changes.
- Rewiring.
- Redesign.
- Dependency changes.
- Data model changes.
- Migration changes.
- Formatting-only churn outside touched scope.

## Required Tags

State how DIFF IDs must be attached to changes, commits, pull requests, or
review notes.

## Verification

List required checks and expected outcomes.

## Completion Criteria

Define the exact conditions required to mark this DIFF complete.

## Out Of Scope Follow-Up

List related work that must remain outside this DIFF.
```

## Template Rules

- `DIFF-000` must be baseline/facts-only and must not include code changes.
- `DIFF-001` and later may be change-bearing only within explicit scope.
- Only one DIFF may be active at a time.
- Locked DIFFs must never be edited.
- No code change is valid unless it is inside the active DIFF scope.
