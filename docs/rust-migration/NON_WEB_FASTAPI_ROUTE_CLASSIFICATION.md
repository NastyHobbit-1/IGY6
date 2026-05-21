# DIFF-137 Non-Web FastAPI Route Classification

Date: 2026-05-20

## Summary

DIFF-137 migrates the duplicate/superseded FastAPI root route to Rust-native
gateway handling. There are 0 FastAPI routes still missing from Rust. The web
route guard remains at
`web_routes_requiring_fallback=0`, so Rust is primary for web-used traffic, but
the repository is not Rust-only.

FastAPI fallback remains configured until DIFF-138 performs the explicit
fallback readiness decision. The machine-readable source of truth is
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

DIFF-137 decision: migrate the duplicate/superseded root route to Rust.

DIFF-137 migrated this route to Rust-native handling:

- `GET /`

The Rust root identity, `/health/live`, `/health/ready`, and
`/rust-migration/status` supersede the old FastAPI scaffold root response.

## Counts

| Bucket | Count | Meaning |
| --- | ---: | --- |
| `active_parity_required` | 0 | No currently classified missing route is in the active medium-risk parity bucket. |
| `intentional_legacy_fallback` | 0 | No remaining route is intentionally retained as FastAPI fallback after DIFF-137. |
| `retireable_unused` | 0 | No missing route is currently proven safe to remove solely as unused. |
| `duplicate_or_superseded` | 0 | No missing duplicate/superseded route remains after DIFF-137. |
| `unsafe_to_migrate_now` | 0 | No currently classified missing route remains in the high-risk artifact/collection bucket. |

## Classification Matrix

| Method | Route | Python handler | Classification | Risk | Future DIFF |
| --- | --- | --- | --- | --- | --- |
| _none_ | _none_ | _none_ | _none_ | _none_ | _none_ |
## Final Posture

- Rust is primary for all routes counted as web-used by the route parity guard.
- FastAPI fallback remains configured until DIFF-138 evaluates fallback
  readiness and decides whether it can be removed.
- Rust-only cannot honestly be claimed.
- No FastAPI routes are missing from Rust after DIFF-137.
