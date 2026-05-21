# DIFF-136 Non-Web FastAPI Route Classification

Date: 2026-05-20

## Summary

DIFF-136 migrates the experiments and improvements FastAPI fallback routes to
Rust-native gateway handling. The 1 route still missing from Rust is classified
below. The web route guard remains at
`web_routes_requiring_fallback=0`, so Rust is primary for web-used traffic, but
the repository is not Rust-only.

FastAPI remains required because the duplicate/superseded root route is still
unresolved until DIFF-137. The machine-readable source of truth is
`configs/legacy-fastapi-route-classification.json`; this document is the
human-readable companion.

DIFF-133 migrated these graph/vector memory routes to Rust-native handlers:

- `GET /memory/graph/nodes/{node_label}/{node_id}/relationships`
- `POST /memory/graph/lineage/sync`
- `POST /memory/graph/schema/ensure`
- `POST /memory/vector/chunks/ensure`
- `POST /memory/vector/chunks/search`
- `POST /memory/vector/chunks/upsert`

DIFF-134 migrated this report/work-item route to Rust-native handling:

- `POST /reports/{report_id}/work-item`

DIFF-135 migrated these artifact and collection ingestion routes to Rust-native
handling:

- `POST /artifacts`
- `POST /collection-runs`
- `POST /collection-runs/local-project`
- `POST /collection-runs/manual-upload/ingest`

DIFF-136 decision: migrate the experiments and improvements route family to
Rust.

DIFF-136 migrated these experiments and improvements routes to Rust-native
handling:

- `GET /experiments`
- `GET /experiments/{experiment_run_id}`
- `POST /experiments`
- `POST /experiments/{experiment_run_id}/status`
- `GET /improvements`
- `GET /improvements/{improvement_item_id}`
- `POST /improvements`

## Counts

| Bucket | Count | Meaning |
| --- | ---: | --- |
| `active_parity_required` | 0 | No currently classified missing route is in the active medium-risk parity bucket. |
| `intentional_legacy_fallback` | 0 | No remaining route is intentionally retained as FastAPI fallback after DIFF-136. |
| `retireable_unused` | 0 | No missing route is currently proven safe to remove solely as unused. |
| `duplicate_or_superseded` | 1 | Functionally covered by Rust health/status surfaces or otherwise superseded. |
| `unsafe_to_migrate_now` | 0 | No currently classified missing route remains in the high-risk artifact/collection bucket. |

## Classification Matrix

| Method | Route | Python handler | Classification | Risk | Future DIFF |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | `services/api/app/main.py::root` | `duplicate_or_superseded` | low | DIFF-137 |
## Final Posture

- Rust is primary for all routes counted as web-used by the route parity guard.
- FastAPI is still required until the duplicate/superseded root route is
  resolved in DIFF-137 and DIFF-138 evaluates fallback readiness.
- Rust-only cannot honestly be claimed.
- The experiments and improvements route family is complete after DIFF-136.
