# DIFF-131 Local LLM Task Routing

Use this instruction card after reading `AGENTS.md`, `docs/agents/AGENT_PROMPT.md`, and `docs/agents/AGENT_PROMPT_CODING.md`.

## Objective

Create DIFF-131 to connect the existing local LLM routing configuration to the Rust Assistant and evidence-answer generation path.

## Baseline

DIFF-130 added `configs/local-llm-routing.json` and Rust validation for task routes. `LLM_PROVIDER=none` remains the default. Ollama remains optional. FastAPI fallback remains required. Do not claim Rust-only operation.

## Required Work

Load and validate `configs/local-llm-routing.json` from the Rust local generation path. Select a route for `code_repo`, `evidence_summary`, `fast_triage`, `report_draft`, `action_explanation`, or `chat_default`. Unknown tasks must use `chat_default`. The selected route should provide model name, system instruction, temperature, and evidence-required behavior.

Fallback must remain deterministic when local generation is disabled, unavailable, invalid, missing evidence, or timed out. Tests must use fake transports only.

## Likely Files

- `crates/igy6-llm/`
- `crates/igy6-evidence-answer/`
- `crates/igy6-gateway/`
- `configs/local-llm-routing.json`
- `configs/rust-cutover-manifest.json`
- `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md`

## Boundaries

Do not install Ollama, pull models, add providers, add credentials, edit `.env`, remove FastAPI fallback, make a Rust-only claim, edit locked DIFFs, or do unrelated cleanup.

## Verification

Run the active DIFF verification, including JSON validation for both config files, Rust formatting, Rust clippy, workspace tests, `cargo test -p igy6-llm`, route parity check, cutover check, and Docker Compose config validation. Run web build only if UI/status text changes.

## Final Report

Report the active DIFF, changed files, verification results, skipped checks, FastAPI fallback status, and Rust-only claim status.
