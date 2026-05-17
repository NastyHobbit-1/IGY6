# DIFF-097: Rust Agent API

Status: Locked

## Type

Change-bearing Rust agent command-plane foundation

## Objective

Add Rust typed agent command-plane classification and capability metadata beside
the existing Python/FastAPI agent API, without executing actions or replacing
Python behavior.

## Baseline Facts

- DIFF-096 completed the Rust `read_only_api` phase.
- The manifest shows `agent_api` as the next pending Rust phase.
- `services/api/app/agent_actions.py` remains the active route, approval,
  audit, runtime capability, and execution implementation.
- This DIFF adds Rust parity for the typed action registry and intent
  classification only.

## Allowed Scope

- `docs/diffs/DIFF-097-rust-agent-api.md`
- Root `Cargo.toml` workspace membership for `crates/igy6-agent-api`
- `Cargo.lock` workspace package metadata
- `crates/igy6-agent-api/`
- `configs/rust-cutover-manifest.json` `agent_api` phase only
- `docs/rust-migration/RUST_MIGRATION_PLAN.md` only if needed for accuracy
- `snippet-vault/rust-equivalents/by-source-language/python/snippets.jsonl`
- `snippet-vault/rust-equivalents/index.jsonl`

## Prohibited Scope

- No locked DIFF edits.
- No Python/FastAPI replacement.
- No action execution.
- No subprocess execution.
- No approval or audit rewiring.
- No API gateway changes.
- No Docker Compose rewrite.
- No `.env` changes.
- No database migrations.
- No runtime/private data reads.
- No archive actions.
- No file deletion.
- No broad refactor.
- No unrelated formatting churn.
- No marking future phases complete.

## Required Tags

- Commit message must include `DIFF-097`.
- Final response must identify `DIFF-097`.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-agent-api`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- Validate changed snippet-vault JSONL files line-by-line as valid JSON.

## Completion Criteria

- Rust agent action registry covers the current Python action names.
- Rust intent classification maps known read-only and system-changing messages
  to the same typed action names.
- System-changing actions require approval and are not executable immediately.
- Required parameters are detected for retrieval preview.
- Dangerous arbitrary/destructive command patterns are rejected as unknown.
- Tests cover read-only classification, approval-required classification,
  missing parameters, dangerous pattern rejection, and registry coverage.
- Manifest `agent_api` phase is marked complete only after verification.
- `cutover_ready` remains false.

## Verification Result

- `git status --short` checked DIFF-097 scoped files plus generated `target/`
  build artifacts, which were removed before commit.
- `git diff --check` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets` passed.
- `cargo test --workspace` passed.
- `cargo test -p igy6-agent-api` passed.
- `scripts/rust-cutover.sh --check` passed with the expected warning that
  `cutover_ready` is false.
- `python3 -m json.tool configs/rust-cutover-manifest.json` passed.
- Snippet-vault JSONL parse validation passed.

## Out Of Scope Follow-Up

- Action execution, script wrappers, approvals, audit writes, database reads,
  retrieval execution, API route replacement, or gateway cutover.
