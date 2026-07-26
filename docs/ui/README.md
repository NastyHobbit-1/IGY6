# IGY6 Web UI Guide (grok branch)

This guide explains the web interface in plain language for the running program on the grok branch.

The program is password protected (default "ThatDog123"). Optional TOTP authenticator support is off by default until you link it in the User & Security section (works with any standard authenticator app). All data stays local only. The UI automatically uses a clear free local URL (dynamic port switching with clear printed address if the preferred port is busy).

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
- Chat web-fetch dock (public / deep / session fetch).
- Retrieval preview and evidence answer history.
- Missing-evidence prompts and local LLM status.
- Agent command / task plan history (collapsed).

A separate Home readiness strip still exists in the page markup for system status; primary navigation uses Chat as the entry point.

### What To Do Here

- Ask over evidence after data has been processed.
- Run public/deep/session fetch when you have a URL.
- Follow onboarding chips if sources or evidence are empty.

### Empty, Loading, And Error States

- Insufficient evidence means no matching local chunks yet — not that the real-world answer does not exist.
- Deterministic packets are used when Ollama is off or has no hits.

## Data

Data is where you register what IGY6 is allowed to use and start upload or collection flows.

### Main Sections

Information lifecycle, sources, guided upload, conversation history import, user observation ingestion, browser/web/router import (grok full-access paths), media import, local project / PC diagnostics.

Connector status matches DIFF-249: manual text paths implemented; browser_export / web_public / media / local paths use full-access + host bridge where configured.

### Buttons And Actions

- Submit manual text / Import conversation / Record observation.
- Deep fetch / Public fetch / Session fetch / Preview panels.
- Save source review; approval-aware pending states.

### What To Do Here

Use Data when you have authorized text or a URL you want IGY6 to remember or review. Then open Work to watch processing and Chat to ask questions.

## Work

Work shows background processing status in normal language: queued, running, completed, failed; work items; pipeline steps.

After adding data, check Work. Open Chat when completed. Treat repeated failures as troubleshooting.

## Settings

Settings contains safety, approval, policy, password/TOTP, and local `.env` configuration. Dry-run verification is required before save. Approvals for collection can be decided here without pasting raw IDs into More.

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

1. Data → guided upload or web fetch.
2. Work → wait for completed processing.
3. Chat → ask over evidence; save answer records when useful.

## Troubleshooting

- UI does not open: `scripts/status.sh`, confirm `web` service, use `WEB_BASE_URL`.
- Not ready: wait, then `scripts/post-cutover-smoke.sh --check`.
- No results: confirm Data upload, then Work status.
- Upload fails: UTF-8 text, source permission, approval if required.

## Safety And Data Rules

- Runtime/private data under `IGY6_DATA_ROOT`.
- Do not commit `.env`, secrets, or collected private data.
- Old Python/FastAPI/Celery services are archived history, not active runtime.
- Local-first and approval-gated for sensitive workflows.

## Current Limitations (aligned to DIFF-249)

- Core manual UTF-8 text paths are the most mature.
- On `grok`, web/public URL, browser export, Session Fetch, media binary, and system/WiFi collection use full-access + host bridge.
- Guided paste/preview panels remain partial for automatic OCR/transcription.
- Host bridge is required for the strongest web tiers.
- No external exfil; empty states are honest.
- Refer to DIFF-249 capability table for exact status per item.
