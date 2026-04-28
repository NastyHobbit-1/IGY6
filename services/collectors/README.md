# Collectors

DIFF: `DIFF-020`

Collector contracts live in `app/contracts.py`. Manual-upload and local-project
connector scaffolds live in `app/manual_upload.py` and
`app/local_project.py`.

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
