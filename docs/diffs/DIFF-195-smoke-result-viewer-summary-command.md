# DIFF-195 Smoke Result Viewer / Summary Command

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- This DIFF does not promote files, merge, cherry-pick, push, touch `main`,
  edit `.env`, remove files, run `sudo`, change user groups, kill processes,
  run destructive commands, print raw uploaded text, or dump runtime/private
  data.

## Purpose

Add a safe way to list and inspect recorded operator smoke results without
manually opening `.igy6-local/smoke-results/*.json`.

The viewer summarizes safe result metadata only and must not expose secrets,
raw runtime/private data, raw uploaded text, `.env` contents, or
`IGY6_DATA_ROOT` contents.

## Baseline

- Branch before work: `dev`.
- HEAD before work: `c0d89ba Complete DIFF-194 operator smoke result recording`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Working tree before work: clean.

## Allowed Scope

- `docs/diffs/DIFF-195-smoke-result-viewer-summary-command.md`
- `scripts/operator-smoke-check.sh`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`

## Prohibited Scope

- No runtime app behavior changes.
- No `.env` edits or secret printing.
- No runtime/private data dumps from `IGY6_DATA_ROOT`.
- No raw uploaded text output.
- No raw full smoke JSON output by default.
- No Docker socket credential, auth token, database row, or private export
  output.
- No destructive cleanup.
- No file removal.
- No `sudo`, user group changes, process killing, or system changes.
- No `main` work, merge, cherry-pick, push, or promotion.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/diffs/DIFF-194-operator-smoke-result-recording.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`
- targeted smoke/operator/result/summary/viewer/status/audit/runtime references
  from `find` and `grep`
- tracked private/dev/build instruction file list from
  `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`

## Implementation

- Extended `scripts/operator-smoke-check.sh` instead of creating a separate
  script.
- Decision: the existing operator script already owns recording, result path
  constants, help text, and `python3` usage for smoke JSON handling, so adding
  read-only viewer modes there is the smallest clean implementation.
- Added viewer interface:
  - `scripts/operator-smoke-check.sh --list-results`
  - `scripts/operator-smoke-check.sh --latest-result`
  - `scripts/operator-smoke-check.sh --show-result PATH`
- Result storage path used:
  - `.igy6-local/smoke-results/`
- `--list-results` lists matching `operator-smoke-*.json` result filenames in
  oldest-first order and does not print raw JSON.
- `--latest-result` summarizes the newest matching result file.
- `--show-result PATH` summarizes a specific result file only when it is under
  `.igy6-local/smoke-results/`, is a file, and matches
  `operator-smoke-*.json`.
- The viewer is read-only and does not modify records.
- The viewer uses embedded `python3` for robust JSON parsing. If `python3` is
  unavailable, it reports that clearly.

## Safe Fields Displayed

- result file path relative to the repo
- `created_at_utc`
- `repo_branch`
- `repo_head`
- `mode`
- `overall_status`
- step total/pass/fail/other counts
- API live/ready/retrieval HTTP status summaries
- web root HTTP status summary
- artifacts/documents/chunks/evidence/retrieval counts
- retrieval answer status
- `stack_started_by_script`
- `stack_stopped_by_script`
- `failure_reason` when present

## Fields Intentionally Not Displayed

- raw full JSON
- `.env` contents or values
- secrets, credentials, cookies, private keys, or auth tokens
- raw runtime/private data
- `IGY6_DATA_ROOT` path or contents
- raw uploaded text
- raw database rows
- Docker socket credentials
- full logs or command output that may contain sensitive data
- synthetic token hash
- arbitrary step messages

## Missing, Empty, Malformed, and Older Record Handling

- Missing result directory: `--list-results` and `--latest-result` print an
  informational message and exit successfully.
- Empty result directory: `--list-results` and `--latest-result` print an
  informational message and exit successfully.
- Malformed JSON: `--latest-result` or `--show-result` prints a clear failure
  and exits nonzero.
- Missing fields from older records: summary output prints `(missing)` for
  absent scalar fields and uses zero counts for missing step lists.
- Failed smoke records: summary output includes `overall_status`, step fail
  count, and `failure_reason` when present.
- Paths outside `.igy6-local/smoke-results/`: `--show-result` rejects them.

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
sed -n '1,380p' docs/diffs/DIFF-194-operator-smoke-result-recording.md
sed -n '1,520p' docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
sed -n '1,700p' scripts/operator-smoke-check.sh
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' README.md
find scripts docs . -maxdepth 4 -type f | sort | grep -E "smoke|operator|result|summary|viewer|status|audit|verification|last-known|runtime" || true
grep -R "operator-smoke\|smoke result\|last known good\|verification result\|smoke-results\|--record\|--run-record" scripts docs -n 2>/dev/null || true
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
bash -n scripts/operator-smoke-check.sh
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --list-results || true
scripts/operator-smoke-check.sh --latest-result || true
latest_result="$(find .igy6-local/smoke-results -maxdepth 1 -type f -name 'operator-smoke-*.json' 2>/dev/null | sort | tail -1)"
if [ -n "$latest_result" ]; then
python3 -m json.tool "$latest_result" >/dev/null
scripts/operator-smoke-check.sh --show-result "$latest_result"
fi
scripts/operator-smoke-check.sh --show-result /tmp/not-a-smoke-result.json || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

Results so far:

- Initial `git status --short`: clean.
- Initial branch: `dev`.
- Initial HEAD: `c0d89ba Complete DIFF-194 operator smoke result recording`.
- Initial `git branch -vv`: `dev` synced with `origin/dev`, no ahead/behind
  marker.
- Initial `git diff --name-status`: no output.
- Initial `git diff --check`: passed.
- `bash -n scripts/operator-smoke-check.sh`: passed after viewer edit.
- `scripts/operator-smoke-check.sh --help`: passed and shows the viewer modes.
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
- `scripts/operator-smoke-check.sh --list-results || true`: passed and listed
  `operator-smoke-20260604T225034Z.json` oldest first.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and printed
  a safe summary for the existing recorded result.
- Specific result validation passed:
  - `python3 -m json.tool "$latest_result" >/dev/null`
  - `scripts/operator-smoke-check.sh --show-result "$latest_result"`
- `--show-result` summary for the existing result displayed only safe fields:
  - file path
  - timestamp
  - branch and HEAD
  - mode and overall status
  - step counts
  - API/web status summaries
  - artifact/document/chunk/evidence/retrieval counts
  - retrieval answer status
  - stack started/stopped booleans
  - failure reason
- Path rejection check passed:
  `scripts/operator-smoke-check.sh --show-result /tmp/not-a-smoke-result.json || true`
  printed `FAIL result path must be under .igy6-local/smoke-results`.
- Private/dev files remained tracked on `dev` before work.
- Stale status scan before work still reported older out-of-scope draft strings
  in DIFF-177, DIFF-180, `DIFF_TEMPLATE.md`, and command transcripts in
  completed DIFF records.
- Stale status scan after verification still reported those older out-of-scope
  draft/template/transcript strings plus DIFF-195 while it was active. After
  this update, DIFF-195 is complete and no longer has active status.
- The broad `find` command reported permission denied for
  `./IGY6_Data/postgres`; that runtime/private data directory was not inspected.

## Files Changed

- `docs/diffs/DIFF-195-smoke-result-viewer-summary-command.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`

## Final Status

DIFF-195 is complete.

## Verification Summary

- The smoke result viewer is integrated into `scripts/operator-smoke-check.sh`
  as read-only modes.
- The implemented interface is:
  - `--list-results`
  - `--latest-result`
  - `--show-result PATH`
- Result records remain under `.igy6-local/smoke-results/`.
- Viewer output is a safe summary and does not print raw JSON.
- Missing result directories and empty result directories are handled with an
  informational message.
- Malformed JSON and invalid paths are handled with clear failures.
- Missing scalar fields display as `(missing)`.
- Existing `--help` and `--check` modes still work.
- Private/dev/build instruction files remained tracked on `dev`.
- No runtime app code, UI code, `.env`, Docker volumes, databases, Qdrant,
  Neo4j, local service data, or `IGY6_DATA_ROOT` contents were edited or
  dumped by source changes.
- No files were removed.
- No full smoke `--run` was needed for this DIFF because an existing ignored
  result file from DIFF-194 was present locally.
- No `main` work, merge, cherry-pick, push, or promotion was performed.
- No `sudo`, user group change, process killing, or destructive command was
  performed.
