# DIFF-193 Operator Docker Permission Preflight and Guidance

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- This DIFF does not promote files, merge, cherry-pick, push, touch `main`,
  edit `.env`, remove files, run `sudo`, change user groups, restart Docker,
  kill processes, or dump runtime/private data.

## Purpose

Improve the operator smoke script so it detects Docker socket permission
problems early and gives clear, safe guidance before trying to start the stack.

## Baseline

- Branch before work: `dev`.
- HEAD before work: `c244c0c Complete DIFF-192 operator smoke script automation`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Working tree before work: clean.

## Allowed Scope

- `docs/diffs/DIFF-193-operator-docker-permission-preflight.md`
- `scripts/operator-smoke-check.sh`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`

## Prohibited Scope

- No user system changes.
- No `sudo`.
- No Docker restart.
- No user group changes.
- No process killing.
- No `.env` edits or secret printing.
- No runtime/private data dumps from `IGY6_DATA_ROOT`.
- No destructive cleanup.
- No file removal.
- No `main` work, merge, cherry-pick, push, or promotion.

## Files Inspected

- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/diffs/DIFF-192-operator-smoke-script-automation.md`
- `scripts/operator-smoke-check.sh`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- tracked private/dev/build instruction file list from
  `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`

## Docker Preflight Behavior Added

- Split ordinary command checks from Docker access checks.
- Added Docker access preflight before Compose config validation and before
  stack startup.
- Detects and reports:
  - missing `docker` command;
  - Docker daemon unavailable;
  - permission denied connecting to `/var/run/docker.sock`;
  - current user cannot run Docker commands.
- Prints safe manual checks when Docker access fails:
  - `id`;
  - `ls -l /var/run/docker.sock`;
  - `docker ps`.
- Mentions the likely permission fix only as manual operator guidance: add the
  user to the `docker` group, then restart the shell or WSL session.
- Does not run any Docker permission fix automatically.

## Script Modes Affected

- `--help`: still works without Docker and documents that the script does not
  change Docker permissions, user groups, or system services.
- `--check`: reports Docker readiness clearly and exits nonzero on Docker
  access failure before Compose config validation.
- `--run`: refuses early on Docker access failure before web build, Compose
  config validation, port startup checks, or stack start.

## Safety Behavior

- No stack starts if Docker preflight fails.
- Docker access failure exits nonzero.
- The script does not modify the user's system.
- The script does not run `sudo`, change groups, restart Docker, kill
  processes, edit `.env`, delete files, remove volumes, or dump runtime/private
  data.

## Documentation Updates

- Updated `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md` with Docker
  permission preflight behavior, safe manual checks, likely manual fix wording,
  and failure handling guidance.

## Verification

Commands run:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -12
git branch -vv
git diff --check
sed -n '1,280p' docs/diffs/DIFF-192-operator-smoke-script-automation.md
sed -n '1,360p' scripts/operator-smoke-check.sh
sed -n '361,720p' scripts/operator-smoke-check.sh
sed -n '1,360p' docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' docs/BRANCH_POLICY.md
sed -n '1,260p' README.md
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
bash -n scripts/operator-smoke-check.sh
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --check || true
```

Results:

- `git diff --check`: passed before source edits.
- `bash -n scripts/operator-smoke-check.sh`: passed.
- `scripts/operator-smoke-check.sh --help`: passed.
- `scripts/operator-smoke-check.sh --check || true`: reported Docker socket
  permission failure early:
  - required repo files present;
  - `curl`, `npm`, and `python3` available;
  - port inspection command available;
  - `.env` exists;
  - `IGY6_DATA_ROOT` key present without value printing;
  - `IGY6_DATA_ROOT` directory exists without content listing;
  - `docker` command available;
  - permission denied connecting to `/var/run/docker.sock`;
  - current user cannot run Docker commands;
  - manual checks printed: `id`, `ls -l /var/run/docker.sock`, `docker ps`;
  - likely manual fix guidance printed without applying it.

## Run Mode

`scripts/operator-smoke-check.sh --run` was skipped because `--check` reported
Docker access failure. This follows the DIFF requirement that no stack should
start if Docker preflight fails.

## Files Changed

- `docs/diffs/DIFF-193-operator-docker-permission-preflight.md`
- `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md`
- `scripts/operator-smoke-check.sh`

## Verification Summary

- Docker permission problems are now caught before Compose validation or stack
  startup.
- The current operator environment still cannot access `/var/run/docker.sock`;
  this is an operator environment issue, not an IGY6 runtime bug.
- Private/dev files remained tracked on `dev`.
- No runtime code, UI code, `.env`, runtime/private data, Docker volumes,
  databases, Qdrant, Neo4j, or local service data were modified by source
  edits.
- No files were removed.
- No `main` work, merge, cherry-pick, push, or promotion was performed.

## Final Status

DIFF-193 is complete.
