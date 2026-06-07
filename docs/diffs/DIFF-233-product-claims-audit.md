# DIFF-233 - Product Claims Audit

Status: Complete

## Scope

DIFF-233 audits README, UI copy, docs/ui, runtime docs, and the active build
plan for claims that could exceed implemented and verified behavior before MVP.
It keeps future roadmap goals intact while distinguishing current support,
metadata/review surfaces, dry-run validation, and unsupported states.

## Claims Inspected

Searched for overclaim-risk language around:

- complete;
- autonomous;
- self-improvement;
- forecasting;
- graph reasoning;
- local LLM default behavior;
- all source types;
- browser/account/connector import;
- backup/restore/delete;
- production-ready language;
- hosted AI;
- PDF/binary/media support.

## Product Copy Changes Made

- Changed evidence-answer/local-LLM help text from deterministic "backup"
  answers to deterministic fallback answers.
- Changed graph memory warning from "not full autonomous graph reasoning yet"
  to lineage/relationship support, not advanced graph reasoning.
- Updated lifecycle audit UI labels to distinguish:
  - metadata export MVP;
  - raw artifacts excluded from the MVP;
  - restore dry-run validation only;
  - full backup archives still future work;
  - destructive delete still future explicit DIFF work.
- Updated the UI guide to state that metadata-only export and restore dry-run
  validation exist, while full service backups and destructive restore do not.
- Clarified the active build plan opening so it reads as a completion target,
  not a current-completeness claim.
- Tightened current limitations copy around graph, prediction/recommendation,
  and improvement workflows so it does not claim advanced graph reasoning,
  forecasting engines, or autonomous self-improvement.

## Accurate Claims Preserved

- Local-first default posture.
- Rust API gateway, Rust worker daemon, and Next.js web UI active runtime.
- UTF-8 text-oriented strongest path.
- Browser/account/connector imports unsupported in current manual flows.
- Binary PDF/image/audio/video parsing unsupported unless a later scoped DIFF
  adds and verifies it.
- Hosted AI calls not used by default.
- Owner WSL live smoke still required for full runtime verification.

## Explicit Non-Claims

- No runtime behavior changed.
- No new API route, schema, persistence, worker behavior, or product control was
  added.
- No future roadmap capability was removed.
- No unsupported source, backup, restore, delete, graph reasoning, forecasting,
  or self-improvement capability is claimed as complete.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-233-product-claims-audit.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.

Full Docker smoke was not run from Codex because the Codex local environment
strips Docker group access and remaps `/var/run/docker.sock` to
`nobody:nogroup`.

## Classification

UI copy plus docs. No new API route, schema, persistence, worker behavior, or
live-stack verification.

## Scope Confirmation

- No hosted AI call was added.
- No browser/account scraping or connector import was added.
- No external service call was added.
- No arbitrary command execution from user text was added.
- No `.env` edit was performed.
- No runtime/private data was dumped.
- No destructive delete or destructive restore was performed.
- No unsafe backup archive was created.
- No main-branch work, merge, cherry-pick, promotion, push, or private/dev file
  removal was performed.
