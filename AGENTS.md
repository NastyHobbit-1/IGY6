# AGENTS.md

## Automatic Codex Entrypoint

Codex and other coding agents must treat this file as the automatic project instruction entrypoint for IGY6 work on `grok`.

This file is dev-only build-agent instruction material. It must not be promoted to `main` unless a later DIFF explicitly changes the branch policy.

## Required First Step

Before making changes, read and follow:

- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- the latest relevant DIFF under `docs/diffs/`
- `README.md`
- `configs/rust-cutover-manifest.json` when runtime ownership, route parity, worker ownership, archive state, or cutover claims are touched
- `docs/ui/README.md` when UI behavior is touched

Do not implement from memory, ambition, or inferred intent. Verify the current repo state first.

## Branch Policy

Work on `grok` for development and build-agent assisted work on this branch.

Do not merge `dev` into `main`.

Do not merge `main` into `dev` unless the owner explicitly instructs it.

Do not cherry-pick cleanup commits from `main` into `dev` unless the owner
explicitly instructs it.

Keep private/dev/build instruction files on `dev`.

`main` remains the public/runtime-clean branch. Later, only necessary
public/runtime-safe files should be selectively promoted to `main` when the
owner explicitly requests promotion.

Dev-only files must stay off `main`, including:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- private build-agent prompts under `docs/agents/`
- dev-only plans under `docs/plans/`

## DIFF Governance

This repository uses a strict DIFF-governed workflow.

Rules:

- One active DIFF at a time.
- No code change is valid unless it is inside the active DIFF scope.
- DIFF-000 is facts-only and must not contain code changes.
- DIFF-001 and later may authorize scoped changes.
- Locked DIFFs are never edited.
- Do not mix scopes.
- Do not perform renames, refactors, behavior changes, rewiring, redesign, dependency changes, data-model changes, migration changes, or broad cleanup unless explicitly allowed by the active DIFF.
- If no active DIFF exists, create or propose the smallest correct DIFF before code changes.

## Current Runtime Truth

The current active application runtime is Rust-based:

- Rust API gateway: active.
- Rust worker daemon: active.
- Next.js web UI: active.
- Legacy Python/FastAPI API: archived and inactive.
- Legacy Python/Celery worker: archived and inactive.
- Celery beat: inactive/retired from active Compose runtime.

Supporting services include PostgreSQL, Qdrant, Neo4j, MLflow, and Phoenix. Redis and Celery are retired from the active Compose runtime.

Do not describe FastAPI, Python/Celery, or beat as active runtime services unless working inside archive/rollback history or an explicit rollback DIFF.

## Product Goal

IGY6 is a private, local-first evidence and decision-support workspace.

The product goal is not a generic chatbot, simple RAG demo, note app, benchmark viewer, prompt wrapper, scraper, or generic agent demo.

IGY6 should collect authorized information, turn it into traceable evidence, support search and reasoning over that evidence, review activity, handle approvals, track outcomes, and produce decision-ready outputs with clear evidence boundaries.

Do not claim unsupported capabilities. Current strongest path is UTF-8 text-oriented source/upload processing into artifacts, documents, chunks, evidence, vector memory, reports, audit records, approvals, and diagnostics where implemented.

Do not claim binary PDF, image, audio, or video parsing is complete unless a later scoped DIFF adds and verifies it.

## Architecture Boundaries

Respect current ownership:

- `apps/web/`: Next.js UI.
- `crates/igy6-gateway/`: Rust API gateway.
- `crates/igy6-worker/`: Rust worker daemon.
- `crates/igy6-agent-api/`: request understanding, action classification, and local action capability contracts.
- `crates/igy6-llm/`: local LLM provider/routing support.
- `crates/igy6-evidence-answer/`: evidence-grounded answer packet construction.
- `crates/igy6-artifacts/`: content-addressed artifact handling.
- `crates/igy6-normalization/`: text normalization.
- `crates/igy6-chunking/`: deterministic chunking.
- `crates/igy6-vector-memory/`: vector memory and Qdrant request behavior.
- `infra/docker-compose.yml`: active local deployment config.
- `archive/legacy-python/`: historical/rollback material only.

Do not reintroduce legacy runtime wiring unless an explicit rollback DIFF authorizes it.

## Agent and Action Safety

Agent behavior must remain typed, bounded, approval-aware, and auditable.

Rules:

- No arbitrary shell execution from user text.
- No user-provided argv execution.
- Only fixed allowlisted local actions may execute.
- System-changing actions require approval.
- Read-only actions must remain non-mutating.
- Dangerous command patterns must be rejected.
- Secrets must be redacted.
- Ambiguous, unsupported, risky, or missing-parameter requests must stay in clarification/approval posture.

## Runtime Data and Secret Rules

Runtime/private data belongs outside the repo under `IGY6_DATA_ROOT`.

Do not commit or print:

- `.env` contents
- credentials
- tokens
- cookies
- private keys
- runtime artifacts
- Docker volume data
- collected private data
- raw private exports

Do not mutate `.env`, runtime data, Docker volumes, databases, Qdrant, Neo4j, or local services unless the active DIFF explicitly allows it and verification requires it.

Prefer non-destructive checks by default.

## UI Rules

The web UI is a normal-user tabbed dashboard.

Current tabs:

- Home
- Add Data
- Work
- Results
- Settings
- Advanced

Keep normal user workflows visible and push low-level controls to Advanced. Empty states must be honest empty states, not fake demo data. Settings changes must stay dry-run/verification gated. Advanced controls must not encourage guessing IDs or bypassing approvals.

### UI styling on `grok` (custom CSS, not Tailwind/shadcn)

The Next.js app under `apps/web/` uses **plain CSS** in `apps/web/src/app/globals.css` plus server-rendered markup in `apps/web/src/app/page.tsx`. There is **no** Tailwind or shadcn dependency in this branch.

When polishing UI components:

- Prefer existing semantic classes (`panel`, `panelInset`, `minimalSlot`, `pill`, `tabList`, etc.) before adding new ones.
- Add responsive layout with CSS media queries or utility classes such as `responsiveToolbar`, `responsiveStatusRow`, and `responsivePanelGrid` (flex-wrap / `auto-fit` grids). Mobile-first: stack vertically below ~640–860px, row layout on wider screens.
- Use `Skeleton` / `SkeletonBlock` in `page.tsx` and matching `.skeleton` rules in `globals.css` for loading states instead of bare `"Loading..."` text.
- Keep **Simple mode** (`body.minimal-ui-mode`) working: `minimalWorkspace` cards must remain usable on narrow viewports.
- Do not introduce Tailwind, shadcn, or a component library unless an active DIFF explicitly authorizes it.

See `docs/ui/README.md` for tab behavior and smoke expectations.

## Coding Rules

For Rust:

- Preserve deterministic behavior where expected.
- Preserve request/response shapes unless the DIFF explicitly changes them.
- Validate inputs explicitly.
- Bound user-controlled sizes, counts, paths, and polling behavior.
- Keep paths traversal-safe and rooted under approved roots.
- Keep network origins bounded and credential-free unless explicitly allowed.
- Add or update tests for behavior changes.

For TypeScript/Next.js:

- Keep API proxy routes aligned with the Rust gateway.
- Avoid stale references to FastAPI unless discussing archive/rollback history.
- Do not expose secrets in UI or API responses.
- Preserve normal-user wording unless diagnostics are explicitly scoped.

For scripts:

- Use safe defaults.
- Prefer check/dry-run modes.
- Do not delete volumes or runtime data by default.
- Print exact commands when useful.
- Do not hide destructive behavior behind broad commands.

## Verification

Use the verification required by the active DIFF. Do not run broad or destructive checks unless scoped.

Common checks include:

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

Record any verification that could not be run and why.

## Required Final Response

Every Codex final response must include:

- active branch
- DIFF ID
- files changed
- summary of what changed
- verification commands run and results
- verification that could not be run and why
- confirmation that prohibited scope was avoided
- whether the work is ready for clean promotion to `main`

## Promotion Rule

If runtime/product work is ready for `main`, do not merge `dev` into `main`.
Wait for explicit owner instructions before promoting any files or commits.
Promote only necessary public/runtime-safe files to `main`, and verify
forbidden private/dev files are absent from the promotion target.
