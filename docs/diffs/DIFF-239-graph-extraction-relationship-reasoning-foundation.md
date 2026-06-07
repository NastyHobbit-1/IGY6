# DIFF-239 - Graph Extraction And Relationship Reasoning Foundation

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `53c6000 Complete DIFF-238 local project PC diagnostics collector hardening`
- `dev` tracking state before work: ahead of `origin/dev` by 4 commits
- Working tree before work: clean

## Purpose

Expand the existing entity, claim, and event review surface with relationship
review candidates tied to local evidence provenance, without claiming full graph
reasoning.

## Files Inspected

- `docs/diffs/DIFF-225-graph-lineage-explanation-ux.md`
- `docs/diffs/DIFF-226-entity-claim-event-extraction-foundation.md`
- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`
- `docs/diffs/DIFF-236-browser-web-router-collector-mvp.md`
- `docs/diffs/DIFF-237-pdf-image-audio-video-import-mvp.md`
- `docs/diffs/DIFF-238-local-project-pc-diagnostics-collector-hardening.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- Current graph/Neo4j, lineage, evidence, claim, entity/event review, and
  source/evidence detail surfaces inspected during batch pre-work

## Implementation

- Extended the structured memory panel to Entity, Claim, Event, And
  Relationship Review.
- Added relationship review candidates derived from loaded relational records:
  - `evidence_observed_from_source`
  - `evidence_extracted_from_document`
  - `evidence_supported_by_chunk`
  - `claim_supported_by_evidence`
- Each relationship row shows:
  - relation type;
  - subject;
  - object;
  - provenance trail;
  - review status;
  - support count;
  - confidence where available.
- Preserved the existing conservative entity, claim, and event review behavior.
- Updated the UI guide to describe relationship candidates as read-only review
  rows rather than persisted graph records.

## Scope Confirmation

This is a UI-only relationship review foundation. It does not add automated
extraction, persistence, Neo4j sync, graph mutation, correlation discovery, or
advanced graph reasoning.

No hosted AI call, external data transfer, evidence mutation, original artifact
mutation, backend route, persistence schema, worker/runtime behavior, Neo4j
write, account scraping, arbitrary filesystem crawl, arbitrary command
execution, `.env` edit, runtime/private data dump, or private/dev file removal
was performed.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-239-graph-extraction-relationship-reasoning-foundation.md`

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Resolved during DIFF:

- Initial `npm --prefix apps/web run build` failed with a terse webpack error.
  The DIFF-239 JSX relationship text was adjusted from an unescaped arrow-style
  separator to plain text and the relationship candidate row type was made
  explicit. The rerun passed.

Not run:

- Rust checks were not required because no Rust files changed.
- Script syntax checks were not required because no scripts changed.
- Full Docker smoke was not run from Codex per owner instruction.

## Verification Summary

- Next.js production build passed after the scoped UI fix.
- Working-tree whitespace check passed.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope draft/status strings in
  historical DIFF records and command examples; this DIFF is
  `Status: Complete`.
