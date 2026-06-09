# User Guide (grok branch)

**This guide is only about running and using the program on the grok branch.** Open the local web UI at the clear URL printed by the start command (normally http://127.0.0.1:3000 or the next free port it automatically switched to).

The UI is deliberately simple and tabbed. On this branch the main areas are:

- Home / overview — readiness, recent activity, quick status.
- Add Data / Collector — deep thorough collection from any reachable target (local, web URLs, system, WiFi, etc.). It extracts complete info and pulls original/full-resolution images and videos directly from their sources.
- Media Library — easy grid of every collected image and video. Click to view full/original resolution in a native browser viewer (real data from the artifact content endpoint). Refresh after scans.
- Work / Results / Evidence — processing, evidence items, answers, reports, graph relationships.
- Settings / User & Security (user section) — change your password (default "ThatDog123"), link or manage optional TOTP authenticator (off by default — works with any standard app like Google Authenticator or Authy), check status.
- Advanced — raw diagnostics when you need them.

## Basic Operating Flow (Setup + Daily Use)

1. Start the program (recommended):
   ```bash
   igy6 start
   ```
   Or `scripts/run.sh` for foreground logs. Both pick free ports when 3000/8000 are
   busy and print the exact `WEB_BASE_URL` to open.

2. Open the URL in your browser. Use the password gate (default "ThatDog123"). If you have enabled TOTP, also supply a current code from your authenticator app for protected actions (deep collector, etc.).

3. Use the Collector area to run deep scans on targets you care about (web pages, local folders, "everything", specific URLs, etc.). The process is thorough: it follows the target, extracts media at original/full res from the actual source, stores everything locally only with full provenance, and feeds evidence + graph.

4. Go to the Media Library to browse and view the full-res images and videos that were collected.

5. Explore Results/Evidence for the extracted claims, relationships, and complete info (all tied together in the real pipelines).

6. In User & Security:
   - Change password anytime (enter current + new).
   - Generate secret + otpauth URL for any authenticator app (provide current password). Add it to your app, then confirm with a code from the app to enable TOTP. Once enabled, protected features will also require a current code.
   - Check status or re-link as needed. TOTP stays off until you explicitly link it.

7. Stop with Ctrl+C or the stop/restart scripts.

All data (including full-res media artifacts, evidence, graph in Neo4j, audit) lives only inside your local instance under the data root you configured. The collector and library are real and end-to-end wired — no scaffolding.

Dynamic URLs: The program (web start + supporting scripts) always finds a free local port if the default is busy and clearly reports the usable http://127.0.0.1:PORT. Just use whatever it tells you.

## Key Features on This Branch

- Deep & thorough scraping on whatever target you give it, with original/full-res images and videos fetched directly from their source.
- Easy Image & Video Library with full-resolution viewing of collected media.
- Password protection with simple changing in the User section.
- Optional TOTP authenticator (off by default, link with any standard app, standard TOTP so Google Authenticator / Authy / etc. all work).
- Everything tied real: collector creates artifacts (full bytes + mime/kind) → evidence → worker normalization/chunking/vectors → graph sync → visible in library and results with complete local audit.
- Local-only by design. Dynamic clear local URLs. Simple focused UI for collection, media viewing, evidence, and security.

Supporting scripts (smoke checks, diagnostics, backup, etc.) are optional and print clear status. Use them when you want extra visibility — not required for normal operation.

All other documents in docs/ (user-guide, operations, ui guide, security-policy, runtime notes, truth table, diffs on this branch, etc.) have been updated to match this program-only, operating-focused view for the grok branch. No build or development content remains in the user-facing instructions.

Start the program, unlock, collect deeply, view media in the library, manage your password and optional authenticator in User & Security. Everything just works locally and is auditable.
5. Work & Processing: check collection and work status.
6. Data & Knowledge -> Evidence: inspect created documents, chunks, evidence,
   and source trails.
7. Assistant: ask a question over local evidence.

Field examples:

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

The test keyword is `blue-raven-117`. If Assistant cannot find that keyword,
first check that worker processing created chunks/evidence.

## Processing Status

Manual upload may create queued work before evidence exists. Check processing
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
provider is unavailable or timed out, Assistant reports fallback status instead
of guessing. If no evidence exists, Assistant says insufficient evidence without
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
diagnostics are available without exposing tokens. Assistant shows the current
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
arbitrary shell text from Assistant input.

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

The (static + runtime) checks verify the Home, Assistant, Data & Knowledge, Work & Processing,
Reports, Safety & Audit, and Settings workflow contract. It also checks that
Assistant action buttons start gated, Advanced panels preserve raw/debug
controls, and manual upload guidance remains visible.

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
