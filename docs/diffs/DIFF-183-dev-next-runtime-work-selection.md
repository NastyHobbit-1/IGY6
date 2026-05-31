# DIFF-183: Dev Next Runtime Work Selection

Status: Complete

## Type

Planning / selection

## Objective

Inspect the current product-completion roadmap and recent dev DIFF history after
DIFF-182 runtime smoke verification, then select the next highest-value runtime
work item.

This DIFF is planning-only. It does not implement runtime, UI, API, worker,
database, Docker Compose, `.env`, Rust migration, merge, cherry-pick, cleanup,
or private/dev file removal work.

## Branch Policy

- Future IGY6 work happens on `dev`.
- Private/dev/build instruction files stay on `dev`.
- `main` remains the public/runtime-clean branch.
- Later, only necessary public/runtime-safe files should be selectively
  promoted to `main`.
- Do not merge `main` into `dev` unless explicitly instructed.
- Do not cherry-pick `main` into `dev` unless explicitly instructed.
- This DIFF removes no private/dev files.

## Baseline Facts

- Branch before work: `dev`.
- HEAD before work:
  `1aee6f7 Complete DIFF-182 dev runtime smoke verification`.
- `dev` was up to date with `origin/dev`.
- Working tree was clean before this DIFF.
- Latest completed dev DIFF before this planning pass was DIFF-182.
- DIFF-182 proved the local runtime stack, API live/ready probes, web UI,
  guided manual upload smoke test, manual upload source creation, permission,
  approval, collection run, raw artifact, normalization work item, completed
  work item, and document/chunk/evidence counts.
- DIFF-182 found no product/runtime bug.
- Private/dev files remained tracked on `dev`.

## Allowed Scope

- This DIFF file only.

Optional documentation files were allowed only if a typo or stale reference
blocked understanding. No optional documentation edit was needed.

## Prohibited Scope

- Do not remove anything from `dev`.
- Do not remove `.codex`.
- Do not remove `AGENTS.md`.
- Do not remove `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`.
- Do not remove `docs/agents`.
- Do not remove `docs/plans`.
- Do not merge `main` into `dev`.
- Do not cherry-pick `main` into `dev`.
- Do not edit runtime code.
- Do not start Rust migration implementation.
- Do not implement the selected next runtime work in this DIFF.
- Do not modify `.env`.
- Do not print secrets.
- Do not read or dump runtime/private data from `IGY6_DATA_ROOT`.
- Do not run destructive commands.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `docs/diffs/DIFF-176-request-understanding-clarification-flow.md`
- `docs/diffs/DIFF-177-main-dev-branch-policy-cleanup.md`
- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `docs/diffs/DIFF-181-dev-governance-status-reconciliation.md`
- `docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- tracked private/dev file inventory from `git ls-files AGENTS.md .codex
  Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents
  docs/plans | sort`

## Roadmap And Gap Findings

- DIFF-178 identified the shortest product path as: guided data add, worker
  processing, evidence/retrieval workflow, report workflow, UI blockers, and
  runtime recovery/logging.
- DIFF-180 implemented the guided manual UTF-8 text source/upload path in Add
  Data without forcing normal users through Advanced route IDs.
- DIFF-182 verified that guided upload can create the source, permission,
  approval, collection run, raw artifact, work item, completed processing, and
  document/chunk/evidence records.
- DIFF-182 also recorded that the first retrieval preview returned no hits
  while worker processing was still pending; after the bounded wait, downstream
  document/chunk/evidence records existed. The next user-visible gap is proving
  and, if necessary, polishing the follow-through from completed manual upload
  into Results retrieval.
- `docs/ui/README.md` already tells users to open Results and ask over evidence
  after processing, so the next DIFF should verify that this normal path works
  with a freshly uploaded synthetic text record.
- The roadmap still lists report workflow UX, work failure recovery, evidence
  answer review, persisted chat/answer history, source trust management, and
  broader ingestion types as later gaps.
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md` is historical migration
  manager context. It is not the current product-roadmap source because the
  Rust-only application API and worker runtime have already been completed.

## Candidate Ranking

### 1. DIFF-184: Manual Upload Evidence Retrieval Followthrough

Problem it solves:

After DIFF-182, IGY6 has proof that manual upload creates evidence records, but
the highest-value normal-user question is whether a freshly uploaded text can be
found and answered from Results after worker completion.

Files likely involved:

- `docs/diffs/DIFF-184-dev-manual-upload-evidence-retrieval-followthrough.md`
- `scripts/e2e-manual-upload-smoke.py`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `apps/web/src/app/api/` retrieval/evidence proxy files if a verified proxy
  bug appears
- `crates/igy6-gateway/src/lib.rs` or retrieval/vector crates only if a
  verified runtime bug appears
- `docs/ui/README.md` only if user-facing behavior changes

Expected behavior:

- A synthetic manual text upload with a unique token completes processing.
- Results retrieval over local evidence can find that unique token after worker
  completion.
- If retrieval succeeds, the DIFF records the verified path and may add or
  extend a bounded smoke helper.
- If retrieval fails after processing is complete, the DIFF scopes the smallest
  fix needed for the verified bug.
- No binary parsing, external model dependency, broad refactor, or private data
  read is introduced.

Verification commands/tests:

- `git status --short`
- `git diff --check`
- `docker compose -f infra/docker-compose.yml --env-file .env config --quiet`
- key-presence check for `IGY6_DATA_ROOT` without printing the value
- `npm --prefix apps/web run build`
- port conflict check for `:3000|:8000|:8765`
- documented stack startup command if no conflict blocks it
- API live and ready checks
- web UI HTTP check
- synthetic manual upload smoke run
- bounded retrieval check for the synthetic unique token after work completion
- final scoped status and diff review

Risk level:

Medium. It exercises the live stack and may uncover a real retrieval, vector,
or UI follow-through bug, but it builds directly on verified DIFF-182 behavior
and can stay small.

Reason to do now:

It closes the biggest user-visible gap after upload: the user needs to know that
uploaded text becomes searchable, cited evidence in Results. It is local-first,
concrete, testable, and directly follows the verified manual upload path.

### 2. DIFF-184: Work Status And Recovery UX Polish

Problem it solves:

Users need clearer completion/failure feedback after upload, including what to
do if processing stalls or fails.

Files likely involved:

- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- work-item API proxy files if needed
- `docs/ui/README.md`
- a new DIFF record

Expected behavior:

- Work gives clearer normal-user status for queued, running, completed, and
  failed items.
- Completed manual upload work points users toward Results.
- Failed work exposes safe troubleshooting guidance without raw route guessing.

Verification commands/tests:

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- targeted UI smoke or source review
- optional live stack check if scoped

Risk level:

Low to medium. Mostly UI-facing, but it could hide or overstate worker state if
not kept tightly aligned with existing API records.

Reason to defer:

It improves the path, but the more fundamental product question is whether
completed upload evidence is actually retrievable from Results.

### 3. DIFF-184: Evidence Answer Review UX

Problem it solves:

Results should show facts, assumptions, uncertainty, missing information,
citations, and source trails in a normal-user review flow.

Files likely involved:

- `apps/web/src/app/page.tsx`
- evidence-answer and retrieval proxy files
- feedback/outcome proxy files if existing controls are wired
- `docs/ui/README.md`
- a new DIFF record

Expected behavior:

- Users can inspect an evidence-grounded answer and its supporting records from
  Results.
- Existing feedback/outcome routes can be reached from the answer review path
  where practical.

Verification commands/tests:

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- route/proxy checks for answer, feedback, and outcome behavior
- optional live stack evidence-answer smoke

Risk level:

Medium. It touches a more complex answer/review surface and may need several
small UI and route checks.

Reason to defer:

This should follow a verified retrieval follow-through so answer review work is
built on a known searchable evidence path.

### 4. DIFF-184: Basic Report Workflow UX And Export

Problem it solves:

Report creation/rendering currently remains more Advanced-oriented, while
normal users need decision-ready outputs.

Files likely involved:

- `apps/web/src/app/page.tsx`
- report proxy files
- report-related Rust route files only if a verified bug appears
- `docs/ui/README.md`
- a new DIFF record

Expected behavior:

- Results exposes a clearer basic report create/render/review path.
- Reports preserve evidence boundaries and do not overclaim unsupported
  intelligence behavior.

Verification commands/tests:

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- report create/render smoke against synthetic evidence if scoped

Risk level:

Medium. Reports are important but depend on reliable retrieval/evidence
follow-through first.

Reason to defer:

It is better sequenced after uploaded text can be reliably retrieved and cited
from Results.

## Selected Next DIFF

Recommended next DIFF:

`DIFF-184: Manual Upload Evidence Retrieval Followthrough`

Rationale:

DIFF-182 proved that a guided manual upload can produce processed evidence
records. The highest-value next product step is proving the normal user can
retrieve and ask over that newly uploaded evidence from Results. This directly
closes the next user-visible gap, preserves local-first behavior, does not
require private/dev file removal or main-branch work, can be implemented as one
bounded DIFF, and has concrete verification.

## Paste-Ready Prompt For Selected Next DIFF

```text
You are working in the IGY6 repo on branch dev.

Branch policy:

* Future IGY6 work happens on dev.
* Do not remove anything from dev.
* Do not remove private/dev/build instruction files from dev.
* main is the public/runtime-clean branch.
* Later, only necessary public/runtime-safe files will be selectively promoted to main.
* Do not merge main into dev unless explicitly instructed.
* Do not cherry-pick main into dev unless explicitly instructed.

Current known state:

* Latest completed dev DIFF: DIFF-183 dev next runtime work selection.
* DIFF-182 proved the runtime stack, API live/ready probes, web UI, guided manual upload smoke test, manual upload source creation, permission, approval, collection run, raw artifact, normalization work item, completed work item, and document/chunk/evidence counts.
* DIFF-182 found no product/runtime bug.
* DIFF-183 selected manual upload evidence retrieval follow-through as the next highest-value runtime work.
* Private/dev files must remain tracked on dev.

Goal:
Create the next available dev DIFF, DIFF-184, to verify and, only if needed, minimally fix the follow-through from a completed guided manual text upload into Results evidence retrieval.

Create:

* docs/diffs/DIFF-184-dev-manual-upload-evidence-retrieval-followthrough.md

This DIFF is verification-first. Do not make code changes unless a specific retrieval/UI/API bug is found during verification and the DIFF is updated to explicitly scope the smallest fix.

Required pre-work inspection:

git status --short
git branch --show-current
git log --oneline --decorate -12
git branch -vv
git diff --name-status
git diff --check
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/BRANCH_POLICY.md
sed -n '1,260p' docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md
sed -n '1,260p' docs/diffs/DIFF-183-dev-next-runtime-work-selection.md
sed -n '1,260p' docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md
sed -n '1,260p' scripts/e2e-manual-upload-smoke.py
sed -n '1,260p' apps/web/src/app/page.tsx
find apps/web/src/app/api -maxdepth 4 -type f | sort
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort

Allowed scope:

* docs/diffs/DIFF-184-dev-manual-upload-evidence-retrieval-followthrough.md
* scripts/e2e-manual-upload-smoke.py only if enhancing the synthetic smoke helper is the smallest safe verification path
* apps/web/src/app/page.tsx only if the Results UI follow-through has a verified small bug or missing state
* apps/web/src/app/api/chat/retrieval-preview/route.ts or adjacent retrieval/evidence proxy files only if a verified proxy bug is found
* smallest Rust retrieval/gateway/vector files only if a verified runtime bug is found and the DIFF is updated before editing
* docs/ui/README.md only if user-facing behavior changes

Prohibited actions:

* Do not remove anything from dev.
* Do not remove private/dev/build instruction files.
* Do not merge main into dev.
* Do not cherry-pick main into dev.
* Do not edit Docker Compose unless a verified Compose bug is explicitly scoped after reporting.
* Do not modify .env.
* Do not print secrets.
* Do not read or dump runtime/private data from IGY6_DATA_ROOT.
* Do not add binary PDF, image OCR, audio/video, browser, web, or router collection.
* Do not start Rust migration work.
* Do not perform broad refactors.
* Do not use external hosted model calls.
* Do not run destructive commands.

Implementation requirements:

1. Create the DIFF-184 record before changing any runtime/UI/script files.
2. Verify the existing guided manual text upload path with a synthetic unique text input.
3. Wait for the related work item to reach a terminal state.
4. After completion, verify that Results/retrieval can find the uploaded text or a unique token from the uploaded text.
5. If retrieval succeeds without code changes, record the verified behavior and consider adding only a bounded smoke-helper enhancement if useful.
6. If retrieval fails after processing is complete, record the bug in DIFF-184 before editing, then make only the smallest scoped fix.
7. Preserve local-first behavior and deterministic fallback behavior.
8. Keep unsupported binary/media/collector claims out of UI and docs.

Suggested verification commands, adjusted only if repo docs require a different command:

git status --short
git diff --check
docker compose -f infra/docker-compose.yml --env-file .env config --quiet
grep -q '^IGY6_DATA_ROOT=' .env && echo "IGY6_DATA_ROOT is set in .env" || echo "IGY6_DATA_ROOT is missing from .env"
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
npm --prefix apps/web run build
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true

If no port conflict blocks startup, run the documented stack command and test:

* API live endpoint
* API ready endpoint
* web UI route
* synthetic guided/manual upload smoke path
* completed work item status
* retrieval preview or Results route for the uploaded unique token after processing completes

Verification before commit:

git status --short
git diff --check
git diff --name-status
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort

Commit:
git add -A
git commit -m "Complete DIFF-184 manual upload retrieval followthrough"

Final response must include:

* new DIFF created
* current branch and HEAD before work
* whether private/dev files remained tracked
* runtime verification commands run
* manual upload retrieval verification result
* bugs found, if any
* files changed
* verification summary
* commit hash, or explicit reason no commit was made
```

## Commands Run

Pre-work inspection:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -12
git branch -vv
git diff --name-status
git diff --check
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/BRANCH_POLICY.md
find docs/diffs -maxdepth 1 -type f | sort | tail -60
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
sed -n '1,260p' docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md
sed -n '1,260p' docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md
sed -n '1,260p' docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md
sed -n '1,260p' docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md
sed -n '1,220p' README.md
sed -n '1,520p' docs/ui/README.md
sed -n '1,180p' docs/diffs/DIFF-176-request-understanding-clarification-flow.md
sed -n '1,180p' docs/diffs/DIFF-177-main-dev-branch-policy-cleanup.md
sed -n '1,220p' docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md
sed -n '1,180p' docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md
sed -n '1,220p' docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md
sed -n '1,220p' docs/diffs/DIFF-181-dev-governance-status-reconciliation.md
```

Verification before commit:

```bash
git status --short
git diff --check
git diff --name-status
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

## Verification Summary

- Pre-work confirmed branch `dev`, clean working tree, and HEAD
  `1aee6f7 Complete DIFF-182 dev runtime smoke verification`.
- `git branch -vv` showed `dev` at `1aee6f7 [origin/dev]`, so `dev` was synced
  with `origin/dev` before this DIFF.
- `git diff --check` passed before editing.
- DIFF-183 was the next available dev DIFF; `find docs/diffs ... | tail`
  showed DIFF-182 as the highest existing DIFF before this record.
- Stale-status grep still returned DIFF-177 and DIFF-180 as `Draft`, plus the
  DIFF template and command strings inside DIFF-182 and this DIFF. Those records
  are outside this DIFF's allowed scope, or are literal recorded commands, and
  were not edited.
- Private/dev files remained tracked on `dev`.
- No runtime code, UI code, API code, Docker Compose, `.env`, private/dev
  instruction files, merge, cherry-pick, cleanup, or Rust migration work was
  changed.

## Final Status

Complete.
