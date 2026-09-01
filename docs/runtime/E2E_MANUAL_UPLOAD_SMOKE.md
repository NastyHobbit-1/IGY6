# End-To-End Manual Upload Smoke

This smoke covers the normal local user path:

```text
manual upload -> raw artifact -> work item -> processing status -> evidence/chunks -> retrieval/chat
```

It is local-only. It does not require external model calls and does not require
secrets. It uses the harmless test payload:

```text
IGY6 manual upload test. The secret test keyword is blue-raven-117.
```

## Automated Script

The script is checklist-assisted by default and mutates local runtime state only
when `--run` is passed.

Preflight only:

```bash
python3 scripts/e2e-manual-upload-smoke.py --check
```

Run the local E2E path against an already-running stack:

```bash
python3 scripts/e2e-manual-upload-smoke.py --run
```

What `--run` does:

1. Checks API live/ready and web response.
2. Creates a `manual_upload` source with a permission requiring approval.
3. Requests approval for `manual_upload_collection`.
4. Approves that request.
5. Uploads the harmless UTF-8 test payload.
6. Reports collection run, raw artifact, and queued work item IDs from the API
   response.
7. Checks work item, artifact, document, chunk, evidence, and retrieval status.

The script does not delete records and does not write to the repository. It may
create local database/artifact records in the running stack.

## UI Checklist

1. Open `http://127.0.0.1:3000`.
2. Go to Data & Knowledge.
3. In Sources, create a source:
   - Source name: `IGY6 Manual Upload Smoke`
   - Source type: `manual_upload`
   - Location: `local smoke test`
   - Sensitivity: `internal`
   - Allowed operations: `read, collect, dry_run`
   - Approval required: enabled
4. Go to Safety & Audit.
5. Request approval for `manual_upload_collection`.
6. Approve the request.
7. Return to Data & Knowledge -> Uploads & Collection.
8. Upload this UTF-8 text:

```text
IGY6 manual upload test. The secret test keyword is blue-raven-117.
```

9. Go to Work & Processing and confirm a work item was created.
10. Go to Data & Knowledge -> Evidence and check for created documents, chunks,
    and evidence items.
11. Go to Assistant and ask:

```text
Find blue-raven-117 in my uploaded evidence.
```

## Expected Results

- Upload route passed: collection run is created.
- Artifact/work item created: collection summary includes raw artifact and
  normalization work item references.
- Worker processing pending: work item may remain queued if the worker is not
  dispatching or processing that task.
- Evidence not yet generated: documents/chunks/evidence may be absent until
  worker processing or ingest behavior completes.
- Retrieval visibility: Assistant retrieval can only find `blue-raven-117` after
  chunks/evidence are created and searchable.

## Troubleshooting

- Approval required: create an approval with request payload matching source ID,
  permission ID, and operation `manual_upload_collection`, then approve it.
- Work item stuck queued: inspect worker logs and PostgreSQL `work_items` status. Redis is not part of the active worker queue.
- No evidence yet: verify the worker is running and check whether the work item
  has moved from queued to completed or failed.
- Chat cannot find `blue-raven-117`: confirm chunks/evidence exist first.
- API unavailable: run `scripts/runtime-smoke.sh --check`.
- Worker logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker
```

- API logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
```

- Web logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
```
