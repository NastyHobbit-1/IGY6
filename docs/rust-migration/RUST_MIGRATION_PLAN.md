# IGY6 Rust Migration Plan

## Purpose

This document defines the controlled Rust migration track for IGY6. The goal is
to move IGY6 toward a Rust-primary architecture one verified component at a
time while preserving the current DIFF-governed workflow and keeping the
repository runnable after every step.

This migration track does not replace `AGENTS.md`, the existing build plan, the
current README, or the DIFF process. It is an implementation migration plan.

## Why Not Rewrite Everything At Once

IGY6 already has working Python/FastAPI, Celery worker, Next.js, PostgreSQL,
Qdrant, Neo4j, MLflow, Phoenix, source, approval, work-item, retrieval, and
evidence workflows. Rewriting all of that in one step would make behavior hard
to verify, weaken auditability, and risk breaking local-first safety controls.

Every Rust migration DIFF must therefore:

- Add Rust beside the existing system.
- Preserve current Python/FastAPI/worker behavior until Rust parity exists.
- Update `configs/rust-cutover-manifest.json`.
- Run targeted verification for the phase it changes.
- Leave `cutover_ready` false until all required phases are complete.

## Current Architecture

During migration, the active system remains:

```text
Next.js web UI
        |
        v
FastAPI API
        |
        +--> Celery workers
        +--> PostgreSQL state/audit store
        +--> Qdrant vector memory
        +--> Neo4j graph memory
        +--> local artifact/export/env backup storage
        +--> Rust tools/sidecars added gradually
```

The current web UI can remain Next.js while backend components move to Rust.
Rust migration does not require a UI rewrite.

## Target Rust Architecture

The intended final architecture is Rust-primary:

```text
Next.js web UI or future local shell
        |
        v
Rust Axum API gateway
        |
        +--> Rust worker runtime
        +--> Rust host bridge
        +--> Rust artifact store
        +--> Rust evidence and retrieval engine
        +--> PostgreSQL
        +--> Qdrant
        +--> Neo4j
```

The Rust API must preserve local-first, permissioned, auditable behavior. It
must not introduce external model calls, arbitrary shell execution, Docker
socket access in the API container, browser/router/account automation, or
system-changing actions without approval.

## DIFF-By-DIFF Migration Sequence

The planned sequence is:

| Phase | Planned DIFF | Purpose |
| --- | --- | --- |
| `host_bridge` | DIFF-086 or next available | Rust local-only host bridge for approved stack-control scripts. |
| `workspace` | Next | Add Cargo workspace, `igy6-core`, `igy6-config`, and `igy6-policy`. |
| `cli` | DIFF-088, corrected by DIFF-089 | Add `igy6` CLI for health, run, stop, run-last-healthy, config check, and snapshot show. |
| `config` | DIFF-090 | Add Rust `.env` validation without replacing Python settings until parity. |
| `artifact_store` | DIFF-091 | Add Rust content-addressed artifact storage bounded by `IGY6_DATA_ROOT`. |
| `normalization` | DIFF-092 | Add UTF-8 text normalization in Rust. |
| `chunking` | DIFF-093 | Add deterministic chunking and evidence-item generation in Rust. |
| `vector_memory` | DIFF-094 | Add Qdrant request planning, upsert/search payloads, and deterministic vectors in Rust. |
| `worker` | DIFF-095 | Add Rust worker MVP for normalization, chunking, and vector-upsert planning. |
| `read_only_api` | DIFF-096 | Add Rust read-only API sidecar foundation while FastAPI remains primary. |
| `agent_api` | DIFF-097 | Add Rust typed agent command-plane classification while Python execution remains primary. |
| `retrieval_preview` | DIFF-098 | Add Rust retrieval-preview planning while preserving `answer_status: not_generated`. |
| `evidence_answer` | DIFF-099 | Add Rust deterministic evidence-answer packet construction. |
| `write_api_batch_1` | DIFF-100 | Move sources, approvals, audit, feedback, and outcomes. |
| `work_queue_reports` | DIFF-101 | Move work items, dispatch, reports, and report rendering. |
| `rust_gateway` | Next | Make Rust Axum the main API gateway; FastAPI becomes fallback or disabled. |
| final cutover | Final DIFF | Run the cutover script and archive deprecated legacy files. |

Because DIFF-085 is the migration-control DIFF, the next implementation DIFF
should normally be the Rust Host Control Bridge unless a later repository state
requires a different next unused DIFF number.

## Active During Migration

These remain active throughout migration unless a later DIFF explicitly replaces
them after verified Rust parity:

- `AGENTS.md`
- `docs/diffs/`
- `docs/agents/`
- `README.md`
- `infra/docker-compose.yml`
- `services/api/`
- `services/worker/`
- `apps/web/`
- `scripts/run.sh`
- `scripts/stop.sh`
- `scripts/run-last-healthy-config.sh`

## Deprecated Only After Rust Parity

Python/FastAPI modules, Celery worker tasks, legacy scripts, old README text,
and old operational docs are deprecated only after equivalent Rust behavior
exists, verification has passed, and the active DIFF updates the manifest.

Do not archive or delete working Python services just because a Rust phase has
started.

## How Rust Crates Will Be Added

Future DIFFs should add crates under `crates/` and update the root
`Cargo.toml` only when the workspace phase is active. Crates should be small and
phase-scoped. Shared types belong in `igy6-core`; configuration logic belongs
in `igy6-config`; safety and permission helpers belong in `igy6-policy`.

Each crate-owning DIFF must add targeted tests and record verification in both
its DIFF file and `configs/rust-cutover-manifest.json`.

## How Python Services Are Gradually Replaced

Python remains primary until a Rust replacement is built and verified. A Rust
sidecar can be introduced beside FastAPI first. After route parity is proven,
the manifest can mark the phase complete. Only the final cutover DIFF may
archive deprecated Python services.

## Web UI During Migration

The Next.js web UI may remain active. If Rust API routes preserve current
request and response shapes, the UI should need minimal changes. If route
versions change, the active DIFF must document and verify the change.

The frontend must continue to call the backend API only. It must not call
PostgreSQL, Qdrant, Neo4j, Redis, local files, Docker, or host bridge endpoints
directly.

## Manifest Requirement For Every Rust DIFF

Every Rust migration DIFF must update `configs/rust-cutover-manifest.json` with:

- Phase status: `pending`, `partial`, or `complete`.
- The DIFF that changed the phase.
- Verification commands that were run.
- Archive/rewrite/create entries only when appropriate.

Do not mark a phase complete unless code exists and verification was run.
