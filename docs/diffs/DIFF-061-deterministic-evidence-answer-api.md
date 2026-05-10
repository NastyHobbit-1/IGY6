# DIFF-061: Deterministic Evidence Answer API

Status: Locked

## Type

Change-bearing.

## Objective

Add a deterministic chat answer endpoint that summarizes retrieved evidence
into a cited answer packet without calling models, executing tools, or taking
actions.

## Baseline Facts

- DIFF-000 through DIFF-060 are locked.
- `POST /chat/retrieval-preview` returns hydrated retrieval context with
  `answer_status: not_generated`.
- Hydrated retrieval exposes chunks, documents, sources, raw artifacts, and
  evidence items.
- No answer packet endpoint exists.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-061-deterministic-evidence-answer-api.md`
- `services/api/app/chat.py`
- `docs/api.md`

Allowed behavior:

- Add `POST /chat/evidence-answer`.
- Reuse hydrated semantic retrieval.
- Return deterministic answer sections for facts, assumptions, inferences,
  uncertainty, missing information, and source trails.
- Cite evidence item IDs or chunk IDs for every fact/inference.
- Return no external model output and execute no actions.
- Document the route and limits.

## Prohibited Scope

This DIFF does not allow conversation persistence, database writes, external
model calls, agent/tool execution, approval decisions, source collection,
artifact content reads, graph traversal changes, vector search changes, worker
dispatch, UI changes, migrations, dependency changes, or broad refactors.

## Required Tags

Use `DIFF-061` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
```

Targeted smoke checks should validate answer packet construction without a
database.

Completed verification:

```bash
.venv/bin/python -m compileall services/api services/worker services/collectors packages/policy
docker compose -f infra/docker-compose.yml --env-file .env.example config
git diff --check
.venv/bin/python -c "import sys; sys.path.insert(0, 'services/api'); from app.chat import build_evidence_answer_packet; from app.retrieval import HydratedChunkSearchResult; packet = build_evidence_answer_packet(HydratedChunkSearchResult(query='what is known?', hits=[])); assert packet.answer_status == 'insufficient_evidence'; assert packet.missing_information; assert packet.facts == []"
```

## Completion Criteria

This DIFF is complete when `POST /chat/evidence-answer` returns a deterministic
evidence-backed answer packet, the route is documented, verification passes,
and no prohibited behavior is introduced.
