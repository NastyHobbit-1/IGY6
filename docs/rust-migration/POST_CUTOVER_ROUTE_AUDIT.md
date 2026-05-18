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
    |   agent intent, retrieval preview, evidence answer, DIFF-106 and
    |   DIFF-107 read-only DB routes, DIFF-108 status/config routes,
    |   DIFF-109 approval request creation, DIFF-110 feedback/outcome
    |   writes, DIFF-111 source creation, DIFF-112 report creation, and
    |   DIFF-113 analysis pattern writes, DIFF-114 collection dry-run
    |   previews, DIFF-115 settings verify/apply, DIFF-116 work-item
    |   creation, DIFF-117 manual upload collection creation, DIFF-118
    |   agent action request/execution routes, and DIFF-120 dynamic web
    |   control routes
    |
    +-- FastAPI fallback service: legacy-api
        for all unsupported routes
```

FastAPI is still required for classified non-web routes. No web-used route
requires FastAPI fallback after DIFF-120, and DIFF-120 records the current
non-web fallback posture in
`configs/legacy-fastapi-route-classification.json` and
`docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`.
`legacy-api` must not be archived, removed, or disabled until route parity
proves every active route is served by Rust or deliberately retired.

## Runtime Topology

`infra/docker-compose.yml` currently defines:

| Service | Runtime role |
| --- | --- |
| `api` | Rust gateway built from `crates/igy6-gateway/Dockerfile`, published on `127.0.0.1:${APP_PORT:-8000}:8000`. |
| `legacy-api` | FastAPI backend built from `services/api`, not directly published, used by Rust gateway as fallback at `http://legacy-api:8000`. |
| `web` | Next.js UI with `API_BASE_URL=http://api:8000`; browser-side helpers also call `http://127.0.0.1:8000`. |
| `worker` and `beat` | Python/Celery execution remains active. |

The web UI calls the Rust gateway endpoint. The route parity guard reports zero
web-used routes requiring fallback, while non-web FastAPI routes continue to be
proxied to `legacy-api`.

## Rust-Native Gateway Routes

These routes are handled directly by `crates/igy6-gateway`:

| Method | Route | Rust status |
| --- | --- | --- |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/rust-migration/status` | Rust-native |
| GET | `/settings/env` | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/memory/vector/chunks` | Rust-native read-only status |
| GET | `/memory/graph/schema` | Rust-native read-only status |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/actions/` | Rust-native fixed action request/audit route |
| POST | `/agent/actions/{action_name}/execute` | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| POST | `/agent/intent` | Rust-native |
| POST | `/chat/retrieval-preview` | Rust-native contract response |
| POST | `/chat/evidence-answer` | Rust-native contract response |
| POST | `/approvals` | Rust-native DB write with audit event |
| GET | `/analysis/patterns` | Rust-native DB read |
| GET | `/analysis/patterns/{pattern_id}` | Rust-native DB read |
| POST | `/analysis/patterns` | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Rust-native DB status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Rust-native DB write with deterministic local detection and audit events |
| GET | `/analysis/hypotheses` | Rust-native DB read |
| GET | `/analysis/hypotheses/{hypothesis_id}` | Rust-native DB read |
| GET | `/analysis/predictions` | Rust-native DB read |
| GET | `/analysis/predictions/{prediction_id}` | Rust-native DB read |
| GET | `/analysis/recommendations` | Rust-native DB read |
| GET | `/analysis/recommendations/{recommendation_id}` | Rust-native DB read |
| GET | `/approvals` | Rust-native DB read |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| POST | `/approvals/{approval_id}/decision` | Rust-native pending-only approval decision with audit event |
| GET | `/artifacts` | Rust-native DB read |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| GET | `/collection-runs` | Rust-native DB read |
| GET | `/collection-runs/{collection_run_id}` | Rust-native DB read |
| POST | `/collection-runs/dry-run` | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/manual-upload` | Rust-native DB/artifact write with source permission, approval, and audit events |
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
| POST | `/feedback` | Rust-native DB write with audit event |
| GET | `/outcomes` | Rust-native DB read |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| POST | `/outcomes` | Rust-native DB write with audit events |
| GET | `/reports` | Rust-native DB read |
| GET | `/reports/{report_id}` | Rust-native DB read |
| POST | `/reports` | Rust-native DB write with audit event |
| POST | `/reports/{report_id}/render` | Rust-native bounded metadata report render with artifact and audit event |
| GET | `/sources` | Rust-native DB read |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| POST | `/sources` | Rust-native DB write with optional permission and audit event |
| GET | `/work-items` | Rust-native DB read |
| GET | `/work-items/{work_item_id}` | Rust-native DB read |
| POST | `/work-items` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Rust-native dispatch validation and non-executing audit marker |

All other routes are forwarded to `legacy-api` when the fallback origin is
configured.

Route parity counts:

| Metric | DIFF-105 | DIFF-106 | DIFF-107 | DIFF-108 | DIFF-109 | DIFF-110 | DIFF-111 | DIFF-112 | DIFF-113 | DIFF-114 | DIFF-115 | DIFF-116 | DIFF-117 | DIFF-118 | DIFF-119 | DIFF-120 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| FastAPI total routes | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 | 91 |
| Rust-native routes | 7 | 24 | 42 | 45 | 46 | 48 | 49 | 50 | 52 | 53 | 55 | 57 | 58 | 60 | 60 | 64 |
| FastAPI routes missing from Rust | 85 | 68 | 50 | 47 | 46 | 44 | 43 | 42 | 40 | 39 | 37 | 36 | 35 | 34 | 34 | 30 |
| Web-used routes | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 41 | 45 |
| Web routes requiring fallback | 36 | 28 | 19 | 16 | 14 | 12 | 11 | 9 | 7 | 6 | 4 | 3 | 2 | 0 | 0 | 0 |

## DIFF-120 Dynamic Web Control Route Parity

DIFF-120 migrates the four dynamically referenced `apps/web` page controls that
DIFF-119 found outside the extractor, and adds them explicitly to
`scripts/rust-route-parity.py` so they cannot be missed again. FastAPI remains
required for the 30 classified routes still missing from Rust, and Rust-only
cannot be claimed.

| Classification | Count |
| --- | ---: |
| `active_parity_required` | 11 |
| `intentional_legacy_fallback` | 7 |
| `retireable_unused` | 0 |
| `duplicate_or_superseded` | 1 |
| `unsafe_to_migrate_now` | 11 |

FastAPI remains required because `intentional_legacy_fallback` and
`unsafe_to_migrate_now` are non-empty. Rust-only cannot honestly be claimed.

## Web-Used Route Matrix

| Method | Route | Web usage | Gateway behavior |
| --- | --- | --- | --- |
| GET | `/agent/capabilities` | Next.js proxy and page data load | Rust-native |
| POST | `/agent/intent` | Next.js proxy and page intent preview | Rust-native |
| POST | `/agent/actions/` | Page dynamic action execution prefix detected by route guard | Rust-native fixed action request/audit route |
| POST | `/agent/actions/{action_name}/execute` | Next.js proxy and page action execution | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| GET | `/analysis/patterns` | Page data load | Rust-native DB read |
| POST | `/analysis/patterns` | Page pattern create | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Page pattern review | Rust-native candidate-only status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Page baseline pattern detection | Rust-native DB write with deterministic local detection and audit events |
| GET | `/analysis/hypotheses` | Page data load | Rust-native DB read |
| GET | `/analysis/predictions` | Page data load | Rust-native DB read |
| GET | `/analysis/recommendations` | Page data load | Rust-native DB read |
| GET | `/approvals` | Next.js proxy and page data load | Rust-native DB read |
| POST | `/approvals` | Next.js proxy and page approval request | Rust-native DB write with audit event |
| POST | `/approvals/{approval_id}/decision` | Page approval decision | Rust-native pending-only decision with audit event |
| POST | `/chat/retrieval-preview` | Next.js proxy and page chat preview | Rust-native contract response |
| POST | `/chat/evidence-answer` | Page evidence answer | Rust-native contract response |
| GET | `/artifacts` | Page data load | Rust-native DB read |
| GET | `/audit-events` | Page data load | Rust-native DB read |
| GET | `/collection-runs` | Page data load | Rust-native DB read |
| POST | `/collection-runs/dry-run` | Page collection preview | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/manual-upload` | Page manual upload collection | Rust-native DB/artifact write with source permission, approval, and audit events |
| GET | `/evidence/documents` | Page data load | Rust-native DB read |
| GET | `/evidence/chunks` | Page data load | Rust-native DB read |
| GET | `/evidence/items` | Page data load | Rust-native DB read |
| GET | `/evidence/claims` | Page data load | Rust-native DB read |
| GET | `/feedback` | Page data load | Rust-native DB read |
| GET | `/outcomes` | Page data load | Rust-native DB read |
| POST | `/feedback` | Page review feedback | Rust-native DB write with audit event |
| POST | `/outcomes` | Page review outcome | Rust-native DB write with audit events |
| GET | `/memory/graph/schema` | Page data load | Rust-native read-only status |
| GET | `/memory/vector/chunks` | Page data load | Rust-native read-only status |
| POST | `/reports` | Page report create | Rust-native DB write with audit event |
| GET | `/reports` | Page data load | Rust-native DB read |
| POST | `/reports/{report_id}/render` | Page report render | Rust-native bounded metadata render with artifact and audit event |
| GET | `/settings/env` | Next.js proxy and page settings load | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Next.js proxy and page settings verify | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Next.js proxy and page settings apply | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/sources` | Page data load | Rust-native DB read |
| POST | `/sources` | Page source create | Rust-native DB write with optional permission and audit event |
| GET | `/work-items` | Page data load | Rust-native DB read |
| POST | `/work-items/` | Page work item creation | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Page work dispatch | Rust-native validation and non-executing dispatch audit marker |

## FastAPI Route Inventory

FastAPI exposes `/` plus the following APIRouter routes. DIFF-105 automated the
count and found 90 APIRouter routes plus `/`; DIFF-104's manual table remains a
human-readable inventory of the active route families and gateway behavior.

| Method | Route | Gateway behavior |
| --- | --- | --- |
| GET | `/agent/capabilities` | Rust-native |
| POST | `/agent/intent` | Rust-native |
| POST | `/agent/actions/{action_name}/execute` | Rust-native fixed action execution with approval, audit, and host-bridge safety gates |
| GET | `/analysis/patterns` | Rust-native DB read |
| POST | `/analysis/patterns` | Rust-native DB write with evidence validation and audit event |
| POST | `/analysis/patterns/{pattern_id}/review` | Rust-native DB status transition with audit event |
| POST | `/analysis/patterns/detect-baseline` | Rust-native DB write with deterministic local detection and audit events |
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
| POST | `/approvals` | Rust-native DB write with audit event |
| GET | `/approvals/{approval_id}` | Rust-native DB read |
| POST | `/approvals/{approval_id}/decision` | Rust-native pending-only decision with audit event |
| GET | `/artifacts` | Rust-native DB read |
| POST | `/artifacts` | Proxied to FastAPI |
| GET | `/artifacts/{artifact_id}` | Rust-native DB read |
| GET | `/audit-events` | Rust-native DB read |
| GET | `/audit-events/{audit_event_id}` | Rust-native DB read |
| POST | `/chat/retrieval-preview` | Rust-native |
| POST | `/chat/evidence-answer` | Rust-native |
| GET | `/collection-runs` | Rust-native DB read |
| POST | `/collection-runs` | Proxied to FastAPI |
| POST | `/collection-runs/dry-run` | Rust-native DB write with source/permission validation and audit events |
| POST | `/collection-runs/manual-upload` | Rust-native DB/artifact write with source permission, approval, and audit events |
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
| POST | `/feedback` | Rust-native DB write with audit event |
| GET | `/feedback/{feedback_id}` | Rust-native DB read |
| GET | `/health/live` | Rust-native |
| GET | `/health/ready` | Rust-native |
| GET | `/improvements` | Proxied to FastAPI |
| POST | `/improvements` | Proxied to FastAPI |
| GET | `/improvements/{improvement_item_id}` | Proxied to FastAPI |
| GET | `/memory/graph/schema` | Rust-native read-only status |
| POST | `/memory/graph/schema/ensure` | Proxied to FastAPI |
| POST | `/memory/graph/lineage/sync` | Proxied to FastAPI |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | Proxied to FastAPI |
| GET | `/memory/vector/chunks` | Rust-native read-only status |
| POST | `/memory/vector/chunks/ensure` | Proxied to FastAPI |
| POST | `/memory/vector/chunks/upsert` | Proxied to FastAPI |
| POST | `/memory/vector/chunks/search` | Proxied to FastAPI |
| GET | `/outcomes` | Rust-native DB read |
| POST | `/outcomes` | Rust-native DB write with audit events |
| GET | `/outcomes/{outcome_id}` | Rust-native DB read |
| GET | `/reports` | Rust-native DB read |
| POST | `/reports` | Rust-native DB write with audit event |
| POST | `/reports/{report_id}/status` | Proxied to FastAPI |
| POST | `/reports/{report_id}/work-item` | Proxied to FastAPI |
| POST | `/reports/{report_id}/render` | Rust-native bounded metadata render with artifact and audit event |
| GET | `/reports/{report_id}` | Rust-native DB read |
| GET | `/retrieval/chunks/{chunk_id}/trail` | Proxied to FastAPI |
| POST | `/retrieval/chunks/search` | Proxied to FastAPI |
| GET | `/settings/env` | Rust-native redacted config metadata |
| POST | `/settings/env/verify` | Rust-native settings validation with redaction and token generation |
| POST | `/settings/env/apply` | Rust-native settings apply with safe `.env` backup/write and audit event |
| GET | `/sources` | Rust-native DB read |
| POST | `/sources` | Rust-native DB write with optional permission and audit event |
| GET | `/sources/{source_id}` | Rust-native DB read |
| GET | `/sources/{source_id}/permissions` | Rust-native DB read |
| POST | `/sources/{source_id}/permissions` | Proxied to FastAPI |
| GET | `/work-items` | Rust-native DB read |
| POST | `/work-items` | Rust-native DB write with intent verification and audit event |
| POST | `/work-items/{work_item_id}/dispatch` | Rust-native validation and non-executing dispatch audit marker |
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
DB read batch and reduces web route fallback dependency again. DIFF-108 adds
Rust-native read-only settings/env metadata, vector status, and graph status
routes without reading `.env` contents or mutating Qdrant/Neo4j. DIFF-109 adds
Rust-native approval request creation with audit event insertion. DIFF-110 adds
Rust-native feedback and outcome writes with validation, audit insertion, and
their Python side-effect parity for source trust, weak-feedback improvement
items, and outcome target updates. DIFF-111 adds Rust-native source creation
with optional initial permission insertion and the deterministic
`source.created` audit event. DIFF-112 adds Rust-native report creation with
deterministic validation and the `report.created` audit event. DIFF-113 adds
Rust-native analysis pattern creation and baseline pattern detection with
evidence validation, deterministic local candidate generation, duplicate
detector-key suppression, and `analysis.pattern.created` audit events.
DIFF-114 adds Rust-native collection dry-run preview creation with
source/permission validation, scaffold connector preview parity, and
`collection_run.created` plus `collection_run.dry_run_preview` audit events.
DIFF-115 adds Rust-native settings verify/apply with allowlisted validation,
secret redaction, candidate-hash token compatibility, safe `.env` backup/write
constraints, and `settings.env.updated` audit events. Rust intentionally does
not execute Docker Compose from HTTP request handlers; Compose validation
remains an operator verification step.
DIFF-116 adds Rust-native work-item creation with intent verification context
validation, supported-type validation, deterministic
`pending_intent_verification` status, and `work_item.created` audit events. It
does not dispatch work, execute agents, migrate manual upload, or change
work-item status routes. DIFF-117 adds Rust-native manual upload collection
creation with source type, source permission, approval, text MIME/content, and
safe filename validation; bounded content-addressed artifact writes via
`crates/igy6-artifacts`; collection run, raw artifact, and queued normalization
work item metadata inserts; and `collection_run.created`, `raw_artifact.created`,
and `work_item.created` audit events. It does not execute normalization,
dispatch workers, ingest into vector/graph memory, or execute agents
synchronously. If the artifact write succeeds but the database transaction later
fails, the content-addressed artifact may remain under the configured safe
artifact root without DB metadata; retrying the same upload reuses the same
hash path.
DIFF-118 adds Rust-native web-used agent action request/execution routes. The
Rust gateway accepts only the existing fixed action allowlist, rejects malformed
action names, rejects user-provided `argv`/command surfaces, requires approved
matching `agent_action` approvals for stack-changing actions, writes
`agent.action.requested`, `agent.action.started`, `agent.action.finished`, and
`agent.action.rejected` audit events where persistence is available, and calls
script-backed actions only through the local-only host bridge with a token and
fixed action name. No arbitrary shell command or raw user text execution is
added.

## Manifest Finding

The manifest accurately showed the DIFF-102 gateway phase complete, but it did
not include an explicit post-cutover route-parity phase or a machine-readable
statement that FastAPI fallback remains required. DIFF-104 adds that status so
future cutover checks cannot be read as proof that FastAPI is removable.

## Follow-Up DIFF Plan

Web-used FastAPI fallback is eliminated as of DIFF-118. FastAPI remains
required for unsupported non-web routes until those routes are implemented or
deliberately retired:

1. DIFF-105: add an automated route parity guard so fallback dependency cannot
   become an undocumented manual finding.
2. DIFF-106: implement Rust native handling for the first safe web-critical
   read-only DB route batch: sources, approvals, work-items, reports, and
   evidence document/item/chunk/claim list/detail reads.
3. DIFF-107: continue fallback reduction with the next safest web-critical
   read routes: audit events, artifacts, collection runs, feedback, outcomes,
   and analysis list/detail reads.
4. DIFF-108: migrate Rust-native read-only settings/env metadata,
   vector-memory status, and graph-memory status without `.env` reads or
   Qdrant/Neo4j mutation.
5. DIFF-109: migrate approval request creation with explicit audit-event
   parity.
6. DIFF-110: migrate feedback and outcome write routes with explicit validation
   and audit parity.
7. DIFF-111: migrate source creation with optional permission and explicit
   `source.created` audit parity.
8. DIFF-112: migrate report creation with explicit `report.created` audit
   parity.
9. DIFF-113: migrate analysis pattern creation and baseline detection with
   explicit evidence validation and `analysis.pattern.created` audit parity.
10. DIFF-114: migrate collection dry-run preview creation with explicit
   source/permission validation and audit parity.
11. DIFF-115: migrate settings env verify/apply with safe redaction, token, and
   audit parity.
12. DIFF-116: migrate work-item creation with intent verification and
   `work_item.created` audit parity, without dispatching work.
13. DIFF-117: migrate manual upload collection creation with source permission,
   approval, artifact storage, queued normalization metadata, and audit parity
   without synchronous ingestion or dispatch.
14. DIFF-118: migrate the final web-used agent action request/execution routes
   with fixed allowlists, approval gates, audit events, local-only host bridge
   execution, timeout bounds, redaction, and no user-provided argv.
15. DIFF-119: audit the remaining non-web FastAPI route inventory and decide
   which routes are active, retireable, or still require Rust parity.
16. Later DIFFs: migrate report render/work-item
   writes, collection writes, retrieval hydration, vector/graph memory,
   analysis, feedback, outcomes, improvements, and experiments.
17. Final retirement DIFF: remove or disable `legacy-api` only after route
   parity tests prove no active route depends on it.
