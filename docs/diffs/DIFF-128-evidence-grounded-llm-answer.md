# DIFF-128: Evidence-Grounded LLM Answer Generation

Status: Locked

## Type

Change-bearing

## Objective

Wire the local LLM provider into evidence answer generation while preserving
evidence-required behavior and deterministic fallback.

## Baseline Facts

- DIFF-127 added the disabled-by-default `igy6-llm` crate and local-Ollama-only
  adapter.
- `/chat/evidence-answer` currently builds deterministic evidence answer
  packets.
- Retrieval hydration remains limited by current Rust gateway behavior; no live
  Qdrant/PostgreSQL retrieval expansion is added in this DIFF.

## Allowed Scope

- Add optional LLM generation logic behind evidence-required checks.
- Preserve deterministic fallback when the provider is disabled, unavailable,
  invalid, timed out, or missing evidence.
- Add tests using fake transports only.
- Update Assistant UI copy and docs to describe evidence-grounded answer,
  deterministic fallback, LLM unavailable, and insufficient evidence.

## Prohibited Scope

- No external model providers.
- No external model calls.
- No arbitrary action execution.
- No LLM-driven tool/action execution.
- No broad retrieval architecture rewrite.
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
- `cargo test -p igy6-gateway`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`

## Completion Notes

- Added optional local LLM answer generation in `igy6-evidence-answer` behind
  evidence-required checks.
- Preserved deterministic fallback for disabled provider, invalid config,
  unavailable/transport errors, and provider failures.
- Preserved insufficient-evidence behavior without calling the provider when no
  facts are available.
- Bounded prompt construction to the configured evidence budget and included
  citation ids/source trails in the prompt packet.
- Wired `/chat/evidence-answer` through the optional generation path while
  keeping current no-evidence gateway behavior deterministic and local.
- Updated Assistant UI copy, README, user guide, LLM plan docs, and the cutover
  manifest.
- No external providers, external model calls, action execution, route removal,
  FastAPI removal, or Rust-only claim was added.

## Verification Results

- `git status --short`: checked DIFF-128 scoped files before locking.
- `git diff --check`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test -p igy6-evidence-answer`: passed.
- `cargo test -p igy6-gateway`: passed.
- `cargo clippy --workspace --all-targets`: passed.
- `cargo test --workspace`: passed.
- `npm --prefix apps/web run build`: passed.
- `npm --prefix apps/web run test:ui-smoke`: passed.
- `python3 scripts/rust-route-parity.py --check`: passed
  (`fastapi=91 rust_native=64 web_used=45 missing_from_rust=30 web_requires_fallback=0`).
- `python3 -m json.tool configs/rust-cutover-manifest.json`: passed.
- `scripts/rust-cutover.sh --check`: passed.
