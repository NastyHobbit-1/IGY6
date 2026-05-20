# DIFF-132 Non-Web FastAPI Route Classification

Date: 2026-05-20

## Summary

DIFF-132 migrates the 11 `active_parity_required` medium-risk FastAPI fallback
routes to Rust-native gateway handlers. The 19 routes still missing from Rust
are classified below. The web route guard remains at
`web_routes_requiring_fallback=0`, so Rust is primary for web-used traffic, but
the repository is not Rust-only.

FastAPI remains required because intentional legacy fallback and
unsafe-to-migrate route buckets are still non-empty. The machine-readable source
of truth is `configs/legacy-fastapi-route-classification.json`; this document is
the human-readable companion.

DIFF-132 migrated these active medium-risk routes to Rust-native handlers:

- `GET /retrieval/chunks/{chunk_id}/trail`
- `POST /analysis/hypotheses`
- `POST /analysis/predictions`
- `POST /analysis/recommendations`
- `POST /evidence/documents`
- `POST /evidence/documents/{document_id}/chunks`
- `POST /evidence/items`
- `POST /reports/{report_id}/status`
- `POST /retrieval/chunks/search`
- `POST /sources/{source_id}/permissions`
- `POST /work-items/{work_item_id}/status`

## Counts

| Bucket | Count | Meaning |
| --- | ---: | --- |
| `active_parity_required` | 0 | No currently classified missing route is in the active medium-risk parity bucket. |
| `intentional_legacy_fallback` | 7 | Temporarily retained Python route with a documented retirement condition. |
| `retireable_unused` | 0 | No missing route is currently proven safe to remove solely as unused. |
| `duplicate_or_superseded` | 1 | Functionally covered by Rust health/status surfaces or otherwise superseded. |
| `unsafe_to_migrate_now` | 11 | High-risk route needing a dedicated parity DIFF before Rust migration. |

## Classification Matrix

| Method | Route | Python handler | Classification | Risk | Future DIFF |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | `services/api/app/main.py::root` | `duplicate_or_superseded` | low | DIFF-137 |
| GET | `/experiments` | `services/api/app/experiments.py::list_experiment_runs` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/experiments/{experiment_run_id}` | `services/api/app/experiments.py::get_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/improvements` | `services/api/app/improvements.py::list_improvement_items` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/improvements/{improvement_item_id}` | `services/api/app/improvements.py::get_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/memory/graph/nodes/{node_label}/{node_id}/relationships` | `services/api/app/graph_memory.py::get_node_relationships` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/artifacts` | `services/api/app/artifacts.py::create_raw_artifact` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs` | `services/api/app/collection_runs.py::create_collection_run` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs/local-project` | `services/api/app/collection_runs.py::create_local_project_collection` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs/manual-upload/ingest` | `services/api/app/collection_runs.py::ingest_manual_upload_collection` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/experiments` | `services/api/app/experiments.py::create_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/experiments/{experiment_run_id}/status` | `services/api/app/experiments.py::update_experiment_run_status` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/improvements` | `services/api/app/improvements.py::create_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/memory/graph/lineage/sync` | `services/api/app/graph_memory.py::sync_lineage_graph` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/memory/graph/schema/ensure` | `services/api/app/graph_memory.py::ensure_graph_schema` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/memory/vector/chunks/ensure` | `services/api/app/vector_memory.py::ensure_chunk_vector_collection` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/memory/vector/chunks/search` | `services/api/app/vector_memory.py::search_chunk_vector_points` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/memory/vector/chunks/upsert` | `services/api/app/vector_memory.py::upsert_chunk_vector_points` | `unsafe_to_migrate_now` | high | DIFF-133 |
| POST | `/reports/{report_id}/work-item` | `services/api/app/reports.py::create_report_work_item` | `unsafe_to_migrate_now` | high | DIFF-134 |

## Final Posture

- Rust is primary for all routes counted as web-used by the route parity guard.
- FastAPI is still required for classified non-web fallback routes.
- Rust-only cannot honestly be claimed.
- The active medium-risk bucket is empty after DIFF-132; future DIFFs should
  migrate or retire the remaining route buckets by risk, starting with the
  graph/vector memory parity work scoped for DIFF-133.
