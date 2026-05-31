# DIFF-182: Dev Runtime Smoke Manual Upload Verification

Status: Complete

## Type

Verification-first runtime smoke / manual upload verification

## Objective

Verify the current `dev` runtime path after DIFF-180 guided manual text upload
work and DIFF-181 governance reconciliation.

This DIFF is verification-first. It must not change code unless a specific bug
is found during verification and this DIFF is updated to explicitly scope the
smallest fix.

## Branch Policy

- Future IGY6 work happens on `dev`.
- Private/dev/build instruction files stay on `dev`.
- `main` remains the public/runtime-clean branch.
- Later, only necessary public/runtime-safe files should be selectively
  promoted to `main`.
- Do not merge `main` into `dev` unless explicitly instructed.
- Do not cherry-pick `main` into `dev` unless explicitly instructed.
- This DIFF must not remove any private/dev files.

## Baseline Facts

- Branch before work: `dev`.
- HEAD before work:
  `eb2a8d4 Complete DIFF-181 dev governance status reconciliation`.
- Working tree was clean before this DIFF.
- Private/dev files remained tracked on `dev`.
- Latest product/runtime work before governance reconciliation was DIFF-180
  guided manual text upload flow.

## Allowed Scope

If no bug is fixed:

- This DIFF file only.

If a directly scoped bug is found and the DIFF is updated before editing:

- This DIFF file.
- Only the smallest runtime/UI/API files necessary to fix the verified bug.

## Prohibited Scope

- No private/dev file removal.
- No removal of `.codex`.
- No removal of `AGENTS.md`.
- No removal of
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`.
- No removal of `docs/agents`.
- No removal of `docs/plans`.
- No `main` branch work.
- No merge.
- No cherry-pick.
- No Rust migration.
- No broad cleanup.
- No `.env` edits.
- No Docker Compose rewrite unless a verified Compose bug is explicitly scoped.
- No database migrations unless explicitly scoped after reporting.
- Do not print secrets.
- Do not print `.env` contents.
- Do not read or dump runtime/private data from `IGY6_DATA_ROOT`.

## Required Verification Areas

- Environment file expectations.
- `IGY6_DATA_ROOT` expectation.
- Docker Compose config validity.
- Web build or documented app startup path.
- API health/readiness if available.
- Manual guided text upload flow added by DIFF-180.
- Request-understanding/intent path if it is part of the current UI/API flow.
- Port conflict status before starting any stack.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `docs/diffs/DIFF-180-guided-manual-text-source-upload-flow.md`
- `scripts/run.sh`
- `scripts/lib/igy6-ops.sh`
- `scripts/e2e-manual-upload-smoke.py`
- `scripts/post-cutover-smoke.sh`
- `apps/web/src/app/api/`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- tracked private/dev file inventory from `git ls-files AGENTS.md .codex
  Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents
  docs/plans | sort`

## Commands Run

Pre-work:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -12
git diff --name-status
git diff --check
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/BRANCH_POLICY.md
find docs/diffs -maxdepth 1 -type f | sort | tail -50
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
```

Verification:

```bash
git status --short
git diff --check
docker compose -f infra/docker-compose.yml --env-file .env config --quiet
grep -q '^IGY6_DATA_ROOT=' .env && echo "IGY6_DATA_ROOT is set in .env" || echo "IGY6_DATA_ROOT is missing from .env"
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
npm --prefix apps/web run build
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
scripts/run.sh
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API live HTTP %{http_code}\n' http://127.0.0.1:8000/health/live
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'API ready HTTP %{http_code}\n' http://127.0.0.1:8000/health/ready
curl --silent --show-error --max-time 10 --output /dev/null --write-out 'Web UI HTTP %{http_code}\n' http://127.0.0.1:3000/
python3 scripts/e2e-manual-upload-smoke.py --check
python3 scripts/e2e-manual-upload-smoke.py --run
curl --silent --show-error --max-time 10 http://127.0.0.1:3000/ | grep -q 'data-guided-manual-upload' && echo 'Guided manual upload UI marker present' || echo 'Guided manual upload UI marker missing'
curl --silent --show-error --max-time 10 -H 'Content-Type: application/json' -d '{"message":"What is the status of this project?","context":{"surface":"DIFF-182 verification"}}' http://127.0.0.1:8000/agent/intent
```

Additional bounded status check:

```bash
python3 - <<'PY'
...
PY
```

The bounded Python check queried only the smoke work-item status and endpoint
record counts; it did not dump runtime/private content.

## Runtime Status

- `.env` exists and declares `IGY6_DATA_ROOT`; the value was not printed.
- `../IGY6_Data` exists; its contents were not read or dumped.
- Docker Compose config validated with `config --quiet`.
- `npm --prefix apps/web run build` passed.
- `scripts/run.sh` built and started the Docker Compose stack.
- API live endpoint returned HTTP 200.
- API ready endpoint returned HTTP 200.
- Web UI returned HTTP 200.
- The stack was stopped with Ctrl+C after verification. Docker Compose performed
  normal container shutdown; volumes were not removed.

## Port Conflict Result

- `ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true` returned no
  listeners before startup.
- No port conflict blocked startup.

## Manual Upload Test Result

- Non-mutating manual-upload smoke preflight passed:
  - API live returned HTTP 200.
  - API ready returned HTTP 200.
  - Web UI returned HTTP 200.
- Mutating local smoke run passed and created harmless local test records:
  - created a `manual_upload` source;
  - created a source permission;
  - created and approved an approval request;
  - created a manual upload collection run;
  - created a raw artifact;
  - created a normalization work item;
  - the work item reached `completed`;
  - documents endpoint record count was 1;
  - chunks endpoint record count was 1;
  - evidence items endpoint record count was 1.
- Initial retrieval preview returned no hits while worker processing was still
  pending; after the bounded wait, the worker had completed and downstream
  document/chunk/evidence records existed.
- The web UI rendered the guided manual upload marker
  `data-guided-manual-upload`.

## Bugs Found

- No product/runtime bug was found.
- A sandbox limitation was observed: the first non-mutating Python smoke helper
  run could not connect to localhost and reported `Operation not permitted`.
  Rerunning the same helper with approved localhost execution passed.
- A transient parallel curl recheck reported connection failures while a
  simultaneous API readiness probe returned HTTP 200; rerunning the UI and
  intent checks individually with approved localhost execution passed.

## Code Fix Made

- No code fix was made.

## Verification Summary

- Pre-work confirmed branch `dev`, clean working tree before this DIFF, and
  HEAD `eb2a8d4 Complete DIFF-181 dev governance status reconciliation`.
- Private/dev files remained tracked on `dev`.
- Compose config, environment-key presence, data-root directory presence, web
  build, stack startup, health/readiness, web UI availability, request intent,
  and guided manual text upload all verified successfully.
- No secrets or `.env` values were printed.
- Runtime/private data contents under `IGY6_DATA_ROOT` were not read or dumped.
- No private/dev files were removed.
- No code, Docker Compose, `.env`, migration, merge, cherry-pick, or Rust
  migration changes were made.

## Final Status

Complete.
