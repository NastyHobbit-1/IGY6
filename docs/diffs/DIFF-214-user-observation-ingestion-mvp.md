# DIFF-214 - User Observation Ingestion MVP

Status: Complete

## Purpose

Add a normal-user path for recording first-party user observations as local
evidence. A user observation is something the owner directly observed, decided,
prefers, corrected, or wants IGY6 to remember as local context.

This DIFF is local/manual only. It does not scrape accounts, read browser data,
call hosted AI, infer hidden personal data, or claim complete long-term memory.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `74959b6a5bfca5dc97a1a67ebcba99c0b05df842`
- `dev` ahead/behind `origin/dev` before commit: aligned with `origin/dev`
  according to `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-210-source-trust-sensitivity-management-ux.md`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-213-conversation-history-import-mvp.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-worker/src/lib.rs`
- `crates/igy6-normalization/src/lib.rs`
- `crates/igy6-chunking/src/lib.rs`
- `scripts/e2e-manual-upload-smoke.py`

## Current `user_observation` Capability Found

- `user_observation` was already accepted by the Rust source creation
  validator and appeared in the Advanced source type dropdown.
- The normal Add Data guided workflows did not provide a first-party
  observation ingestion path.
- The Rust `/collection-runs/manual-upload` and
  `/collection-runs/manual-upload/ingest` routes accepted `manual_upload` and
  `conversation_history` sources, but rejected `user_observation` sources before
  this DIFF.
- Existing worker behavior already preserves source, artifact, document, chunk,
  and evidence lineage once a UTF-8 raw artifact is accepted for normalization.

## Product Workflow Gap Found

Users could paste generic text or conversation history, but there was no
normal-user way to record an owner-provided observation, decision, preference,
correction, or note as local evidence with observation-specific metadata.
Advanced could create a `user_observation` source, but that source type could
not honestly complete the manual text ingestion path.

## UX/API/Backend Changes Made

- Added a normal Add Data user observation ingestion workflow.
- The workflow can create a new `user_observation` source or select an existing
  enabled `user_observation` source.
- The workflow accepts owner-entered UTF-8 text only.
- The workflow clearly states observations are owner-provided first-party local
  context and are not automatically verified truth.
- The workflow clearly states it does not extract hidden memory, scrape
  accounts, read browsers, use connectors, call hosted AI, or read external
  services.
- The workflow creates approval requests and stops in pending state when the
  selected source permission requires approval.
- The Rust gateway now allows `user_observation` sources through the existing
  manual UTF-8 upload and ingest routes.
- The Rust gateway work payload records `source_type: user_observation` through
  the existing source-type metadata path.
- Added focused Rust assertions covering `user_observation` manual text
  collection support.
- Updated the UI guide with the new workflow.

## Source/Artifact/Document/Chunk/Evidence Lineage Behavior

- The ingestion path creates or uses a `user_observation` source.
- The selected source permission is used for `read`/`collect` checks.
- Accepted observation text is stored as a raw artifact through the existing
  local artifact store.
- A collection run records the source and upload summary.
- A normalization work item is queued through the existing worker pipeline.
- The worker can create normalized documents, chunks, evidence items, and
  vector work from that raw artifact using existing lineage fields.
- No raw artifact, document, chunk, evidence, or source deletion behavior was
  added.

## Fields/Metadata Stored

Observation metadata submitted with the raw artifact:

- `submitted_from`
- `title`
- `observation_title`
- `observation_type`
- `observed_at_or_decided_at`
- `confidence`
- `tags`
- `related_record_labels_or_ids`
- `related_links_validated`
- `sensitivity_flag`
- `owner_provided_first_party_context`
- `automatic_truth_verification`
- `hidden_memory_extraction`
- `account_or_browser_scraping`
- `hosted_ai_processing`
- `external_service_collection`

Source metadata:

- `created_from`
- `import_path`
- `owner_provided_first_party_context`
- `automatic_verification`

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
- Hidden memory extraction results.
- Hosted AI output.
- Binary PDF, image, audio, video, screenshot, or OCR payloads.
- Runtime/private data dumps from `IGY6_DATA_ROOT`.
- Validated relationship edges for related source/evidence/task IDs; the MVP
  stores those entries as plain text labels/IDs only.

## Unsupported States Handled

- Observation text is not automatically verified as true.
- User observations are not claimed to be externally confirmed facts.
- Complete long-term memory is not claimed.
- Automatic personal-data extraction is not claimed.
- Browser scraping is not implemented.
- Account scraping is not implemented.
- Connector import is not implemented.
- External service collection is not implemented.
- Hosted AI is not called.
- Binary/media parsing is not claimed.
- Related source/evidence/task entries are not validated links in this DIFF.
- Approval-required observation sources stop in pending approval state instead
  of uploading before approval.
- Existing observation sources without usable permissions show a diagnostic
  message instead of fake collection.

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
- A live observation ingestion against PostgreSQL was not run because it
  requires the owner's normal WSL stack/database. Backend behavior was covered
  by Rust validation tests and the UI by the Next.js build.

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
- `docs/diffs/DIFF-214-user-observation-ingestion-mvp.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Rust formatting passed.
- Rust workspace tests passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-214; DIFF-214 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No browser scraping was added.
- No account scraping was added.
- No connector import was added.
- No external service call was added.
- No hosted AI call was added.
- No hidden external data transfer was added.
- No binary PDF/image/audio/video import claim was added.
- No full long-term memory claim was added.
- No automatic personal-data extraction claim was added.
- No arbitrary command execution was added.
- No fake controls were added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
