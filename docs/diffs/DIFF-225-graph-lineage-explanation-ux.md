# DIFF-225 - Graph Lineage Explanation UX

Status: Complete

## Purpose

Explain why records are connected. Users should be able to inspect source →
artifact → document → chunk → evidence → answer/report/task lineage without
needing raw database knowledge.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `e2d715e6a4cbad54a9b239cb9d57358ba20de343`
- DIFF-224 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-worker/src/lib.rs`
- `crates/igy6-normalization/src/lib.rs`

## Existing Capability Found

- The UI already loads sources, collection runs, artifacts, documents, chunks,
  evidence, evidence answers, reports, task plans, and Neo4j graph schema
  status.
- Source and evidence detail panels already expose scoped lineage fragments.
- Neo4j schema visibility exists through `/memory/graph/schema`, but full graph
  reasoning is not complete.

## Product Changes Made

- Added a normal-user `Lineage Explanation` panel in Results.
- The panel explains source-to-output lineage using loaded relational records:
  - source to artifact;
  - artifact to document;
  - document to chunk;
  - chunk to evidence;
  - evidence to answer/report/task links where available.
- The panel shows:
  - source trust level;
  - source sensitivity;
  - enabled/disabled state;
  - correction/supersession review states from linked evidence;
  - record counts per source;
  - safe next action.
- Neo4j schema status is shown honestly when visible.
- If graph schema is missing or incomplete, the UI uses relational fallback and
  states that full graph reasoning is not claimed.
- Updated the UI guide.

## Backend/API Changes

No backend or proxy changes were required.

Existing read routes already expose the data needed for normal-user lineage
explanation.

## Unsupported States Handled

- The panel does not claim full graph reasoning.
- The panel does not claim correlation discovery.
- The panel does not dump raw artifact contents.
- The panel does not expose secrets or raw runtime paths.
- Missing links are shown as incomplete lineage rather than hidden.

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-225-graph-lineage-explanation-ux.md`
- `docs/ui/README.md`

## Classification

UI only.

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports older draft/template/status-command strings
  outside DIFF-225; DIFF-225 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No hidden data transfer was added.
- No arbitrary command execution was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete, restore, or backup archive creation was performed.
- No full graph-reasoning, full forecasting, or autonomous self-improvement
  claim was added.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
