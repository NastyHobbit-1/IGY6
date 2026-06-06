# DIFF-218 - Local LLM Provider Status And Routing UX

Status: Complete

## Purpose

Make local AI status clear to normal users by showing whether local LLM/Ollama
routing is enabled, which provider/model is configured, whether deterministic
fallback is active, and what limitations apply.

This DIFF is product work, not smoke-tooling work.

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `45964f90c2e383cf934b752e66ec1ab2db7fccb2`
- DIFF-217 was committed and the working tree was clean before starting.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-126-local-llm-provider-plan.md`
- `docs/diffs/DIFF-127-local-llm-adapter.md`
- `docs/diffs/DIFF-128-evidence-grounded-llm-answer.md`
- `docs/diffs/DIFF-129-local-llm-ui-status.md`
- `docs/diffs/DIFF-130-ollama-routing.md`
- `docs/diffs/DIFF-212-persisted-evidence-answer-chat-session-records.md`
- `docs/diffs/DIFF-217-evidence-detail-page-panel.md`
- `README.md`
- `docs/ui/README.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/`
- `crates/igy6-llm/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`

## Current Local LLM Capability Found

- `crates/igy6-llm` supports provider values `none` and `ollama`.
- Ollama provider URLs are bounded to local HTTP hosts and credentials are
  rejected.
- Evidence-answer behavior already has deterministic fallback when provider
  config is disabled, invalid, unavailable, or insufficiently evidenced.
- `/settings/env` already exposes the relevant local LLM settings without
  dumping secrets:
  - `LLM_PROVIDER`
  - `OLLAMA_BASE_URL`
  - `OLLAMA_MODEL`
  - `LLM_TIMEOUT_SECONDS`
  - `LLM_EVIDENCE_REQUIRED`
- The web UI already had a basic Local LLM status panel, but it did not show all
  DIFF-218 requested normal-user status fields explicitly.

## Product Changes Made

- Expanded the normal Local LLM status panel in Assistant and Settings.
- The panel now shows:
  - enabled/disabled state;
  - configured provider;
  - configured model;
  - health/configuration status;
  - answer mode;
  - routing state;
  - deterministic fallback state;
  - evidence-required mode;
  - hosted/external AI default state.
- Added clear guidance for disabled, configured, and missing-model states.
- Added normal-user limitation notes:
  - local generation is evidence-grounded;
  - deterministic fallback remains available;
  - this UI does not install or pull models;
  - Settings still uses dry-run/save flow for `.env` edits;
  - no hosted AI is called by default;
  - the status panel performs no hidden source-data transfer.
- Updated raw diagnostics with the new derived status fields.
- Updated the UI guide.

## Backend/API Changes

No Rust backend, LLM crate, or Next.js proxy changes were required.

The existing `/settings/env` response already exposed enough non-secret local
LLM configuration for honest status display.

## Unsupported States Handled

- The panel does not contact Ollama for health checks.
- The panel does not install models or pull model files.
- The panel does not edit `.env`.
- The panel does not call hosted AI.
- The panel does not claim local LLM answers are available by default.
- The panel does not claim full reasoning, full memory, hidden memory, or
  autonomous reasoning.
- Missing model configuration is shown as unavailable until configured.

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Full Docker smoke was not run from Codex because the Codex local environment
  strips Docker group access and remaps `/var/run/docker.sock` to
  `nobody:nogroup`.
- Live Ollama health or generation was not run because this DIFF intentionally
  does not contact providers, install models, or modify local model state.
- Rust checks were not required because no Rust files changed in this DIFF.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-218-local-llm-provider-status-routing-ux.md`
- `docs/ui/README.md`

## Verification Summary

- The web build passed.
- Private/dev instruction files remained tracked on `dev`.
- Stale-status scan still reports pre-existing draft/template/status-command
  strings outside DIFF-218; DIFF-218 is `Status: Complete`.

## Scope Confirmation

- No smoke-tooling-only work was performed.
- No hosted AI call was added.
- No local model install or pull behavior was added.
- No `.env` edit was performed.
- No external service call was added.
- No hidden data transfer was added.
- No full reasoning or memory claim was added.
- No browser/account scraping or connector import was added.
- No arbitrary command execution was added.
- No runtime/private data was dumped.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
