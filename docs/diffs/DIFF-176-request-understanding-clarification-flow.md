# DIFF-176: Request Understanding Clarification Flow

Status: Active

## Type

Change-bearing

## Objective

Add the first request-understanding layer so IGY6 can classify plain-language
user requests before answering, creating work, or suggesting actions.

## Baseline Facts

- Rust-only application API and worker runtime are already cut over.
- `main` is the clean product/runtime branch.
- `dev` preserves build plans and agent context.
- The existing Rust agent API already has a typed local action registry and
  `/agent/intent` preview route.
- Existing work item creation already records intent verification and starts in
  `pending_intent_verification`.

## Allowed Scope

- `crates/igy6-agent-api`
- `crates/igy6-gateway`
- `crates/igy6-work-queue-reports` only if needed for request/work posture
- `crates/igy6-policy` only if needed for approval posture
- `apps/web/src/app/api/agent/*`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `README.md` and `docs/ui/README.md` for user-facing behavior changes
- This DIFF document
- `docs/plans/IGY6_DEV_BUILD_PLAN.md` only if needed for status tracking

## Prohibited Scope

- No broad worker execution.
- No live queue processing.
- No runtime/private data reads or writes.
- No Docker Compose ownership changes.
- No external service calls.
- No hidden automation.
- No locked historical DIFF edits.
- No blind `dev` to `main` merge.

## Required Tags

Reference `DIFF-176` in change summaries, completion notes, and any commit or
pull request text.

## Verification

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo test -p igy6-agent-api`
- `cargo test -p igy6-gateway`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `scripts/rust-cutover.sh --check`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke` if UI changes

## Completion Criteria

- `/agent/intent` returns an explicit request category and plain-language
  request summary.
- Supported categories include:
  `evidence_question`, `add_data`, `check_work_status`, `create_report`,
  `request_action`, `system_changing_action`, `feedback`, `record_outcome`,
  `correction`, `diagnostics`, `project_status`,
  `experiment_or_improvement`, and `unclear`.
- The summary reports evidence, clarification, approval, work-item, and
  unsupported/unsafe posture.
- Ambiguous requests do not silently create work.
- Risky/system-changing requests require approval posture.
- Unsupported requests return clear unsupported or needs-more-info posture.
- Tests cover classification, clarification-needed cases, approval-required
  cases, unsupported cases, and work-item-needed cases.
- Runtime ownership remains unchanged.

## Out Of Scope Follow-Up

- Creating work items automatically from request understanding.
- Running collectors, workers, reports, experiments, or queue dispatch.
- Full conversational confirmation UI.
- Advanced ML/LLM intent classification.
- Promotion of self-improvement methods.
