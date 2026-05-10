# DIFF-073: Web Preview Same-Origin Proxy

Status: Locked

## Type

Change-bearing

## Objective

Make the Chat Retrieval Preview work when the browser reaches the web UI
through a forwarded or remote URL where browser-local `127.0.0.1:8000` does
not point at the workspace FastAPI service.

## Baseline Facts

- `POST /chat/retrieval-preview` works from the workspace.
- CORS preflight works for direct local browser origins after DIFF-072.
- The preview form still fails in the user's browser with `Failed to fetch`.
- The client-side form currently posts directly to `NEXT_PUBLIC_API_BASE_URL`
  or `http://127.0.0.1:8000`.
- Server-rendered inventory calls already work because the Next.js server can
  reach FastAPI via `API_BASE_URL`.

## Allowed Scope

- `docs/diffs/DIFF-073-web-preview-same-origin-proxy.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`

Allowed behavior change:

- Add a same-origin Next.js route that proxies only the retrieval-preview
  request to FastAPI.
- Point the Chat Retrieval Preview form at the same-origin proxy path.

## Prohibited Scope

- No backend API, database, migration, worker, collector, graph, vector,
  approval, policy, or Docker changes.
- No model calls or generated answers.
- No broad API proxy.
- No action-console behavior changes.
- No dependency changes.
- No refactors or renames.

## Required Tags

Use `DIFF-073` in change summaries, commits, and review notes for this work.

## Verification

Run:

```bash
curl -i -sS -X POST http://127.0.0.1:3001/api/chat/retrieval-preview \
  -H 'Content-Type: application/json' \
  -d '{"message":"What does the system know?","limit":5}'

curl --max-time 20 -sS http://127.0.0.1:3001 | rg 'data-api-base-url="/api"|Chat Retrieval Preview'
```

Expected outcomes:

- Same-origin preview proxy returns `200 OK`.
- Response contains `answer_status: not_generated` and an empty hit list when
  no evidence has been ingested.
- Rendered preview form points at `/api`.

## Completion Criteria

- Chat Retrieval Preview no longer requires browser access to FastAPI port
  `8000`.
- Existing evidence/retrieval semantics are unchanged.
- Verification results are recorded before locking this DIFF.

## Verification Result

- Passed: `POST http://127.0.0.1:3002/api/chat/retrieval-preview` returns
  `200 OK` with `answer_status: not_generated` and an empty hit list when no
  evidence has been ingested.
- Passed: rendered web page on `http://127.0.0.1:3002` includes
  `data-api-base-url="/api"` for the Chat Retrieval Preview form.
- Passed: `GET /health/ready` reports PostgreSQL, Redis, Qdrant, MLflow,
  Phoenix, and Neo4j as `ok`.

## Out Of Scope Follow-Up

- Same-origin proxy support for the MVP Action Console.
- Configurable external URLs for deployed non-local environments.
