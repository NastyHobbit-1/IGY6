# User Guide (grok branch)

**This guide is only about running and using the program on the grok branch.** Open the local web UI at the clear URL printed by the start command (normally http://127.0.0.1:3000 or the next free port it automatically switched to).

The UI is deliberately simple and tabbed. Visible tab labels (from `HomePage.tsx`) are:

- **Chat** (default) — evidence-grounded assistant, retrieval preview, web-fetch dock, LLM status, task history.
- **Data** — sources, guided manual upload, conversation history, user observations, web/browser/router fetch, media import (PDF/image/audio/video), local project/PC diagnostics.
- **Work** — processing queue, work items, pipeline status.
- **Settings** — password/TOTP (User & Security), env dry-run, approvals, safety/policy, lifecycle audit.
- **More** — diagnostics and advanced route console.

Internal panel headings such as Home readiness, Add Data, Results/Evidence, and Advanced still appear inside content panels; the five tab labels above are what users click. Media Library (full-res images/videos) is reachable from Data / Chat workflows after deep collection or media import.

## Basic Operating Flow (Setup + Daily Use)

1. Start the program (recommended):
   ```bash
   igy6 start
   ```
   Or `scripts/run.sh` for foreground logs. Both pick free ports when 3000/8000 are
   busy and print the exact `WEB_BASE_URL` to open.

2. Open the URL in your browser. If prompted, set a program password in Settings → User & Security. If you enable TOTP, also supply a current code from your authenticator app for protected actions (deep collector, etc.).

3. Use **Data** to register sources and bring in authorized information:
   - Guided text upload, conversation history, user observations.
   - Web fetch: Public fetch, Automated deep fetch, Session-assisted fetch (host bridge may be required for strongest tiers).
   - Media import: upload PDF/image/audio/video; worker extracts text with local tools (pdftotext / tesseract / ffmpeg+whisper) when installed.
   - Local project / PC diagnostics (bounded paths only).

4. Open **Work** to watch normalization, extraction, chunking, and vector upsert status.

5. Open **Chat** to ask questions over local evidence, run retrieval preview, or use the web-fetch dock.

6. In **Settings** → User & Security:
   - Change password anytime (enter current + new).
   - Generate secret + otpauth URL for any authenticator app (provide current password). Add it to your app, then confirm with a code from the app to enable TOTP. Once enabled, protected features will also require a current code.
   - Check status or re-link as needed. TOTP stays off until you explicitly link it.

7. Use **More** only for diagnostics or the advanced route console when needed.

8. Stop with `igy6 stop` or the matching stop/restart scripts.

All data (including full-res media artifacts, evidence, graph in Neo4j, audit) lives only inside your local instance under the data root you configured. Collection, media import, and library paths are real and end-to-end wired.

Dynamic URLs: The program always finds a free local port if the default is busy and clearly reports the usable http://127.0.0.1:PORT. Just use whatever it tells you.

## Key Features on This Branch

- Deep & thorough collection on targets you authorize, with original/full-res images and videos fetched from their source when using full-access paths.
- Media import with local extraction (PDF text layer, OCR, transcription) and artifact storage.
- Password protection with simple changing in Settings → User & Security.
- Optional TOTP authenticator (off by default, link with any standard app).
- End-to-end pipeline: artifacts → documents → chunks → evidence → vectors/graph → Chat answers with local audit.
- Local-only by design. Dynamic clear local URLs. Chat-first tabbed UI.

Supporting scripts (smoke checks, diagnostics, backup, etc.) are optional and print clear status. Use them when you want extra visibility — not required for normal operation.

## Guided Upload Field Examples

- Source name: `Router Troubleshooting Notes` or `IGY6 Build Logs`
- Source type: `manual_upload`
- Location: `local notes folder` or `local repo logs`
- Sensitivity: `private` or `internal`
- Allowed operations: `read, collect` or `read, collect, dry_run`
- Approval reason: `Allow IGY6 to process this uploaded troubleshooting note.`
- Coder approval reason: `Approve processing this local build log for evidence extraction.`
- Report render reason: `Create a summary of this uploaded bill.` or
  `Render a route parity verification summary.`

For a fuller local smoke checklist, read:

```text
docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md
```

The helper script has a safe preflight mode:

```bash
python3 scripts/e2e-manual-upload-smoke.py --check
```

Run the local E2E path only when you are comfortable creating harmless local
runtime records:

```bash
python3 scripts/e2e-manual-upload-smoke.py --run
```

The test keyword is `blue-raven-117`. If Chat cannot find that keyword,
first check that worker processing created chunks/evidence.

## Processing Status

Manual upload or media import may create queued work before evidence exists. Check processing
status with:

```bash
python3 scripts/processing-status-smoke.py
```

The live worker is the Rust `igy6-worker` daemon. Gateway dispatch records work
in PostgreSQL for the worker to claim. See `docs/runtime/PROCESSING_STATUS.md` for pipeline
details, status meanings, and log commands.

## Optional Local LLM Plan

IGY6 does not call an external model by default. Local LLM support is optional,
starts with Ollama on the user's machine, and remains disabled when
`LLM_PROVIDER=none`. Evidence answers keep deterministic fallback. If a local
provider is unavailable or timed out, Chat reports fallback status instead
of guessing. If no evidence exists, Chat says insufficient evidence without
calling the provider.

Planned local defaults:

- `LLM_PROVIDER=none`
- `OLLAMA_BASE_URL=http://host.docker.internal:11434`
- `OLLAMA_MODEL=` left blank until the user chooses a local model
- `LLM_TIMEOUT_SECONDS=60`
- `LLM_EVIDENCE_REQUIRED=true`

Local LLM answers must cite retrieved evidence or say insufficient evidence.
They must not execute actions or change approval requirements. See
`docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`.

Settings shows provider `none` or `ollama`, local Ollama base URL, model name,
health/status text, timeout, and evidence-required state. Advanced provider
diagnostics are available without exposing tokens. Chat shows the current
answer mode: deterministic evidence, local LLM evidence-grounded, or unavailable.

Examples:

- Normal user: `Use local model to summarize uploaded warranty note using only evidence.`
- Coder: `Use local model to explain build log failure with citations.`

### Ollama Local Setup

Ollama is optional. IGY6 still works without it by using deterministic evidence
fallback. No external model calls are made by default.

Check local state:

```bash
scripts/ollama-local-setup.sh --check
scripts/ollama-local-setup.sh --list-recommended
ollama --version
curl http://127.0.0.1:11434/api/tags
```

Install manually:

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

Pull only the default recommended models:

```bash
ollama pull qwen2.5-coder:7b
ollama pull llama3.1:8b
ollama pull gemma3:4b
```

Test a model manually:

```bash
ollama run qwen2.5-coder:7b
```

Configure IGY6 after installing local models:

```env
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_MODEL=qwen2.5-coder:7b
LLM_EVIDENCE_REQUIRED=true
```

Use `LLM_PROVIDER=none` to return to deterministic mode.

Model recommendations:

| Model | Best for |
| --- | --- |
| `qwen2.5-coder:7b` | Code, repo state, logs, scripts, DIFFs, route parity |
| `llama3.1:8b` | General evidence summaries and default chat |
| `gemma3:4b` | Fast triage and short explanations |
| `gemma3:12b` | Optional longer report drafts if local performance is acceptable |

Task routing is defined in `configs/local-llm-routing.json`. Each task has a
different system instruction, model, temperature, purpose, and
`evidence_required=true`.

## Safety Notes

IGY6 is local-first and evidence-only by default. The API path no longer uses
FastAPI fallback after DIFF-138, but full Rust-only repository/runtime operation
is not claimed while legacy Python/Celery services remain archived. It
does not send evidence to an external model by default. It does not run
arbitrary shell text from Chat input.

System-changing actions must clearly show approval requirements. Stack
start/stop/recovery actions require approval and fixed allowlisted runtime
capability.

## UI Smoke Check

Run the focused UI workflow smoke check with:

```bash
npm --prefix apps/web run test:ui-smoke
npm --prefix apps/web run test:ui-runtime-smoke
npm --prefix apps/web run typecheck
# combined: npm --prefix apps/web run check
```

The (static + runtime) checks verify the Chat, Data, Work, Settings, and More
workflow contract. It also checks that Chat action buttons start gated, Advanced
panels preserve raw/debug controls, and manual upload guidance remains visible.

## Runtime Smoke Check

Check an already-running local stack:

```bash
scripts/runtime-smoke.sh --check
```

The runtime smoke check validates Docker Compose config, expected running
services, and HTTP checks against `APP_PORT` / `WEB_PORT` from `.env` (defaults
8000 and 3000). It prints clear PASS/FAIL lines and does not start or stop
services in default check mode.

Start explicitly:

```bash
scripts/runtime-smoke.sh --start --detached
```

Stop explicitly:

```bash
scripts/runtime-smoke.sh --stop
```

The stop command uses `docker compose down`, not `down -v`. Do not use
`down -v` as a normal stop command because it can delete stored Docker volume
data.

Long Docker commands:

```bash
docker compose -f infra/docker-compose.yml --env-file .env up --build
docker compose -f infra/docker-compose.yml --env-file .env down
docker compose -f infra/docker-compose.yml --env-file .env ps
docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200
```

WSL aliases:

```bash
igy6-start
igy6-stop
igy6-ps
igy6-logs
```

Troubleshooting:

- Empty `ps` output means the stack is probably not running for the selected
  Compose project/env file.
- Web UI connection refused usually means the web container is stopped, still
  starting, or failed during Next.js startup. Check `WEB_PORT` in `.env` — it may
  be 3001, 3002, etc. if 3000 was busy. Wrong app (e.g. Open WebUI) on a port
  means another service took that port before IGY6 started; run `igy6 start` again
  or stop the conflicting container.
- Phoenix `GET / 200 OK` log lines are normal local health/readiness probes.
- For API logs: `docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api`
- For web logs: `docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web`
- Media extraction empty: rebuild worker so tools are present (`docker compose -f infra/docker-compose.yml build worker && up -d worker`); optional host tools via `./install.sh` / `install.ps1`.
