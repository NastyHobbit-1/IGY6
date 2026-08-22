# DIFF-294: Production-Readiness and Productization Pass

Status: Active

## Type

Change-bearing

## Objective

Take the `grok` branch from its current state to a coherent, installable, documented, tested, usable release-quality IGY6 product. Complete intended implementations, wire UI↔API↔runtime, finish installer/bootstrap and configuration profiles, modernize UI within existing Next.js + plain CSS, and make documentation match verified reality.

This DIFF authorizes a production-readiness pass, not a rewrite from scratch and not a merge to `main`.

## Baseline Facts

- Work branch is lowercase `grok` only. Do not merge or promote to `main` unless the owner explicitly instructs it.
- No active change-bearing DIFF existed before this file. Highest prior record is DIFF-293 (nightly RITR, no product code). DIFF-176 is Complete (DIFF-181).
- Active runtime is Rust gateway + Rust worker + Next.js. Legacy FastAPI/Celery are archived.
- UI is Chat / Data / Work / Settings / More, plain CSS only.
- One active DIFF at a time. Locked DIFFs must not be edited.

## Allowed Scope

- `crates/`, `apps/web/`, `infra/`, installer/bootstrap and `scripts/`
- Config templates/presets/wizard (no real secrets)
- Product docs, tests, this DIFF, optional inventory docs
- Terminology cleanup and redundancy removal that does not drop working capability

## Prohibited Scope

- Other branches; merges; promotion to `main`; locked DIFF edits
- Tailwind/shadcn; reactivating FastAPI/Celery; arbitrary shell; committing secrets
- Hiding incomplete features by deleting UI/APIs; capability cuts for naming; blind rewrites

## Required Tags

Reference `DIFF-294` in commits and PR text.

## Verification

`git status --short`; `git diff --check`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets`; `cargo test --workspace`; `npm --prefix apps/web run build`; existing UI smoke; post-cutover / fresh-clone / lifecycle scripts; installer on a clean path; e2e install→start→use. Record anything skipped.

## Completion Criteria

Intended features implemented and reachable; UI states complete; installer portable and idempotent; config profiles have real runtime effect; docs match verified reality; no promotion under this DIFF.

## Out Of Scope Follow-Up

Promotion to `main`; new connector platforms; replacing the CSS stack; reopening locked DIFFs.
