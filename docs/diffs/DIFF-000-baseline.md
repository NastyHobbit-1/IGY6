# DIFF-000: Baseline

Status: Locked

## Type

Baseline/facts-only.

## Objective

Record the known repository baseline, current structure, governance setup,
branch state, untracked files, known commands, and known verification results.

This DIFF does not authorize code changes.

## Baseline Facts

Repository: `IGY6`.

Current branch state at creation:

- Branch: `main`.
- Remote tracking branch: `origin/main`.
- Current tracked HEAD: `df418fb Rename AGENT.md to AGENTS.md`.
- Recent commits:
  - `df418fb Rename AGENT.md to AGENTS.md`.
  - `af9a06c Implement Phase 0 skeleton`.
  - `451eeb8 Initial commit`.

Current uncommitted tracked changes at creation:

- `AGENTS.md` modified to add the automatic Codex entrypoint and DIFF
  governance references.

Current untracked files at creation:

- `.codex`
- `docs/agents/README.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `docs/diffs/README.md`
- `docs/diffs/DIFF_PROCESS.md`
- `docs/diffs/DIFF_TEMPLATE.md`
- `docs/diffs/DIFF-000-baseline.md`

Current top-level repository structure:

- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- `README.md`
- `apps/`
- `configs/`
- `docs/`
- `infra/`
- `packages/`
- `services/`
- `storage/`

Current service and package structure:

- `apps/web`
- `configs/evals`
- `configs/guardrails`
- `configs/sources`
- `docs/agents`
- `docs/diffs`
- `docs/planning`
- `infra/migrations`
- `infra/neo4j`
- `infra/qdrant`
- `packages/policy`
- `packages/schemas`
- `services/api`
- `services/collectors`
- `services/ml`
- `services/reports`
- `services/self_improvement`
- `services/worker`
- `storage/artifacts`
- `storage/exports`

Current Phase 0 application baseline:

- FastAPI API skeleton exists under `services/api`.
- Celery worker skeleton exists under `services/worker`.
- Next.js web status UI exists under `apps/web`.
- Docker Compose stack exists at `infra/docker-compose.yml`.
- Alembic configuration and initial migration exist under
  `services/api/migrations`.
- Placeholder package and service directories exist for policy, schemas,
  collectors, ML, reports, and self-improvement.
- Documentation exists for architecture, API, operations, security policy, user
  guide, and Phase 0 planning.

Current governance setup:

- Root `AGENTS.md` is the automatic Codex instruction entrypoint.
- `docs/diffs/DIFF_PROCESS.md` defines the strict DIFF workflow.
- `docs/diffs/DIFF_TEMPLATE.md` defines the reusable DIFF template.
- `docs/agents/AGENT_PROMPT.md` defines the generic agent prompt.
- `docs/agents/AGENT_PROMPT_CODING.md` defines the coding-agent prompt.
- `docs/diffs/README.md` and `docs/agents/README.md` describe the governance
  directories.

Known commands:

```bash
git status --short --branch --untracked-files=all
git diff --stat
python3 -m compileall services/api services/worker
docker compose -f infra/docker-compose.yml --env-file .env.example config
docker compose -f infra/docker-compose.yml --env-file .env.example up -d
curl http://127.0.0.1:8000/health/live
curl http://127.0.0.1:8000/health/ready
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T api alembic current
docker compose -f infra/docker-compose.yml --env-file .env.example exec -T worker celery -A app.celery_app:celery_app inspect ping
docker compose -f infra/docker-compose.yml --env-file .env.example down
```

Known verification results:

- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says Python syntax compilation passed for `services/api` and
  `services/worker`.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says Docker Compose configuration rendered successfully with `.env.example`.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says API, worker, and web images built successfully.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says Docker Compose stack started locally with localhost-bound ports.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says `/health/live` returned `ok`.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says `/health/ready` returned `ok` for PostgreSQL, Redis, Qdrant, Neo4j,
  MLflow, and Phoenix.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says Alembic current revision was `0001_phase0_foundation`.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says PostgreSQL contained the foundational Phase 0 tables.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says Celery worker inspection returned `pong`.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says the web status page rendered and displayed service readiness.
- Earlier Phase 0 verification recorded in `docs/planning/PHASE-0-PLAN.md`
  says web dependency install reported zero npm vulnerabilities after patched
  Next/React pins and a PostCSS override.
- Current-session `python3 -m compileall services/api services/worker` passed.
- Current-session `docker compose -f infra/docker-compose.yml --env-file
  .env.example config` passed.
- Current-session Docker runtime inspection could not run because the Docker
  daemon was unavailable: `Cannot connect to the Docker daemon at
  unix:///var/run/docker.sock. Is the docker daemon running?`

## Allowed Scope

This DIFF allows facts-only documentation of the baseline in this file.

It does not authorize application code changes.

## Prohibited Scope

This DIFF prohibits all change-bearing work, including:

- Application code changes.
- Configuration behavior changes.
- Dependency changes.
- Data model changes.
- Migration changes.
- Renames.
- Refactors.
- Behavior changes.
- Rewiring.
- Redesign.
- Formatting churn outside this baseline document.

## Required Tags

Use `DIFF-000` when referring to this baseline in summaries, reviews, commits,
or follow-up DIFFs.

## Verification

No application verification is required by this facts-only DIFF.

Baseline verification for this DIFF is limited to confirming that the document
exists and that git status records the current working tree state.

## Completion Criteria

This DIFF is complete when:

- `docs/diffs/DIFF-000-baseline.md` exists.
- It records the repository baseline facts known at creation time.
- It clearly states that it does not authorize code changes.
- It clearly states that `DIFF-001` and later are required before any
  change-bearing work.

## Out Of Scope Follow-Up

Any change-bearing work requires `DIFF-001` or later.

Future DIFFs must be explicit about active status, allowed scope, prohibited
scope, verification, and completion criteria before code changes occur.
