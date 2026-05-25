# Agent Notes

This directory is not used by the IGY6 runtime on `main`.

Private coordination material belongs on the local `dev` branch, not on the product-facing `main` branch.

Runtime chat and action behavior lives in the application code and configuration:

- `crates/igy6-agent-api/`
- `crates/igy6-gateway/`
- `crates/igy6-llm/`
- `crates/igy6-evidence-answer/`
- `configs/local-llm-routing.json`
- `apps/web/src/app/api/agent/`
- `apps/web/src/app/api/chat/`
