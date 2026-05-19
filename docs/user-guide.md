# User Guide

Open the local web UI:

```text
http://127.0.0.1:3000
```

IGY6 is organized around seven workflows:

- Home
- Assistant
- Data & Knowledge
- Work & Processing
- Reports
- Safety & Audit
- Settings

The old developer-console split between Chat, Agent Command, Sources, Evidence,
Memory, Work Queue, Approvals, Audit, Reports, and Settings has been folded into
these workflow sections. Advanced IDs, raw JSON, approval IDs, and route/debug
details are still present under Advanced panels.

## Normal PC User Examples

- Upload warranty text and ask when the warranty expires.
- Upload router or internet troubleshooting notes and ask what changed.
- Upload a folder inventory/export and ask what files look duplicated.
- Create a summary report from notes without sending anything externally.
- Ask Assistant: `What did I upload today?`
- Ask Assistant: `What does this document say about my bill?`
- Request approval before a system-changing action.

## Coder Examples

- Upload a build log and ask for the likely failure cause with evidence.
- Upload a repo status report and ask for the next DIFF recommendation.
- Use Assistant to show git status.
- Use Assistant to show latest DIFF.
- Create a work item for code review or route parity follow-up.
- Review audit events after an agent action.
- Inspect chunks/evidence created from a technical document.
- Render a migration or verification summary in Reports.

## Manual Upload Flow

1. Data & Knowledge -> Sources: create or select a `manual_upload` source.
2. Check source permission and approval status.
3. Safety & Audit -> Approvals: request approval if required.
4. Data & Knowledge -> Uploads & Collection: upload UTF-8 text.
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

## Safety Notes

IGY6 is local-first and evidence-only by default. It does not claim Rust-only
operation while the manifest still requires FastAPI fallback. It does not send
evidence to an external model by default. It does not run arbitrary shell text
from Assistant input.

System-changing actions must clearly show approval requirements. Stack
start/stop/recovery actions require approval and fixed allowlisted runtime
capability.

## UI Smoke Check

Run the focused UI workflow smoke check with:

```bash
npm --prefix apps/web run test:ui-smoke
```

The check verifies the Home, Assistant, Data & Knowledge, Work & Processing,
Reports, Safety & Audit, and Settings workflow contract. It also checks that
Assistant action buttons start gated, Advanced panels preserve raw/debug
controls, and manual upload guidance remains visible.

## Runtime Smoke Check

Check an already-running local stack:

```bash
scripts/runtime-smoke.sh --check
```

The runtime smoke check validates Docker Compose config, expected running
services, `http://127.0.0.1:8000/health/live`,
`http://127.0.0.1:8000/health/ready`, and `http://127.0.0.1:3000`. It prints
clear PASS/FAIL lines and does not start or stop services in default check mode.

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
- `127.0.0.1:3000` refused usually means the web container is stopped, still
  starting, or failed during Next.js startup.
- Phoenix `GET / 200 OK` log lines are normal local health/readiness probes.
- For API logs: `docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api`
- For web logs: `docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web`
