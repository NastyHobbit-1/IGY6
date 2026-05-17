# DIFF-104 Post-Cutover Route Audit

Date: 2026-05-17

## Summary

DIFF-103 completed cutover governance, but the runtime is not Rust-only. The
current topology is Rust-primary with required FastAPI fallback:

```text
Next.js web
    |
    v
Rust gateway service: api
    |
    +-- Rust-native routes for health, migration status, agent capability,
    |   agent intent, retrieval preview, evidence answer, and the DIFF-106
    |   DIFF-106 and DIFF-107 read-only DB route batches
    |
    +-- FastAPI fallback service: legacy-api
        for all unsupported routes
```

FastAPI is still required. It must not be archived, removed, or disabled until
route parity proves every active route is served by Rust or deliberately
retired.

## Runtime Topology

`infra/docker-compose.yml` currently defines:

| Service | Runtime role |
| --- | --- |
| `api` | Rust gateway built from `crates/igy6-gateway/Dockerfile`, published on `127.0.0.1:${APP_PORT:-8000}:8000`. |
| `legacy-api` | FastAPI backend built from `services/api`, not directly published, used by Rust gateway as fallback at `http://legacy-api:8000`. |
| `web` | Next.js UI with `API_BASE_URL=http://api:8000`; browser-side helpers also call `http://127.0.0.1:8000`. |
| `worker` and `beat` | Python/Celery execution remains active. |

The web UI calls the Rust gateway endpoint, but many web workflows still depend
on gateway proxying to `legacy-api`.

## Rust-Native Gateway Routes

These routes are handled directly by `crates/igy6-gateway`:

| Method | Route | Rust status |
| --- | --- | --- |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/rust-migration/status` | Rust-native |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/intent` | Rust-native |
| POST | `/chat/retrieval-preview` | Rust-native contract response |
| POST | `/chat/evidence-answer` | Rust-native contract response |
| GET | `/analysis/patterns` | Rust-native DB read |
| GET | `/analysis/patterns/{pattern_id}` | Rust-native DB read |
| GET | `/analysis/hypotheses` | Rust-native DB read |
| GET | `/analysis/hypotheses/{hypothesis_id}` | Rust-native DB read |
| GET | `/analysis/predictions` | Rust-native DB read |
| GET | `/analysis/predictions/{prediction_id}` | Rust-native DB read |
| GET | `/analysis/recommendations` | Rust-native DB read |
| GET | `/analysis/recommendations/{recommendation_id}` | Rust-native DB read |
| GET | `/approvals` | Rust-native DB read |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| GET | `/artifacts` | Rust-native DB read |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| GET | `/collection-runs` | Rust-native DB read |
| GET | `/collection-runs/{collection_run_id}` | Rust-native DB read |
| GET | `/evidence/documents` | Rust-native DB read |
| GET | `/evidence/documents/{document_id}` | Rust-native DB read |
| GET | `/evidence/items` | Rust-native DB read |
| GET | `/evidence/items/{evidence_item_id}` | Rust-native DB read |
| GET | `/evidence/chunks` | Rust-native DB read |
| GET | `/evidence/chunks/{chunk_id}` | Rust-native DB read |
| GET | `/evidence/claims` | Rust-native DB read |
| GET | `/evidence/claims/{claim_id}` | Rust-native DB read |
| GET | `/feedback` | Rust-native DB read |
| GET | `/feedback/{feedback_id}` | Rust-native DB read |
| GET | `/outcomes` | Rust-native DB read |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| GET | `/reports` | Rust-native DB read |
| GET | `/reports/{report_id}` | Rust-native DB read |
| GET | `/sources` | Rust-native DB read |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| GET | `/work-items` | Rust-native DB read |
| GET | `/work-items/{work_item_id}` | Rust-native DB read |

All other routes are forwarded to `legacy-api` when the fallback origin is
configured.

Route parity counts:

| Metric | DIFF-105 | DIFF-106 | DIFF-107 |
| --- | ---: | ---: | ---: |
| FastAPI total routes | 91 | 91 | 91 |
| Rust-native routes | 7 | 24 | 42 |
| FastAPI routes missing from Rust | 85 | 68 | 50 |
| Web-used routes | 41 | 41 | 41 |
| Web routes requiring fallback | 36 | 28 | 19 |

## Web-Used Route Matrix

| Method | Route | Web usage | Gateway behavior |
| --- | --- | --- | --- |
| GET | `/agent/capabilities` | Next.js proxy and page data load | Rust-native |
| POST | `/agent/intent` | Next.js proxy and page intent preview | Rust-native |
| POST | `/agent/actions/{action_name}/execute` | Next.js proxy and page action execution | Proxied to FastAPI |
| GET | `/analysis/patterns` | Page data load | Rust-native DB read |
| POST | `/analysis/patterns` | Page pattern create | Proxied to FastAPI |
| POST | `/analysis/patterns/detect-baseline` | Page baseline pattern detection | Proxied to FastAPI |
| GET | `/analysis/hypotheses` | Page data load | Rust-native DB read |
| GET | `/analysis/predictions` | Page data load | Rust-native DB read |
| GET | `/analysis/recommendations` | Page data load | Rust-native DB read |
| GET | `/approvals` | Next.js proxy and page data load | Rust-native DB read |
| POST | `/approvals` | Next.js proxy and page approval request | Proxied to FastAPI |
| POST | `/approvals/{approval_id}/decision` | Page approval decision | Proxied to FastAPI |
| POST | `/chat/retrieval-preview` | Next.js proxy and page chat preview | Rust-native contract response |
| POST | `/chat/evidence-answer` | Page evidence answer | Rust-native contract response |
| GET | `/artifacts` | Page data load | Rust-native DB read |
| GET | `/audit-events` | Page data load | Rust-native DB read |
| GET | `/collection-runs` | Page data load | Rust-native DB read |
| GET | `/evidence/documents` | Page data load | Rust-native DB read |
| GET | `/evidence/chunks` | Page data load | Rust-native DB read |
| GET | `/evidence/items` | Page data load | Rust-native DB read |
| GET | `/evidence/claims` | Page data load | Rust-native DB read |
| GET | `/feedback` | Page data load | Rust-native DB read |
| GET | `/outcomes` | Page data load | Rust-native DB read |
| GET | `/memory/graph/schema` | Page data load | Proxied to FastAPI |
| GET | `/memory/vector/chunks` | Page data load | Proxied to FastAPI |
| POST | `/reports` | Page report create | Proxied to FastAPI |
| GET | `/reports` | Page data load | Rust-native DB read |
| POST | `/reports/{report_id}/render` | Page report render | Proxied to FastAPI |
| GET | `/settings/env` | Next.js proxy and page settings load | Proxied to FastAPI |
| POST | `/settings/env/verify` | Next.js proxy and page settings verify | Proxied to FastAPI |
| POST | `/settings/env/apply` | Next.js proxy and page settings apply | Proxied to FastAPI |
| GET | `/sources` | Page data load | Rust-native DB read |
| POST | `/sources` | Page source create | Proxied to FastAPI |
| GET | `/work-items` | Page data load | Rust-native DB read |
| POST | `/work-items/{work_item_id}/dispatch` | Page work dispatch | Proxied to FastAPI |

## FastAPI Route Inventory

FastAPI exposes `/` plus the following APIRouter routes. DIFF-105 automated the
count and found 90 APIRouter routes plus `/`; DIFF-104's manual table remains a
human-readable inventory of the active route families and gateway behavior.

| Method | Route | Gateway behavior |
| --- | --- | --- |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/intent` | Rust-native |
| POST | `/agent/actions/{action_name}/execute` | Proxied to FastAPI |
| GET | `/analysis/patterns` | Rust-native DB read |
| POST | `/analysis/patterns` | Proxied to FastAPI |
| POST | `/analysis/patterns/{pattern_id}/review` | Proxied to FastAPI |
| POST | `/analysis/patterns/detect-baseline` | Proxied to FastAPI |
| GET | `/analysis/patterns/{pattern_id}` | Rust-native DB read |
| GET | `/analysis/hypotheses` | Rust-native DB read |
| POST | `/analysis/hypotheses` | Proxied to FastAPI |
| GET | `/analysis/hypotheses/{hypothesis_id}` | Rust-native DB read |
| GET | `/analysis/predictions` | Rust-native DB read |
| POST | `/analysis/predictions` | Proxied to FastAPI |
| GET | `/analysis/predictions/{prediction_id}` | Rust-native DB read |
| GET | `/analysis/recommendations` | Rust-native DB read |
| POST | `/analysis/recommendations` | Proxied to FastAPI |
| GET | `/analysis/recommendations/{recommendation_id}` | Rust-native DB read |
| GET | `/approvals` | Rust-native DB read |
| POST | `/approvals` | Proxied to FastAPI |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| POST | `/approvals/{approval_id}/decision` | Proxied to FastAPI |
| GET | `/artifacts` | Rust-native DB read |
| POST | `/artifacts` | Proxied to FastAPI |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| POST | `/chat/retrieval-preview` | Rust-native |
| POST | `/chat/evidence-answer` | Rust-native |
| GET | `/collection-runs` | Rust-native DB read |
| POST | `/collection-runs` | Proxied to FastAPI |
| POST | `/collection-runs/dry-run` | Proxied to FastAPI |
| POST | `/collection-runs/manual-upload` | Proxied to FastAPI |
| POST | `/collection-runs/manual-upload/ingest` | Proxied to FastAPI |
| POST | `/collection-runs/local-project` | Proxied to FastAPI |
| GET | `/collection-runs/{collection_run_id}` | Rust-native DB read |
| GET | `/evidence/documents` | Rust-native DB read |
| GET | `/evidence/documents/{document_id}` | Rust-native DB read |
| POST | `/evidence/documents` | Proxied to FastAPI |
| POST | `/evidence/documents/{document_id}/chunks` | Proxied to FastAPI |
| GET | `/evidence/items` | Rust-native DB read |
| GET | `/evidence/items/{evidence_item_id}` | Rust-native DB read |
| GET | `/evidence/chunks` | Rust-native DB read |
| GET | `/evidence/chunks/{chunk_id}` | Rust-native DB read |
| GET | `/evidence/claims` | Rust-native DB read |
| GET | `/evidence/claims/{claim_id}` | Rust-native DB read |
| POST | `/evidence/items` | Proxied to FastAPI |
| GET | `/experiments` | Proxied to FastAPI |
| POST | `/experiments` | Proxied to FastAPI |
| POST | `/experiments/{experiment_run_id}/status` | Proxied to FastAPI |
| GET | `/experiments/{experiment_run_id}` | Proxied to FastAPI |
| GET | `/feedback` | Rust-native DB read |
| POST | `/feedback` | Proxied to FastAPI |
| GET | `/feedback/{feedback_id}` | Rust-native DB read |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/improvements` | Proxied to FastAPI |
| POST | `/improvements` | Proxied to FastAPI |
| GET | `/improvements/{improvement_item_id}` | Proxied to FastAPI |
| GET | `/memory/graph/schema` | Proxied to FastAPI |
| POST | `/memory/graph/schema/ensure` | Proxied to FastAPI |
| POST | `/memory/graph/lineage/sync` | Proxied to FastAPI |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | Proxied to FastAPI |
| GET | `/memory/vector/chunks` | Proxied to FastAPI |
| POST | `/memory/vector/chunks/ensure` | Proxied to FastAPI |
| POST | `/memory/vector/chunks/upsert` | Proxied to FastAPI |
| POST | `/memory/vector/chunks/search` | Proxied to FastAPI |
| GET | `/outcomes` | Rust-native DB read |
| POST | `/outcomes` | Proxied to FastAPI |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| GET | `/reports` | Rust-native DB read |
| POST | `/reports` | Proxied to FastAPI |
| POST | `/reports/{report_id}/status` | Proxied to FastAPI |
| POST | `/reports/{report_id}/work-item` | Proxied to FastAPI |
| POST | `/reports/{report_id}/render` | Proxied to FastAPI |
| GET | `/reports/{report_id}` | Rust-native DB read |
| GET | `/retrieval/chunks/{chunk_id}/trail` | Proxied to FastAPI |
| POST | `/retrieval/chunks/search` | Proxied to FastAPI |
| GET | `/settings/env` | Proxied to FastAPI |
| POST | `/settings/env/verify` | Proxied to FastAPI |
| POST | `/settings/env/apply` | Proxied to FastAPI |
| GET | `/sources` | Rust-native DB read |
| POST | `/sources` | Proxied to FastAPI |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| POST | `/sources/{source_id}/permissions` | Proxied to FastAPI |
| GET | `/work-items` | Rust-native DB read |
| POST | `/work-items` | Proxied to FastAPI |
| POST | `/work-items/{work_item_id}/dispatch` | Proxied to FastAPI |
| POST | `/work-items/{work_item_id}/status` | Proxied to FastAPI |
| GET | `/work-items/{work_item_id}` | Rust-native DB read |

## Cutover Script Finding

`scripts/rust-cutover.sh` correctly enforces manifest shape, Rust checks,
`cutover_ready=true`, and a clean worktree before `--execute`. It does not prove
route parity and did not claim to do so in code. The DIFF-103 outcome was
therefore governance-complete, not operationally Rust-complete.

DIFF-105 adds `scripts/rust-route-parity.py` and runs it from
`scripts/rust-cutover.sh --check`. The guard inventories source-defined routes
and validates that the manifest still marks FastAPI fallback as required while
parity is incomplete. DIFF-106 extends the guard to count the Rust gateway route
registry and records the reduced fallback counts. DIFF-107 records the second
DB read batch and reduces web route fallback dependency again.

## Manifest Finding

The manifest accurately showed the DIFF-102 gateway phase complete, but it did
not include an explicit post-cutover route-parity phase or a machine-readable
statement that FastAPI fallback remains required. DIFF-104 adds that status so
future cutover checks cannot be read as proof that FastAPI is removable.

## Follow-Up DIFF Plan

FastAPI remains required until these parity batches are implemented or
deliberately retired:

1. DIFF-105: add an automated route parity guard so fallback dependency cannot
   become an undocumented manual finding.
2. DIFF-106: implement Rust native handling for the first safe web-critical
   read-only DB route batch: sources, approvals, work-items, reports, and
   evidence document/item/chunk/claim list/detail reads.
3. DIFF-107: continue fallback reduction with the next safest web-critical
   read routes: audit events, artifacts, collection runs, feedback, outcomes,
   and analysis list/detail reads.
4. DIFF-108: continue with the remaining web fallbacks that are not simple
   PostgreSQL reads: settings/env, vector/graph memory status, writes,
   approval decisions, report creation/rendering, collection write workflows,
   and agent action execution.
5. Later DIFFs: migrate settings/env write/verify, agent action execution,
   source/report/work-item writes, collection writes, retrieval hydration,
   vector/graph memory, analysis, feedback, outcomes, improvements, and
   experiments.
6. Final retirement DIFF: remove or disable `legacy-api` only after route
   parity tests prove no active route depends on it.
