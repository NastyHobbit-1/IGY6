# DIFF-179: Runtime Wording Drift And Proxy Error Cleanup

Status: Complete

Reconciliation note: DIFF-181 changed this status from Draft to Complete
because this DIFF already contains completed Result and Verification Result
sections and git history includes completed DIFF-179 commits before DIFF-180.

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

Completed.

Cleaned active current-runtime wording drift without changing routes,
contracts, Docker Compose, runtime services, worker behavior, local LLM
behavior, migrations, dependencies, or runtime/private data.

Changes made:

- Replaced stale `FastAPI returned a non-JSON response` and
  `Failed to reach FastAPI ...` text in active Next.js API proxy error paths
  with `Rust API` wording.
- Replaced visible runtime posture labels in the web UI from retired service
  names to neutral legacy labels: `Legacy API`, `Legacy worker`, and
  `Legacy scheduler`.
- Replaced active settings wording exposed by the Rust gateway from
  `Redis / Celery` to `Redis`.
- Reworded retained `CELERY_*` setting descriptions as archived rollback
  settings rather than active service settings.
- Removed active `beat restart` wording from Redis/archived-Celery settings
  warnings.

Scoped search classification:

- Active-runtime drift fixed:
  - `apps/web/src/app/api/**`: stale `FastAPI returned...` and
    `Failed to reach FastAPI...` proxy error text.
  - `apps/web/src/app/page.tsx`: visible runtime posture labels that used
    retired service names.
  - `crates/igy6-gateway/src/lib.rs`: active Settings API group/description
    wording that exposed Celery/beat as if they were current runtime settings.
- Historical/archive/rollback references kept:
  - `README.md`, `docs/ui/README.md`, and
    `docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md` references
    that explicitly say legacy Python/FastAPI/Python-Celery/Celery beat are
    archived, inactive, retired, or unsupported as active runtime.
  - `configs/rust-cutover-manifest.json` migration, rollback, and route-parity
    history entries.
  - `crates/igy6-gateway/src/lib.rs` route-parity/test/status references that
    explicitly report fallback as removed, disabled, or absent.
- Irrelevant/no change:
  - generic `fallback` variable/function names for local data fallback and
    deterministic answer fallback.
  - `fastapi_fallback` status keys that are compatibility/status fields and
    report `false`, `removed`, or `none`.

## Verification Result

Passed.

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -5
git diff --name-status
git diff --check
npm --prefix apps/web run build
rg -n "FastAPI" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "Python/Celery" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "Celery" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "\\bbeat\\b|\\bBeat\\b" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "legacy-api" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "fallback" apps/web/src/app/api apps/web/src/app/page.tsx README.md docs/ui/README.md docs/plans/IGY6_PRODUCT_COMPLETION_ROADMAP_AND_GAP_AUDIT.md crates/igy6-gateway configs/rust-cutover-manifest.json
rg -n "FastAPI|Python/Celery|Celery|\\bbeat\\b|\\bBeat\\b|legacy-api|fallback" apps/web/src/app/api apps/web/src/app/page.tsx
rg -n "FastAPI returned|Failed to reach FastAPI|Python/Celery worker|Celery beat|FastAPI fallback|legacy-api" apps/web/src/app/api apps/web/src/app/page.tsx
```

Results:

- `git branch --show-current` returned `dev`.
- `git diff --check` passed.
- `npm --prefix apps/web run build` passed.
- Scoped active UI/proxy search found no active FastAPI, Python/Celery, Celery
  beat, or legacy-api wording describing retired Python services as live
  backend/runtime services.
- Remaining active `apps/web` search hits are generic `fallback` variable names
  used for local data fallback, not retired runtime-service claims.
- No live service start, stop, restart, Docker volume, database, Qdrant, Neo4j,
  `.env`, or runtime/private data operation was run.
