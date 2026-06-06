# IGY6 Web UI Guide

This guide explains the IGY6 web interface in plain language.

IGY6 is a local-first evidence workspace. You use it to add authorized
information, check processing, ask questions over stored evidence, review
results, and inspect safety settings. It is for normal local use first; the
Advanced tab is only for troubleshooting and technical checks.

The main tabs are:

- Home
- Add Data
- Work
- Results
- Settings
- Advanced

## Open The UI

Start IGY6:

```bash
scripts/run.sh
```

Check whether services are running:

```bash
scripts/status.sh
```

Open the web UI:

```text
http://127.0.0.1:3000
```

Stop IGY6:

```bash
scripts/stop.sh
```

The scripts require `.env`. If it is missing, create it from `.env.example`
before starting the stack.

## Home

Home is the first tab. It answers three questions:

- Is the system ready?
- What needs attention?
- What should I do next?

### Main Sections

Readiness strip:

- `System ready` means the application API is expected to be available.
- `Background worker ready` means background processing is expected to be
  available.
- `Old Python services archived` means the old Python runtime is not active.

Home overview:

- Service readiness shows how many readiness checks are passing.
- Recent data activity counts recent collection and artifact records.
- Recent work counts recent processing records.
- Pending approvals counts approval requests waiting for a decision.
- Recent audit events counts recent activity records.

Primary workflow cards:

- `Open Add Data` takes you to the Add Data tab.
- `Open Work` takes you to the Work tab.
- `Open Results` takes you to the Results tab.

### What To Do Here

Start here after opening the UI. If Home says everything is ready, choose the
next step:

- Add data if IGY6 has no sources yet.
- Check processing if you recently added data.
- Open Results if evidence or reports already exist.

### Empty, Loading, And Error States

- Empty counts usually mean no data has been added yet.
- A readiness error means the stack may not be fully running.
- Pending approvals mean a workflow is waiting for your review before it can
  continue.

### What Not To Use Home For

Home is not where you enter raw IDs, run diagnostics, or inspect service
internals. Use Advanced only when troubleshooting.

## Add Data

Add Data is where you register what IGY6 is allowed to use and start simple
upload or collection flows.

### Main Sections

Information lifecycle:

- Shows the path from source to upload, artifact, document, chunks, evidence,
  memory, and retrieval.

Overview:

- Gives examples of useful data, such as troubleshooting notes, bills, build
  logs, or verification summaries.

Sources:

- A source is a place or category of information IGY6 may use.
- Source name is the human name, such as `Router Troubleshooting Notes`.
- Source type describes the supported path. `manual_upload` is the simplest
  current path, `conversation_history` is for manually pasted prior
  conversation/history text, and `user_observation` is for owner-provided
  observations, decisions, preferences, corrections, and notes.
- Location is a description or path for the source.
- Sensitivity tells IGY6 how private the data is.
- Permissions control what IGY6 may do with the source.
- Source trust and sensitivity review lets you mark a source as `trusted`,
  `noisy`, `sensitive`, `disabled`, or `review-needed` from the normal Add Data
  workflow.
- Source review updates the source record and audit trail. It does not delete
  sources, rewrite historical evidence, silently hide evidence in Results, or
  claim new retrieval ranking or policy enforcement behavior.
- Source detail lets you inspect one source's label, type, trust state,
  sensitivity, permissions, collection runs, artifact metadata, documents,
  chunks, evidence previews, direct feedback/outcome links, and correction
  indicators.
- Source detail shows metadata and bounded previews only. It does not dump raw
  artifact contents, expose secrets, or claim complete policy enforcement.

Guided Upload:

- Provides a normal-user manual text form for pasted UTF-8 text.
- Lets you create a new `manual_upload` source or choose an existing enabled
  manual upload source without entering raw source or permission IDs.
- Explains that current guided upload is text-only and does not parse binary
  PDF, image, audio, video, screenshot, web page, or OCR input.
- If the selected source permission requires approval, creates an approval
  request and shows a pending state instead of uploading before approval.
- If a matching approval is already approved, the guided path uses it
  automatically; normal users do not need to paste raw approval IDs into
  Advanced for supported manual text collection.
- Shows recent collection runs after upload or dry-run records exist.

Conversation History Import:

- Provides a normal-user import form for prior conversation/history text.
- Lets you create a new `conversation_history` source or choose an existing
  enabled conversation source without entering raw source or permission IDs.
- Accepts manual pasted UTF-8 text only.
- Stores safe labels such as conversation title, date/time range, participants
  or roles, context note, and whether the import contains corrections,
  decisions, or instructions/preferences.
- Uses the existing local source, artifact, document, chunk, and evidence
  pipeline where processing succeeds.
- Does not scrape browsers, accounts, chat services, Gmail, Discord, ChatGPT,
  or any external service.
- Does not parse binary PDF, image, audio, video, screenshot, or OCR input.
- Browser/account/connector imports are planned future capabilities and require
  separate permission, dry-run, audit, and source-policy work.

User Observation Ingestion:

- Provides a normal-user form for first-party local context.
- Lets you create a new `user_observation` source or choose an existing enabled
  observation source without entering raw source or permission IDs.
- Accepts manual UTF-8 text entered by the owner.
- Supports safe labels such as observation title, observation type, observed or
  decided time if known, confidence, tags, optional related source/evidence/task
  IDs or labels, and a sensitivity flag.
- Uses the existing local source, artifact, document, chunk, and evidence
  pipeline where processing succeeds.
- Records user-provided context only. It does not automatically verify truth or
  turn the observation into an externally confirmed fact.
- Does not extract hidden memory, scrape accounts, read browser data, use
  connectors, call hosted AI, or read external services.
- Related IDs or labels are stored as plain text in this MVP; they are not
  validated links unless a later DIFF adds that behavior.

### Buttons And Actions

- `Submit manual text` sends pasted UTF-8 text through the existing manual
  upload collection path when the source permission allows immediate
  collection.
- `Import conversation text` sends manually pasted UTF-8 conversation/history
  text through the same local text pipeline under a `conversation_history`
  source.
- `Record observation` sends owner-entered UTF-8 observation text through the
  same local text pipeline under a `user_observation` source.
- `Save source review` records a trust/sensitivity review update for an
  existing source and keeps linked evidence counts visible.
- Approval-required source permissions stop in a pending state and tell you to
  review approvals before collection continues. After approval, return to the
  same guided Add Data workflow and submit again; IGY6 matches the approved
  approval record automatically.
- Advanced keeps the raw source, permission, approval, and upload controls for
  low-level troubleshooting.

### What To Do Here

Use Add Data when you have authorized text you want IGY6 to remember or review.

Good examples:

- A warranty note copied into text.
- A router troubleshooting note.
- A build log snippet.
- A project verification summary.
- A prior support chat or project discussion copied into plain text.
- A first-party observation, decision, preference, correction, or note you want
  IGY6 to retain as local context.

Basic flow:

1. Choose an existing manual source or create a new manual text source.
2. Enter a source name and description if creating a source.
3. Paste authorized UTF-8 text.
4. Submit the manual text.
5. Open Work to inspect processing status.
6. Open Results to inspect collection runs, artifacts, documents, chunks, and
   evidence after processing.

Conversation history flow:

1. Choose an existing conversation source or create a new
   `conversation_history` source.
2. Enter a conversation title, date/time range if known, participants or roles,
   and purpose/context note when useful.
3. Mark whether the conversation contains corrections, decisions, or
   instructions/preferences.
4. Paste authorized UTF-8 conversation/history text.
5. Import the conversation text.
6. Open Work to inspect processing status.
7. Open Results to inspect evidence after processing.

User observation flow:

1. Choose an existing observation source or create a new `user_observation`
   source.
2. Enter an observation title, type, observed or decided time if known,
   confidence, tags, related labels or IDs if useful, and sensitivity flag if
   relevant.
3. Enter the owner-provided observation, decision, preference, correction, or
   note.
4. Record the observation.
5. Open Work to inspect processing status.
6. Open Results to inspect evidence after processing.
7. Use source and evidence review states later if the observation needs
   correction, supersession, verification, or dispute review.

### Empty, Loading, And Error States

- `No sources registered yet` means IGY6 has no allowed source records.
- `No collection runs recorded yet` means no upload or collection run has been
  recorded.
- Endpoint errors mean the UI could not read that record type from the local
  API.

### What Not To Use Add Data For

Do not use it for unsupported binary parsing. Binary PDF, image, audio, and
video parsing are not claimed unless a later DIFF adds them. Convert important
content to text first.

## Work

Work shows background processing status in normal language.

### Main Sections

Processing totals:

- Queued means waiting to run.
- Running means currently being processed.
- Completed means finished.
- Failed means attention is needed.

Processing pipeline:

- Shows the expected path from raw artifact to document, chunks, evidence,
  vector memory, and graph memory.

Work items:

- Lists recent background tasks.
- Shows the task type, status, requester, error message if present, and time.

Advanced dispatch details:

- Hidden in a collapsible area.
- Normal users usually do not need it.

### Buttons And Actions

The Work tab mainly helps you inspect status. If you need to run low-level
dispatch controls, use Advanced only when you know the exact work item and why
you are dispatching it.

### What To Do Here

After adding data, check Work.

- If queued items exist, wait for processing.
- If running items exist, let them finish.
- If completed items exist, open Results.
- If failed items exist, read the error and check Troubleshooting below.

### Empty, Loading, And Error States

- `No work items recorded yet` means nothing has been queued.
- Failed work usually means the input, permission, or processing path needs
  review.
- Repeated failures should be treated as troubleshooting, not ignored.

### What Not To Use Work For

Do not use Work to force broad processing or bypass approvals. Background work
must remain bounded and auditable.

## Results

Results is where you inspect evidence, ask questions, and review reports.

### Main Sections

Assistant:

- Lets you ask over local evidence.
- Shows retrieval context when matching evidence exists.
- Does not need an external model by default.

Ask Over Evidence:

- Question or request is where you type a plain-language question.
- Evidence limit controls how many matching chunks to show.
- `Ask over evidence` searches local evidence and returns context.

Local LLM Status:

- Shows whether local model generation is disabled, configured, or waiting for
  a model.
- Shows the configured provider, configured model, routing state, fallback
  state, and evidence-required mode.
- Deterministic evidence answers remain available without an online model.
- Settings does not contact Ollama, install models, call hosted AI, or transfer
  source data. Local model availability is verified only when an evidence-answer
  request runs.
- If the local model is unavailable or not configured, use deterministic
  evidence answers and check Settings/Ollama locally before expecting
  model-drafted wording.

Evidence And Documents:

- Shows collection runs, artifacts, documents, chunks, evidence items, and
  claims.
- Evidence correction and supersession lets you mark an evidence item as
  `needs correction`, `corrected`, `superseded`, `disputed`, or `verified`.
- Evidence correction records review metadata and audit history only. It does
  not delete evidence, rewrite raw artifacts, rewrite documents or chunks,
  silently hide superseded evidence, or claim retrieval ranking changes.
- Evidence detail lets you inspect a bounded evidence preview, source trail,
  document/chunk lineage, source trust and sensitivity context, correction or
  supersession state, feedback, outcome links where present, related saved
  answer records, and task plan/report metadata links where present.
- Evidence detail is read-only. It does not delete, mutate, hide, or dump
  excessive raw text.

Search Memory And Findings:

- Shows vector collection state, graph schema state, patterns, hypotheses,
  predictions, and recommendations.

Reports:

- Lists report records if any exist.
- Report create/render controls currently live in Advanced.

### Buttons And Actions

- `Ask over evidence` searches local evidence for your question.
- `Save evidence review` records correction or supersession state for an
  existing evidence item while preserving original history.
- `Open Results search` brings you back to the Results search area.
- Advanced report and evidence-answer controls are available in Advanced.

### What To Do Here

Ask questions after data has been processed.

Examples:

- `What did I upload today?`
- `What does this warranty note say I need to do next?`
- `What failed in this build log? Cite the evidence.`

### Empty, Loading, And Error States

- No evidence means nothing has been processed into evidence yet.
- No chunks means documents have not been chunked yet.
- No reports means no report records exist.
- If Results is empty after upload, check Work.

### What Not To Use Results For

Do not treat missing results as proof that no information exists. It may only
mean the source has not been added, processed, chunked, or embedded yet.

## Settings

Settings contains safety, approval, policy, and local configuration
information.

### Main Sections

Safety, Approvals, And Policy:

- Pending approvals show requests waiting for a decision.
- Blocked actions show actions that cannot currently run.
- Approval-required actions show workflows that need explicit approval.
- External model policy shows the local-first default.

Approvals:

- Lists recent approval requests and decisions.
- Source collection approvals for manual uploads, conversation history imports,
  and user observations can be approved or denied from Settings without copying
  raw IDs into Advanced.
- Approving a collection request does not upload data by itself. Return to Add
  Data and submit the same guided workflow so the approved record can be used.

Feedback and Outcomes:

- Show user review records and outcome records when they exist.

Safety Rules:

- Shows approval defaults, allowed operation classes, external model policy,
  and runtime capability.

Settings:

- Shows local-only configuration and `.env` status.
- Shows storage paths and local model settings.
- Supports dry-run verification before saving settings.

### Buttons And Actions

- `Verify Dry Run` checks pending settings edits before save.
- `Save Settings` stays disabled until a matching dry run passes.

### What To Do Here

Use Settings to review safety posture, approve or inspect local policy state,
and verify configuration changes before saving.

### Empty, Loading, And Error States

- No approvals means no workflow is waiting for approval.
- No feedback or outcomes means nothing has been reviewed yet.
- `.env` missing or read-only status means settings cannot be saved normally.

### What Not To Use Settings For

Do not paste secrets into unrelated fields. Do not change storage paths unless
you understand the restart and data-location implications.

## Advanced

Advanced is for diagnostics and troubleshooting. Normal users usually do not
need it.

### Main Sections

Diagnostics:

- Shows technical runtime posture.
- Confirms the current Rust API and worker runtime.
- Shows old Python services as inactive or archived.
- Shows service readiness details.
- Shows recent audit events.

Advanced Route Console:

- Contains low-level API-backed controls.
- Includes source, approval, dry-run, upload, dispatch, evidence answer,
  review, pattern, and report forms.
- Shows raw JSON results.

### Buttons And Actions

Advanced actions call real local API paths. Use them only when you understand
the required IDs, permissions, and approval state.

### What To Do Here

Use Advanced when:

- Troubleshooting with support notes.
- You need raw IDs or JSON.
- You need to inspect service status.
- You need to confirm old Python services are archived.

### What Not To Use Advanced For

Do not use Advanced to guess IDs, bypass approvals, or run unsupported
workflows. The normal tabs are safer for everyday use.

## Interface Item Guide

Readiness and status messages:

- Green or ready states mean the matching check is expected to be usable.
- Error or failed states mean you should inspect the related tab.
- Empty states usually mean no records exist yet.

Primary action buttons:

- `Ask with evidence` opens Results.
- `Add data` opens Add Data.
- `Check processing` opens Work.

Recent activity and current work:

- Recent activity is based on records returned by the local API.
- It is not fake demo data.
- Empty recent activity means no matching records were returned.

Upload and source areas:

- Sources define what IGY6 is allowed to use.
- Uploads create records that can later become documents, chunks, and evidence.
- Current manual upload is best for UTF-8 text.

Processing and work status:

- Queued means waiting.
- Running means active.
- Completed means done.
- Failed means review the message and troubleshoot.

Evidence, results, and reports:

- Evidence is local record material that can support answers.
- Reports are saved output records when report workflows have been used.
- No results usually means data has not been processed yet.

Settings and safety:

- Approvals protect sensitive workflows.
- External model policy is blocked by default.
- Settings changes must pass dry-run verification before save.

Advanced diagnostics:

- Advanced can show technical names, raw JSON, route details, and service
  checks.
- Action Preview shows what IGY6 thinks a plain-language request means before
  it runs a supported action or asks for approval.
- Ambiguous, unsupported, or risky requests show clarification or approval
  posture instead of silently becoming work.
- It is for troubleshooting, not everyday use.

## Workflows

### Workflow A: Start IGY6 And Check Readiness

1. Start IGY6:

   ```bash
   scripts/run.sh
   ```

2. In another terminal, check status:

   ```bash
   scripts/status.sh
   ```

3. Open:

   ```text
   http://127.0.0.1:3000
   ```

4. On Home, check readiness.

5. If Home shows pending approvals or errors, open Settings or Work.

### Workflow B: Add Data

1. Open Add Data.
2. Review the source and upload guidance.
3. Use a simple text source when possible.
4. Prefer UTF-8 text such as notes, logs, or copied document text.
5. If approval is required, create or review the approval before collection.
6. After adding data, open Work.

What happens next:

- IGY6 records collection activity.
- Raw input can become an artifact.
- Background processing can create documents, chunks, evidence, and memory
  records.

### Workflow C: Check Processing

1. Open Work.
2. Read the queued, running, completed, and failed counts.
3. If work is queued, wait for background processing.
4. If work is running, let it finish.
5. If work completed, open Results.
6. If work failed, read the error message and check Troubleshooting.

### Workflow D: View Results

1. Open Results.
2. Review evidence and document counts.
3. Ask a question in Ask Over Evidence.
4. Save an answer record when you want the retrieval review to persist after
   reload.
5. Review saved evidence answer records to see the question, answer status,
   deterministic summary, evidence IDs, source/chunk/document trail labels, and
   feedback state.
6. If reports exist, review them in Reports.
7. If no results exist, return to Add Data or Work.

Example questions:

- `What did I upload today?`
- `What does this note say I need to do next?`
- `What failed in this log? Cite the evidence.`

Saved answer records are additive review records. They do not delete evidence,
rewrite documents or chunks, hide superseded evidence, change retrieval ranking,
or create full chat memory. Feedback can target a saved answer record. Outcomes
for answer records are not offered because the current outcome API supports only
reports, work items, predictions, recommendations, hypotheses, and patterns.

### Workflow E: Use Advanced Only When Needed

1. Open Advanced only for troubleshooting or raw diagnostics.
2. Preview a request when you need to check whether IGY6 understands it as a
   question, data-add request, report request, feedback, outcome, action,
   diagnostics, project status, or unclear request.
3. Review service readiness or old-runtime status if needed.
4. Use route console controls only when you know the exact source, approval,
   work item, or report IDs.
5. Return to normal tabs for everyday work.

## Troubleshooting

UI does not open:

- Run `scripts/status.sh`.
- Make sure the `web` service is running.
- Open `http://127.0.0.1:3000`.

System is not ready:

- Wait for services to finish starting.
- Check `scripts/status.sh`.
- Run `scripts/post-cutover-smoke.sh --check` for a non-destructive runtime
  check.

Data does not appear:

- Confirm you added a source or upload.
- Check Add Data for collection records.
- Check Work for queued or failed processing.

Processing seems stuck:

- Open Work.
- Look for queued, running, or failed items.
- Run `scripts/runtime-lifecycle-check.sh --check` to validate lifecycle
  command posture without starting or stopping services.

Upload fails:

- Confirm the content is UTF-8 text.
- Confirm the source and permission are correct.
- Confirm approval exists if approval is required.

No results yet:

- Add data first.
- Check Work for processing.
- Remember that missing evidence may mean processing has not completed.

When to run the smoke check:

```bash
scripts/post-cutover-smoke.sh --check
```

Run it when the runtime posture seems wrong, after pulling changes, or before
starting product work.

When to run lifecycle validation:

```bash
scripts/runtime-lifecycle-check.sh --check
```

Run it when start, stop, or restart commands are unclear or after editing
operator docs or scripts.

## Safety And Data Rules

- Runtime/private data belongs under `IGY6_DATA_ROOT`.
- Do not commit private data, `.env`, storage folders, artifacts, credentials,
  cookies, tokens, or collected personal data.
- The Advanced tab may show technical diagnostics and raw IDs.
- Old Python/FastAPI/Celery services are archived history, not active runtime.
- IGY6 is local-first and approval-gated for sensitive or system-changing
  workflows.

## Current Limitations

- Manual upload is best for UTF-8 text.
- Binary PDF, image, audio, and video parsing are not claimed unless a later
  DIFF adds them.
- Some source types are planned or metadata-only until their collector workflow
  is completed.
- Empty states are real empty states, not demo placeholders.
- Advanced controls may require exact IDs and approvals.
- Reports, graph reasoning, forecasting, and self-improvement workflows are
  only as complete as their current API-backed records and later DIFFs make
  them.
