# DIFF-126: Local LLM Provider Integration Plan

Status: Locked

## Type

Documentation and configuration planning

## Objective

Plan optional local LLM integration before writing model-calling code.

## Baseline Facts

- IGY6 is local-first and evidence-only by default.
- No external model calls are made by default.
- Deterministic evidence answers remain the fallback behavior.
- Rust-only operation cannot be claimed while the manifest still requires
  FastAPI fallback.
- Later implementation DIFFs must preserve evidence-required behavior: retrieved
  evidence first, citations/source trails where possible, and insufficient
  evidence when no evidence supports an answer.

## Allowed Scope

- Add a local LLM provider plan under `docs/llm/`.
- Add safe local LLM configuration defaults to `.env.example`.
- Update user-facing docs with optional Ollama/local LLM posture.
- Update manifest/docs honestly if needed.

## Prohibited Scope

- No model-calling code.
- No Ollama runtime calls.
- No external model calls.
- No Assistant answer-generation wiring.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No secrets or private runtime data commits.
- No locked DIFF edits.

## Verification

- `git status --short`
- `git diff --check`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json` if changed
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`

## Completion Notes

- Added `docs/llm/LOCAL_LLM_PROVIDER_PLAN.md` with the optional local provider
  contract, starting with Ollama, and evidence-grounded answer rules.
- Added disabled-by-default safe local LLM keys to `.env.example`:
  `LLM_PROVIDER=none`, `OLLAMA_BASE_URL`, `OLLAMA_MODEL`,
  `LLM_TIMEOUT_SECONDS`, and `LLM_EVIDENCE_REQUIRED=true`.
- Updated `README.md` and `docs/user-guide.md` to explain optional local-first
  LLM posture, deterministic evidence fallback, insufficient-evidence behavior,
  and citation requirements.
- Updated `configs/rust-cutover-manifest.json` with an honest partial planning
  milestone that explicitly does not claim provider code, model calls, Assistant
  wiring, external model calls, or Rust-only operation.
- No backend behavior changed. No model calls were added or executed.

## Verification Results

- `git status --short`: checked DIFF-126 scoped files only before locking.
- `git diff --check`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed
  (`fastapi=91 rust_native=64 web_used=45 missing_from_rust=30 web_requires_fallback=0`).
- `scripts/rust-cutover.sh --check`: passed.
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`: passed.
- `npm --prefix apps/web run build`: passed.
