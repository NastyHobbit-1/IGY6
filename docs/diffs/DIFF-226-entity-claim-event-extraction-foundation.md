# DIFF-226 - Entity / Claim / Event Extraction Foundation

Status: Complete

## Scope

DIFF-226 adds the first normal-user review foundation for structured entities,
claims, and events from local evidence. This is product UI/docs work over
existing evidence and claim reads. It does not add schema, persistence, worker
extraction, hosted AI, or full graph/NLP reasoning.

## Current Support Found

- Evidence items already expose source, document, chunk, statement, confidence,
  metadata, and timestamps.
- Claim records are exposed through the Rust gateway read routes with
  `evidence_ids`, confidence, status, and metadata.
- No entity or event persistence route is currently exposed.
- No claim-create route is currently exposed through the gateway.

Because persistence is incomplete for entity/event records and claim creation,
this DIFF implements a review surface rather than a write workflow.

## Product Behavior Added

- Added an Entity, Claim, and Event Review panel in Results.
- Shows structured-memory counts for evidence, stored claims, entity candidates,
  claim candidates, and event candidates.
- Derives conservative local review candidates from already loaded evidence:
  - entity candidates from simple capitalized phrase hints;
  - claim candidates from evidence statements not already linked to a stored
    claim;
  - event candidates from date-like text or known observation/decision metadata.
- Shows provenance to source, document, chunk, and evidence where available.
- Shows stored claims with linked evidence IDs and review status.
- Marks all derived candidates as review-only or needs-review.
- States that capitalization/date hints are unverified and require owner review.

## Explicit Non-Claims

- No broad NLP extraction is claimed.
- No complete entity resolution is claimed.
- No full graph reasoning is claimed.
- No original evidence is mutated.
- No entity/event persistence is added.
- No hosted AI or external service call is used.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-226-entity-claim-event-extraction-foundation.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.
