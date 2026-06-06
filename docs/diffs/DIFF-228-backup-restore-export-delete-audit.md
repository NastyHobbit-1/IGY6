# DIFF-228 - Backup / Restore / Export / Delete Audit

Status: Complete

## Scope

DIFF-228 audits data lifecycle boundaries before destructive controls are
implemented. It adds a small non-destructive Settings panel and this concrete
audit. It does not delete, restore, dump runtime data, create a backup archive,
edit `.env`, or modify runtime services.

## Storage Classes Inspected

- PostgreSQL relational records:
  - sources
  - source permissions
  - approvals
  - audit events
  - collection runs
  - raw artifact metadata
  - normalized documents
  - chunks
  - evidence items
  - claims
  - evidence answer records
  - feedback events
  - outcomes
  - work items
  - agent task plans
  - reports
  - patterns
  - hypotheses
  - predictions
  - recommendations
  - improvement items
  - experiment runs
- Content-addressed artifacts under the configured artifact store path.
- Rendered markdown report artifacts linked from report records.
- Vector memory in Qdrant.
- Graph memory in Neo4j.
- MLflow/Phoenix runtime stores where configured by the local stack.
- `.env` and `.env` backups, which are operational configuration, not product
  export content.

## Backup Candidates

- PostgreSQL records for source, evidence, review, analysis, report, and audit
  metadata.
- Content-addressed artifact store when the owner explicitly chooses to include
  raw/generated artifacts.
- Rendered markdown report artifacts.
- Qdrant collections for chunk vectors.
- Neo4j graph database state.
- MLflow/Phoenix experiment and observability stores if experiments/traces are
  in active use.

Backups must exclude secrets unless a future encrypted operator backup DIFF
explicitly defines secret handling. Product exports should not include `.env`.

## Export Candidates

- Report markdown artifacts.
- Report metadata and citation/evidence appendix IDs.
- Evidence, source, document, chunk, answer, feedback, outcome, pattern,
  prediction, recommendation, and task metadata where safe.
- Owner-selected raw artifacts only when a future workflow clearly warns that
  raw artifacts may contain private data.
- Audit summaries without secret values or raw runtime/private dumps.

## Restore Candidates

- PostgreSQL metadata restore from a future validated backup format.
- Artifact store restore with content-hash/path reconciliation.
- Qdrant and Neo4j service-specific restores.
- Rendered report artifact restore.

Restore is not implemented in this DIFF. Future restore work needs schema
version checks, conflict handling, dependency ordering, rollback instructions,
and explicit operator confirmation.

## Delete Candidates

Delete controls are not implemented in this DIFF. Future delete candidates need
separate explicit DIFFs because they are destructive:

- individual report metadata/artifact links;
- selected source records and dependent permissions/collection runs;
- selected raw artifacts and downstream documents/chunks/evidence;
- selected evidence, claims, answer records, feedback, outcomes, patterns,
  predictions, recommendations, and improvement/experiment metadata;
- vector and graph entries tied to deleted chunks/evidence;
- retention cleanup for old audit-safe diagnostics.

Delete workflows must record audit events, handle dependencies, avoid silent
evidence hiding, and clearly distinguish metadata deletion from raw artifact,
vector, and graph deletion.

## Retention Considerations

- Audit events should be retained long enough to explain approvals, review
  actions, source changes, report rendering, and destructive lifecycle actions.
- Evidence and report retention should preserve lineage unless the owner
  explicitly requests deletion.
- Raw artifacts may require stricter retention because they can contain private
  source text.
- Vector and graph stores need retention coupled to chunk/evidence lineage.
- `.env` backups are operational safety material and must not be bundled into
  ordinary product exports.

## Secret And Runtime Data Exclusions

- `.env` contents
- credentials
- tokens
- cookies
- private keys
- raw runtime database dumps printed to UI/logs
- unselected raw artifacts
- Docker volume data printed or copied into the repository
- runtime/private data under `IGY6_DATA_ROOT`

## Product UX Added

- Added a Settings lifecycle audit panel.
- Shows current metadata counts by data class.
- Shows configured/not-reported status for lifecycle-related settings without
  printing raw values.
- Shows vector/graph visibility from existing status endpoints.
- Shows excluded classes and future dangerous operations.
- States that backup archive creation, restore, and destructive delete are not
  implemented.

## Explicit Non-Claims

- No complete backup/restore system is claimed.
- No complete data deletion is claimed.
- No backup archive is created.
- No runtime/private data is dumped.
- No `.env` edit is performed.
- No destructive delete or restore is performed.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-228-backup-restore-export-delete-audit.md`

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Rust checks were not required because no Rust files changed.
