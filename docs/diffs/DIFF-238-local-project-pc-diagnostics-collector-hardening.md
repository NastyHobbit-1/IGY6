# DIFF-238 - Local Project And PC Diagnostics Collector Hardening

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `e22037c Complete DIFF-237 pdf image audio video import MVP`
- `dev` tracking state before work: ahead of `origin/dev` by 3 commits
- Working tree before work: clean

## Purpose

Harden the local project collection posture and add an authorized PC diagnostics
import preview that stays explicit, bounded, dry-run oriented, and
non-destructive.

## Files Inspected

- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`
- `docs/diffs/DIFF-236-browser-web-router-collector-mvp.md`
- `docs/diffs/DIFF-237-pdf-image-audio-video-import-mvp.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- Existing local_project/source, permission, dry-run, artifact, document,
  chunk, evidence, and diagnostics-related UI/docs inspected during the batch
  pre-work

## Implementation

- Added a Local Project And PC Diagnostics Hardening panel in Add Data.
- Added dry-run preview modes:
  - local project manifest
  - PC diagnostics export
- Required explicit user-provided scope or selected path label.
- Added include/exclude posture fields and bounded preview caps for file count
  and bytes.
- Added pasted manifest/diagnostics text preview that reports length only and
  does not echo the content.
- Redacted path-like scope output in preview results.
- Added secret-signal warnings for `.env`, SSH/private key, credential, token,
  cookie, API key, and similar terms.
- Updated the UI guide with the hardening behavior and limits.

## Scope Confirmation

This is a UI-only product hardening DIFF. It does not implement automated
filesystem collection or live diagnostics probing.

No arbitrary filesystem crawling, path traversal, file reads, live system
probing, shell/system command execution, browser profile collection, credential
collection, `.env` collection/edit, SSH key collection, cookie/token collection,
raw private path dump, runtime/private data dump, backend route, persistence
schema, or worker behavior was added.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-238-local-project-pc-diagnostics-collector-hardening.md`

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Rust checks were not required because no Rust files changed.
- Script syntax checks were not required because no scripts changed.
- Full Docker smoke was not run from Codex per owner instruction.

## Verification Summary

- Next.js production build passed.
- Working-tree whitespace check passed.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope draft/status strings in
  historical DIFF records and command examples; this DIFF is
  `Status: Complete`.
