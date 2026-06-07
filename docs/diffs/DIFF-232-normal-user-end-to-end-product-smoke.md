# DIFF-232 - Normal-User End-To-End Product Smoke

Status: Complete

## Scope

DIFF-232 defines a normal-user product-path smoke for Add Data -> Work ->
Results -> Answer -> Report -> Feedback -> Outcome -> Detail Review. It is
product verification, not generic smoke-tooling expansion.

## Current Support Found

- The UI already exposes normal-user guided manual text upload.
- Work status records are visible in the Work tab.
- Results includes Ask Over Evidence, answer save, report workflow,
  feedback/outcome workflow, and source/evidence history surfaces.
- Existing operator smoke can run live synthetic checks in owner WSL, but Codex
  must not run full Docker smoke.

## Product Behavior Added

- Added `docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md`.
- Added `scripts/normal-user-product-smoke.sh`.
- The helper supports:
  - `--check`: Codex-safe source marker check only;
  - `--owner-commands`: owner-run WSL command guidance.
- The checklist defines the product-level path:
  - guided supported source creation;
  - manual synthetic UTF-8 text add path;
  - work item processing;
  - retrieval/evidence-grounded answer surface;
  - persisted answer record;
  - report workflow;
  - feedback capture;
  - outcome capture for API-supported targets;
  - source/evidence detail review.

## Safety Behavior

- Synthetic data only.
- No Docker command is run by the Codex-safe check.
- No `.env` read is required.
- No runtime/private data is dumped.
- Unsupported outcome targets remain unsupported.
- The DIFF does not claim the live product path is verified until owner WSL
  smoke and the manual checklist pass.

## Explicit Non-Claims

- This DIFF does not add new runtime behavior.
- This DIFF does not claim full live-stack product readiness.
- This DIFF does not claim binary parsing, PDF export, browser/account import,
  connector import, or complete backup/restore/delete behavior.

## Files Changed

- `scripts/normal-user-product-smoke.sh`
- `docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md`
- `docs/diffs/DIFF-232-normal-user-end-to-end-product-smoke.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `bash -n scripts/normal-user-product-smoke.sh`
- `scripts/normal-user-product-smoke.sh --check`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

Product smoke checklist plus Codex-safe source marker helper. No new API route,
schema, persistence, worker behavior, or UI behavior.

## Scope Confirmation

- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No arbitrary command execution from user text was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete or destructive restore was performed.
- No unsafe backup archive was created.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
