# Collectors

DIFF: `DIFF-019`

Collector contracts live in `app/contracts.py`. A manual-upload connector
scaffold lives in `app/manual_upload.py`.

The current scaffold validates scope and produces dry-run metadata only. It does
not perform real collection or normalization yet.

The contract defines the expected connector methods:

- `validate_scope`
- `dry_run`
- `collect`
- `normalize`
- `classify_sensitivity`
- `extract_metadata`
- `cleanup`
