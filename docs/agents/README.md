# Agent Notes

This directory is not used by the IGY6 runtime on `main`.

Private coordination material belongs on the local `dev` branch, not on the product-facing `main` branch.

## Required Baseline For Future Codex Prompts

Use `docs/agents/CODEX_PROMPT_BASELINE.md` as the baseline for every future Codex prompt.

That baseline contains the required project-building instructions for:

- branch policy;
- DIFF governance;
- runtime/product posture;
- Rust/Next.js architecture;
- runtime data and secret boundaries;
- agent/action safety;
- UI rules;
- coding rules;
- verification rules;
- clean promotion back to `main`.

Runtime chat and action behavior lives in the application code and configuration:

- `crates/igy6-agent-api/`
- `crates/igy6-gateway/`
- `crates/igy6-llm/`
- `crates/igy6-evidence-answer/`
- `configs/local-llm-routing.json`
- `apps/web/src/app/api/agent/`
- `apps/web/src/app/api/chat/`
