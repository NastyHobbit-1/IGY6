# DIFF-185 Evidence Answer Review UX

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- Public/runtime-safe changes can be promoted to `main` later by explicit instruction.
- This DIFF does not merge, cherry-pick, remove private/dev files, or start Rust migration work.

## Purpose

Make the user-facing Results evidence review path clear, useful, and trustworthy now that DIFF-184 backed `/chat/retrieval-preview` with live evidence retrieval.

The review path should show whether evidence was retrieved, what was retrieved, why the status is trustworthy or limited, which chunks/evidence items support the result, and what the next safe action is for empty, partial, successful, or error states.

## Scope

Allowed:

- This DIFF record.
- Smallest necessary `apps/web` files for Results/retrieval review UX.
- Smallest necessary Next API proxy files only if the proxy blocks the UX.
- Smallest necessary Rust gateway files only if additive response fields or tests are required.

Not allowed:

- Removing anything from `dev`.
- Removing `.codex`, `AGENTS.md`, `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, `docs/agents`, or `docs/plans`.
- Merging `main` into `dev`.
- Cherry-picking `main` into `dev`.
- Runtime refactors, Docker Compose rewrites, `.env` edits, Rust migration work, or LLM answer generation.

## Current Branch And HEAD Before Work

- Branch: `dev`
- HEAD before work: `e3b452e Complete DIFF-184 manual upload evidence retrieval followthrough`
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker in `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- DIFF inventory under `docs/diffs`
- Tracked private/dev inventory under `.codex`, `AGENTS.md`, `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, `docs/agents`, and `docs/plans`

## Current UI/API Behavior Before Changes

- The Results page `ChatRetrievalPreview` component posts to `/api/chat/retrieval-preview`.
- The Next proxy forwards requests to the Rust gateway `/chat/retrieval-preview` route.
- The Rust route returns live retrieval data with `answer_status`, `retrieval_context.hits`, `items`, source, document, chunk, score, and evidence item fields.
- The existing UI showed a compact status line and a basic list of hit title, score, chunk, source, and evidence item count.
- Empty results showed only `No retrieval context returned.`
- Error responses updated the status text but did not render an in-panel review state.

## Changes Made

- Updated `apps/web/src/app/page.tsx` in `ChatRetrievalPreview`.
- Added a Results review summary rendered after retrieval completes.
- Added explicit empty, successful, and error review states.
- Added user-facing guidance that the review is evidence-backed only when hits are present, and that empty results mean insufficient evidence rather than proof of absence.
- Added hit-level review details where returned by the live retrieval response:
  - `answer_status`
  - hit count
  - collection availability
  - score
  - retrieval mode from returned payload fields
  - chunk and document identifiers
  - source label or identifier
  - evidence item count
  - short chunk/evidence text preview
  - first returned evidence item snippets
- Added DOM markers for verification:
  - `data-retrieval-review-guidance`
  - `data-retrieval-review-summary`
  - `data-retrieval-review-hit`
  - `data-retrieval-review-error`

## API/Contract Changes

- None.
- The existing Rust `/chat/retrieval-preview` response already returned enough live retrieval data for this UX.
- The Next `/api/chat/retrieval-preview` proxy was unchanged.
- No Rust files were changed.
- No LLM answer generation was added.

## Test Token Strategy

Runtime verification will use synthetic text and a unique token only. No `.env` contents or runtime/private data from `IGY6_DATA_ROOT` will be printed or dumped.

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -14
git branch -vv
git diff --name-status
git diff --check
sed -n '1,240p' AGENTS.md
sed -n '1,220p' docs/BRANCH_POLICY.md
sed -n '1,280p' docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
find docs/diffs -maxdepth 1 -type f | sort | tail -80
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
find apps/web crates/igy6-gateway crates/igy6-agent-api docs scripts -maxdepth 5 -type f | sort | grep -E "result|retrieval|evidence|answer|chat|intent|route|page|ui|manual|upload" || true
grep -R "retrieval-preview\|answer_status\|evidence\|Results\|retrieved\|no evidence\|manual upload\|chunk\|source" apps/web crates/igy6-gateway crates/igy6-agent-api docs scripts -n 2>/dev/null | head -300 || true
sed -n '1040,1165p' apps/web/src/app/page.tsx
sed -n '2350,2475p' apps/web/src/app/page.tsx
sed -n '1,220p' apps/web/src/app/api/chat/retrieval-preview/route.ts
rg -n "live_retrieval_preview|retrieval_preview_response|answer_status|retrieval_context" crates/igy6-gateway/src/lib.rs
docker compose -f infra/docker-compose.yml --env-file .env config
grep -n '^IGY6_DATA_ROOT=' .env
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
npm --prefix apps/web run build
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
scripts/run.sh
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API live HTTP %{http_code}\n' http://127.0.0.1:8000/health/live
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API ready HTTP %{http_code}\n' http://127.0.0.1:8000/health/ready
curl --silent --show-error --max-time 20 --output /tmp/igy6-diff185/page.html --write-out 'Web UI HTTP %{http_code}\n' http://127.0.0.1:3000/
curl ... POST http://127.0.0.1:8000/sources
curl ... POST http://127.0.0.1:8000/approvals
curl ... POST http://127.0.0.1:8000/approvals/{approval_id}/decision
curl ... POST http://127.0.0.1:8000/collection-runs/manual-upload
curl ... GET http://127.0.0.1:8000/work-items
curl ... GET http://127.0.0.1:8000/evidence/documents
curl ... GET http://127.0.0.1:8000/evidence/chunks
curl ... GET http://127.0.0.1:8000/evidence/items
curl ... POST http://127.0.0.1:8000/chat/retrieval-preview
curl ... POST http://127.0.0.1:3000/api/chat/retrieval-preview
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
```

Verification results:

- `git diff --check` passed before runtime testing.
- Docker Compose config rendered successfully.
- `IGY6_DATA_ROOT` key was present in `.env`; the configured adjacent data-root directory exists.
- `npm --prefix apps/web run build` passed.
- No listeners were present on ports 3000, 8000, or 8765 before stack start.
- `scripts/run.sh` started the local stack.
- API live probe returned HTTP 200.
- API ready probe returned HTTP 200.
- Web UI probe returned HTTP 200.
- Synthetic source creation returned HTTP 201.
- Synthetic approval creation returned HTTP 201.
- Approval decision returned HTTP 200 after two transient local curl connection failures when using shell-sourced environment variables; a literal URL retry succeeded immediately while live/ready probes were healthy.
- Manual upload returned HTTP 201.
- The upload created one raw artifact and one normalization work item.
- The normalization work item reached `completed`.
- Evidence record surfaces returned HTTP 200:
  - documents: 3 total records, 1 synthetic-token match;
  - chunks: 3 total records, 1 synthetic-token match;
  - evidence items: 3 total records, 1 synthetic-token match.
- Rust `/chat/retrieval-preview` returned HTTP 200 with `answer_status=retrieved`, 3 hits, a synthetic-token match, and retrieval mode `local_hash_v1`.
- Next `/api/chat/retrieval-preview` returned HTTP 200 with `answer_status=retrieved`, 3 hits, a synthetic-token match, and matching live retrieval behavior.
- Browser automation was not used; UI verification used build-level and curl/grep checks.
- The server-rendered Results page contained the review guidance marker.
- The server-rendered script contained the review summary and hit markers used by the client-side retrieval review rendering.
- After stopping the stack, ports 3000, 8000, and 8765 were not listening.

## Bugs Found

- No product/runtime retrieval bug was found.
- The only observed issue was a transient local curl connection failure during approval decision retries with shell-sourced variables. Live/ready probes were healthy, and the same approval decision succeeded with a literal URL. This matches the transient probe behavior recorded in DIFF-184 and was not treated as a DIFF-185 product bug.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`

## Final Status

Complete.
