# DIFF-023: Collector Normalization Scaffold

Status: Locked

## Type

Change-bearing.

## Objective

Add collector-local normalization helpers and wire the existing connector
scaffolds to return structural normalized document references and sensitivity
labels.

This DIFF does not authorize real extraction, file reading, artifact writes,
API integration, worker scheduling, or filesystem traversal.

## Baseline Facts

- DIFF-000 through DIFF-022 are locked.
- Manual-upload and local-project connector scaffolds exist.
- A connector registry and dry-run runner already exist in the collectors
  package.
- The collectors package still lacks a shared normalization helper.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-023-collector-normalization-scaffold.md`
- `services/collectors/README.md`
- `services/collectors/app/__init__.py`
- `services/collectors/app/local_project.py`
- `services/collectors/app/manual_upload.py`
- `services/collectors/app/normalization.py`

Allowed behavior:

- Add a shared helper for scaffold normalized document references.
- Add a shared helper for scaffold sensitivity classification.
- Implement the connector `normalize` methods using the shared helper.
- Keep the connector `classify_sensitivity` methods structural and pure.
- Export the normalization helpers from the collectors package.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- API endpoint changes.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Real collection execution.
- Artifact collection.
- File content extraction.
- Filesystem traversal.
- Artifact writes.
- Export generation.
- Browser automation.
- External model calls.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-023` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/collectors python3 -c "from app.contracts import RawArtifactRef; from app.normalization import build_normalized_document_ref, classify_sensitivity_label; raw = RawArtifactRef(id='raw-test', source_id='src-test', content_hash='abc123', storage_path='/tmp/raw.txt'); doc = build_normalized_document_ref(raw, text_content='hello'); print(doc.id, doc.sensitivity, classify_sensitivity_label(doc.sensitivity))"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/collectors python3 -c "from app.contracts import RawArtifactRef; from app.normalization import build_normalized_document_ref, classify_sensitivity_label; raw = RawArtifactRef(id='raw-test', source_id='src-test', content_hash='abc123', storage_path='/tmp/raw.txt'); doc = build_normalized_document_ref(raw, text_content='hello'); print(doc.id, doc.sensitivity, classify_sensitivity_label(doc.sensitivity))"`

## Completion Criteria

This DIFF is complete when:

- A shared normalization helper module exists.
- The existing connector scaffolds use the helper for normalization.
- The helper returns structural normalized document references only.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Real raw artifact creation.
- File and folder extraction.
- Evidence generation.
- API integration with normalized document creation.
- Worker integration.
