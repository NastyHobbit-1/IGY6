# DIFF-227 - Report Template And Export MVP

Status: Complete

## Scope

DIFF-227 improves the normal-user report workflow with basic templates and
local markdown export using existing report routes. It does not add PDF export,
external export, hosted AI, raw artifact reads, or a full report authoring suite.

## Current Support Found

- The Rust gateway already supports `POST /reports` and
  `POST /reports/{report_id}/render`.
- Rendered reports are stored as local content-addressed `text/markdown` raw
  artifacts and linked back to the report through `artifact_path`.
- Supported report types are `summary`, `evidence_review`, `decision_note`,
  `handoff`, and `experiment_summary`.
- The existing renderer creates markdown inventory reports from local metadata
  and optional notes; it does not read raw artifact contents.

## Product Behavior Added

- Replaced invalid normal-user report-type choices with supported templates.
- Added templates:
  - Evidence brief (`evidence_review`)
  - Decision note (`decision_note`)
  - Handoff (`handoff`)
  - Inventory summary (`summary`)
- Template metadata records:
  - selected template key;
  - planned section list;
  - markdown export format;
  - unsupported PDF export state;
  - citation evidence IDs;
  - linked evidence answer record IDs.
- Render notes now include template sections, an uncertainty/missing-info
  oriented structure, citation/evidence appendix IDs, linked answer records, and
  optional owner notes.
- Added a citation appendix preview in Results before report creation.
- Kept unsupported export states honest.

## Explicit Non-Claims

- No PDF export is implemented or claimed.
- No full report authoring suite is claimed.
- No hosted AI call is made.
- No secrets or raw artifact contents are dumped.
- No new API route, schema, worker behavior, or external service call is added.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-227-report-template-export-mvp.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.
