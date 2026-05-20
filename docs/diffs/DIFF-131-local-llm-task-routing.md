# DIFF-131: Local LLM Task Routing

Status: Locked

## Type

Change-bearing

## Objective

Connect the existing local LLM routing configuration to the Rust Assistant and
evidence-answer generation path while preserving optional Ollama behavior,
deterministic fallback, and FastAPI fallback.

## Baseline Facts

- DIFF-130 added `configs/local-llm-routing.json` and Rust validation for local
  LLM task routes.
- `LLM_PROVIDER=none` remains the default.
- Ollama remains optional and local-only.
- FastAPI fallback remains required.
- Rust-only operation must not be claimed.

## Allowed Scope

- `crates/igy6-llm/`
- `crates/igy6-evidence-answer/`
- `crates/igy6-gateway/`
- `configs/local-llm-routing.json`
- `configs/rust-cutover-manifest.json`
- `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`

## Prohibited Scope

- No Ollama install.
- No model pulls.
- No external model calls.
- No added providers.
- No added credentials or secrets.
- No `.env` edits.
- No FastAPI fallback removal.
- No Rust-only claim.
- No locked DIFF edits.
- No unrelated cleanup, renames, rewiring, redesign, dependency changes, data
  model changes, or migration changes.

## Required Tags

Use `DIFF-131` in change summaries and review notes. Inline comments are only
allowed when useful for a non-obvious DIFF-specific behavior.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/local-llm-routing.json`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-llm`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Run web build only if UI/status text changes.

## Completion Criteria

- Rust local generation loads and validates `configs/local-llm-routing.json`
  when local generation is enabled.
- Supported task names select route-specific model, system instruction,
  temperature, and evidence-required behavior.
- Unknown tasks fall back to `chat_default`.
- Disabled, unavailable, invalid, missing-evidence, or timed-out local
  generation keeps deterministic fallback.
- Tests use fake transports only.
- FastAPI fallback remains required and documented.

## Out Of Scope Follow-Up

- Installing Ollama or pulling models.
- Adding non-Ollama providers.
- Runtime UI controls for selecting task routes.
- Removing legacy FastAPI fallback.

## Completion Notes

- Added Rust loading for `configs/local-llm-routing.json` through the local LLM
  crate.
- Added route selection that maps known task names to route-specific model,
  system instruction, temperature, and evidence-required behavior.
- Unknown task names fall back to `chat_default`.
- Rust evidence-answer generation now applies selected local LLM routes when
  `LLM_PROVIDER=ollama`; disabled local generation still uses deterministic
  fallback.
- Missing evidence still avoids provider calls and returns insufficient
  evidence.
- Tests use fake transports only.
- FastAPI fallback remains required; no Rust-only claim was added.

## Verification Results

- `git diff --check`: passed.
- `python3 -m json.tool configs/local-llm-routing.json`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `cargo test -p igy6-llm`: passed.
- `cargo test -p igy6-evidence-answer`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed with
  `missing_from_rust=30` and `web_requires_fallback=0`.
- `scripts/rust-cutover.sh --check`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`:
  passed.
- Web build was not run because DIFF-131 did not change UI/status text.
