# Codex Prompt Baseline for IGY6

Use this baseline in every future Codex prompt for IGY6 work.

This file is branch-local build-agent instruction material for the active `grok` branch. It must stay on `grok` unless a later DIFF explicitly changes the branch policy. Do not promote this file to `main` under DIFF-294.

## Current Active DIFF

- DIFF-294 — Production-Readiness and Productization (Active, change-bearing)

## Required Opening Context

You are working in the IGY6 repository.

Before making changes, read and follow:

- `docs/BRANCH_POLICY.md`
- the latest relevant DIFF under `docs/diffs/`
- `README.md`
- `docs/ui/README.md` when UI behavior is touched
- `configs/rust-cutover-manifest.json` when runtime ownership, Rust migration, cutover, route parity, worker ownership, or archive state is touched

Do not start implementation from memory, ambition, or inferred intent. Verify the current repo state first.

## Branch Rules

- Work on `grok` only. Do not develop on other branches unless explicitly authorized by a later DIFF.
- Do not merge `grok` into `main`.
- Do not add build-agent instruction files to `main`.
- Under DIFF-294, promotion to `main` is out of scope. If a later DIFF authorizes promotion, do so via a clean branch from `main` and cherry-pick only the scoped runtime/product commit; verify forbidden files are absent.

Forbidden on `main` unless a later DIFF explicitly changes policy:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/agents/CODEX_DIFF_123_129_PROMPT.md`
- `docs/agents/CODEX_DIFF_130_OLLAMA_ROUTING_PROMPT.md`
- `docs/agents/RUST_COMPLETION_MANAGER_PROMPT.md`
- `docs/plans/DEV_BRANCH_POLICY.md`
- `docs/plans/IGY6_DEV_BUILD_PLAN.md`

Allowed tracked product docs such as `docs/agents/README.md` and `docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md` may remain on `main` only if they do not contain private build-agent instructions.

## DIFF Governance

Every change must be governed by one active DIFF.

Rules:

- Do not edit locked DIFFs.
- Do not mix scopes from multiple DIFFs.
- Do not create broad cleanup outside the active DIFF.
- Do not refactor, rename, rewire, redesign, change dependencies, alter data models, or change migrations unless the active DIFF explicitly allows it.
- If no active DIFF exists, create or propose the smallest correct DIFF before code changes.
- Each DIFF must define objective, baseline facts, allowed scope, prohibited scope, verification, completion criteria, result, and verification result.

## Current Product/Runtime Posture

IGY6 is a local-first evidence and decision-support workspace.

Current active runtime posture:

- Rust API gateway is active.
- Rust worker daemon is active.
- Next.js web UI is active.
- Legacy Python/FastAPI API is archived and inactive.
- Legacy Python/Celery worker is archived and inactive.
- Celery beat is inactive/retired from active Compose runtime.

Supporting services include PostgreSQL, Qdrant, Neo4j, MLflow, and Phoenix.

Do not claim unsupported capabilities. The strongest current product path is UTF-8 text-oriented workflows: source/upload, artifact/document/chunk/evidence/vector processing, evidence answers, reports, audit records, approvals, and local diagnostics where implemented.

Do not claim binary PDF, image, audio, or video parsing unless a later scoped DIFF adds and verifies it.

## Runtime Data and Secret Rules

Runtime/private data belongs outside the repo under `IGY6_DATA_ROOT`.

Do not commit or print:

- `.env` contents
- credentials
- tokens
- cookies
- private keys
- collected private data
- runtime artifacts
- Docker volume data
- raw private exports

Do not mutate `.env`, runtime data, Docker volumes, databases, Qdrant, Neo4j, or local services unless the active DIFF explicitly allows it and verification requires it.

Prefer non-destructive checks by default.

## Architecture Rules

Respect the current architecture:

- `apps/web/` is the Next.js UI.
- `crates/igy6-gateway/` owns the Rust API gateway.
- `crates/igy6-worker/` owns the Rust worker daemon.
- `crates/igy6-agent-api/` owns request understanding, action classification, and local action capability contracts.
- `crates/igy6-llm/` owns local LLM provider/routing support.
- `crates/igy6-evidence-answer/` owns evidence-grounded answer packet construction.
- `crates/igy6-artifacts/`, `crates/igy6-normalization/`, `crates/igy6-chunking/`, and `crates/igy6-vector-memory/` own artifact, normalization, chunking, and vector behavior.
- `infra/docker-compose.yml` is the active local deployment config.
- `archive/legacy-python/` is historical/rollback material only and not the active runtime path.

Do not reintroduce FastAPI, Python/Celery, or beat as active runtime services unless an explicit rollback DIFF authorizes it.

## Agent and Action Safety Rules

Agent behavior must remain typed, bounded, approval-aware, and audit-oriented.

Rules:

- No arbitrary shell execution from user text.
- No user-provided argv execution.
- Only fixed allowlisted local actions may execute.
- System-changing actions require approval.
- Read-only actions must stay non-mutating.
- Dangerous command patterns must be rejected.
- Outputs that may contain secrets must be redacted.
- Request understanding must clarify ambiguous, unsupported, risky, or missing-parameter requests before action.

## UI Rules

The UI is a normal-user tabbed dashboard, not a developer-only console.

Current visible tabs in the grok UI:

- Chat (default)
- Data
- Work
- Settings
- More

Note: internal panel sections still use headings like Home readiness, Add Data,
Results/Evidence, and Advanced; the tab bar labels above are what users see.

Rules:

- Keep normal user workflows visible and technical controls pushed to Advanced.
- Do not show fake demo data as real state.
- Empty states must be honest empty states.
- Do not claim unsupported routes, source types, or parsing behavior.
- Settings changes must remain dry-run/verification gated.
- Advanced controls must not encourage guessing IDs or bypassing approvals.

## Coding Rules

For Rust:

- Keep behavior deterministic where the existing design requires deterministic behavior.
- Preserve request/response shapes unless the active DIFF explicitly versions or changes them.
- Validate inputs explicitly.
- Use bounded limits for user-controlled sizes, counts, and polling behavior.
- Keep filesystem paths traversal-safe and rooted under approved roots.
- Keep network origins bounded and credential-free unless explicitly allowed.
- Add or update tests for behavior changes.

For TypeScript/Next.js:

- Keep API proxy routes aligned with the Rust API gateway.
- Avoid stale references to FastAPI unless discussing archive/rollback history.
- Do not expose secrets in UI or API responses.
- Preserve normal-user wording unless the active DIFF asks for developer diagnostics.

For scripts:

- Use safe defaults.
- Do not delete volumes or runtime data by default.
- Prefer check/dry-run modes for validation.
- Print exact commands when useful.
- Do not hide destructive behavior behind broad commands.

## Verification Requirements

Use the verification required by the active DIFF. For common changes, include only checks relevant to touched scope.

Common checks:

- `git status --short`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `npm --prefix apps/web run build`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`

Do not run destructive live runtime checks unless explicitly scoped.

## Required Final Response Format

Every Codex final response should include:

- active branch
- DIFF ID
- files changed
- summary of what changed
- verification commands run and results
- any verification that could not be run and why
- explicit confirmation that prohibited scope was avoided
- whether the work is ready for clean promotion to `main`

## Promotion Checklist for Runtime/Product Work

Before asking to promote work to `main`, verify:

- work is committed on `grok`
- one commit or clearly identified commit range contains the runtime/product change
- dev-only files are not part of the commit to promote
- clean branch from `main` can cherry-pick the intended commit
- forbidden-file check passes
- `git diff --check` passes
- active DIFF result and verification result are complete

Promotion must be via clean branch and PR into `main`, never by merging `grok` into `main`.
