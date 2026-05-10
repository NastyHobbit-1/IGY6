# DIFF-072: Local Web API CORS

Status: Locked

## Type

Change-bearing

## Objective

Allow the local Next.js web UI to call the local FastAPI backend from browser
client-side forms without weakening the local-first boundary.

## Baseline Facts

- The web UI is running on `http://127.0.0.1:3001`.
- The API is running on `http://127.0.0.1:8000`.
- Direct `POST /chat/retrieval-preview` requests return `200`.
- Browser preflight for `POST /chat/retrieval-preview` returns `405` because
  the API has no CORS middleware.
- Qdrant and API readiness are healthy.

## Allowed Scope

- `docs/diffs/DIFF-072-local-web-api-cors.md`
- `services/api/app/main.py`

Allowed behavior change:

- Add FastAPI CORS middleware for localhost/loopback web development origins
  only.

## Prohibited Scope

- No database, migration, model, worker, collector, vector-memory, graph-memory,
  chat-answer, retrieval, approval, policy, Docker, or frontend behavior changes.
- No wildcard CORS origin.
- No remote/network exposure.
- No dependency changes.
- No refactors or renames.

## Required Tags

Use `DIFF-072` in change summaries, commits, and review notes for this work.

## Verification

Run:

```bash
curl -i -sS -X OPTIONS http://127.0.0.1:8000/chat/retrieval-preview \
  -H 'Origin: http://127.0.0.1:3001' \
  -H 'Access-Control-Request-Method: POST' \
  -H 'Access-Control-Request-Headers: content-type'

curl -i -sS -X POST http://127.0.0.1:8000/chat/retrieval-preview \
  -H 'Origin: http://127.0.0.1:3001' \
  -H 'Content-Type: application/json' \
  -d '{"message":"What does the system know?","limit":5}'
```

Expected outcomes:

- Preflight returns `200 OK`.
- Responses include `access-control-allow-origin: http://127.0.0.1:3001`.
- Retrieval preview still returns `answer_status: not_generated` with an empty
  hit list when no evidence has been ingested.

## Completion Criteria

- The local browser UI can call FastAPI client-side forms from `127.0.0.1:3001`.
- CORS remains limited to loopback/local development origins.
- Verification results are recorded before locking this DIFF.

## Verification Result

- Passed: browser-style preflight for `Origin: http://127.0.0.1:3001`
  returns `200 OK` with `access-control-allow-origin:
  http://127.0.0.1:3001`.
- Passed: browser-origin `POST /chat/retrieval-preview` returns `200 OK`
  with `answer_status: not_generated` and an empty hit list when no evidence
  has been ingested.
- Passed: `GET /health/ready` reports PostgreSQL, Redis, Qdrant, MLflow,
  Phoenix, and Neo4j as `ok`.

## Out Of Scope Follow-Up

- Configurable CORS origins for future non-local deployment profiles.
- UI workflow improvements beyond making the existing client-side fetch work.
