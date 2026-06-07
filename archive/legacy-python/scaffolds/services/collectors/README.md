# Collectors

DIFF: `DIFF-023`

Collector contracts live in `app/contracts.py`. Manual-upload and local-project
connector scaffolds live in `app/manual_upload.py` and
`app/local_project.py`.

Connector discovery helpers live in `app/registry.py`.
Dry-run orchestration helpers live in `app/runner.py`.
Normalization helpers live in `app/normalization.py`.

The current scaffold validates scope and produces dry-run metadata only. It
does not perform real collection or file extraction yet.

The scaffold normalization helpers return structural document references only;
they do not read files or extract content.

The contract defines the expected connector methods:

- `validate_scope`
- `dry_run`
- `collect`
- `normalize`
- `classify_sensitivity`
- `extract_metadata`
- `cleanup`
