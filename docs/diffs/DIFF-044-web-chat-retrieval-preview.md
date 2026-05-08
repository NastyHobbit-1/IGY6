# DIFF-044: Web Chat Retrieval Preview

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal web UI control for the retrieval-only chat preview endpoint.
The UI lets the user submit a message to FastAPI and inspect the returned
hydrated retrieval context with `answer_status: not_generated`.

This DIFF does not add answer generation, conversation persistence, external
model calls, source writes, approvals, or actions.

## Baseline Facts

- DIFF-000 through DIFF-043 are locked.
- `POST /chat/retrieval-preview` exists and returns retrieval context with no
  generated answer.
- The web UI is currently server-rendered read-only inventory and inspector
  sections.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-044-web-chat-retrieval-preview.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `docs/user-guide.md`

Allowed behavior:

- Add a client-side form that posts to FastAPI `POST /chat/retrieval-preview`.
- Display `answer_status`.
- Display returned hydrated retrieval hits.
- Keep the UI explicit that no answer is generated.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Backend changes.
- Database model changes.
- Migration changes.
- Source creation.
- Collection execution.
- Upload.
- Evidence creation.
- Approval decisions.
- Conversation persistence.
- External model calls.
- Generated answers.
- Direct PostgreSQL, artifact-store, local file, Qdrant, Neo4j, Redis, MLflow,
  or Phoenix access from the frontend.
- Worker scheduling.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-044` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
```

Results:

- Passed: `npm --prefix apps/web run build`
- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`

## Completion Criteria

This DIFF is complete when:

- Web UI has a chat retrieval preview form.
- Form calls only FastAPI `POST /chat/retrieval-preview`.
- UI displays `answer_status: not_generated`.
- UI displays returned retrieval hit metadata.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Evidence-backed generated chat answers.
- Conversation persistence.
- Feedback controls on chat responses.
- Approval-gated actions.
