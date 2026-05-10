# IGY6

IGY6 is a local-first adaptive intelligence foundation for collecting authorized
information, preserving evidence, and reviewing what the system knows through
auditable local workflows.

It is not a finished autonomous agent, not ComfyUI, not a generic chatbot, and
not a local AI-stack model manager. The current implementation does not generate
LLM answers and does not call external models. It supports deterministic local
evidence, retrieval, metadata, reporting, and review workflows. System-changing
actions are not implemented, and the system is read-only or scaffolded by
default unless a current endpoint explicitly records local metadata.

## Current Capabilities

Implemented or scaffolded behavior in the current repository:

| Area | Current behavior |
| --- | --- |
| Local stack | Docker Compose starts the web UI, API, PostgreSQL, Redis, Celery worker, Celery beat, Qdrant, Neo4j, MLflow, and Phoenix on localhost-bound ports. |
| Web UI | Next.js dark AI-console shell with source, evidence, memory, work, approval, report, audit, retrieval preview, and MVP Action Console panels. |
| API | FastAPI gateway with health, source, approval, collection, artifact, evidence, retrieval, chat, graph, vector, feedback, outcome, report, improvement, experiment, work-item, and audit routes. |
| State and audit | PostgreSQL stores sources, permissions, collection runs, artifacts, normalized documents, chunks, evidence, claims, patterns, hypotheses, predictions, recommendations, work items, approvals, feedback, outcomes, reports, experiments, improvements, and audit events. |
| Source registry | Sources can be registered with type, location, sensitivity, trust, enabled state, metadata, and optional permission. |
| Source permissions | Permissions store scope JSON, allowed operations, external model policy, and approval requirement. |
| Approvals | Approval requests can be created and approved or denied. Collection approval matching requires exact source, permission, and operation payload values. |
| Collection | Manual upload collection supports base64 UTF-8 text. Local project collection supports scoped paths under a container-visible source directory. |
| Artifacts | Raw artifacts are stored in local content-addressed storage with PostgreSQL metadata. |
| Normalization | Worker normalization reads raw artifacts as UTF-8 text and creates normalized text documents. |
| Chunking and evidence | Worker chunking creates chunks and evidence items from normalized documents. |
| Vector memory | Deterministic local hash embeddings are upserted to Qdrant. Qdrant search and hydrated retrieval trails are available. |
| Retrieval policy | Retrieval hydration excludes hits from disabled sources and preserves hits with no source. |
| Chat | `/chat/retrieval-preview` returns retrieval context with `answer_status: not_generated`. `/chat/evidence-answer` returns a deterministic local evidence-summary packet. |
| Work queue | Work items enforce intent verification metadata before queued dispatch. Supported dispatch types are listed below. |
| Graph memory | Neo4j schema constraints, lineage sync, and relationship lookup are available as foundation endpoints. |
| Analysis metadata | Patterns, hypotheses, predictions, and recommendations can be recorded against existing evidence. Baseline pattern detection exists. |
| Feedback and outcomes | Feedback and outcome metadata can be recorded. Weak feedback can create proposed improvement items. Source trust feedback can update source trust/enabled state. |
| Reports | Report metadata can be created and rendered to deterministic local markdown artifacts from local metadata counts and recent records. |
| Improvement and experiments | Improvement items and experiment run metadata can be recorded. Production self-improvement execution is not implemented. |
| Reserved services | MLflow and Phoenix run as reserved local services, but production experiment execution and tracing integration are not complete. |
| Settings | The UI can edit the local `.env` through a backend verify-dry-run-before-save workflow with secret masking, backups, atomic writes, and audit events. |

## Not Implemented Yet

Current boundaries:

- No LLM-generated answers.
- No autonomous source collection.
- No browser automation.
- No ComfyUI or local AI-stack functionality.
- No image generation, model manager, model selector, or model download flow.
- No external model calls.
- No system-changing actions.
- No automatic predictions or advice generation.
- No production self-improvement execution.
- No production method changes.
- No binary, PDF, image, audio, or rich document normalization.
- No full authentication system.
- No advanced graph reasoning.
- No advanced ML forecasting.
- No full connector suite for router, web account, PC diagnostics, conversation history, or approved website collection.

## Services

Default local endpoints:

| Service | URL |
| --- | --- |
| Web UI | `http://127.0.0.1:3000` |
| API | `http://127.0.0.1:8000` |
| API docs | `http://127.0.0.1:8000/docs` |
| Qdrant | `http://127.0.0.1:6333` |
| Neo4j Browser | `http://127.0.0.1:7474` |
| MLflow | `http://127.0.0.1:5000` |
| Phoenix | `http://127.0.0.1:6006` |

The Compose file binds published service ports to `127.0.0.1`.

## Storage Model

The repository is intended to hold code, configuration, and documentation.
Runtime/private/persistent data lives in a separate data folder controlled by
`IGY6_DATA_ROOT`.

Default:

```text
IGY6_DATA_ROOT=../IGY6_Data
```

Docker Compose bind-mounts `IGY6_DATA_ROOT` into the API, worker, and beat
containers as `/workspace/storage`, so the app can keep using stable container
paths:

```text
ARTIFACT_STORE_PATH=/workspace/storage/artifacts
EXPORT_STORE_PATH=/workspace/storage/exports
ENV_BACKUP_DIR=/workspace/storage/env_backups
ENV_FILE_PATH=/workspace/project/.env
```

Persistent service data is also stored under `IGY6_DATA_ROOT`. IGY6 no longer
uses Docker named volumes for app persistent data.

Expected data folder contents:

```text
IGY6_Data/
  artifacts/
  exports/
  env_backups/
  postgres/
  qdrant/
  neo4j/
    data/
    logs/
  mlflow/
  phoenix/
```

Recommended Windows layout:

```text
D:/Projects/IGY6
D:/Projects/IGY6_Data
```

`.env` example:

```text
IGY6_DATA_ROOT=D:/Projects/IGY6_Data
```

Use forward slashes in `.env` paths on Windows. Do not use backslash paths.

Backup and move rules:

- Stop the Docker stack before copying, moving, zipping, syncing, or backing up
  runtime data.
- Copy both the repo folder and the `IGY6_DATA_ROOT` folder when moving IGY6 to
  another machine or drive.
- Do not put the live data folder inside OneDrive, iCloud, Dropbox, Google
  Drive, or another sync tool.
- Do not commit runtime data.

Existing users:

- Before DIFF-079, data may have lived in Docker named volumes.
- This change does not migrate old named-volume data automatically.
- Starting with a fresh `IGY6_DATA_ROOT` can look like a fresh system.
- If old named-volume data needs migration, do it in a separate manual
  procedure or future DIFF.
- Do not delete old Docker named volumes until you confirm the data is no
  longer needed.

## Run Locally

Prerequisites:

- Docker and Docker Compose.
- Git.
- A shell. Examples below use bash-style commands.

1. Clone and open the repository.

```bash
git clone https://github.com/NastyHobbit-1/IGY6.git
cd IGY6
```

2. Create a local environment file.

```bash
cp .env.example .env
```

3. Review `.env` and choose the data root.

The checked-in example uses local-only placeholder values such as
`change-me-local-only`. Keep secrets local and do not commit `.env`.

For the default sibling data folder:

```bash
mkdir -p ../IGY6_Data
```

PowerShell equivalent:

```powershell
New-Item -ItemType Directory -Force -Path ..\IGY6_Data
```

On Windows, an absolute `.env` value should use forward slashes:

```text
IGY6_DATA_ROOT=D:/Projects/IGY6_Data
```

The settings editor expects the API container to see the project `.env` at
`ENV_FILE_PATH=/workspace/project/.env` and to place backups under
`ENV_BACKUP_DIR=/workspace/storage/env_backups`. These are local container paths
defined in `.env.example` and mounted by Docker Compose.

4. Start the stack.

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

Relative `IGY6_DATA_ROOT` values are resolved by Docker Compose from the
Compose file location. Use an absolute forward-slash path when you want a
specific folder outside the repository, especially on Windows.

5. Open the web UI.

```text
http://127.0.0.1:3000
```

6. Open the API or API docs.

```text
http://127.0.0.1:8000
http://127.0.0.1:8000/docs
```

7. Stop the stack.

```bash
docker compose -f infra/docker-compose.yml --env-file .env down
```

## First Use Workflow

Some steps are easiest through API calls today. The web UI shows current records
and provides an MVP Action Console for several existing FastAPI endpoints.

1. Start the stack.
2. Check API readiness with `/health/ready`.
3. Create a source.
4. Create a source permission if one was not created with the source.
5. If the permission has `approval_required: true`, create an approval whose
   payload exactly matches `source_id`, `source_permission_id`, and collection
   `operation`.
6. Approve the approval.
7. Run a dry-run collection preview.
8. Run manual upload collection or local project collection.
9. List work items and dispatch the queued `collection_normalization` item.
10. List work items again and dispatch the chained `document_chunking` item.
11. List work items again and dispatch the chained `chunk_vector_upsert` item.
12. Use chat retrieval preview or evidence answer.
13. Review retrieved chunks, source trails, evidence, audit events, and reports.
14. Record feedback, outcomes, or reports if desired.

Only ingested, normalized, chunked, and vector-upserted evidence can be returned
by retrieval.

## API Examples

Set a local API variable:

```bash
API=http://127.0.0.1:8000
```

Replace placeholder IDs such as `SOURCE_ID`, `SOURCE_PERMISSION_ID`,
`APPROVAL_ID`, `WORK_ITEM_ID`, and `REPORT_ID` with IDs returned by earlier
calls.

### Health

```bash
curl "$API/health/live"
curl "$API/health/ready"
```

### Create a manual upload source with a permission

```bash
curl -X POST "$API/sources" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Manual UTF-8 notes",
    "source_type": "manual_upload",
    "location": null,
    "sensitivity": "internal",
    "permission": {
      "scope_json": {
        "description": "Authorized manual UTF-8 text notes only"
      },
      "allowed_operations": ["dry_run", "read", "collect"],
      "external_model_policy": "blocked",
      "approval_required": true
    }
  }'
```

Use the response `id` as `SOURCE_ID` and the first permission `id` as
`SOURCE_PERMISSION_ID`.

### Create an approval for manual upload collection

```bash
curl -X POST "$API/approvals" \
  -H "Content-Type: application/json" \
  -d '{
    "request_type": "manual_upload_collection",
    "requested_by_actor_id": "local-owner",
    "request_payload_json": {
      "source_id": "SOURCE_ID",
      "source_permission_id": "SOURCE_PERMISSION_ID",
      "operation": "manual_upload_collection"
    }
  }'
```

### Approve the approval

```bash
curl -X POST "$API/approvals/APPROVAL_ID/decision" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "approved",
    "decided_by_actor_id": "local-owner",
    "decision_reason": "Approved local manual UTF-8 test collection"
  }'
```

### Run a dry-run

```bash
curl -X POST "$API/collection-runs/dry-run" \
  -H "Content-Type: application/json" \
  -d '{
    "source_id": "SOURCE_ID",
    "source_permission_id": "SOURCE_PERMISSION_ID",
    "requested_by_actor_id": "local-owner"
  }'
```

### Run manual upload collection with base64 UTF-8 text

```bash
TEXT_B64=$(printf 'IGY6 local UTF-8 evidence note.' | base64 | tr -d '\n')

curl -X POST "$API/collection-runs/manual-upload" \
  -H "Content-Type: application/json" \
  -d "{
    \"source_id\": \"SOURCE_ID\",
    \"source_permission_id\": \"SOURCE_PERMISSION_ID\",
    \"approval_id\": \"APPROVAL_ID\",
    \"filename\": \"manual-note.txt\",
    \"mime_type\": \"text/plain\",
    \"content_base64\": \"$TEXT_B64\",
    \"requested_by_actor_id\": \"local-owner\"
  }"
```

The collection response summary includes raw artifact IDs and the queued
normalization work item ID.

### List work items

```bash
curl "$API/work-items"
```

### Dispatch a queued work item

```bash
curl -X POST "$API/work-items/WORK_ITEM_ID/dispatch" \
  -H "Content-Type: application/json" \
  -d '{
    "actor_id": "local-owner"
  }'
```

Repeat list and dispatch for the chained `document_chunking` and
`chunk_vector_upsert` work items.

### Run chat retrieval preview

```bash
curl -X POST "$API/chat/retrieval-preview" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What does the system know?",
    "limit": 5
  }'
```

### Run deterministic evidence answer

```bash
curl -X POST "$API/chat/evidence-answer" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What does the system know?",
    "limit": 5
  }'
```

### Create a report

```bash
curl -X POST "$API/reports" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Local evidence summary",
    "report_type": "summary",
    "status": "requested",
    "requested_by_actor_id": "local-owner"
  }'
```

### Render a report

```bash
curl -X POST "$API/reports/REPORT_ID/render" \
  -H "Content-Type: application/json" \
  -d '{
    "actor_id": "local-owner",
    "notes": "Rendered from local metadata only."
  }'
```

Rendering creates a local markdown artifact and marks the report `ready`.

## Local Project Collection

Local project collection is implemented for scoped files that the API container
can see.

- The source `source_type` must be `local_project`.
- The source `location` must be an existing directory inside the
  container-visible environment. With the default Compose file, the repository
  `storage` directory is mounted into the API container at `/workspace/storage`.
- The permission `scope_json` must include a non-empty `paths` list.
- Relative paths are resolved under the source location.
- Absolute or relative paths must not escape the source location.
- `max_files` defaults to `100` when omitted and must be at least `1`.
- `max_file_bytes` defaults to `1000000` when omitted and must be at least `1`.
- Symlinks are skipped.
- Files over `max_file_bytes` are skipped.
- Artifacts are stored as raw bytes, but current worker normalization supports
  UTF-8 text only. Binary files may collect and then fail normalization with an
  explicit UTF-8-only error.

Create an approval for local project collection with
`operation: "local_project_collection"` and use
`/collection-runs/local-project` to collect.

Example source payload shape:

```json
{
  "name": "Container-visible project notes",
  "source_type": "local_project",
  "location": "/workspace/storage/project-notes",
  "sensitivity": "internal",
  "permission": {
    "scope_json": {
      "paths": ["."],
      "max_files": 25,
      "max_file_bytes": 1000000
    },
    "allowed_operations": ["dry_run", "read", "collect"],
    "external_model_policy": "blocked",
    "approval_required": true
  }
}
```

## Work-Item Dispatch Notes

Work items require recorded intent verification metadata before they can be
queued or dispatched. Only `queued` work items can dispatch.

Supported dispatch types:

- `collection_normalization`
- `document_chunking`
- `chunk_vector_upsert`

Unsupported work-item dispatch types return an error. Worker tasks may create
chained queued work items after successful normalization or chunking.

Terminal work items such as `completed`, `failed`, or `canceled` do not move
casually back to `queued` or `running`.

## Retrieval And Chat Notes

- `POST /chat/retrieval-preview` returns retrieval context only with
  `answer_status: not_generated`.
- `POST /chat/evidence-answer` returns a deterministic local evidence packet
  with facts, assumptions, inferences, uncertainty, missing information, source
  trails, and retrieval context.
- No LLM answer is generated.
- No external model call happens.
- No hidden reasoning or autonomous action happens.
- Disabled sources are filtered out of hydrated retrieval results.
- Retrieval can only find chunks that have been ingested, normalized, chunked,
  and vector-upserted.
- Retrieval scores are similarity signals, not proof of correctness.

## UI Usage

The web UI is a dark AI-console shell for the existing IGY6 workflows. It shows:

- Local readiness and status.
- Source registry and source permission summaries.
- Collection runs, raw artifacts, normalized documents, chunks, evidence items,
  and claims.
- Vector memory and graph schema status.
- Patterns, hypotheses, predictions, and recommendations.
- Work items, approvals, feedback, outcomes, reports, and audit events.
- Chat Retrieval Preview, which calls same-origin
  `/api/chat/retrieval-preview`.
- Settings, which calls same-origin `/api/settings/env`,
  `/api/settings/env/verify`, and `/api/settings/env/apply`.
- MVP Action Console controls that call existing FastAPI endpoints only.

Scaffolded visual controls are disabled or labeled honestly. The UI does not add
ComfyUI, image generation, model management, model downloads, autonomous agents,
or AI-stack backend behavior.

## Help Bubbles

IGY6 includes hover/focus help bubbles for technical terms in the web UI. Keep
the pointer or keyboard focus on the small `?` marker for about one second to
see a plain-language explanation.

Help bubbles are cosmetic only. They do not change API names, saved data,
backend behavior, settings behavior, or labels stored in the database. Common
words such as Completed, Failed, Reports, Settings, Search, and Save may not
have bubbles because they are already clear in context.

Exact technical keys remain visible where needed, especially in Settings. For
example, `ENV_FILE_PATH`, `ENV_BACKUP_DIR`,
`IGY6_DATA_ROOT`, `QDRANT_CHUNK_VECTOR_SIZE`,
`EXTERNAL_MODEL_POLICY_DEFAULT`, and `APPROVAL_REQUIRED_DEFAULT` keep their
exact names while explaining what they control and what restart or safety
limits apply.

## Settings Page

The Settings section edits the local IGY6 `.env` only. It is a sensitive,
system-changing workflow, so the UI never blindly saves edits.

Current flow:

1. The UI loads sanitized settings from `GET /settings/env`.
2. Editable allowlisted keys are grouped by App / Web, PostgreSQL, Redis /
   Celery, Qdrant, Neo4j, MLflow, Phoenix, Storage, and Policy / Safety.
3. Secret values such as passwords and URLs containing passwords are masked by
   default.
4. To change a secret, use the replacement field; the current secret is not
   shown in plain text.
5. Click `Verify Dry Run`.
6. The API validates the proposed candidate without writing `.env`.
7. If dry-run passes, the API returns a candidate hash/verification token.
8. `Save Settings` is enabled only for the exact candidate that passed dry-run.
9. Save revalidates the same candidate, requires the matching token, creates a
   timestamped backup, atomically writes normalized `.env` content, and records
   an audit event without secret values.

Dry-run validates:

- Unknown keys are not editable. Existing unknown `.env` keys are shown as
  read-only unmanaged keys and preserved.
- `IGY6_DATA_ROOT` is not empty, is not a filesystem or drive root, does not use
  Windows backslashes, and does not contain traversal except the default
  `../IGY6_Data`.
- Required allowlisted keys are present.
- Ports are valid integers from `1` to `65535`.
- Boolean values parse for `SINGLE_USER_MODE` and
  `APPROVAL_REQUIRED_DEFAULT`.
- URLs and URIs are syntactically plausible.
- `DATABASE_URL`, `NEO4J_URI`, and `QDRANT_URL` agree with their matching host,
  port, user, password, and database fields where applicable.
- Storage paths are absolute container paths and do not contain obvious path
  traversal.
- External model policy and audit log level values are constrained.
- Qdrant vector size is a positive integer, with a warning that changing it can
  require rebuilding vector storage.
- Changing `IGY6_DATA_ROOT` warns that Docker stack restart/recreate is required,
  existing data is not migrated, and the target folder must exist or be
  creatable by Docker.
- Docker Compose config validation is attempted from the API runtime only when
  Docker CLI and the Compose file are available. If unavailable, dry-run returns
  a warning instead of failing solely for that reason.

Dry-run passing means the candidate configuration is syntactically and
structurally valid. It does not guarantee every service will work after restart.

Saved settings are written to `.env`; they are not applied to running
containers. Restart or recreate the Docker stack before expecting changed values
to take effect. No automatic restart or container recreate is implemented.
Saving a new `IGY6_DATA_ROOT` does not move existing data.

Backups are written to `ENV_BACKUP_DIR`. Automatic rollback is not implemented
in this DIFF. Manual rollback is:

```bash
cp storage/env_backups/.env.TIMESTAMP.bak .env
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

Use the actual backup path returned by the Settings save response.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| Docker daemon unavailable | Confirm Docker Desktop or the Docker daemon is running, then rerun the Compose command. |
| Port already in use | Another local process may already be bound to `3000`, `8000`, `5432`, `6379`, `6333`, `7474`, `7687`, `5000`, or `6006`. Stop the other process or change the local port mapping deliberately. |
| API readiness is not `ok` | Call `/health/ready` and inspect which dependency is degraded: PostgreSQL, Redis, Qdrant, Neo4j, MLflow, or Phoenix. |
| Worker not responding | Check the `worker` service logs and run the Celery ping command in the verification section. |
| Migrations not current | Run `docker compose -f infra/docker-compose.yml --env-file .env exec -T api alembic current` and inspect API startup logs. |
| Qdrant search returns no hits | Make sure collection, normalization, document chunking, and vector upsert work items have completed. Also verify the vector collection exists. |
| Manual upload rejected as non-UTF-8 | Manual upload collection currently accepts UTF-8 text only. Convert the content to UTF-8 text before upload. |
| Local project path escapes source location | Keep every permission path under the source `location`; scoped paths cannot escape that root. |
| No evidence answer is returned | There may be no ingested, chunked, and vector-upserted evidence yet, or matching evidence may belong to a disabled source. |
| Settings dry-run passes but services fail after restart | Dry-run validates structure, not live service availability. Review the changed keys and Docker logs after restart. |
| Settings save fails because `.env` is not writable | Confirm Docker Compose mounted the repository at `/workspace/project` and that the local `.env` file exists and is writable. |
| Fresh system after changing storage | `IGY6_DATA_ROOT` may point at an empty data folder. This DIFF does not migrate old Docker named-volume data automatically. |
| `npm --prefix apps/web run lint` is unavailable | The web package currently defines `dev`, `build`, and `start`, but no `lint` script. Use `npm --prefix apps/web run build` for web build verification. |

## Verification

Useful local checks:

```bash
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
docker compose -f infra/docker-compose.yml --env-file .env.example up -d
curl http://127.0.0.1:8000/health/ready
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T api alembic current
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T worker celery -A app.celery_app:celery_app inspect ping
npm --prefix apps/web run build
```

Use `.env.example` for configuration validation and disposable local smoke
checks. Use `.env` for normal local runs after reviewing local-only values.

For documentation-only changes, the narrow repository checks are:

```bash
git diff --check
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

## Development Notes

- Follow `AGENTS.md` and the DIFF process in `docs/diffs`.
- Do not add features outside the active DIFF.
- Do not treat retrieved content as trusted instructions.
- Do not claim generated answers, external model calls, browser automation,
  ComfyUI, image generation, or autonomous system-changing actions unless those
  behaviors are implemented in code and covered by an active DIFF.
