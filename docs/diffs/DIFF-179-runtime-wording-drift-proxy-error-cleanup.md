# DIFF-179: Runtime Wording Drift And Proxy Error Cleanup

Status: Draft

## Type

UI/API wording hygiene

## Objective

Clean up stale current-runtime wording that still describes FastAPI, Python/Celery, Celery beat, legacy-api fallback, or other retired Python services as active live runtime services.

This DIFF exists because DIFF-178 identified visible runtime wording drift as the first recommended completion task and a product/troubleshooting risk.

This is a wording-only DIFF. It must not change route behavior, backend contracts, runtime architecture, Docker Compose, worker behavior, database schema, migrations, or local LLM behavior.

## Baseline Facts

- `main` is the clean runtime/product branch.
- `dev` is the development/build-agent branch.
- `dev` must not be merged directly into `main`.
- Active application runtime is Rust API gateway plus Rust worker daemon plus Next.js web UI.
- Legacy Python/FastAPI API is archived/inactive.
- Legacy Python/Celery worker is archived/inactive.
- Celery beat is inactive/retired from active Compose runtime.
- Historical, archive, migration, and rollback documentation may correctly mention FastAPI, Python/Celery, Celery beat, legacy-api, or fallback when clearly framed as historical/inactive/rollback context.
- DIFF-178 recommends DIFF-179 as: Runtime Wording Drift And Proxy Error Cleanup.

## Required Inputs To Inspect

Codex must inspect at minimum:

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/api/`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/`
- `configs/rust-cutover-manifest.json`

Codex may inspect additional files if needed to classify wording drift, but must keep the change scoped.

## Allowed Scope

Codex may:

- replace stale FastAPI wording in active Next.js API proxy error messages;
- replace visible active-runtime wording that incorrectly describes FastAPI, Python/Celery, Celery beat, legacy-api, or fallback as live runtime services;
- replace stale wording in docs only when the text describes current active runtime incorrectly;
- keep historical/archive/rollback/migration references intact when they clearly refer to inactive legacy behavior;
- update this DIFF Result and Verification Result sections;
- run scoped searches and classify matches as active-runtime drift, historical/archive/rollback reference, or irrelevant/no change.

## Prohibited Scope

Codex must not:

- change route behavior;
- change request/response contracts;
- change Rust gateway behavior;
- change Rust worker behavior;
- change Next.js functional behavior beyond text/error wording;
- change Docker Compose;
- change `.env` or `.env.example`;
- add dependencies;
- edit migrations;
- mutate runtime/private data;
- start, stop, or restart services;
- edit locked DIFFs;
- merge `dev` into `main`;
- promote dev-only instruction files to `main`.

## Search Requirements

Codex must run scoped searches for stale current-runtime wording, including at least:

- `FastAPI`
- `Python/Celery`
- `Celery`
- `beat`
- `legacy-api`
- `fallback`

Each meaningful match must be classified as one of:

- active-runtime drift to fix;
- historical/archive/rollback reference to keep;
- irrelevant/no change.

## Completion Criteria

This DIFF is complete when:

- active UI/API proxy error text no longer describes FastAPI, Python/Celery, Celery beat, or legacy-api as the live backend/runtime;
- historical/archive/rollback references remain intact where accurate;
- `docs/diffs/DIFF-179-runtime-wording-drift-proxy-error-cleanup.md` has Result and Verification Result updated;
- no route behavior or runtime behavior is changed;
- `npm --prefix apps/web run build` passes or the failure is documented with exact cause;
- `git diff --check` passes;
- scoped search confirms no active UI/proxy text still describes retired Python services as active runtime.

## Verification

Required verification:

- `git status --short`
- `git branch --show-current`
- `git diff --name-status`
- `git diff --check`
- `npm --prefix apps/web run build`
- scoped search proving no active UI/proxy text describes FastAPI, Python/Celery, Celery beat, or legacy-api as the live backend/runtime

Do not run live service start/stop/restart for this DIFF.

## Result

Pending.

## Verification Result

Pending.
