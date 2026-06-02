# DIFF-191 Promotion Candidate Audit

Status: Complete

## Branch Policy

- Work happened on `dev`.
- This DIFF is audit/planning only.
- No files were promoted.
- `main` was not checked out, touched, merged into, or cherry-picked from/to.
- Private/dev/build instruction files stay tracked on `dev`.
- Public/runtime-safe work may be promoted later only by explicit owner
  instruction and only through selective promotion.

## Purpose

Audit DIFF-184 through DIFF-190 and produce a precise, safe promotion plan for
`main`. This DIFF does not perform promotion.

## Current References

- Branch before work: `dev`.
- HEAD before work: `508e8ac Complete DIFF-190 operator smoke verification bundle`.
- `origin/dev` before work: `508e8ac`.
- `dev` ahead/behind `origin/dev` before work: synced, no ahead/behind marker.
- Local `main` after fetch inspection: `56039c5`.
- `origin/main` after fetch inspection: `7fccb5f`.
- Candidate audit range: `e3b452e^..508e8ac`.

## Files Inspected

- `AGENTS.md`
- `docs/agents/CODEX_PROMPT_BASELINE.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `configs/rust-cutover-manifest.json`
- `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md`
- `docs/diffs/DIFF-185-evidence-answer-review-ux.md`
- `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md`
- `docs/diffs/DIFF-187-basic-report-workflow-ux.md`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`
- `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md`
- `docs/diffs/DIFF-190-operator-smoke-verification-bundle.md`
- Commit/file summaries for `e3b452e`, `7b7871c`, `a05011f`, `a1b561a`,
  `6492825`, `80feffd`, and `508e8ac`.
- `main..dev` and `origin/main..dev` comparisons without changing branches.
- Tracked private/dev inventory for `.codex`, `AGENTS.md`,
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`,
  `docs/agents`, and `docs/plans`.

## Candidate Range Summary

DIFF-184 through DIFF-190 changed only these paths:

```text
M apps/web/src/app/globals.css
M apps/web/src/app/page.tsx
M crates/igy6-gateway/src/lib.rs
A docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
A docs/diffs/DIFF-185-evidence-answer-review-ux.md
A docs/diffs/DIFF-186-work-status-recovery-ux-polish.md
A docs/diffs/DIFF-187-basic-report-workflow-ux.md
A docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
A docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
A docs/diffs/DIFF-190-operator-smoke-verification-bundle.md
A docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
```

## File Classification

| File | Category | Reason |
| --- | --- | --- |
| `crates/igy6-gateway/src/lib.rs` | A. `promote_candidate_public_runtime` | Rust gateway runtime code. DIFF-184 replaced a contract-only retrieval preview path with live retrieval behavior; DIFF-188 fixed a narrow outcome timestamp insertion bug. Public/runtime-safe if promoted with tests. |
| `apps/web/src/app/page.tsx` | A. `promote_candidate_public_runtime` | Next.js UI source. DIFF-185 through DIFF-189 added Results/Work UX for retrieval review, work status, basic reports, feedback/outcomes, and source/evidence history. Public/runtime-safe if promoted with web build. |
| `apps/web/src/app/globals.css` | A. `promote_candidate_public_runtime` | Next.js styling for DIFF-186 work status UX. Public/runtime-safe when promoted with the matching `page.tsx` changes. |
| `docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md` | A. `promote_candidate_public_runtime` | Runtime/operator documentation. It uses synthetic text and non-secret checks, and does not include private runtime data. |
| `docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md` | B. `dev_only_keep_on_dev` | Dev DIFF record with build-agent workflow detail and local verification notes. Exclude from public promotion unless owner separately authorizes DIFF record promotion. |
| `docs/diffs/DIFF-185-evidence-answer-review-ux.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |
| `docs/diffs/DIFF-186-work-status-recovery-ux-polish.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |
| `docs/diffs/DIFF-187-basic-report-workflow-ux.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |
| `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |
| `docs/diffs/DIFF-189-source-evidence-history-detail-ux.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |
| `docs/diffs/DIFF-190-operator-smoke-verification-bundle.md` | B. `dev_only_keep_on_dev` | Dev DIFF record. Exclude from public promotion. |

No DIFF-184 through DIFF-190 file is classified as C
`promotion_needs_review` or D `do_not_promote`.

## Public Runtime Promotion Candidates

Recommended candidate files:

```text
apps/web/src/app/globals.css
apps/web/src/app/page.tsx
crates/igy6-gateway/src/lib.rs
docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md
```

These files are public/runtime-safe candidates because they are normal runtime
source or runtime operator documentation and do not include private/dev/build
instruction material.

## Excluded Files

Keep these only on `dev` for this promotion:

```text
docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
docs/diffs/DIFF-185-evidence-answer-review-ux.md
docs/diffs/DIFF-186-work-status-recovery-ux-polish.md
docs/diffs/DIFF-187-basic-report-workflow-ux.md
docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
docs/diffs/DIFF-190-operator-smoke-verification-bundle.md
```

Also exclude all known private/dev/build files from any promotion target:

```text
.codex
AGENTS.md
Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md
docs/agents/
docs/plans/
```

## Promotion Method Analysis

Cherry-picking selected commits is not recommended as the primary method. The
candidate commits also add dev DIFF records that this audit excludes from
promotion. Cherry-picking would require extra cleanup before commit and could
accidentally carry dev-only documentation.

Checking out selected paths from `dev` to `main` is also not recommended as the
primary method. `main..dev` includes earlier dev-only and runtime changes outside
DIFF-184 through DIFF-190, so path checkout from the current `dev` tip could
silently include unaudited content in files such as `apps/web/src/app/page.tsx`
or `crates/igy6-gateway/src/lib.rs`.

Recommended method: create a clean public promotion branch from
`origin/main` or the owner-specified `main` reference, then apply only the
audited DIFF-184 through DIFF-190 patch hunks for the four promotion candidate
files. Do not merge `dev`. Do not checkout whole files from `dev` unless the
future promotion DIFF first confirms that all existing `dev` changes in that
file are also intended for promotion. Commit only the four candidate files after
reviewing the staged diff.

## Recommended Promotion Command Shape

The future promotion DIFF should adapt commands to the owner-selected base
branch, but the safe shape is:

```bash
git fetch origin
git switch -c promotion/diff-184-190-runtime origin/main
git diff e3b452e^..508e8ac -- \
  apps/web/src/app/globals.css \
  apps/web/src/app/page.tsx \
  crates/igy6-gateway/src/lib.rs \
  docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md > /tmp/igy6-diff184-190-public.patch
git apply --index /tmp/igy6-diff184-190-public.patch
git diff --cached --name-status
git diff --cached --check
```

If `git apply` reports context conflicts because `main` does not have earlier
dev-only context, resolve manually by extracting only the audited public runtime
hunks from DIFF-184 through DIFF-190. Do not solve conflicts by merging `dev` or
checking out whole files from `dev`.

## Risks

- `main` and `origin/main` differ locally: local `main` is `56039c5`, while
  `origin/main` is `7fccb5f` after fetch. The promotion owner must choose the
  exact public base before starting.
- `main..dev` and `origin/main..dev` include private/dev files and historical
  dev-only branch-management changes. Wholesale merge, broad cherry-pick, or
  whole-path checkout from `dev` is unsafe.
- `git diff --check main..dev` and `git diff --check origin/main..dev` fail on
  existing trailing whitespace in
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`, a private/dev
  file that must not be promoted. The audited candidate range
  `e3b452e^..508e8ac` passes `git diff --check`.
- `apps/web/src/app/page.tsx` and `crates/igy6-gateway/src/lib.rs` may have
  dependencies on earlier dev changes outside DIFF-184 through DIFF-190.
  Promotion must build and test from the public promotion branch after applying
  the selected hunks.
- DIFF-190 runtime docs refer to existing scripts and UI markers. If only part
  of the runtime/UI patch is promoted, the doc must be rechecked against the
  promotion branch before commit.

## Future Promotion Prompt

Paste-ready prompt for a later owner-authorized promotion DIFF:

```text
You are working in the IGY6 repo.

Goal:
Create and complete the next DIFF for selective public promotion of the
DIFF-184 through DIFF-190 runtime-safe candidate files audited by DIFF-191.

Use DIFF-191 as the promotion audit. Do not merge all of dev. Do not promote
private/dev/build instruction files. Do not promote dev-only DIFF records unless
the owner explicitly expands the scope.

Allowed promotion files only:

- apps/web/src/app/globals.css
- apps/web/src/app/page.tsx
- crates/igy6-gateway/src/lib.rs
- docs/runtime/OPERATOR_SMOKE_VERIFICATION_BUNDLE.md

Excluded files:

- .codex
- AGENTS.md
- Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md
- docs/agents/
- docs/plans/
- docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
- docs/diffs/DIFF-185-evidence-answer-review-ux.md
- docs/diffs/DIFF-186-work-status-recovery-ux-polish.md
- docs/diffs/DIFF-187-basic-report-workflow-ux.md
- docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
- docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
- docs/diffs/DIFF-190-operator-smoke-verification-bundle.md

Required approach:

1. Read AGENTS.md, docs/BRANCH_POLICY.md, README.md, docs/ui/README.md,
   configs/rust-cutover-manifest.json, and
   docs/diffs/DIFF-191-promotion-candidate-audit.md.
2. Ask the owner to confirm the exact public base, either main or origin/main,
   before switching branches.
3. Only after explicit owner confirmation, create a clean promotion branch from
   the confirmed public base.
4. Apply only the audited DIFF-184 through DIFF-190 public/runtime-safe hunks for
   the four allowed files. Prefer an explicit range patch:
   `git diff e3b452e^..508e8ac -- <allowed files>`.
5. If the patch conflicts, manually extract only audited public/runtime hunks.
   Do not merge dev. Do not cherry-pick broad commits. Do not checkout whole
   files from dev unless you first prove the whole file contains only intended
   public/runtime changes.
6. Verify no excluded files are staged or present as newly added files on the
   promotion branch.
7. Run public/runtime cleanliness checks:
   - git status --short
   - git diff --cached --name-status
   - git diff --cached --check
   - git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans
8. Run build/tests:
   - cargo fmt --all --check
   - cargo test -p igy6-gateway
   - npm --prefix apps/web run build
   - docker compose -f infra/docker-compose.yml --env-file .env.example config
9. Commit only the allowed public/runtime-safe files and the new promotion DIFF
   record if the owner allows a public DIFF record for this promotion.
10. Do not push main. Require explicit owner confirmation before any push to
    main or PR creation.

Final response must include branch, base reference, files promoted, excluded
files verified absent, verification results, whether main was pushed, and the
commit hash.
```

## Verification Summary

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
find docs/diffs -maxdepth 1 -type f | sort | tail -110
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/ui/README.md
sed -n '1,260p' configs/rust-cutover-manifest.json
sed -n '1,360p' docs/diffs/DIFF-184-manual-upload-evidence-retrieval-followthrough.md
sed -n '1,360p' docs/diffs/DIFF-185-evidence-answer-review-ux.md
sed -n '1,360p' docs/diffs/DIFF-186-work-status-recovery-ux-polish.md
sed -n '1,360p' docs/diffs/DIFF-187-basic-report-workflow-ux.md
sed -n '1,360p' docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md
sed -n '1,360p' docs/diffs/DIFF-189-source-evidence-history-detail-ux.md
sed -n '1,360p' docs/diffs/DIFF-190-operator-smoke-verification-bundle.md
git show --name-status --oneline e3b452e
git show --name-status --oneline 7b7871c
git show --name-status --oneline a05011f
git show --name-status --oneline a1b561a
git show --name-status --oneline 6492825
git show --name-status --oneline 80feffd
git show --name-status --oneline 508e8ac
git fetch origin
git log --oneline --decorate main..dev
git log --oneline --decorate origin/main..dev || true
git diff --name-status main..dev
git diff --name-status origin/main..dev || true
git diff --check main..dev
git diff --check origin/main..dev || true
git diff --name-status e3b452e^..508e8ac
git diff --check e3b452e^..508e8ac
git rev-parse --short dev
git rev-parse --short origin/dev
git rev-parse --short main
git rev-parse --short origin/main
git status --short --branch
```

Results:

- Initial working tree was clean.
- Current branch was `dev`.
- `dev` and `origin/dev` were both `508e8ac`.
- `git fetch origin` completed.
- `main` was `56039c5`; `origin/main` was `7fccb5f`.
- Candidate range `e3b452e^..508e8ac` changed 11 files and passed
  `git diff --check`.
- Full `main..dev` and `origin/main..dev` comparisons showed private/dev files,
  dev-only documentation, and historical deletions/additions outside this
  promotion audit.
- Full `main..dev` and `origin/main..dev` `git diff --check` failed only because
  `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md` has existing
  trailing whitespace in the broad dev-only comparison. That file is excluded
  from promotion and was not edited.
- The active/draft grep still reports stale older `Status: Draft` strings in
  DIFF-177 and DIFF-180 plus command transcripts in completed DIFF records; this
  audit did not edit those older locked/out-of-scope records.
- Private/dev files remain tracked on `dev`.
- No runtime code, UI code, `.env`, runtime data, Docker volumes, databases, or
  services were modified.
- No promotion, branch switch to `main`, merge, cherry-pick, push, or destructive
  command was performed.

## Final Status

DIFF-191 is complete as an audit/planning DIFF.
