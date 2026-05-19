# IGY6

IGY6 is a local-first evidence workspace. It helps you register data sources,
upload or collect allowed material, turn that material into evidence, search it,
run safe local assistant actions, review approvals, and audit what happened.

It is not a finished autonomous agent, not a generic chatbot, not ComfyUI, and
not an external-model workflow by default. Current answers are evidence previews
or deterministic evidence packets from local records.

## Current Backend Posture

IGY6 is Rust-primary through the gateway and route parity work completed through
DIFF-120. It is not Rust-only. The route manifest still marks FastAPI fallback as
required for remaining legacy/non-web routes, so do not remove FastAPI or claim
Rust-only operation until the manifest and route parity scripts prove it.

Current web-used route parity is tracked by:

```bash
python3 scripts/rust-route-parity.py --check
```

The web UI should show the same truth: local-first, evidence-only, no external
model by default, and approval-gated for system-changing actions.

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
