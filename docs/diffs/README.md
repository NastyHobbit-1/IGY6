# DIFF Workflow

This directory contains DIFF-governed task specifications.

Before making repository changes, agents must inspect this directory, identify
the active DIFF, and keep all edits within that DIFF's stated scope.

Each DIFF should define:

- Objective.
- Allowed files or areas.
- Explicitly prohibited changes.
- Required verification steps.
- Completion criteria.
- Any follow-up work that must remain out of scope.

If no active DIFF exists, agents must not infer broad implementation scope from
the product plan. They should propose the smallest next DIFF or ask for the
missing task definition before changing code.
