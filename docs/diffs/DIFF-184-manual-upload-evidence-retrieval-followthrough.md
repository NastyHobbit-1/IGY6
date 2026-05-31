# DIFF-184: Manual Upload Evidence Retrieval Followthrough

Status: Complete

## Type

Verification-first runtime followthrough

## Objective

Verify that text submitted through the guided/manual upload path can be
retrieved afterward through the current Results/evidence/search surfaces. If
retrieval is broken, scope and make only the smallest direct fix.

## Branch Policy

- Future IGY6 work happens on `dev`.
- Private/dev/build instruction files stay on `dev`.
- `main` remains the public/runtime-clean branch.
- Later, only necessary public/runtime-safe files should be selectively
  promoted to `main`.
- Do not merge `main` into `dev` unless explicitly instructed.
- Do not cherry-pick `main` into `dev` unless explicitly instructed.
- This DIFF removes no private/dev files.

## Baseline Facts

- Branch before work: `dev`.
- HEAD before work:
  `1c41068 Complete DIFF-183 dev next runtime work selection`.
- `dev` was up to date with `origin/dev` before this DIFF.
- Working tree was clean before this DIFF.
- DIFF-183 selected manual upload evidence retrieval followthrough as the next
  highest-value runtime work.
- DIFF-182 verified guided manual text upload through source, permission,
  approval, collection run, raw artifact, normalization work item, completed
  work item, and document/chunk/evidence counts.
- Private/dev files remained tracked on `dev`.

## Allowed Scope

If no retrieval bug is found:

- This DIFF file only.

If a directly scoped retrieval bug is found:

- This DIFF file.
- The smallest necessary app/API/script/test files directly related to manual
  upload evidence retrieval.

## Prohibited Scope

- Do not remove anything from `dev`.
- Do not remove `.codex`.
- Do not remove `AGENTS.md`.
- Do not remove `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`.
- Do not remove `docs/agents`.
- Do not remove `docs/plans`.
- Do not merge `main` into `dev`.
- Do not cherry-pick `main` into `dev`.
- Do not start unrelated Rust migration work.
- Do not perform broad refactors.
- Do not edit Docker Compose unless a verified DIFF-184 bug is specifically in
  Compose.
- Do not edit `.env`.
- Do not print secrets.
- Do not print `.env` contents.
- Do not dump runtime/private data from `IGY6_DATA_ROOT`.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md`
- `docs/diffs/DIFF-183-dev-next-runtime-work-selection.md`
- `scripts/e2e-manual-upload-smoke.py`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-retrieval-preview/src/lib.rs`
- `crates/igy6-vector-memory/src/lib.rs`
- `crates/igy6-worker/src/lib.rs`
- tracked private/dev file inventory from `git ls-files AGENTS.md .codex
  Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents
  docs/plans | sort`

## Manual Upload Test Phrase Strategy

Use a synthetic DIFF-184 phrase with a timestamp-derived token. The phrase must
be harmless, local, and generated for this verification only. It must not come
from private runtime data, `.env`, or `IGY6_DATA_ROOT` contents.

## Commands Run

Pre-work inspection:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -12
git branch -vv
git diff --name-status
git diff --check
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/BRANCH_POLICY.md
sed -n '1,260p' docs/diffs/DIFF-182-dev-runtime-smoke-manual-upload-verification.md
sed -n '1,260p' docs/diffs/DIFF-183-dev-next-runtime-work-selection.md
find docs/diffs -maxdepth 1 -type f | sort | tail -70
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
find apps services crates scripts docs -maxdepth 4 -type f | sort | grep -E "manual|upload|evidence|result|retrieval|search|source|work|chunk|artifact|intent|route|api" || true
grep -R "manual upload\|guided manual\|evidence\|Results\|retrieval\|search\|chunks\|work item\|source" apps services crates scripts docs -n 2>/dev/null | head -240 || true
sed -n '1,320p' scripts/e2e-manual-upload-smoke.py
sed -n '1,260p' apps/web/src/app/api/chat/retrieval-preview/route.ts
rg -n "manual|upload|evidence|Results|retrieval|search|chunks|work item|source" apps/web/src/app/page.tsx
rg -n "manual|upload|evidence|retrieval|search|chunks|work_item|collection-runs|artifacts|documents" crates/igy6-gateway/src/lib.rs crates/igy6-retrieval-preview/src/lib.rs crates/igy6-vector-memory/src/lib.rs crates/igy6-worker/src/lib.rs
```

Runtime verification:

```bash
docker compose -f infra/docker-compose.yml --env-file .env config --quiet
grep -q '^IGY6_DATA_ROOT=' .env && echo "IGY6_DATA_ROOT is set in .env" || echo "IGY6_DATA_ROOT is missing from .env"
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
npm --prefix apps/web run build
scripts/run.sh
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API live HTTP %{http_code}\n' http://127.0.0.1:8000/health/live
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API ready HTTP %{http_code}\n' http://127.0.0.1:8000/health/ready
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'Web UI HTTP %{http_code}\n' http://127.0.0.1:3000/
python3 scripts/e2e-manual-upload-smoke.py --check
```

The first Python smoke-helper check failed under sandbox localhost restrictions
with `Operation not permitted`, matching the DIFF-182 sandbox behavior. The same
helper passed when rerun with approved localhost access.

Synthetic upload and retrieval verification:

```bash
curl ... POST http://127.0.0.1:8000/sources
curl ... POST http://127.0.0.1:8000/approvals
curl ... POST http://127.0.0.1:8000/approvals/{approval_id}/decision
curl ... POST http://127.0.0.1:8000/collection-runs/manual-upload
curl ... GET http://127.0.0.1:8000/work-items
curl ... GET http://127.0.0.1:8000/evidence/documents
curl ... GET http://127.0.0.1:8000/evidence/chunks
curl ... GET http://127.0.0.1:8000/evidence/items
curl ... POST http://127.0.0.1:8000/chat/retrieval-preview
curl ... POST http://127.0.0.1:8000/retrieval/chunks/search
curl ... POST http://127.0.0.1:8000/memory/vector/chunks/search
curl ... POST http://127.0.0.1:3000/api/chat/retrieval-preview
```

The `curl ...` commands used temporary JSON payload files under
`/tmp/igy6-diff184`. They did not read `.env` contents and did not dump
`IGY6_DATA_ROOT` contents.

Fix verification:

```bash
cargo fmt --all --check
cargo test -p igy6-gateway retrieval_preview_requires_live_database_and_evidence_answer_is_contract_only
scripts/run.sh
curl ... POST http://127.0.0.1:8000/chat/retrieval-preview
curl ... POST http://127.0.0.1:3000/api/chat/retrieval-preview
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
```

## Upload Verification Result

- Synthetic token strategy used a generated harmless local token:
  `diff184-retrieval-token-1780202776`.
- Source creation returned HTTP 201.
- Approval creation returned HTTP 201.
- Approval decision returned HTTP 200 after one transient connection failure and
  immediate retry.
- Manual upload returned HTTP 201.
- The normalization work item reached `completed`.
- The evidence record surfaces returned HTTP 200:
  - documents: 2 total records, 1 synthetic-token match;
  - chunks: 2 total records, 1 synthetic-token match;
  - evidence items: 2 total records, 1 synthetic-token match.

## Retrieval / Results / Evidence Verification Result

- Before the fix, the lower-level record surfaces proved the upload had been
  processed, and `/memory/vector/chunks/search` returned two vector hits, but
  `/chat/retrieval-preview` returned HTTP 200 with zero hits and no usable
  `retrieval_context`.
- `/retrieval/chunks/search` returned HTTP 200 with zero hits for the full
  natural-language query because that route performs literal text matching.
- After the fix, `POST /chat/retrieval-preview` returned:
  - HTTP 200;
  - `answer_status: retrieved`;
  - `retrieval_context.collection_exists: true`;
  - 2 retrieval hits;
  - 1 hit containing the synthetic token.
- After the fix, the Next.js Results-facing proxy
  `POST /api/chat/retrieval-preview` returned:
  - HTTP 200;
  - `answer_status: retrieved`;
  - 2 retrieval hits;
  - 1 hit containing the synthetic token.

## Bugs Found

- Product/runtime bug found: the user-facing Results retrieval path used
  `/chat/retrieval-preview`, but the Rust gateway implementation was still a
  contract-only stub. It returned `items: []` and no live hydrated retrieval
  context even when uploaded evidence existed and vector search could find the
  matching chunk.
- The break was route wiring/API behavior, not missing upload processing,
  missing evidence records, or missing vector upsert.
- A transient approval-decision curl attempt failed to connect to port 8000 and
  passed immediately when rerun with the literal approval URL. API live/ready
  checks passed before and after, so this was recorded as a transient runtime
  probe artifact rather than a product bug.

## Fixes Made

- Replaced the `/chat/retrieval-preview` contract-only response path with a
  live Rust retrieval preview response.
- The live route now:
  - accepts `message` or `query`;
  - first tries the existing PostgreSQL text search/hydration path;
  - falls back to existing Qdrant vector search when text search has no hits;
  - hydrates vector hits through the existing chunk trail logic;
  - returns `retrieval_context` plus `items` for compatibility with the current
    Results UI script.
- Updated the scoped gateway unit test so a no-database unit request now expects
  the live DB route to report the missing database URL instead of returning a
  contract-only preview.

## Files Changed

- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`

## Verification Summary

- Pre-work confirmed branch `dev`, clean working tree, and HEAD
  `1c41068 Complete DIFF-183 dev next runtime work selection`.
- `git branch -vv` showed `dev` at `1c41068 [origin/dev]`, synced with
  `origin/dev` before this DIFF.
- Private/dev files remained tracked on `dev`.
- Docker Compose config validated with `.env`.
- `IGY6_DATA_ROOT` key presence and directory existence were checked without
  printing `.env` contents or dumping runtime/private data.
- No port conflict existed before startup.
- `npm --prefix apps/web run build` passed.
- `scripts/run.sh` built and started the stack.
- API live, API ready, and web UI probes returned HTTP 200.
- Synthetic manual upload, processing, document/chunk/evidence presence, Rust
  API retrieval, and Next.js Results proxy retrieval were verified.
- The stack was stopped after runtime testing, and no listeners remained on
  ports 3000, 8000, or 8765.
- `cargo fmt --all --check` passed.
- `cargo test -p igy6-gateway retrieval_preview_requires_live_database_and_evidence_answer_is_contract_only`
  passed.
- No private/dev files were removed.
- No `main` work, merge, cherry-pick, Docker Compose edit, `.env` edit, broad
  refactor, unrelated Rust migration, or runtime/private data dump was done.
