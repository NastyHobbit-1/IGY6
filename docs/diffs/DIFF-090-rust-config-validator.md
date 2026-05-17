# DIFF-090: Rust Config Validator

Status: Locked

## Type

Change-bearing Rust config validator

## Objective

Add the Rust `config` migration phase by implementing deterministic,
repo-visible configuration validation in `crates/igy6-config` and exposing it
through `igy6 config check` without replacing Python settings/env validation.

## Baseline Facts

- DIFF-085 created the Rust migration cutover plan.
- DIFF-086 completed the Rust host control bridge.
- DIFF-087 completed the Rust workspace foundation.
- DIFF-088 added the initial Rust CLI foundation.
- DIFF-089 corrected DIFF-088 CLI contract drift and completed the required
  `igy6` CLI command surface.
- The next manifest phase is `config`.
- Python settings/env validation remains active until a later parity DIFF
  explicitly replaces it.
- `.env` values must never be printed.
- Runtime/private `IGY6_DATA_ROOT` contents must not be read.

## Allowed Scope

- `docs/diffs/DIFF-090-rust-config-validator.md`
- `crates/igy6-config/`
- `crates/igy6-cli/` only to expose the validator through `igy6 config check`
- `configs/rust-cutover-manifest.json` `config` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/by-source-language/bash/snippets.jsonl`
- `snippet-vault/rust-equivalents/by-source-language/other/snippets.jsonl` only
  if source language is unclear
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No backend rewrite.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No `.env` commits.
- No database migrations.
- No runtime/private data reads from `IGY6_DATA_ROOT` contents.
- No file deletion.
- No archive execution.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.
- No treating the DIFF-120 planning horizon as permission to do multiple DIFFs
  in this run.

## Required Tags

- Commit message must include `DIFF-090`.
- Final response must identify `DIFF-090`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo run -p igy6-cli --bin igy6 -- config check`
- `cargo run -p igy6-cli --bin igy6 -- health`
- `cargo run -p igy6-cli --bin igy6 -- --help`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- If snippet-vault JSONL files are created or updated, validate that each added
  JSONL line parses as valid JSON.

## Completion Criteria

- `crates/igy6-config` parses env-style config lines safely.
- Blank lines and comments are ignored.
- Malformed non-comment lines without `=` are rejected.
- Empty keys are rejected.
- Duplicate keys are detected.
- Required keys are validated.
- Boolean-like fields, port fields, URL/URI-like fields, policy fields, log
  levels, and local path strings are validated without network calls.
- `.env.example` is validated.
- `.env`, when present, is read only for key presence and safe structural
  validation, with no values printed.
- `igy6 config check` uses the Rust config validator from `crates/igy6-config`.
- `cutover_ready` remains false.
- Only the `config` phase is marked complete in the manifest.

## Verification Result

- `git status --short` checked DIFF-090 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed after formatting.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo run -p igy6-cli --bin igy6 -- config check` passed.
- `cargo run -p igy6-cli --bin igy6 -- health` passed.
- `cargo run -p igy6-cli --bin igy6 -- --help` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Replacing Python settings/env validation.
- Writing `.env` or runtime/private data.
- Reading runtime/private `IGY6_DATA_ROOT` contents.
- Docker Compose rewrites or API gateway changes.
- Future Rust migration phases after DIFF-090.

## Forward Plan Through DIFF-120

- DIFF-091 artifact_store.
- DIFF-092 normalization.
- DIFF-093 chunking.
- DIFF-094 vector_memory.
- DIFF-095 worker.
- DIFF-096 read_only_api.
- DIFF-097 agent_api.
- DIFF-098 retrieval_preview.
- DIFF-099 evidence_answer.
- DIFF-100 write_api_batch_1.
- DIFF-101 work_queue_reports.
- DIFF-102 rust_gateway.
- DIFF-103 final Rust cutover readiness audit.
- DIFF-104 final Rust cutover execution only if the manifest is ready.
- DIFF-105 post-cutover docs and operations audit.
- DIFF-106 source/evidence parity audit.
- DIFF-107 vector/graph parity audit.
- DIFF-108 prediction/outcome parity audit.
- DIFF-109 self-improvement parity audit.
- DIFF-110 safety/approval/audit hardening.
- DIFF-111 backup/export/restore hardening.
- DIFF-112 UI integration parity review.
- DIFF-113 performance and local deployment hardening.
- DIFF-114 report/export completion check.
- DIFF-115 acceptance criteria audit against `AGENTS.md`.
- DIFF-116 drift cleanup only if documented and approved.
- DIFF-117 long-term operations polish.
- DIFF-118 final project-plan gap closure.
- DIFF-119 release candidate verification.
- DIFF-120 final completion audit, only if still needed.

If Rust migration completes before DIFF-120, future DIFFs shift to acceptance
hardening, parity audits, safety audits, export/backup hardening, and final
project-plan completion checks. Future work must preserve IGY6 as a
local-first adaptive intelligence system with source permissions, evidence
ledger, artifact store, vector and graph memory, pattern detection,
predictions/recommendations, outcomes, feedback, self-improvement, approval
gates, auditability, and reports/exports.
