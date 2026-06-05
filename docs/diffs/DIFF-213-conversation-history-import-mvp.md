# DIFF-213 - Conversation History Import MVP

Status: Complete

## Purpose

Add a controlled normal-user path for importing prior conversation/history text
as authorized local evidence.

This DIFF is the manual local import MVP only. It does not implement browser
extraction, account scraping, connector import, or external service calls.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `566aaad52389b7cee871110ecc26aa19bd16c9a9`
- `dev` ahead/behind `origin/dev` before commit: aligned with `origin/dev`
  according to `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/approvals/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-worker/src/lib.rs`
- `scripts/e2e-manual-upload-smoke.py`

## Current `conversation_history` Capability Found

- `conversation_history` was already accepted by the Rust source creation
  validator and was already listed in low-level Advanced source creation.
- The normal Add Data guided upload path only created or selected
  `manual_upload` sources.
- The Rust `/collection-runs/manual-upload` and
  `/collection-runs/manual-upload/ingest` routes rejected any source whose
  `source_type` was not exactly `manual_upload`.
- Existing worker behavior already preserves source, artifact, document, chunk,
  and evidence lineage once a UTF-8 raw artifact is accepted for normalization.

## Product Workflow Gap Found

Users could paste generic UTF-8 notes, but there was no normal-user path for
labeling prior conversation/history text as conversation history with safe
metadata such as title, date range, participants, context, corrections,
decisions, or instructions/preferences. The existing manual upload pipeline
could process the text once accepted, but the source-type gate blocked
`conversation_history`.

## UX/API/Backend Changes Made

- Added a normal Add Data conversation history import workflow.
- The workflow can create a new `conversation_history` source or select an
  existing enabled `conversation_history` source.
- The workflow accepts manual pasted UTF-8 text only.
- The workflow stores safe metadata on the raw artifact submission.
- The UI states that browser/account/connector import is planned future work
  and not implemented in this DIFF.
- The UI states that binary/media import is not part of this DIFF.
- The Rust gateway now allows `conversation_history` sources through the
  existing manual UTF-8 upload and ingest routes.
- The Rust gateway summary/work payload now records the actual source type
  while preserving the existing manual upload route contract.
- Added focused Rust tests covering conversation-history source acceptance and
  rejection of unsupported source types.
- Updated the UI guide with the new workflow.

## Source/Artifact/Document/Chunk/Evidence Lineage Behavior

- The import creates or uses a `conversation_history` source.
- The selected source permission is used for `read`/`collect` checks.
- Accepted pasted text is stored as a raw artifact through the existing local
  artifact store.
- A collection run records the source and upload summary.
- A normalization work item is queued through the existing worker pipeline.
- The worker can create normalized documents, chunks, evidence items, and
  vector work from that raw artifact using existing lineage fields.
- No raw artifact, document, chunk, evidence, or source deletion behavior was
  added.

## Fields/Metadata Stored

Conversation import metadata submitted with the raw artifact:

- `submitted_from`
- `title`
- `conversation_title`
- `conversation_date_range`
- `participants`
- `context_note`
- `contains_corrections`
- `contains_decisions`
- `contains_instructions_preferences`
- `manual_local_import_only`
- `browser_account_connector_import`
- `binary_media_import`

Source metadata:

- `created_from`
- `import_path`
- `manual_local_import_only`

Permission scope metadata:

- `path`
- `entered_from`
- `import_type`

## Fields Intentionally Not Stored

- Account credentials, tokens, cookies, private keys, or `.env` values.
- Browser session data.
- Account exports fetched by IGY6.
- Connector credentials.
- External service responses.
- Hidden summaries or hosted AI output.
- Binary PDF, image, audio, video, screenshot, or OCR payloads.
- Runtime/private data dumps from `IGY6_DATA_ROOT`.

## Unsupported States Handled

- Browser extraction is not implemented.
- Account scraping is not implemented.
- Connector import is not implemented.
- External service collection is not implemented.
- Binary/media parsing is not claimed.
- Full chat memory is not claimed.
- Hosted AI is not called.
- Approval-required conversation sources stop in pending approval state instead
  of uploading before approval.
- Existing conversation sources without usable permissions show a diagnostic
  message instead of fake collection.

## Future Browser/Account/Connector/External-Service Capability Note

Browser extraction, account imports, connector imports, and external service
collection remain future planned capabilities. They require separate scoped
DIFFs with explicit permission, dry-run preview, audit events, source policy,
bounded user-controlled scope, and no hidden external data transfer.

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- Live browser/API import was not run because it requires the owner's normal WSL
  stack/database. Backend behavior was covered by Rust validation tests and the
  UI by the Next.js build.

## Full Docker Smoke

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`. The owner should run full operator smoke locally in normal
WSL.

Owner-run local WSL verification commands:

```bash
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result
```

## Files Changed

- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-213-conversation-history-import-mvp.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Rust formatting passed.
- Rust workspace tests passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-213; DIFF-213 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No browser scraping was added.
- No account scraping was added.
- No connector import was added.
- No external service call was added.
- No hosted AI call was added.
- No hidden external data transfer was added.
- No binary PDF/image/audio/video import claim was added.
- No full chat-memory claim was added.
- No arbitrary command execution was added.
- No fake controls were added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
