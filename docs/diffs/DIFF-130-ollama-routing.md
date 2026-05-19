# DIFF-130: Ollama Setup And Task Routing

Status: Locked

## Type

Change-bearing

## Objective

Add safe local Ollama setup support and task-based local model routing for IGY6
without making Ollama mandatory, pulling large models by default, adding cloud
providers, or removing deterministic evidence fallback.

## Baseline Facts

- DIFF-126 through DIFF-129 added the local LLM plan, local Ollama adapter,
  evidence-grounded fallback path, and Assistant/Settings status UI.
- `LLM_PROVIDER=none` remains the default.
- FastAPI fallback remains required for classified legacy/non-web routes.
- Rust-only operation cannot be claimed.

## Allowed Scope

- Add `configs/local-llm-routing.json`.
- Add routing selection/validation structures and tests to `crates/igy6-llm`.
- Add `scripts/ollama-local-setup.sh`.
- Update README, user guide, local LLM docs, and relevant runtime smoke docs.
- Update manifest honestly if useful.

## Prohibited Scope

- No external model calls.
- No cloud providers.
- No API keys or secrets.
- No `.env` commit.
- No automatic install during tests.
- No automatic model pull during tests.
- No large default model pulls.
- No destructive commands.
- No Ollama model deletion.
- No LLM action execution.
- No approval bypass.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No locked DIFF edits.

## Verification

- `git status --short`
- `git diff --check`
- `bash -n scripts/ollama-local-setup.sh`
- `scripts/ollama-local-setup.sh --help`
- `scripts/ollama-local-setup.sh --check`
- `scripts/ollama-local-setup.sh --list-recommended`
- `python3 -m json.tool configs/local-llm-routing.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `npm --prefix apps/web run build` if docs/UI touched in a way that affects web
- `npm --prefix apps/web run test:ui-smoke` if UI copy changed
- `npm --prefix apps/web test` if package scripts exist
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Notes

- Added `configs/local-llm-routing.json` for task-based Ollama routing on a
  practical RTX 3060 12GB local target.
- Added `scripts/ollama-local-setup.sh` with safe check-only defaults, explicit
  install/pull/env-write flags, and no model deletion behavior.
- Added Rust validation and tests for the local routing config in
  `crates/igy6-llm`.
- Updated `.env.example`, README, user guide, local LLM planning docs, runtime
  processing docs, and the cutover manifest without claiming Rust-only status.
- Default pull set is limited to `qwen2.5-coder:7b`, `llama3.1:8b`, and
  `gemma3:4b`; `gemma3:12b` is documented as optional; larger models remain
  explicitly non-default.
- `scripts/ollama-local-setup.sh --check` was run in check-only mode. It
  validated the routing config and detected that Ollama was not installed and
  that the local API was not reachable; no install, pull, or `.env` mutation was
  performed during verification.

## Verification Results

- `git diff --check`: passed.
- `bash -n scripts/ollama-local-setup.sh`: passed.
- `scripts/ollama-local-setup.sh --help`: passed.
- `scripts/ollama-local-setup.sh --check`: passed with optional Ollama reported
  unavailable on this machine.
- `scripts/ollama-local-setup.sh --list-recommended`: passed.
- `python3 -m json.tool configs/local-llm-routing.json`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `cargo fmt --all --check`: passed after applying formatter.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test -p igy6-llm`: passed.
- `cargo test --workspace`: passed.
- `npm --prefix apps/web run build`: passed.
- `npm --prefix apps/web test`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed with
  `missing_from_rust=30` and `web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`:
  passed.
