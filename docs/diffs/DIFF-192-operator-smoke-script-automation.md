# DIFF-192 Operator Smoke Script Automation

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- This DIFF does not promote files, merge, cherry-pick, push, touch `main`,
  edit `.env`, remove files, or dump runtime/private data.

## Purpose

Convert the DIFF-190 operator smoke verification checklist into a safe,
repeatable operator script that can verify the proven local manual-upload
evidence path without reconstructing long command sequences.

## Baseline

- Branch before work: `dev`.
- HEAD before work: `837b298 Complete DIFF-191 promotion candidate audit`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Working tree before work: clean.

## Allowed Scope

- Add `scripts/operator-smoke-check.sh`.
- Update `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md` to reference the
  new script.
- Complete this DIFF record.

## Prohibited Scope

- No runtime app behavior changes.
- No `.env` edits or secret printing.
- No runtime/private data dumps from `IGY6_DATA_ROOT`.
- No destructive cleanup.
- No file removal.
- No `main` work, merge, cherry-pick, push, or promotion.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/diffs/DIFF-190-operator-smoke-verification-bundle.md`
- `docs/diffs/DIFF-191-promotion-candidate-audit.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/e2e-manual-upload-smoke.py`
- `scripts/run.sh`
- `scripts/stop.sh`
- `scripts/runtime-smoke.sh`
- `scripts/post-cutover-smoke.sh`
- `scripts/lib/igy6-ops.sh`
- `apps/web/scripts/ui-smoke.mjs`
- targeted marker and route references in `apps/web/src/app/page.tsx`
- targeted route references in `crates/igy6-gateway/src/lib.rs`

## Implementation Notes

- Added `scripts/operator-smoke-check.sh` as the smallest repo-root operator
  wrapper matching the DIFF-190 checklist.
- Kept the naming under `scripts/` because existing runtime/operator checks
  already live there, including `scripts/runtime-smoke.sh`,
  `scripts/post-cutover-smoke.sh`, and
  `scripts/e2e-manual-upload-smoke.py`.
- Implemented modes:
  - `--help`: prints usage and safety notes.
  - `--check`: verifies prerequisites/configuration without starting the stack
    or mutating runtime data.
  - `--run`: runs the full local smoke path using synthetic data.
- `--run` reuses:
  - `scripts/run.sh` for stack startup;
  - `scripts/stop.sh` for shutdown;
  - `scripts/e2e-manual-upload-smoke.py --run` for the synthetic manual upload
    and evidence/retrieval checks.
- Updated `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md` to point
  operators at the new script and explain `--check`, `--run`, success, failure,
  no-secret behavior, and synthetic-data behavior.

## Safety Behavior

- Uses `set -Eeuo pipefail`.
- Refuses to proceed when required repo files are missing.
- Checks required commands before use.
- Validates Docker Compose config without printing `.env`.
- Checks `IGY6_DATA_ROOT` key presence without printing the value.
- Checks the `IGY6_DATA_ROOT` directory exists without listing or dumping
  contents.
- Checks ports `3000`, `8000`, and `8765` before startup.
- Does not kill existing processes.
- Uses synthetic manual-upload smoke data only.
- Does not edit `.env`, remove files, remove volumes, or delete runtime data.
- Stops the stack on exit if the script started it.
- Prints `PASS`/`FAIL` step summaries and exits nonzero on failure.
- Detects early `scripts/run.sh` exit before waiting through every health probe.

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -20
git branch -vv
git diff --name-status
git diff --check
sed -n '1,280p' AGENTS.md
sed -n '1,260p' docs/BRANCH_POLICY.md
sed -n '1,360p' docs/diffs/DIFF-190-operator-smoke-verification-bundle.md
sed -n '1,360p' docs/diffs/DIFF-191-promotion-candidate-audit.md
sed -n '1,360p' docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' README.md
sed -n '1,320p' scripts/e2e-manual-upload-smoke.py 2>/dev/null || true
find scripts docs apps/web crates services -maxdepth 5 -type f | sort | grep -E "smoke|e2e|manual|upload|retrieval|work|status|report|feedback|outcome|source|evidence|verify|check|operator" || true
grep -R "manual upload\|retrieval-preview\|work item\|smoke\|e2e\|verification\|operator\|IGY6_DATA_ROOT\|feedback\|outcome\|report\|source history" scripts docs apps/web crates services -n 2>/dev/null | head -600 || true
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
sed -n '1,260p' scripts/run.sh
sed -n '1,220p' scripts/stop.sh
sed -n '1,280p' scripts/runtime-smoke.sh
sed -n '1,300p' scripts/post-cutover-smoke.sh
sed -n '1,220p' scripts/lib/igy6-ops.sh
sed -n '1,260p' apps/web/scripts/ui-smoke.mjs
rg -n "data-guided-manual-result|data-work-status-item|data-chat-preview-results|data-basic-report-workflow|data-evidence-feedback-workflow|data-source-evidence-history|data-retrieval-preview" apps/web/src/app/page.tsx crates/igy6-gateway/src/lib.rs docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md scripts/e2e-manual-upload-smoke.py
rg -n "POST /reports|/reports|feedback|outcomes|source history|source-evidence" crates/igy6-gateway/src/lib.rs apps/web/src/app/page.tsx scripts docs/diffs/DIFF-187-basic-report-workflow-ux.md docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
git rev-parse --short HEAD
git status --short --branch
bash -n scripts/operator-smoke-check.sh
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --check
npm --prefix apps/web run build
scripts/operator-smoke-check.sh --run
tail -30 /tmp/igy6-operator-smoke-run.log
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
git diff --check
git diff --name-status
git status --short
```

Results:

- `bash -n scripts/operator-smoke-check.sh`: passed.
- `scripts/operator-smoke-check.sh --help`: passed.
- `scripts/operator-smoke-check.sh --check`: passed.
  - Required repo files present.
  - `docker`, `curl`, `npm`, and `python3` available.
  - Port inspection command available.
  - Docker Compose plugin available.
  - `.env` exists.
  - `IGY6_DATA_ROOT` key present without value printing.
  - `IGY6_DATA_ROOT` directory exists without content listing.
  - Docker Compose config valid.
  - Ports `3000`, `8000`, and `8765` clear.
- `npm --prefix apps/web run build`: passed.
- `scripts/operator-smoke-check.sh --run`: blocked by Docker socket
  permissions.
  - The script completed prerequisite checks and web build.
  - Ports `3000`, `8000`, and `8765` were clear before startup.
  - Startup used `scripts/run.sh`.
  - `scripts/run.sh` exited before API live became ready.
  - `/tmp/igy6-operator-smoke-run.log` reported:
    `unable to get image 'redis:7': permission denied while trying to connect
    to the docker API at unix:///var/run/docker.sock`.
  - Cleanup attempted `scripts/stop.sh`, but Docker socket access was also
    denied for `docker compose down`.
  - Final checked ports were clear.
- `git diff --check`: passed.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older locked/out-of-scope `Status: Draft`
  strings in DIFF-177 and DIFF-180 plus command transcripts in completed DIFF
  records; after this update DIFF-192 no longer has `Status: Active`.

## Files Changed

- `docs/diffs/DIFF-192-operator-smoke-script-automation.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`

## Verification Summary

- The non-mutating `--check` path passed.
- The web build passed.
- The full `--run` path is implemented but could not complete in this
  environment because Docker socket access was denied before stack startup.
- No runtime code, UI code, `.env`, runtime/private data, Docker volumes,
  databases, Qdrant, Neo4j, or local service data were modified by source
  edits.
- No files were removed.
- No `main` work, merge, cherry-pick, push, or promotion was performed.
- This script/documentation change is public/runtime-safe in content, but no
  promotion to `main` was performed and any promotion requires explicit owner
  instruction.

## Final Status

DIFF-192 is complete.
