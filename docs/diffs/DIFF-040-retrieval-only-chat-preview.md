# DIFF-040: Retrieval-Only Chat Preview

Status: Locked

## Type

Change-bearing.

## Objective

Add a minimal retrieval-only chat preview endpoint that accepts a user message
and returns hydrated retrieval context with `answer_status` set to
`not_generated`.

This DIFF creates a safe backend contract for future evidence-backed chat
without generating answers, calling external models, storing conversations, or
triggering actions.

## Baseline Facts

- DIFF-000 through DIFF-039 are locked.
- Hydrated semantic retrieval exists at `POST /retrieval/chunks/search`.
- No chat endpoint exists yet.
- Chat answer generation is still out of scope.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-040-retrieval-only-chat-preview.md`
- `docs/api.md`
- `services/api/app/chat.py`
- `services/api/app/main.py`

Allowed behavior:

- Add `POST /chat/retrieval-preview`.
- Accept a user message and bounded retrieval limit.
- Reuse hydrated semantic retrieval to return context.
- Return `answer_status: "not_generated"`.
- Return no generated answer text.

## Prohibited Scope

This DIFF does not allow:

- Editing locked DIFFs.
- Database model changes.
- Migration changes.
- UI changes.
- Worker task changes.
- Source collection.
- Artifact content reads.
- Changes to vector embedding or search behavior.
- Neo4j traversal.
- External model calls.
- Generated answers.
- Conversation persistence.
- Approval workflow changes.
- Self-improvement queueing.
- Writes to PostgreSQL, Qdrant, Neo4j, or artifact storage.
- Dependency changes.
- Docker rewiring.
- Renames.
- Broad refactors.
- Formatting churn outside allowed files.

## Required Tags

Use `DIFF-040` in change summaries, commits, and review notes for this work.

## Verification

Required checks:

```bash
python3 -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
PYTHONPATH=services/api .venv/bin/python -c "from app.chat import ChatRetrievalPreviewRequest; print(ChatRetrievalPreviewRequest(message='what do we know?').limit)"
```

Results:

- Passed: `python3 -m compileall services/api services/worker services/collectors packages/policy`
- Passed: `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- Passed: `PYTHONPATH=services/api .venv/bin/python -c "from app.chat import ChatRetrievalPreviewRequest; print(ChatRetrievalPreviewRequest(message='what do we know?').limit)"`

## Completion Criteria

This DIFF is complete when:

- Retrieval-only chat preview route exists.
- Route returns hydrated retrieval context.
- Route returns `answer_status: "not_generated"`.
- Route does not return generated answer text.
- Required verification passes or any blockage is recorded.
- No prohibited files or behavior are changed.

## Out Of Scope Follow-Up

Future DIFFs must cover:

- Conversation persistence.
- Evidence-backed answer generation.
- Feedback controls on chat responses.
- Retrieval planning across PostgreSQL, Qdrant, and Neo4j.
