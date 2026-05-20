# Rust Completion Manager Prompt

Use this prompt with Codex or another coding agent after reading `AGENTS.md`, `docs/agents/AGENT_PROMPT.md`, and `docs/agents/AGENT_PROMPT_CODING.md`.

## Mission

Finish the remaining Rust migration work safely in sequential DIFFs until FastAPI fallback is no longer required, or until every remaining FastAPI route is explicitly retired or documented as intentionally retained.

## Current Baseline

DIFF-131 is complete and locked. IGY6 is Rust-primary, not Rust-only. Web-used routes should not require FastAPI fallback, but non-web FastAPI fallback is still required until route parity and manifest state prove otherwise. Do not claim Rust-only operation unless it is factually true.

## Required First Reads

- `AGENTS.md`
- `docs/agents/AGENT_PROMPT.md`
- `docs/agents/AGENT_PROMPT_CODING.md`
- `configs/rust-cutover-manifest.json`
- `configs/legacy-fastapi-route-classification.json`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`
- `docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md`
- `docs/diffs/DIFF-131-local-llm-task-routing.md`

## Global Rules

Work one DIFF at a time. Do not edit locked DIFFs. Do not combine unrelated route buckets. Do not perform broad refactors or unrelated cleanup. Keep the repo runnable after every DIFF. Do not mutate `.env`. Do not touch runtime/private data under `IGY6_DATA_ROOT`. Do not add cloud providers, credentials, or secrets. Do not remove Python/FastAPI fallback until route parity, docs, and manifest state prove it is safe.

## DIFF-132: Active Medium-Risk Route Parity

Migrate only the `active_parity_required` routes:

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

Preserve request/response contracts, audit behavior, approval checks, permission checks, and status-transition checks. Add Rust tests for validation, success, not found, invalid state, and audit behavior where applicable. Update route classification, manifest, and route audit docs. Do not remove FastAPI fallback or claim Rust-only.

## DIFF-133: Graph And Vector Memory Route Parity

Migrate only:

- `GET /memory/graph/nodes/{node_label}/{node_id}/relationships`
- `POST /memory/graph/lineage/sync`
- `POST /memory/graph/schema/ensure`
- `POST /memory/vector/chunks/ensure`
- `POST /memory/vector/chunks/search`
- `POST /memory/vector/chunks/upsert`

Preserve missing-service behavior, missing-collection behavior, vector-size/config validation, and graph schema safety. Use bounded service calls and fake clients or planning layers in tests where possible. Do not migrate artifact or collection ingestion here.

## DIFF-134: Report Work-Item Route Parity

Migrate or safely redesign:

- `POST /reports/{report_id}/work-item`

Preserve work-item creation semantics, report/audit expectations, and approval/permission checks where present. Prefer bounded Rust planning plus DB/audit records. Do not broaden into artifact or collection ingestion.

## DIFF-135: Artifact And Collection Ingestion Route Parity

Migrate only:

- `POST /artifacts`
- `POST /collection-runs`
- `POST /collection-runs/local-project`
- `POST /collection-runs/manual-upload/ingest`

Preserve artifact storage safety, `IGY6_DATA_ROOT` boundaries, content-addressing/dedup behavior, source permission checks, approval checks, audit events, and bounded input behavior. Do not add PDF/image/audio parsing unless required by route parity.

## DIFF-136: Experiments And Improvements Fallback Resolution

Resolve:

- `GET /experiments`
- `GET /experiments/{experiment_run_id}`
- `POST /experiments`
- `POST /experiments/{experiment_run_id}/status`
- `GET /improvements`
- `GET /improvements/{improvement_item_id}`
- `POST /improvements`

Choose one explicit family decision: migrate to Rust, retire as unused/deprecated, or keep as intentional fallback with a documented reason and retirement condition. Do not leave the decision implicit.

## DIFF-137: Duplicate Root Route Resolution

Resolve:

- `GET /`

Confirm whether Rust health/status/root behavior supersedes it. Retire or migrate with minimal scope. Update route classification, parity guard, manifest, and route audit docs.

## DIFF-138: FastAPI Fallback Readiness Gate

Determine whether fallback can be removed. If parity/retirement is incomplete, do not remove FastAPI; document exact blockers and keep `fastapi_fallback_required=true`. If safe, set `fastapi_fallback_required=false`, update operational status honestly, update README/docs, remove `legacy-api` fallback wiring from Compose, and remove fallback proxy behavior only if no longer needed.

## DIFF-139: Legacy Python Archive Plan Or Execution

Only after fallback removal is proven safe, archive or preserve legacy Python components honestly. Archive `services/api/` only if no longer required. Archive `services/worker/` only if Rust worker execution parity exists and is verified. Preserve governance files and locked DIFF history.

## DIFF-140: Final Rust Completion Audit

Produce the final audit. State either:

A. Rust-only complete: no FastAPI fallback required, all active API routes are Rust-native or retired, Compose has no `legacy-api` fallback, and the manifest says `fastapi_fallback_required=false`.

B. Rust-primary with documented remaining Python: name the exact Python service still required, why it remains, and the future DIFF needed.

## Required Verification For Each DIFF

Run the active DIFF verification and, unless clearly inapplicable, include:

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json` when present or touched
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `python3 scripts/rust-route-parity.py --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build` when web-facing behavior or contracts change
- `npm --prefix apps/web run test:ui-smoke` when UI workflow/status text changes and the script is available

## Required Report After Each DIFF

Report:

- Active DIFF number and status.
- Commit SHA if committed.
- Files changed.
- Routes migrated, retired, or intentionally kept.
- Verification commands and results.
- Remaining FastAPI fallback count.
- `fastapi_fallback_required` value.
- Whether Rust-only operation is claimed.
- Next recommended DIFF.

Unless Rust-only is fully proven, include: `IGY6 remains Rust-primary with required FastAPI fallback. Rust-only is not claimed.`
