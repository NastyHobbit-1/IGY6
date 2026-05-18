# DIFF-120 Non-Web FastAPI Route Classification

Date: 2026-05-18

## Summary

DIFF-120 classifies the 30 FastAPI routes that remain missing from the Rust
gateway after migrating the four dynamically referenced web control routes found
by DIFF-119. The web route guard now explicitly tracks those dynamic controls
and remains at `web_routes_requiring_fallback=0`, so Rust is primary for
web-used traffic, but the repository is not Rust-only.

FastAPI remains required because intentional legacy fallback and
unsafe-to-migrate route buckets are still non-empty. The machine-readable source
of truth is `configs/legacy-fastapi-route-classification.json`; this document is
the human-readable companion.

DIFF-120 migrated these dynamic page controls to Rust-native handlers:

- `POST /analysis/patterns/{pattern_id}/review`
- `POST /approvals/{approval_id}/decision`
- `POST /reports/{report_id}/render`
- `POST /work-items/{work_item_id}/dispatch`

`POST /work-items/{work_item_id}/dispatch` intentionally records a bounded
non-executing dispatch marker and audit event rather than invoking Celery from
the Rust HTTP gateway.

## Counts

| Bucket | Count | Meaning |
| --- | ---: | --- |
| `active_parity_required` | 11 | Intended product route that should become Rust-native in a scoped future DIFF. |
| `intentional_legacy_fallback` | 7 | Temporarily retained Python route with a documented retirement condition. |
| `retireable_unused` | 0 | No missing route is currently proven safe to remove solely as unused. |
| `duplicate_or_superseded` | 1 | Functionally covered by Rust health/status surfaces or otherwise superseded. |
| `unsafe_to_migrate_now` | 11 | High-risk route needing a dedicated parity DIFF before Rust migration. |

## Classification Matrix

| Method | Route | Python handler | Classification | Risk | Future DIFF |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | `services/api/app/main.py::root` | `duplicate_or_superseded` | low | DIFF-121 |
| GET | `/experiments` | `services/api/app/experiments.py::list_experiment_runs` | `intentional_legacy_fallback` | medium | DIFF-121 |
| GET | `/experiments/{experiment_run_id}` | `services/api/app/experiments.py::get_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-121 |
| GET | `/improvements` | `services/api/app/improvements.py::list_improvement_items` | `intentional_legacy_fallback` | medium | DIFF-121 |
| GET | `/improvements/{improvement_item_id}` | `services/api/app/improvements.py::get_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-121 |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | `services/api/app/graph_memory.py::get_node_relationships` | `unsafe_to_migrate_now` | high | DIFF-122 |
| GET | `/retrieval/chunks/{chunk_id}/trail` | `services/api/app/retrieval.py::get_chunk_retrieval_trail` | `active_parity_required` | medium | DIFF-121 |
| POST | `/analysis/hypotheses` | `services/api/app/analysis.py::create_hypothesis` | `active_parity_required` | medium | DIFF-121 |
| POST | `/analysis/predictions` | `services/api/app/analysis.py::create_prediction` | `active_parity_required` | medium | DIFF-121 |
| POST | `/analysis/recommendations` | `services/api/app/analysis.py::create_recommendation` | `active_parity_required` | medium | DIFF-121 |
| POST | `/artifacts` | `services/api/app/artifacts.py::create_raw_artifact` | `unsafe_to_migrate_now` | high | DIFF-124 |
| POST | `/collection-runs` | `services/api/app/collection_runs.py::create_collection_run` | `unsafe_to_migrate_now` | high | DIFF-124 |
| POST | `/collection-runs/local-project` | `services/api/app/collection_runs.py::create_local_project_collection` | `unsafe_to_migrate_now` | high | DIFF-124 |
| POST | `/collection-runs/manual-upload/ingest` | `services/api/app/collection_runs.py::ingest_manual_upload_collection` | `unsafe_to_migrate_now` | high | DIFF-124 |
| POST | `/evidence/documents` | `services/api/app/evidence.py::create_document` | `active_parity_required` | medium | DIFF-121 |
| POST | `/evidence/documents/{document_id}/chunks` | `services/api/app/evidence.py::generate_document_chunks` | `active_parity_required` | medium | DIFF-121 |
| POST | `/evidence/items` | `services/api/app/evidence.py::create_evidence_item` | `active_parity_required` | medium | DIFF-121 |
| POST | `/experiments` | `services/api/app/experiments.py::create_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-121 |
| POST | `/experiments/{experiment_run_id}/status` | `services/api/app/experiments.py::update_experiment_run_status` | `intentional_legacy_fallback` | medium | DIFF-121 |
| POST | `/improvements` | `services/api/app/improvements.py::create_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-121 |
| POST | `/memory/graph/lineage/sync` | `services/api/app/graph_memory.py::sync_lineage_graph` | `unsafe_to_migrate_now` | high | DIFF-122 |
| POST | `/memory/graph/schema/ensure` | `services/api/app/graph_memory.py::ensure_graph_schema` | `unsafe_to_migrate_now` | high | DIFF-122 |
| POST | `/memory/vector/chunks/ensure` | `services/api/app/vector_memory.py::ensure_chunk_vector_collection` | `unsafe_to_migrate_now` | high | DIFF-122 |
| POST | `/memory/vector/chunks/search` | `services/api/app/vector_memory.py::search_chunk_vector_points` | `unsafe_to_migrate_now` | high | DIFF-122 |
| POST | `/memory/vector/chunks/upsert` | `services/api/app/vector_memory.py::upsert_chunk_vector_points` | `unsafe_to_migrate_now` | high | DIFF-122 |
| POST | `/reports/{report_id}/status` | `services/api/app/reports.py::update_report_status` | `active_parity_required` | medium | DIFF-121 |
| POST | `/reports/{report_id}/work-item` | `services/api/app/reports.py::create_report_work_item` | `unsafe_to_migrate_now` | high | DIFF-123 |
| POST | `/retrieval/chunks/search` | `services/api/app/retrieval.py::search_retrieval_chunks` | `active_parity_required` | medium | DIFF-121 |
| POST | `/sources/{source_id}/permissions` | `services/api/app/sources.py::create_source_permission` | `active_parity_required` | medium | DIFF-121 |
| POST | `/work-items/{work_item_id}/status` | `services/api/app/work_items.py::update_work_item_status` | `active_parity_required` | medium | DIFF-121 |

## Final Posture

- Rust is primary for all routes counted as web-used by the route parity guard.
- FastAPI is still required for classified non-web fallback routes.
- Rust-only cannot honestly be claimed.
- Future DIFFs should migrate or retire routes by bucket, starting with
  `active_parity_required` routes that have medium risk and no external runtime
  service mutation.
