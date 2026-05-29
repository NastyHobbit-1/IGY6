# DIFF-178: Product Completion Roadmap And Gap Audit

Status: Draft

## Type

Documentation / planning / audit

## Objective

Create a grounded product completion roadmap and gap audit for IGY6 after DIFF-177.

The purpose of this DIFF is to inspect the current repository, current runtime posture, UI documentation, API/worker architecture, and existing DIFF history, then produce a practical completion plan that separates:

- what is already complete;
- what is partially implemented;
- what is missing;
- what is required for a basic usable product;
- what is required for a solid local MVP;
- what is required for the full adaptive-intelligence vision.

This DIFF is planning-only. It must not change runtime code.

## Baseline Facts

- `main` is the clean runtime/product branch.
- `dev` is the development/build-agent branch and may contain dev-only instruction material.
- `dev` must not be merged directly into `main`.
- DIFF-176 is on `main` and implements the request-understanding clarification flow.
- DIFF-177 is on `main` and documents the main/dev branch boundary.
- `AGENTS.md` on `dev` is the Codex entrypoint and points to `docs/agents/CODEX_PROMPT_BASELINE.md`.
- The active application runtime is Rust API gateway plus Rust worker daemon plus Next.js web UI.
- Legacy Python/FastAPI and Python/Celery worker source trees are archived/inactive, not active runtime services.
- The strongest current user-facing path is UTF-8 text-oriented source/upload processing into artifacts, documents, chunks, evidence, vector memory, reports, audit records, approvals, and diagnostics where implemented.
- Binary PDF, image, audio, and video parsing must not be claimed complete unless a later scoped DIFF adds and verifies them.

## Required Inputs To Inspect

Codex must inspect at minimum:

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `configs/rust-cutover-manifest.json`
- `infra/docker-compose.yml`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-gateway/`
- `crates/igy6-worker/`
- `crates/igy6-agent-api/`
- `crates/igy6-evidence-answer/`
- `crates/igy6-llm/`
- `crates/igy6-artifacts/`
- `crates/igy6-normalization/`
- `crates/igy6-chunking/`
- `crates/igy6-vector-memory/`
- relevant existing DIFF records under `docs/diffs/`

Codex may inspect additional files when needed, but must keep the audit bounded and read-only unless writing the allowed roadmap output.

## Allowed Scope

Codex may:

- inspect repository files;
- summarize current product/runtime state;
- identify completed, partial, and missing capabilities;
- create or update a roadmap document under `docs/plans/` on `dev`;
- update this DIFF result and verification sections;
- define the next ordered DIFF sequence;
- estimate rough DIFF count for basic usable product, solid local MVP, and full completion;
- explicitly list risks, blockers, and unknowns;
- explicitly list unsupported claims that must not be made yet.

Preferred output file:

- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`

## Prohibited Scope

Codex must not:

- change runtime code;
- change Rust crates;
- change Next.js UI code;
- change Docker Compose;
- change `.env` or `.env.example`;
- mutate runtime/private data;
- start, stop, or restart services;
- run destructive commands;
- add new dependencies;
- modify database migrations;
- alter API routes;
- alter worker execution behavior;
- alter local LLM behavior;
- claim unsupported capabilities as complete;
- edit locked DIFFs;
- merge `dev` into `main`;
- promote dev-only instruction files to `main`.

## Required Roadmap Contents

The roadmap/audit document must include:

1. Current repo/runtime summary.
2. Current user-facing workflow summary.
3. Completed capability list.
4. Partial capability list.
5. Missing capability list.
6. Unsupported capability claims to avoid.
7. Product risk/blocker list.
8. Ordered next-DIFF plan.
9. DIFF count estimate for:
   - basic usable product;
   - solid local MVP;
   - full adaptive-intelligence product.
10. Recommended next 10 to 15 DIFFs in order, with one-paragraph scope each.
11. Verification commands used.
12. Open questions that need owner decision.

## Completion Criteria

This DIFF is complete when:

- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md` exists on `dev`.
- The roadmap is grounded in inspected repo files, not speculation.
- The roadmap separates complete, partial, missing, and unsupported capabilities.
- The roadmap defines an ordered sequence of next DIFFs.
- The roadmap gives rough DIFF counts for basic usable product, solid local MVP, and full completion.
- This DIFF file has Result and Verification Result updated.
- No runtime code is changed.
- No main promotion is performed.

## Verification

Required minimum verification:

- `git status --short`
- `git branch --show-current`
- `git diff --name-status HEAD~1..HEAD` or equivalent scoped diff review
- `git diff --check`
- confirm changed files are limited to this DIFF and the roadmap/audit document

Optional read-only verification when useful:

- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`

Do not run live service start/stop or destructive verification for this DIFF.

## Result

Completed.

Created `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md` on
`dev`.

The roadmap records:

- current Rust API gateway, Rust worker daemon, Next.js UI, and local service
  runtime posture;
- current normal-user workflow through Home, Add Data, Work, Results, Settings,
  and Advanced;
- completed, partial, missing, and unsupported capabilities;
- risks and blockers that must not be hidden by product wording;
- an ordered next-DIFF plan;
- rough DIFF-count estimates for a basic usable product, solid local MVP, and
  full adaptive-intelligence product;
- recommended DIFF-179 through DIFF-193 scopes;
- verification commands used during read-only inspection;
- owner decisions still needed.

No runtime code, Rust crates, Next.js UI code, Docker Compose, `.env`,
`.env.example`, migrations, API routes, worker behavior, local LLM behavior, or
runtime/private data were changed.

## Verification Result

Passed for the DIFF-178 documentation scope.

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -5
git diff --stat
git diff --name-status
git diff --check
```

Read-only inspection also covered the required files and directories listed in
this DIFF, including `AGENTS.md`, `docs/agents/CODEX_PROMPT_BASELINE.md`,
`docs/BRANCH_POLICY.md`, `README.md`, `docs/ui/README.md`,
`configs/rust-cutover-manifest.json`, `infra/docker-compose.yml`,
`apps/web/src/app/page.tsx`, `apps/web/src/app/api/`, the required Rust crates,
`docs/diffs/DIFF_PROCESS.md`, `docs/diffs/DIFF_TEMPLATE.md`, and relevant
recent DIFF records.

`git diff --check` passed.

Changed files are limited to:

- `docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`

Optional live/runtime checks were not run because this DIFF is planning-only and
explicitly prohibits starting, stopping, restarting, or mutating runtime
services or runtime/private data.
