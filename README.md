# IGY6

IGY6 is a local-first evidence workspace. It helps you register data sources,
upload or collect allowed material, turn that material into evidence, search it,
run safe local assistant actions, review approvals, and audit what happened.

It is not a finished autonomous agent, not a generic chatbot, not ComfyUI, and
not an external-model workflow by default. Current answers are evidence previews
or deterministic evidence packets from local records.

## Current Backend Posture

IGY6's API path is Rust-native after DIFF-138 and DIFF-140 records the final
Rust API cutover audit. Route parity records zero FastAPI routes missing from
Rust and Docker Compose no longer wires the FastAPI `legacy-api` fallback
service into the runtime API path.

DIFF-139 archives the legacy FastAPI API source at
`archive/legacy-python/services-api`. This does not claim Python worker parity:
Python/Celery `worker` and `beat` services remain part of the local stack until
a later DIFF proves their replacement or explicitly documents long-term
retention. DIFF-141 recommends migrating worker execution to Rust one job family
at a time while retaining Python/Celery until execution parity is complete.
DIFF-143 through DIFF-145 add Rust `collection_normalization`,
`document_chunking`, and `chunk_vector_upsert` execution parity planning and
executor contracts. DIFF-146 decides the worker process cutover is not ready:
beat posture and live worker process ownership remain Python/Celery-backed
until a Rust worker runtime actually polls, claims, executes DB/audit writes,
executes Qdrant side effects, and resolves scheduler posture. DIFF-147 adds a
safe Rust worker CLI/runtime harness with `--check`, `--dry-run`, and `--once`
modes, but live execution remains disabled and Python/Celery still owns live
worker side effects. DIFF-148 adds an explicit one-job canary gate with
`--once --canary-live --canary-work-item ID`, plus side-effect verification
planning. DIFF-149 implements the gated one-job Rust live side-effect executor
for the selected canary work item only: PostgreSQL claim/status writes,
worker audit events, scoped artifact reads, parity DB writes, and Qdrant
collection/point work for `chunk_vector_upsert`. It does not enable broad queue
polling, Compose Rust worker ownership, beat replacement, or Rust-only runtime.

Current web-used route parity is tracked by:

```bash
python3 scripts/rust-route-parity.py --check
```

The web UI should show the same truth: local-first, evidence-only, no external
model by default, and approval-gated for system-changing actions.

Optional local LLM support is planned but disabled by default. The initial
planned provider is Ollama running locally. Deterministic evidence answers remain
the fallback, and any future LLM answer must cite retrieved evidence or say
there is insufficient evidence.

## Start And Stop Locally

Create `.env` from `.env.example` before normal local runs.

Start the stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
```

Stop the stack:

```bash
docker compose -f infra/docker-compose.yml --env-file .env down
```

Show running services:

```bash
docker compose -f infra/docker-compose.yml --env-file .env ps
```

Follow logs:

```bash
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200
```

Do not use `down -v` as a normal stop command. `down -v` can delete stored
Docker volume data.

Run the safe runtime smoke check against an already-running stack:

```bash
scripts/runtime-smoke.sh --check
```

The smoke script validates Docker Compose config, expected running services, API
live/ready endpoints, and the web UI. By default it does not start or stop
anything. To start explicitly:

```bash
scripts/runtime-smoke.sh --start --detached
```

To stop explicitly:

```bash
scripts/runtime-smoke.sh --stop
```

The stop command uses `docker compose down`, never `down -v`.

## Simple WSL Aliases

Add aliases like these to your shell profile, adjusting the path:

```bash
alias igy6-start='cd /home/nasty/projects/IGY6 && docker compose -f infra/docker-compose.yml --env-file .env up --build'
alias igy6-stop='cd /home/nasty/projects/IGY6 && docker compose -f infra/docker-compose.yml --env-file .env down'
alias igy6-ps='cd /home/nasty/projects/IGY6 && docker compose -f infra/docker-compose.yml --env-file .env ps'
alias igy6-logs='cd /home/nasty/projects/IGY6 && docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200'
```

Then use:

```bash
igy6-start
igy6-ps
igy6-logs
igy6-stop
```

## Local URLs

| Service | URL |
| --- | --- |
| Web UI | `http://127.0.0.1:3000` |
| API gateway | `http://127.0.0.1:8000` |
| API readiness | `http://127.0.0.1:8000/health/ready` |

## UI Navigation

The web UI is organized by workflow:

- Home: system status, service readiness, recent data, recent work, recent audit,
  and next recommended action.
- Assistant: one place to ask evidence questions, preview safe actions, request
  approval, run read-only actions, and view action results.
- Data & Knowledge: sources, uploads, collection runs, raw artifacts, documents,
  chunks, evidence, memory, analysis, and search.
- Work & Processing: queue status, work item detail, dispatch status, worker
  readiness, and the processing pipeline.
- Reports: report list, report detail, render controls, and output/status.
- Safety & Audit: approvals, audit log, safety rules, local-first state, and
  external-model policy.
- Settings: environment status, redacted config, verify/apply settings, runtime
  status, storage paths, and developer diagnostics.

Old navigation was split across Chat, Agent Command, Sources, Evidence, Memory,
Work Queue, Approvals, Audit, Reports, and Settings. Advanced IDs, raw JSON,
approval IDs, and route/debug details are still available, but they are under
Advanced sections instead of the primary workflow.

## Quickstart For Normal PC Users

1. Open `http://127.0.0.1:3000`.
2. Go to Data & Knowledge.
3. Create or use a `manual_upload` source such as `Router Troubleshooting Notes`.
4. Request approval if the source permission requires it.
5. Upload text from a warranty note, router troubleshooting note, bill note, or
   folder inventory/export.
6. Check Work & Processing for queued or completed processing.
7. Ask Assistant questions like:

```text
When does this warranty expire?
What changed in these router troubleshooting notes?
What files look duplicated?
What did I upload today?
What does this document say about my bill?
```

IGY6 does not send this evidence to an external model by default.

## Quickstart For Seasoned Coders

1. Start the stack and verify readiness.
2. Upload a build log, repo status report, route parity summary, or migration
   verification note through Data & Knowledge.
3. Ask Assistant:

```text
What failed in this build log? Cite the evidence.
Show git status.
Show latest DIFF.
Show work items.
```

4. Use Work & Processing to inspect queue state and dispatch metadata.
5. Use Safety & Audit to review approval decisions and audit events after an
   agent action.
6. Use Reports to render a migration or verification summary.

## Manual Upload Test Flow

Use this as a small local smoke test:

1. Data & Knowledge -> Sources: create a source.
   - Source name: `Router Troubleshooting Notes` or `IGY6 Build Logs`
   - Source type: `manual_upload`
   - Location: `local notes folder` or `local repo logs`
   - Sensitivity: `private` or `internal`
   - Allowed operations: `read, collect` or `read, collect, dry_run`
2. Safety & Audit -> Approvals: request approval if required.
   - Normal user reason: `Allow IGY6 to process this uploaded troubleshooting note.`
   - Coder reason: `Approve processing this local build log for evidence extraction.`
3. Data & Knowledge -> Uploads & Collection: upload UTF-8 text.
4. Work & Processing: confirm collection/work records were created.
5. Data & Knowledge -> Evidence: inspect documents, chunks, evidence items, and
   source trails.
6. Assistant: ask an evidence question and verify citations or retrieval context.

Current manual upload works best with UTF-8 text. Binary PDF/image/audio parsing
is not claimed by this flow unless a later DIFF adds it.

For a guided end-to-end checklist and local helper script, see
`docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md`.

Non-mutating preflight:

```bash
python3 scripts/e2e-manual-upload-smoke.py --check
```

Run the local smoke path against an already-running stack:

```bash
python3 scripts/e2e-manual-upload-smoke.py --run
```

The `--run` mode creates harmless local runtime records using the test keyword
`blue-raven-117`. It does not write test data to the repository. Worker
processing may remain queued; the script reports upload success, artifact/work
item creation, evidence availability, and retrieval visibility separately.

## Processing Status Diagnostics

Manual upload creates raw artifact metadata and queued processing work. Live
end-to-end processing is still owned by the Python/Celery worker. The Rust
worker crate has DIFF-143 collection normalization, DIFF-144 document chunking,
and DIFF-145 chunk vector upsert parity planning and executor contracts, and
DIFF-146 retains Python/Celery because there is not yet a Rust worker process
that polls, claims, and executes queued jobs. DIFF-147 adds the safe
`igy6-worker` harness for check/dry-run/once planning, but it is non-mutating
and does not replace Celery. DIFF-148 adds an opt-in one-job canary gate and
verification plan. DIFF-149 adds the gated live side-effect executor for one
explicitly selected canary work item, but Python/Celery remains the production
worker path until process ownership and beat posture are cut over. The Rust
gateway dispatch route records safe dispatch metadata without invoking Celery
directly.

Check worker/processing status:

```bash
python3 scripts/processing-status-smoke.py
```

See `docs/runtime/PROCESSING_STATUS.md` for the pipeline, status meanings, and
worker/API/Redis log commands.

## Optional Local LLM Plan

IGY6 does not call an external model by default. DIFF-126 adds a plan for a
future optional local LLM adapter, DIFF-127 adds the Rust local-Ollama-only
adapter crate, and DIFF-128 wires optional local generation into evidence-answer
logic behind evidence-required checks and deterministic fallback.

Safe local planning defaults in `.env.example`:

```env
LLM_PROVIDER=none
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_MODEL=
LLM_TIMEOUT_SECONDS=60
LLM_EVIDENCE_REQUIRED=true
```

`LLM_PROVIDER=none` means no model calls and deterministic evidence fallback is
active. If `LLM_PROVIDER=ollama` is configured, evidence-answer generation is
still evidence-required, timeout-bound, citation-oriented, and local-only.
Provider disabled, unavailable, invalid, or timed out states fall back to the
deterministic answer. If no evidence exists, Assistant says insufficient
evidence without calling the provider. See `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`.

Settings shows local LLM provider status, provider name, redacted local Ollama
base URL, model name, timeout, evidence-required state, and Advanced raw provider
diagnostics. Assistant shows whether the answer path is deterministic evidence,
local LLM evidence-grounded, or unavailable. Normal user example: use a local
model to summarize an uploaded warranty note using only evidence. Coder example:
use a local model to explain a build log failure with citations.

### Ollama Setup And Task Routing

DIFF-130 adds a safe local setup helper. The default mode is check-only:

```bash
scripts/ollama-local-setup.sh --check
scripts/ollama-local-setup.sh --list-recommended
```

It does not install Ollama, pull models, edit `.env`, delete models, run Docker
destructive commands, or call cloud APIs unless you pass explicit flags.

Manual Ollama checks:

```bash
ollama --version
curl http://127.0.0.1:11434/api/tags
```

Manual install and default model pulls:

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull qwen2.5-coder:7b
ollama pull llama3.1:8b
ollama pull gemma3:4b
ollama run qwen2.5-coder:7b
```

Scripted local setup when intentionally requested:

```bash
scripts/ollama-local-setup.sh --install --yes
scripts/ollama-local-setup.sh --pull-default-models
scripts/ollama-local-setup.sh --write-env code_repo
```

`--write-env` backs up `.env` first, preserves unrelated values, and writes:

```env
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_MODEL=qwen2.5-coder:7b
LLM_TIMEOUT_SECONDS=60
LLM_EVIDENCE_REQUIRED=true
```

Revert to deterministic mode with:

```env
LLM_PROVIDER=none
```

Recommended local models for a normal PC such as an RTX 3060 12GB setup:

| Model | Use | Default pull |
| --- | --- | --- |
| `qwen2.5-coder:7b` | Code/repo work, logs, stack traces, DIFFs, route parity, scripts | Yes |
| `llama3.1:8b` | General evidence summaries and default chat | Yes |
| `gemma3:4b` | Fast/lightweight triage and action explanations | Yes |
| `gemma3:12b` | Optional longer report drafting if performance is acceptable | No |

Do not pull `qwen2.5-coder:32b`, `llama3.1:70b`, `llama3.1:405b`,
`gemma3:27b`, or other large models by default on a 12GB VRAM setup.

Task routing lives in `configs/local-llm-routing.json`:

- `code_repo` -> `qwen2.5-coder:7b`
- `evidence_summary` -> `llama3.1:8b`
- `fast_triage` -> `gemma3:4b`
- `report_draft` -> `llama3.1:8b`, optional `gemma3:12b`
- `action_explanation` -> `gemma3:4b`
- `chat_default` -> `llama3.1:8b`

Each route includes its own system instruction, temperature, purpose, and
`evidence_required=true`.

## Safety And Approvals

Read-only actions, retrieval preview, and local status checks are designed to be
safe by default. System-changing actions must show approval requirements and use
matching approval records before execution.

Supported assistant action labels include:

- Show project health
- Show git status
- Show latest DIFF
- Show work items
- Run retrieval preview
- Start stack
- Stop stack
- Run last healthy stack

Stack start/stop/recovery actions require approval and only run if the API
runtime has the fixed allowlisted host bridge capability it needs. IGY6 does not
run arbitrary shell text from user input.

## Settings

Settings shows environment status, redacted config values, runtime service
status, storage paths, and developer diagnostics. Secret-like values are masked.
The UI should never display raw secrets and should not encourage pasting tokens
or private keys into unsafe fields.

Settings changes use verify-before-apply behavior where supported. Some values
are read-only or require service restart/recreate after editing.

## Troubleshooting

- Web UI unavailable: run `igy6-ps` or the long `docker compose ... ps` command
  and check the `web` container.
- Empty `ps` output usually means the stack is not running for the selected
  Compose project/env file. Start with `igy6-start` or
  `scripts/runtime-smoke.sh --start --detached`.
- `127.0.0.1:3000` refused usually means the `web` container is not running, is
  still building, or failed during Next.js startup. Check `igy6-logs` or
  `docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web`.
- API unavailable: open `http://127.0.0.1:8000/health/ready` and check API logs.
- Phoenix logs that show `GET / 200 OK` are normal health/readiness probes for
  the local Phoenix service; they do not mean external model calls are happening.
- Assistant action blocked: check Safety & Audit for approval requirements and
  runtime capability status.
- Upload blocked: confirm the source exists, permission operations include the
  needed collection operation, and approval exists if required.
- No evidence returned: confirm processing created documents, chunks, and
  evidence items before asking Assistant.
- Settings save blocked: run verify first and read the redacted validation
  warnings.

## Verification Commands

Common checks:

```bash
git status --short
git diff --check
npm --prefix apps/web run build
npm --prefix apps/web run test:ui-smoke
scripts/runtime-smoke.sh --check
python3 scripts/rust-route-parity.py --check
scripts/rust-cutover.sh --check
```

`npm --prefix apps/web run test:ui-smoke` checks the reorganized workflow UI
contract: top-level sections, Assistant action gating controls, Advanced panels,
manual upload guidance, empty/next-step states, and safety posture text.

Rust checks when backend/Rust files change:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Config checks when JSON config files change:

```bash
python3 -m json.tool configs/rust-cutover-manifest.json
python3 -m json.tool configs/legacy-fastapi-route-classification.json
```

Compose wiring check when Compose/runtime wiring changes:

```bash
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

## DIFF Discipline

Every change-bearing task must be recorded under the next available DIFF in
`docs/diffs/`. Locked DIFFs are historical records and must not be edited.
