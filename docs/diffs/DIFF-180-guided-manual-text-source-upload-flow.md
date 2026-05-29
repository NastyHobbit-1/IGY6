# DIFF-180: Guided Manual Text Source And Upload Flow

Status: Draft

## Type

UI/product workflow

## Objective

Make the normal Add Data path usable for manual UTF-8 text ingestion without forcing the user into Advanced ID-driven route controls.

The goal is a guided normal-user workflow that can create or select the needed manual text source context, handle permission/approval requirements where already supported, submit UTF-8 text for upload/processing through existing routes, and show clear next steps without claiming unsupported binary parsing.

This DIFF is scoped to the guided manual text path only. It must not expand binary parsing, add new source collectors, or change worker semantics.

## Baseline Facts

- `main` is the clean runtime/product branch.
- `dev` is the development/build-agent branch.
- `dev` must not be merged directly into `main`.
- DIFF-179 is on `main` and cleaned stale active-runtime wording.
- DIFF-178 identifies the current Add Data workflow as partial because core source/upload controls still rely on Advanced route forms and exact IDs.
- The strongest current ingestion path is UTF-8 text-oriented manual upload/local project processing.
- Binary PDF, image, audio, and video parsing must not be claimed complete.
- The UI is a normal-user tabbed dashboard with Home, Add Data, Work, Results, Settings, and Advanced.
- Advanced may retain raw route/debug controls, but normal manual text intake should not require raw ID guessing.

## Required Inputs To Inspect

Codex must inspect at minimum:

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-agent-api/src/lib.rs`
- relevant source, permission, approval, upload, collection, and work routes in the Rust gateway and web proxy layer

Codex may inspect additional files if needed, but must keep the implementation bounded to this workflow.

## Allowed Scope

Codex may:

- add or improve normal-user Add Data UI for manual UTF-8 text upload;
- hide raw source/permission/approval IDs from the primary manual-text path where existing APIs allow it;
- use existing source, permission, approval, collection/upload, work, and status routes;
- add minimal web API proxy helpers only if they preserve existing backend contracts;
- add clear text-only limitation messaging;
- add honest success, pending, error, and next-step states;
- preserve Advanced raw controls for diagnostics;
- update `docs/ui/README.md` if the user-facing workflow changes;
- update this DIFF Result and Verification Result sections;
- add or update narrowly scoped tests if practical and supported by the existing test setup.

## Prohibited Scope

Codex must not:

- add binary PDF parsing;
- add image OCR;
- add audio/video transcription;
- add browser/web/router collectors;
- add new external connectors;
- add new dependencies unless explicitly justified and approved in this DIFF result;
- change database migrations unless explicitly required and approved before implementation;
- change worker execution semantics;
- change service configuration for Qdrant, Neo4j, MLflow, Phoenix, Redis, or PostgreSQL;
- change Docker Compose;
- change `.env` or `.env.example`;
- start, stop, or restart services;
- remove Advanced route controls;
- claim unsupported source types are complete;
- edit locked DIFFs;
- merge `dev` into `main`;
- promote dev-only instruction files to `main`.

## UX Requirements

The guided Add Data flow must:

1. Clearly say this path supports manual UTF-8 text only.
2. Provide a normal-user way to enter or paste text.
3. Provide a normal-user way to name or describe the text/source.
4. Create/use the needed manual text source context through existing APIs when possible.
5. Handle approval-required state clearly when existing APIs require approval.
6. Submit the text through the existing manual upload/collection path.
7. Show what happened next: queued work, created artifact/document/evidence path, or clear error.
8. Tell the user where to inspect processing status in Work and results in Results.
9. Avoid raw ID guessing in the main path.
10. Keep Advanced available for low-level inspection and troubleshooting.

## Completion Criteria

This DIFF is complete when:

- normal users can start manual UTF-8 text ingestion from Add Data without using Advanced route forms for the primary path;
- binary/media limitations are visible and honest;
- the workflow uses existing backend contracts or explicitly documented minimal proxy helpers;
- errors are understandable and do not require knowing internal IDs for the normal path;
- `docs/ui/README.md` is updated if the Add Data user flow changes;
- this DIFF file has Result and Verification Result updated;
- no prohibited runtime, service, dependency, migration, or collector expansion occurs.

## Verification

Required verification:

- `git status --short`
- `git branch --show-current`
- `git diff --name-status`
- `git diff --check`
- `npm --prefix apps/web run build`
- scoped review confirming changed files stay inside DIFF-180 allowed scope
- confirm no binary/media parsing claims were added
- confirm no `dev` to `main` merge occurred

Recommended verification if practical:

- a synthetic/manual UI or route walkthrough using safe sample text;
- unit or integration tests only where existing setup supports them without service start/stop.

Do not run live service start/stop/restart for this DIFF.

## Result

Pending.

## Verification Result

Pending.
