# Policy Package

DIFF: `DIFF-004`

Shared approval, sensitivity, and source safety policy definitions live in
`app/rules.py`.

The current package provides constants and pure helper functions only. It is not
yet wired into API, worker, collector, or UI runtime enforcement.

All future source access and sensitive/system-changing actions must be modeled
as policy-checked and auditable before execution.
