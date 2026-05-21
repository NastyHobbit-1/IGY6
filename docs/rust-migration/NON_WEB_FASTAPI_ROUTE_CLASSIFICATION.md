# DIFF-134 Non-Web FastAPI Route Classification

Date: 2026-05-20

## Summary

DIFF-134 migrates the report work-item FastAPI fallback route to Rust-native
gateway handling. The 12 routes still missing from Rust are
classified below. The web route guard remains at
`web_routes_requiring_fallback=0`, so Rust is primary for web-used traffic, but
the repository is not Rust-only.

FastAPI remains required because intentional legacy fallback and
unsafe-to-migrate route buckets are still non-empty. The machine-readable source
of truth is `configs/legacy-fastapi-route-classification.json`; this document is
the human-readable companion.

DIFF-133 migrated these graph/vector memory routes to Rust-native handlers:

- `GET /memory/graph/nodes/{node_label}/{node_id}/relationships`
- `POST /memory/graph/lineage/sync`
- `POST /memory/graph/schema/ensure`
- `POST /memory/vector/chunks/ensure`
- `POST /memory/vector/chunks/search`
- `POST /memory/vector/chunks/upsert`

DIFF-134 migrated this report/work-item route to Rust-native handling:

- `POST /reports/{report_id}/work-item`

## Counts

| Bucket | Count | Meaning |
| --- | ---: | --- |
| `active_parity_required` | 0 | No currently classified missing route is in the active medium-risk parity bucket. |
| `intentional_legacy_fallback` | 7 | Temporarily retained Python route with a documented retirement condition. |
| `retireable_unused` | 0 | No missing route is currently proven safe to remove solely as unused. |
| `duplicate_or_superseded` | 1 | Functionally covered by Rust health/status surfaces or otherwise superseded. |
| `unsafe_to_migrate_now` | 4 | High-risk route needing a dedicated parity DIFF before Rust migration. |

## Classification Matrix

| Method | Route | Python handler | Classification | Risk | Future DIFF |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | `services/api/app/main.py::root` | `duplicate_or_superseded` | low | DIFF-137 |
| GET | `/experiments` | `services/api/app/experiments.py::list_experiment_runs` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/experiments/{experiment_run_id}` | `services/api/app/experiments.py::get_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/improvements` | `services/api/app/improvements.py::list_improvement_items` | `intentional_legacy_fallback` | medium | DIFF-136 |
| GET | `/improvements/{improvement_item_id}` | `services/api/app/improvements.py::get_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/artifacts` | `services/api/app/artifacts.py::create_raw_artifact` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs` | `services/api/app/collection_runs.py::create_collection_run` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs/local-project` | `services/api/app/collection_runs.py::create_local_project_collection` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/collection-runs/manual-upload/ingest` | `services/api/app/collection_runs.py::ingest_manual_upload_collection` | `unsafe_to_migrate_now` | high | DIFF-135 |
| POST | `/experiments` | `services/api/app/experiments.py::create_experiment_run` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/experiments/{experiment_run_id}/status` | `services/api/app/experiments.py::update_experiment_run_status` | `intentional_legacy_fallback` | medium | DIFF-136 |
| POST | `/improvements` | `services/api/app/improvements.py::create_improvement_item` | `intentional_legacy_fallback` | medium | DIFF-136 |
## Final Posture

- Rust is primary for all routes counted as web-used by the route parity guard.
- FastAPI is still required for classified non-web fallback routes.
- Rust-only cannot honestly be claimed.
- The report work-item bucket is complete after DIFF-134; future DIFFs should
  migrate or retire the remaining route buckets by risk, starting with artifact
  and collection ingestion parity scoped for DIFF-135.
