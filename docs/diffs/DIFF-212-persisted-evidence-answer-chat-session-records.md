# DIFF-212 - Persisted Evidence Answer / Chat Session Records

Status: Complete

## Purpose

Add a safe persistence layer and normal-user review surface for evidence-backed
answer records. Users can save a retrieval/evidence-answer result, review prior
saved answer records, see the evidence identifiers used, and attach feedback
where the backend supports the target.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `d7f3cdeb04f974f120458211fb5c8b5627a601a0`
- `dev` ahead/behind `origin/dev` before commit: aligned with `origin/dev`
  according to `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`
- `docs/diffs/DIFF-209-persist-evidence-check-summary-on-task-plans.md`
- `docs/diffs/DIFF-211-evidence-correction-supersession-ux.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-evidence-answer/src/lib.rs`

## Current Persisted Answer/Chat Capability Found

- `/chat/retrieval-preview` is the live local evidence retrieval path used by
  Results.
- `/chat/evidence-answer` returns a bounded contract/fallback answer packet,
  but it is not the live hydrated Results path and does not persist records.
- Evidence answer packet code can represent `answer_status`, facts,
  assumptions, inferences, uncertainty, missing information, and source trails.
- Reports, feedback, outcomes, and task plans are already persisted, but no
  answer/chat/session record route or table existed before this DIFF.
- Feedback did not accept `evidence_answer` targets before this DIFF.
- Outcomes remain limited to prediction, recommendation, work item,
  hypothesis, pattern, and report targets.

## Product Workflow Gap Found

Users could ask over local evidence and inspect the immediate retrieval result,
but that answer/retrieval review was transient. After reload, there was no
normal Results history showing what question was asked, what answer status was
returned, which evidence identifiers were used, or where feedback could be
attached to the answer record.

## Persistence Approach Chosen

- Added a focused Rust gateway persistence surface:
  - `POST /evidence-answers`
  - `GET /evidence-answers`
  - `GET /evidence-answers/{answer_id}`
- The gateway creates the scoped `evidence_answer_records` table if needed.
- Records are additive review/session records. They do not mutate evidence,
  documents, chunks, sources, raw artifacts, reports, or retrieval behavior.
- The Results UI saves the current retrieval-preview result as a safe answer
  record only when the user chooses `Save answer record`.

## UX/API/Backend Changes Made

- Added a Results save control for the Ask Over Evidence workflow.
- Added a Results answer history section listing recent saved answer records.
- The history shows question summary, answer status, deterministic answer text,
  evidence/document/chunk/source identifiers, retrieval mode/count, local model
  status, and immutable-history guidance.
- Added answer records to the Results feedback target list.
- Added backend validation for answer record payloads.
- Added audit event creation with event type `evidence_answer.created`.
- Added `evidence_answer` to backend feedback target validation.
- Added a Next.js proxy route:
  - `POST /api/evidence-answers`
- Updated UI documentation for persisted answer records.

## Safe Fields Stored

- `id`
- `user_question`
- `answer_status`
- `answer_text`
- `facts`
- `assumptions`
- `inferences`
- `uncertainty`
- `missing_information`
- `evidence_item_ids`
- `document_ids`
- `chunk_ids`
- `source_ids`
- `safe_labels`
- `retrieval_mode`
- `retrieval_count`
- `local_model_status`
- `metadata_json`
- `created_at`
- `updated_at`

## Fields Intentionally Not Stored

- `.env` values.
- Credentials, tokens, cookies, private keys, or secrets.
- Raw artifact contents.
- Full normalized document text.
- Full chunk text or evidence snippets from retrieval-preview rendering.
- Full retrieval JSON payloads.
- Hosted AI responses.
- External conversation memory.
- Runtime/private data dumps from `IGY6_DATA_ROOT`.

## Feedback/Outcome Target Behavior

- Feedback can target persisted answer records through target type
  `evidence_answer`.
- Outcomes are not offered for answer records because the existing outcome route
  validates only prediction, recommendation, work item, hypothesis, pattern, and
  report targets.
- The UI states this unsupported outcome state honestly and does not render fake
  outcome controls for answer records.

## Unsupported States Handled

- This DIFF does not claim full chat memory.
- This DIFF does not claim autonomous reasoning.
- This DIFF does not call hosted AI.
- This DIFF does not change retrieval ranking, filtering, or policy
  enforcement.
- Saved records do not silently hide superseded or corrected evidence.
- Empty history remains an honest empty state.
- Missing or invalid persistence support is shown as an error instead of a fake
  saved state.

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
- A live answer-record save against PostgreSQL was not run because that requires
  the owner's normal WSL stack/database. Backend behavior was covered by Rust
  route and validation tests.

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

- `apps/web/src/app/api/evidence-answers/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Rust formatting passed.
- Rust workspace tests passed.
- Private/dev instruction files remained tracked on `dev`.
- No stale active/in-progress/draft DIFF was introduced by DIFF-212.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No full chat-memory claim was added.
- No autonomous reasoning claim was added.
- No hosted AI call was added.
- No hidden external data transfer was added.
- No arbitrary command execution was added.
- No fake controls were added.
- No `.env` file was edited.
- No runtime/private data was dumped.
- No evidence, raw artifact, document, chunk, or source deletion was added.
- No retrieval-ranking, retrieval-filtering, or policy-enforcement claim was
  added.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
