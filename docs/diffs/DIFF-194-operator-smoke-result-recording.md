# DIFF-194 Operator Smoke Result Recording

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- This DIFF does not promote files, merge, cherry-pick, push, touch `main`,
  edit `.env`, remove files, run `sudo`, change user groups, kill processes,
  run destructive commands, or dump runtime/private data.

## Purpose

Add a safe, repeatable way to record an operator smoke result summary as a local
development "last known good" marker without dumping secrets, `.env` contents,
raw runtime/private data, raw uploaded text, or `IGY6_DATA_ROOT` contents.

## Baseline

- Branch before work: `dev`.
- HEAD before work: `0d00dfc Complete DIFF-193 operator Docker permission preflight`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Working tree before work: clean.

## Allowed Scope

- `docs/diffs/DIFF-194-operator-smoke-result-recording.md`
- `scripts/operator-smoke-check.sh`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `.gitignore` only to ignore `.igy6-local/smoke-results/`

## Prohibited Scope

- No runtime app behavior changes.
- No `.env` edits or secret printing.
- No runtime/private data dumps from `IGY6_DATA_ROOT`.
- No raw uploaded text recording.
- No full log recording.
- No Docker socket credential, auth token, database row, or private export
  recording.
- No destructive cleanup.
- No file removal.
- No `sudo`, user group changes, process killing, or system changes.
- No `main` work, merge, cherry-pick, push, or promotion.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `.gitignore`
- `docs/diffs/DIFF-192-operator-smoke-script-automation.md`
- `docs/diffs/DIFF-193-operator-docker-permission-preflight.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`
- `scripts/e2e-manual-upload-smoke.py`
- targeted smoke/operator/result/status/audit/runtime references from `find`
  and `grep`
- tracked private/dev/build instruction file list from
  `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`

## Implementation

- Added optional result recording to `scripts/operator-smoke-check.sh`.
- Implemented interface:
  - `scripts/operator-smoke-check.sh --run --record`
  - `scripts/operator-smoke-check.sh --run-record`
- Kept existing modes backward compatible:
  - `--help`
  - `--check`
  - `--run`
- Plain `--run` remains non-recording.
- Docker preflight still happens before Compose validation and before stack
  startup.
- No stack starts if Docker preflight fails.
- Recording writes a JSON summary under:
  - `.igy6-local/smoke-results/operator-smoke-YYYYMMDDTHHMMSSZ.json`
- `.gitignore` ignores `.igy6-local/smoke-results/`.

## Safe Fields Recorded

- `schema_version`
- `created_at_utc`
- `repo_branch`
- `repo_head`
- `smoke_script`
- `mode`
- `overall_status`
- `steps` with `name`, `status`, and `message`
- synthetic token marker plus SHA-256 hash, not raw private data
- API live/ready/retrieval HTTP status summaries
- web root HTTP status summary
- artifacts/documents/chunks/evidence/retrieval counts when available
- retrieval answer status when available
- checked ports
- `igy6_data_root_present` boolean only
- `stack_started_by_script`
- `stack_stopped_by_script`
- `failure_reason` when present

## Fields Intentionally Not Recorded

- `.env` contents or values
- secrets, credentials, cookies, private keys, or auth tokens
- raw runtime/private data
- `IGY6_DATA_ROOT` path or contents
- raw uploaded text
- raw database rows
- Docker socket credentials
- full logs or command output that may contain sensitive data

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -20
git branch -vv
git diff --name-status
git diff --check
sed -n '1,300p' AGENTS.md
sed -n '1,280p' docs/BRANCH_POLICY.md
sed -n '1,360p' docs/diffs/DIFF-192-operator-smoke-script-automation.md
sed -n '1,360p' docs/diffs/DIFF-193-operator-docker-permission-preflight.md
sed -n '1,420p' docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
sed -n '1,520p' scripts/operator-smoke-check.sh
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' README.md
find scripts docs . -maxdepth 4 -type f | sort | grep -E "smoke|operator|result|status|audit|verification|last-known|runtime" || true
grep -R "operator-smoke\|smoke result\|last known good\|verification result\|IGY6_DATA_ROOT\|runtime data" scripts docs -n 2>/dev/null || true
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
sed -n '1,240p' .gitignore
sed -n '1,360p' scripts/e2e-manual-upload-smoke.py
sed -n '360,760p' scripts/e2e-manual-upload-smoke.py
bash -n scripts/operator-smoke-check.sh
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run
scripts/operator-smoke-check.sh --run --record
find .igy6-local/smoke-results -maxdepth 1 -type f | sort | tail -5
python3 -m json.tool "$(find .igy6-local/smoke-results -maxdepth 1 -type f | sort | tail -1)" >/dev/null
git status --short
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

Results so far:

- Initial `git status --short`: clean.
- Initial branch: `dev`.
- Initial HEAD: `0d00dfc Complete DIFF-193 operator Docker permission preflight`.
- Initial `git branch -vv`: `dev` synced with `origin/dev`, no ahead/behind
  marker.
- Initial `git diff --name-status`: no output.
- Initial `git diff --check`: passed.
- `bash -n scripts/operator-smoke-check.sh`: passed after script edit.
- `scripts/operator-smoke-check.sh --help`: passed.
- `scripts/operator-smoke-check.sh --check` in the sandbox: failed early on
  Docker socket permission before Compose validation or stack startup. This did
  not start the stack or mutate runtime data.
- `scripts/operator-smoke-check.sh --check` with escalated Docker access:
  passed.
  - Required repo files present.
  - `curl`, `npm`, and `python3` available.
  - Port inspection command available.
  - `.env` exists.
  - `IGY6_DATA_ROOT` key present without value printing.
  - `IGY6_DATA_ROOT` directory exists without content listing.
  - Docker daemon accessible.
  - Docker Compose plugin available.
  - Compose config valid.
  - Ports `3000`, `8000`, and `8765` clear.
- `scripts/operator-smoke-check.sh --run` with escalated Docker access: passed.
  - Web build passed.
  - Stack started by `scripts/run.sh`.
  - API live, API ready, and web UI returned HTTP 200.
  - Synthetic manual upload smoke passed.
  - Work item creation and endpoint count checks passed.
  - Retrieval preview returned 5 hits.
  - UI markers were present.
  - Stack stopped through `scripts/stop.sh`.
  - Ports `3000`, `8000`, and `8765` were clear after stop.
- `scripts/operator-smoke-check.sh --run --record` with escalated Docker
  access: passed.
  - Web build passed.
  - Stack started by `scripts/run.sh`.
  - API live, API ready, and web UI returned HTTP 200.
  - Synthetic manual upload smoke passed.
  - Count summary from the recorded run:
    - artifacts: 9
    - documents: 7
    - chunks: 7
    - evidence items: 7
    - retrieval items: 5
  - Stack stopped through `scripts/stop.sh`.
  - Ports `3000`, `8000`, and `8765` were clear after stop.
  - Result summary written to
    `.igy6-local/smoke-results/operator-smoke-20260604T225034Z.json`.
- JSON validation passed with:
  `python3 -m json.tool "$(find .igy6-local/smoke-results -maxdepth 1 -type f | sort | tail -1)" >/dev/null`.
- Result JSON summary inspection showed:
  - `overall_status`: `passed`
  - `mode`: `run-record`
  - API live/ready/retrieval status values: `200`
  - web root status: `200`
  - `stack_started_by_script`: `true`
  - `stack_stopped_by_script`: `true`
- Final `git status --short` after recording did not show `.igy6-local/` or
  smoke result JSON files.
- Final port check after stack runs produced no output, meaning checked ports
  were clear.
- Private/dev files remained tracked on `dev` before work.
- Stale status scan before work still reported older out-of-scope draft strings
  in DIFF-177, DIFF-180, `DIFF_TEMPLATE.md`, and command transcripts in
  completed DIFF records.
- Stale status scan after verification still reported those older out-of-scope
  draft/template/transcript strings. DIFF-194 is complete and no longer reports
  as active after this update.
- The broad `find` command reported permission denied for
  `./IGY6_Data/postgres`; that runtime/private data directory was not inspected.

## Files Changed

- `.gitignore`
- `docs/diffs/DIFF-194-operator-smoke-result-recording.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`

## Final Status

DIFF-194 is complete.

## Verification Summary

- Optional result recording was added without making recording mandatory for
  plain `--run`.
- The preferred interface is `scripts/operator-smoke-check.sh --run --record`.
- `scripts/operator-smoke-check.sh --run-record` is also accepted as a shorthand.
- Result records are stored under the gitignored local path
  `.igy6-local/smoke-results/`.
- The result file format is JSON and machine-readable.
- Safe summary fields are recorded, and prohibited secret/private/raw fields are
  intentionally not recorded.
- Non-recording `--run` passed end to end.
- Recording `--run --record` passed end to end and produced valid JSON.
- Private/dev/build instruction files remained tracked on `dev`.
- No runtime app code, UI code, `.env`, Docker volumes, databases, Qdrant,
  Neo4j, local service data, or `IGY6_DATA_ROOT` contents were edited or
  dumped by source changes.
- No files were removed.
- No `main` work, merge, cherry-pick, push, or promotion was performed.
- No `sudo`, user group change, process killing, or destructive command was
  performed.
