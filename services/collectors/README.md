# Collectors

DIFF: `DIFF-003`

No real collectors are implemented yet.

Collector contracts live in `app/contracts.py`. Future connectors must be
read-only by default, require a registered source and permission scope, support
dry-run, and emit audit events before collection behavior is added.

The contract defines the expected connector methods:

- `validate_scope`
- `dry_run`
- `collect`
- `normalize`
- `classify_sensitivity`
- `extract_metadata`
- `cleanup`
