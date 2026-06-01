# DIFF-186 Work Status And Recovery UX Polish

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- Public/runtime-safe changes can be promoted to `main` later by explicit instruction.
- This DIFF does not merge, cherry-pick, remove private/dev files, or start Rust migration work.

## Purpose

Improve the user-facing work status and recovery experience so users can
understand what IGY6 is doing after creating or processing work items,
especially after guided manual upload.

## Scope

Allowed:

- This DIFF record.
- Smallest necessary `apps/web` files for work status/recovery UX.
- Smallest necessary Next API proxy files if required.
- Smallest necessary Rust gateway/API files only if additive response fields or tests are required.
- Tests/scripts directly related to work status/recovery verification if needed.

Not allowed:

- Removing anything from `dev`.
- Removing `.codex`, `AGENTS.md`, `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, `docs/agents`, or `docs/plans`.
- Merging `main` into `dev`.
- Cherry-picking `main` into `dev`.
- Runtime refactors, Docker Compose rewrites, `.env` edits, Rust migration work, unrelated retrieval/report/export work, or fake recovery actions.

## Current Branch And HEAD Before Work

- Branch: `dev`
- HEAD before work: `7b7871c Complete DIFF-185 evidence answer review UX`
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker in `git branch -vv`

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `README.md`
- `docs/ui/README.md`
- `configs/rust-cutover-manifest.json`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-worker/src/lib.rs`
- `crates/igy6-work-queue-reports/src/lib.rs`
- `scripts/e2e-manual-upload-smoke.py`
- DIFF inventory under `docs/diffs`
- Tracked private/dev inventory under `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/agents`, and `docs/plans`

## Current UI/API Behavior Before Changes

- The Add Data guided manual upload flow posts directly from the browser to the
  Rust gateway `/collection-runs/manual-upload` route.
- The Rust manual upload route creates a completed collection run, raw artifact,
  and queued `collection_normalization` work item. The collection run
  `summary_json` includes `normalization_work_item_id` and `raw_artifact_ids`.
- The Work tab reads `/work-items` directly from the Rust gateway through the
  shared server-side `getJson` helper. There is no Next work-items proxy.
- The Rust `/work-items` list/detail routes already return `id`, `work_type`,
  `status`, `requested_by_actor_id`, `payload_json`, `error_message`,
  `created_at`, and `updated_at`.
- The existing Work tab showed counts plus a compact work item row with work
  type, status, requester or error, and created time.
- The existing Work tab did not clearly show work item IDs, related
  collection/source/artifact/document/chunk identifiers, updated time, or
  status-specific next guidance.
- The existing guided upload success state told users to open Work and Results
  but did not visibly surface the created collection run, work item, source, or
  artifact identifiers.
- No safe retry/recover route was identified. Existing dispatch/status routes
  are low-level Advanced controls, not automatic retry recovery.

## UX Changes Made

- Updated Add Data guided manual upload success feedback to visibly show:
  - source id;
  - collection run id;
  - normalization work item id;
  - work type;
  - raw artifact id list;
  - expected status progression.
- Updated the Work tab to show up to eight recent work items instead of four so
  the result of a recent manual upload is less likely to be hidden.
- Updated each Work tab item to show:
  - work item id;
  - work type;
  - current status;
  - created time;
  - updated time where returned by the API;
  - related collection, source, permission, artifact, document, chunk, and
    parent work identifiers where present in `payload_json`;
  - status-specific guidance for queued, pending intent verification, running,
    completed, failed, canceled, and unknown states.
- Added DOM markers for verification:
  - `data-guided-manual-work-status`
  - `data-work-status-item`
- Failed-state guidance is honest: it tells the user to read the error and
  verify source, permission, and uploaded UTF-8 text. It does not expose a fake
  retry/recover button.

## API/Contract Changes

- None.
- The existing Rust `/work-items` list/detail routes already return
  `payload_json` and `updated_at`.
- The existing Rust `/collection-runs/manual-upload` response already returns
  `summary_json.normalization_work_item_id` and `summary_json.raw_artifact_ids`.
- No Next proxy, Rust gateway, database, or worker contract changes were made.

## Test Token Strategy

Runtime verification will use synthetic text and a unique token only. No `.env`
contents or runtime/private data from `IGY6_DATA_ROOT` will be printed or
dumped.

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -16
git branch -vv
git diff --name-status
git diff --check
sed -n '1,240p' AGENTS.md
sed -n '1,240p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,220p' docs/BRANCH_POLICY.md
sed -n '1,280p' docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
sed -n '1,280p' docs/diffs/DIFF-185-evidence-answer-review-ux.md
sed -n '1,220p' README.md
sed -n '1,260p' docs/ui/README.md
sed -n '1,240p' configs/rust-cutover-manifest.json
find docs/diffs -maxdepth 1 -type f | sort | tail -90
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
find apps/web crates services scripts docs -maxdepth 5 -type f | sort | grep -E "work|status|queue|recovery|retry|run-last|health|ready|manual|upload|source|approval|collection|artifact|normalize|normalization|task|job|result|evidence" || true
grep -R "work item\|work-items\|status\|pending\|running\|completed\|failed\|retry\|recovery\|run-last\|manual upload\|normalization\|queue" apps/web crates services scripts docs -n 2>/dev/null | head -360 || true
rg -n "Work|workItems|work-items|Guided|manual|upload|collectionRuns|selectedTab|setActiveTab|activeTab|work item|status" apps/web/src/app/page.tsx
find apps/web/src/app/api -maxdepth 5 -type f | sort
rg -n "work-items|collection-runs|manual-upload|WorkItem|work_item|retry|failed|queued|running|completed" crates/igy6-gateway/src/lib.rs crates/igy6-worker/src/lib.rs crates/igy6-work-queue-reports/src/lib.rs apps/web/src/app/api apps/web/src/app/page.tsx scripts
sed -n '120,180p' apps/web/src/app/page.tsx
sed -n '1510,1700p' apps/web/src/app/page.tsx
sed -n '2020,2125p' apps/web/src/app/page.tsx
sed -n '2630,2678p' apps/web/src/app/page.tsx
sed -n '1320,1488p' crates/igy6-gateway/src/lib.rs
sed -n '520,650p' crates/igy6-gateway/src/lib.rs
sed -n '8440,8508p' crates/igy6-gateway/src/lib.rs
sed -n '3486,3528p' crates/igy6-worker/src/lib.rs
```

Build and runtime verification:

```bash
npm --prefix apps/web run build
docker compose -f infra/docker-compose.yml --env-file .env config --quiet
grep -q '^IGY6_DATA_ROOT=' .env && echo "IGY6_DATA_ROOT key present" || echo "IGY6_DATA_ROOT key missing"
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
scripts/run.sh
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API live HTTP %{http_code}\n' http://127.0.0.1:8000/health/live
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API ready HTTP %{http_code}\n' http://127.0.0.1:8000/health/ready
curl --silent --show-error --max-time 20 --output /tmp/igy6-diff186-page.html --write-out 'Web UI HTTP %{http_code}\n' http://127.0.0.1:3000/
grep -E 'data-work-status-item|data-guided-manual-work-status|No automatic retry action is exposed here|Work item:' /tmp/igy6-diff186-page.html | head -20 || true
python3 /tmp/igy6_diff186_flow.py
curl --silent --show-error --max-time 20 --output /tmp/igy6-diff186-current.html --write-out 'Web UI HTTP %{http_code}\n' http://127.0.0.1:3000/
grep -q 'Processing completed successfully.' /tmp/igy6-diff186-current.html && echo 'completed guidance visible in current page' || echo 'completed guidance not visible in current page'
grep -q 'data-guided-manual-work-status' /tmp/igy6-diff186-current.html && echo 'guided upload work-status marker present' || echo 'guided upload work-status marker missing'
grep -q 'No automatic retry action is exposed here.' /tmp/igy6-diff186-current.html && echo 'failed guidance marker present in rendered payload' || echo 'failed guidance marker not present without a recent failed item'
grep -q 'Waiting for background processing.' /tmp/igy6-diff186-current.html && echo 'queued guidance visible in current page' || echo 'queued guidance not visible after work completed'
grep -n 'Waiting for background processing.\|Processing is in progress.\|No automatic retry action is exposed here.' apps/web/src/app/page.tsx
grep -n 'data-guided-manual-work-status\|data-work-status-item' apps/web/src/app/page.tsx
scripts/stop.sh
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
```

Verification results:

- `npm --prefix apps/web run build` passed.
- Docker Compose config validated with `.env` through `config --quiet`.
- `IGY6_DATA_ROOT` key was present in `.env`; the configured adjacent data root
  directory exists. `.env` contents were not printed.
- No listeners were present on ports 3000, 8000, or 8765 before stack start.
- `scripts/run.sh` built and started the local stack.
- API live probe returned HTTP 200.
- API ready probe returned HTTP 200.
- Web UI probe returned HTTP 200.
- The first Python localhost verification attempt failed under sandbox socket
  restrictions with `Operation not permitted`; the same helper passed with
  approved localhost access.
- Synthetic upload verification used token:
  `diff186-work-status-token-1780310379`.
- Synthetic source creation returned a source id:
  `source-18b4eedba7a621f0`.
- Manual upload returned collection run:
  `collection-18b4eedba8fe862a`.
- Manual upload returned normalization work item:
  `work-18b4eedba8fe8b15`.
- Manual upload returned raw artifact:
  `artifact-18b4eedba8fe8a09`.
- The work item initial status observed by the verification helper was
  `queued`.
- The work item final status was `completed`.
- `/work-items` listed the work item.
- The current web Work page included the work item and the completed guidance.
- The server-rendered page included the guided-upload work-status marker.
- Documents, chunks, and evidence item APIs each had one synthetic-token match.
- Rust `/chat/retrieval-preview` returned `answer_status: retrieved` and a
  synthetic-token match.
- Next `/api/chat/retrieval-preview` returned `answer_status: retrieved` and a
  synthetic-token match.
- Failed guidance was verified in source because there was no recent failed
  work item in the rendered page during runtime verification.
- Running guidance was verified in source because the synthetic work completed
  too quickly to capture a running rendered state.
- After `scripts/stop.sh`, ports 3000, 8000, and 8765 had no listeners.
- Browser automation was not available; UI verification used build-level,
  curl, source marker, and rendered HTML checks.
- `cargo fmt --all --check` and cargo tests were not run because no Rust files
  changed.

## Bugs Found

- No backend work status bug was found.
- No retry/recover API suitable for a normal-user Work tab action was found.
- A transient current-page curl check initially failed to connect to port 3000
  immediately after other live checks; a direct retry returned HTTP 200. The
  stack session logs showed the web server was serving requests, so this was
  recorded as a transient local probe artifact, not a product bug.

## Files Changed

- `apps/web/src/app/globals.css`
- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`

## Final Status

Complete.
