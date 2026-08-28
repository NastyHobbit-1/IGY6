# IGY6 Web UI Guide (grok branch)

This guide explains the web interface in plain language for the running program on the grok branch.

The program is password protected (set on first run). Optional TOTP authenticator support is off by default until you link it in the User & Security section (works with any standard authenticator app). All data stays local only. The UI automatically uses a clear free local URL (dynamic port switching with clear printed address if the preferred port is busy).

## Start & Open

Recommended:

```bash
igy6 start
```

Or `scripts/run.sh` for foreground logs. Note the usable local URL printed by
`igy6` or read `WEB_BASE_URL` from `.env` (default `http://127.0.0.1:3000`; auto-
switches to 3001, 3002, … when busy). The page title must be **IGY6 Local
Evidence Workspace**.

Stop with `igy6 stop` or the matching stop/restart scripts.

## Main Areas (visible tab labels)

Visible tabs in the UI (from `HomePage.tsx`):

- **Chat** (default): evidence-grounded assistant, retrieval preview, web-fetch dock, LLM status, task history.
- **Data**: sources, guided manual upload, conversation history, user observations, web/browser/router fetch, media import, local project/PC diagnostics.
- **Work**: processing queue, work items, pipeline status.
- **Settings**: password/TOTP, env dry-run, approvals, safety/policy, lifecycle audit.
- **More**: diagnostics, advanced route console, technical posture.

Internal panel content still uses headings such as Home readiness, Add Data, Results/Evidence, and Advanced; the tab bar labels above are what users see and click.

Media Library (full-res images/videos) lives with collected artifacts and is reachable from Data / Chat workflows after deep collection.

Use the collector for aggressive local/web/system collection (password + optional totp_code required for protected calls). View results in Chat and Data areas. Manage your password and optional authenticator in Settings → User & Security. Everything is tied into real local artifacts, evidence, graph, and audit.

Dynamic URL: the program always picks a free port if needed and tells you the exact address — just use what it prints.

All other docs have been aligned to this program-only operating view for the grok branch.

## Chat (default tab)

Chat is the default tab. It answers questions over local evidence and surfaces next steps.

### Main Sections

- Unified chat hub with plain-language requests.
- Chat web-fetch dock (public / automated deep / session-assisted fetch).
- Retrieval preview and evidence answer history.
- Missing-evidence prompts and local LLM status.
- Agent command / task plan history (collapsed).

A separate Home readiness strip still exists in the page markup for system status; primary navigation uses Chat as the entry point.

### What To Do Here

- Ask over evidence after data has been processed.
- Run public/automated deep/session-assisted fetch when you have a URL.
- Follow onboarding chips if sources or evidence are empty.

### Empty, Loading, And Error States

- Insufficient evidence means no matching local chunks yet — not that the real-world answer does not exist.
- Deterministic packets are used when Ollama is off or has no hits.

## Data

Data is where you register what IGY6 is allowed to use and start upload or collection flows.

### Main Sections

Information lifecycle, sources, guided upload, conversation history import, user observation ingestion, browser/web/router import (grok full-access paths), media import, local project / PC diagnostics.

Connector status: manual text paths implemented; browser_export / web_public / media_file / local paths use full-access + host bridge where configured. Media import (DIFF-268) uploads the binary and runs local extraction in the worker (pdftotext, tesseract, ffmpeg+whisper) when tools are installed in the worker image / host.

### Buttons And Actions

- Submit manual text / Import conversation / Record observation.
- Automated deep fetch / Public fetch / Session-assisted fetch / Preview panels.
- Media import: choose type, select file, Upload media file (stores binary; worker extracts text).
- Save source review; approval-aware pending states.

### What To Do Here

Use Data when you have authorized text, a media file, or a URL you want IGY6 to remember or review. Then open Work to watch processing and Chat to ask questions.

## Work

Work shows background processing status in normal language: queued, running, completed, failed; work items; pipeline steps.

After adding data, check Work. Open Chat when completed. Treat repeated failures as troubleshooting.

## Settings

Settings contains safety, approval, policy, password/TOTP, local `.env` configuration, and troubleshooting logs. Dry-run verification is required before save. Approvals for collection can be decided here without pasting raw IDs into More. Startup and error logs live under `IGY6_DATA_ROOT/ops/` (`startup.log`, `error.log`) and are shown in Settings → Troubleshooting.

## More (diagnostics)

More is for diagnostics and the advanced route console. Normal users usually do not need it. Do not use it to guess IDs or bypass approvals.

## Interface Item Guide

- Primary actions: ask with evidence (Chat), add data (Data), check processing (Work).
- Empty states are real (no fake demo data).
- Advanced / More can show technical names and raw JSON.

## Workflows

### Start and readiness

1. `igy6 start` (or `scripts/run.sh`).
2. `scripts/status.sh`.
3. Open `WEB_BASE_URL`.
4. Use Chat for questions; Data to add sources; Work for queue; Settings for policy.

### Add data → process → ask

1. Data → guided upload, media upload, or web fetch.
2. Work → wait for completed processing (normalization / extraction for media).
3. Chat → ask over evidence; save answer records when useful.

## Troubleshooting

- UI does not open: `scripts/status.sh`, confirm `web` service, use `WEB_BASE_URL`.
- Not ready: wait, then `scripts/post-cutover-smoke.sh --check`.
- No results: confirm Data upload, then Work status.
- Upload fails: UTF-8 text for paste paths; for media, confirm API is up and worker image was rebuilt after install so extraction tools are present.
- Media extraction empty: rebuild worker (`docker compose -f infra/docker-compose.yml build worker && up -d worker`); optional host tools via `./install.sh` / `install.ps1`.
- Operator logs: Settings → Troubleshooting, or `IGY6_DATA_ROOT/ops/startup.log` and `ops/error.log`. Sensitive values are redacted.

## Safety And Data Rules

- Runtime/private data under `IGY6_DATA_ROOT`.
- Do not commit `.env`, secrets, or collected private data.
- Old Python/FastAPI/Celery services are archived history, not active runtime.
- Local-first and approval-gated for sensitive workflows.
- Extracted OCR/transcript text and original binaries stay inside IGY6 only.

## Current Limitations (aligned to DIFF-268 / DIFF-249)

- Core manual UTF-8 text paths remain the most mature for paste workflows.
- On `grok`, web/public URL, browser export, session-assisted fetch, media binary, and system/WiFi collection use full-access + host bridge where required.
- Media import is implemented: binary upload stores the original; worker normalization extracts text with local tools (pdftotext for PDF text layer, tesseract for images, ffmpeg+whisper for audio/video). Quality depends on those local engines, not cloud services. Worker image must be rebuilt after install so tools are present in the container.
- Image-only PDFs with no text layer may yield empty extraction until page-render OCR is added in a later DIFF.
- Host bridge is required for the strongest web tiers.
- No external exfil; empty states are honest.
- Refer to DIFF-249 capability table and DIFF-268 for exact status per item.

## Browser API calls

Client-side buttons post to same-origin `/api/*` Next.js proxies. Those proxies forward to the Rust gateway using container `API_BASE_URL`. Guided upload, media import, local project, reports, approvals, and Advanced console actions all use `/api`, not `http://127.0.0.1:8000`.
