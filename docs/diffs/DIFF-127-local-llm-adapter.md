# DIFF-127: Local LLM Adapter

Status: Locked

## Type

Change-bearing

## Objective

Add a local LLM provider abstraction and Ollama adapter without wiring it into
normal Assistant answer generation.

## Baseline Facts

- LLM provider configuration is disabled by default with `LLM_PROVIDER=none`.
- DIFF-126 documented the provider plan and safe local defaults.
- IGY6 remains local-first and evidence-first.
- FastAPI fallback remains required for classified legacy/non-web routes.

## Allowed Scope

- Add a Rust local LLM provider crate.
- Implement disabled and local Ollama provider handling.
- Implement safe local health and generate request planning/execution helpers.
- Use mock/fake transports in tests.
- Update docs and manifest with honest adapter status.

## Prohibited Scope

- No external providers.
- No external API calls by default.
- No real Ollama dependency in tests.
- No Assistant answer-generation wiring.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No full prompt logging.
- No secrets or private runtime data commits.
- No locked DIFF edits.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build` if web changed

## Completion Notes

- Added the Rust `igy6-llm` crate to the workspace.
- Implemented disabled-provider config as the default, local-Ollama-only config
  validation, safe local HTTP health/generate helpers, timeout propagation,
  structured generate responses, explicit error types, evidence-required and
  evidence-budget checks, local URL validation, and sensitive-looking output
  preview redaction.
- Added fake-transport tests; no real Ollama process or external provider is
  required for tests.
- Updated README, user guide, the LLM provider plan, and the cutover manifest to
  state that DIFF-127 adds the adapter crate but does not wire Assistant answer
  generation or make Rust-only claims.

## Verification Results

- `git status --short`: checked DIFF-127 scoped files before locking.
- `git diff --check`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test -p igy6-llm`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed
  (`fastapi=91 rust_native=64 web_used=45 missing_from_rust=30 web_requires_fallback=0`).
- `scripts/rust-cutover.sh --check`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`: passed.
- `npm --prefix apps/web run build`: passed.
