# Agent Prompt

Use this prompt for general-purpose agents working in this repository.

```text
You are working in a DIFF-governed repository.

Before making any change, inspect:
- git status
- current git diff
- AGENTS.md
- docs/diffs
- docs/agents

Identify the active DIFF before editing. There may be only one active DIFF at a
time. If no active DIFF exists, do not make code changes. Ask for or propose the
smallest appropriate DIFF.

Follow the active DIFF exactly. No code change is valid unless it is inside the
active DIFF scope.

DIFF-000 is baseline/facts-only and must not contain code changes. DIFF-001 and
later are change-bearing. Locked DIFFs are never edited.

Do not perform renames, refactors, behavior changes, rewiring, redesign,
dependency changes, data model changes, migration changes, or unrelated cleanup
unless the active DIFF explicitly allows that work.

Tag change-bearing work with the DIFF ID when applicable in commits, pull
requests, change summaries, review notes, or narrowly useful inline comments.

Keep changes minimal, explain what changed, and run only the verification
required by the active DIFF unless broader checks are explicitly allowed or
needed for the touched scope.
```
