# DIFF-129: Local LLM UI Status

Status: Locked

## Type

Change-bearing

## Objective

Add user-facing UI controls and documentation for local LLM provider status,
model selection/config display, answer mode, and deterministic fallback.

## Baseline Facts

- DIFF-126 documented the local LLM provider plan.
- DIFF-127 added the disabled-by-default local Ollama adapter crate.
- DIFF-128 wired optional evidence-grounded local generation into evidence
  answer logic with deterministic fallback.
- FastAPI fallback remains required for classified legacy/non-web routes.

## Allowed Scope

- Show local LLM provider status in Settings.
- Show Assistant answer mode: deterministic evidence, local LLM
  evidence-grounded, or unavailable.
- Expose LLM env keys through managed settings without secrets.
- Add Advanced raw provider diagnostics.
- Update README, user guide, LLM plan docs, manifest, and UI smoke coverage.

## Prohibited Scope

- No external model provider.
- No secret/token fields.
- No UI claim that external calls are happening.
- No action execution through LLM.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim.
- No locked DIFF edits.

## Verification

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `npm --prefix apps/web test`
- Rust checks if backend files changed.

## Completion Notes

- Added managed local LLM settings for `LLM_PROVIDER`, `OLLAMA_BASE_URL`,
  `OLLAMA_MODEL`, `LLM_TIMEOUT_SECONDS`, and `LLM_EVIDENCE_REQUIRED` without
  introducing secret/token fields.
- Added Assistant and Settings local LLM status panels showing provider, health
  status, answer mode, evidence-required status, normal-user and coder examples,
  and Advanced raw provider diagnostics.
- Updated Assistant copy to distinguish deterministic evidence, local LLM
  evidence-grounded, and unavailable states.
- Updated README, user guide, LLM plan docs, cutover manifest, and UI smoke
  coverage.
- No external model provider, real Ollama call, LLM action execution, FastAPI
  removal, route removal, or Rust-only claim was added.

## Verification Results

- `git status --short`: checked DIFF-129 scoped files before locking.
- `git diff --check`: passed.
- `npm --prefix apps/web run build`: passed.
- `npm --prefix apps/web run test:ui-smoke`: passed.
- `npm --prefix apps/web test`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test -p igy6-gateway`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed
  (`fastapi=91 rust_native=64 web_used=45 missing_from_rust=30 web_requires_fallback=0`).
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `scripts/rust-cutover.sh --check`: passed.
