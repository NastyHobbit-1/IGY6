# IGY6 Product Completion Roadmap And Gap Audit

DIFF: DIFF-178

Status: current-state roadmap and gap audit for `dev`.

## 1. Current Repo And Runtime Summary

IGY6 currently presents as a local-first evidence and decision-support workspace, not a generic chatbot or simple RAG demo. The active application runtime is Rust API gateway, Rust worker daemon, Next.js web UI, PostgreSQL, Qdrant, Neo4j, MLflow, and Phoenix. The legacy Python/FastAPI API and Python/Celery worker are archived under `archive/legacy-python/` and are not active runtime services. Base Docker Compose builds `api` from `crates/igy6-gateway/Dockerfile`, builds `worker` from `crates/igy6-worker/Dockerfile`, runs the Rust worker with `igy6-worker --daemon`, and defines no `legacy-api`, Python/Celery worker, or Celery beat service.

The strongest implemented product path is UTF-8 text-oriented ingestion and processing. The current path can register sources and permissions, create collection/dry-run/upload records, store content-addressed artifacts, normalize UTF-8 text, chunk documents, create evidence items, upsert deterministic local hash vectors to Qdrant, inspect local records, ask over retrieved evidence, build deterministic evidence answer packets, optionally call a local Ollama provider when configured, record approvals, feedback, outcomes, reports, audit events, improvement metadata, and experiment metadata.

The current implementation is not yet the full adaptive-intelligence product. Binary PDF/image/audio/video parsing is not complete. Web/router/browser collectors are not implemented as usable read-only collectors. Graph memory exists as schema/lineage/relationship surfaces, but full graph reasoning and correlation discovery are not complete. Pattern detection is baseline and record-oriented. Prediction, recommendation, outcome, improvement, and experiment records exist, but forecasting, method optimization, MLflow/Optuna execution, Phoenix tracing integration, and production method-change approval loops are not complete.

## 2. Current User-Facing Workflow Summary

The normal UI is a tabbed dashboard with Home, Add Data, Work, Results, Settings, and Advanced. Home summarizes readiness, counts, recent activity, and next actions. Add Data explains the information lifecycle and points users toward source registration and text-oriented upload. Work shows queued/running/completed/failed work items and processing state. Results exposes assistant retrieval, evidence/document/chunk records, memory/finding records, reports, and local LLM status. Settings exposes safety, approvals, feedback, outcomes, audit events, local-first model policy, and dry-run-gated `.env` settings. Advanced contains low-level route controls for source creation, approvals, dry-runs, manual upload, work dispatch, evidence answer, feedback/outcome/pattern review, pattern detection, and report creation/rendering.

The practical current workflow is:

1. Start the local stack outside this DIFF with `scripts/run.sh`.
2. Open the web UI at `WEB_BASE_URL` from `.env` (default `http://127.0.0.1:3000`).
3. Check Home readiness.
4. Register a source and permission, usually `manual_upload` or `local_project`.
5. Create approval when required by the source permission.
6. Collect UTF-8 text by manual upload or supported local project collection path.
7. Let the Rust worker daemon process queued normalization, chunking, evidence, and vector work.
8. Inspect Work for failures and Results for evidence, retrieval, reports, and records.
9. Record feedback or outcomes through review controls.
10. Use Advanced only for raw API-backed operations and troubleshooting.

This workflow is real but still uneven: several normal-user actions are described in the UI while their forms remain in Advanced, and the user must understand source IDs, permission IDs, and approval IDs for some operations.

## 3. Completed Capability List

- Rust-only application runtime posture is complete: Rust gateway API and Rust worker daemon are active; legacy Python/FastAPI and Python/Celery are archived/inactive.
- Docker Compose local runtime defines PostgreSQL, Qdrant, Neo4j, MLflow, Phoenix, Rust API, Rust worker, and web UI with localhost-bound ports.
- Route parity and fallback removal are complete for the former FastAPI surface according to the manifest and Rust gateway route registry.
- Non-destructive runtime validation scripts exist for post-cutover smoke, fresh-clone startup checks, and lifecycle command-shape checks.
- Source records and source permission records can be created and read.
- Approval records can be created, decided, listed, and audited.
- Collection dry-runs and collection run records exist.
- Manual upload and local-project collection routes exist for supported text-oriented inputs.
- Content-addressed artifact storage exists with SHA-256 layout, bounded path checks, and hash verification.
- UTF-8 text normalization exists with source/raw-artifact lineage metadata.
- Deterministic chunk planning exists with bounded chunk sizes and evidence item planning.
- Rust worker daemon can claim supported queued work and execute collection normalization, document chunking, and chunk vector upsert.
- Qdrant deterministic local hash vector planning, collection ensure, point upsert, and search request behavior exist.
- Evidence/document/chunk/claim read surfaces exist.
- Retrieval preview and hydrated chunk trails exist for local evidence search.
- Deterministic evidence answer packets separate facts, assumptions, inferences, uncertainty, missing information, citations, and source trails.
- Optional local Ollama routing/config support exists, with deterministic fallback when disabled, unavailable, or insufficiently evidenced.
- Baseline pattern record creation, baseline detection, and pattern review routes exist.
- Hypothesis, prediction, recommendation, feedback, outcome, improvement item, experiment run, report, work item, audit, and settings routes exist.
- Feedback can apply limited source trust side effects, and outcome records can update supported target statuses.
- Report creation, status updates, report work-item creation, and markdown artifact rendering exist.
- Agent request understanding and action classification exist with clarification/approval posture, fixed action registry, dangerous-pattern rejection, and bounded local actions.
- Settings `.env` inspect/verify/apply flow is dry-run gated and records audit events on apply.
- UI documentation describes the current tabbed normal-user interface and current limitations.

## 4. Partial Capability List

- Normal-user data add flow is partial because core source/upload controls still rely on Advanced route forms for exact IDs.
- Source type support is partial. `manual_upload` and `local_project` are useful; `user_observation` and `conversation_history` are source metadata categories without complete specialized ingestion/review workflows; router, web, and PC diagnostic types are planned.
- Permission policy is partial. Source permissions, allowed operations, approval requirements, external model policy, and audit records exist, but full central policy middleware across every sensitive workflow is not complete.
- Evidence-backed chat is partial. Retrieval preview and evidence answer packets exist, but persisted chat sessions, conversational memory, richer answer review controls, and graph-assisted reasoning are incomplete.
- Local LLM support is partial. Ollama routing can be configured, but local model setup, health, prompt tracing, and evaluation are not full product workflows.
- Graph memory is partial. Neo4j service, schema, lineage sync, and relationship inspection exist, but complete entity/claim/event extraction and advanced graph reasoning are not done.
- Pattern detection is partial. Baseline recurrence/gap/agreement-style records exist, but the full required pattern set, statistical validation, conflict/drift/anomaly detection, and review UX are incomplete.
- Predictions and recommendations are partial. Records and review/outcome routes exist, but automatic evidence-backed prediction generation, forecasting, calibration, and recommendation workflows are incomplete.
- Feedback/outcome learning is partial. Records and some side effects exist, but the system does not yet reliably change future reasoning behavior based on prior failed advice or successful methods.
- Self-improvement is partial. Improvement item and experiment run metadata exist, but no complete Optuna/MLflow/Phoenix experiment runner, baseline comparison, report, or production method approval loop exists.
- Reports are partial. Report metadata and render artifacts exist, but report templates, user-facing report authoring flow, export bundles, and decision-ready report formats are incomplete.
- Runtime operations are partial. Start/stop/restart scripts and validation exist, but backup/restore, diagnostics bundles, live recovery, performance tests, and long-term operations hardening are incomplete.
- UI completion is partial. The tab structure is aligned with normal use, but several controls remain low-level, some copy or proxy errors still contain stale FastAPI wording, and end-to-end guided workflows need polish.

## 5. Missing Capability List

- Complete guided source onboarding for every required source type.
- Browser/web/router collectors with domain scope enforcement, read-only default, screenshots, page text extraction, and approval-controlled write prevention.
- PC diagnostics import workflow for authorized diagnostic exports.
- Conversation history import workflow that preserves prior user intent, corrections, and decision context.
- Binary PDF parsing, image OCR, audio/video transcription, and media-specific sensitivity handling.
- Full connector contract implementation for validate_scope, dry_run, collect, normalize, classify_sensitivity, extract_metadata, and cleanup across all source types.
- Full source trust/noisy/sensitive/disabled management UX and policy effects.
- Full immutable evidence correction/supersession model in the user workflow.
- Complete entity, claim, event, and relationship extraction into Neo4j.
- Full graph traversal explanations that answer why records are connected.
- Complete evidence-backed chat with persisted conversations, answer feedback buttons, evidence cards, source disagreement display, and missing-information prompts.
- Complete pattern set: recurrence, temporal association, configuration drift, cross-source agreement, cross-source conflict, failed-advice recurrence, successful-method recurrence, anomaly, and missing-information gap.
- Automatic evidence-backed hypothesis, prediction, and recommendation generation with confidence, uncertainty, expected result, and disproof criteria.
- Forecasting baselines with scikit-learn and StatsForecast, plus NeuralForecast only after enough clean data exists and it beats simpler baselines.
- Outcome-driven learning that prevents repeated failed advice from being presented as new.
- Full self-improvement runner using controlled experiments, Optuna, MLflow, Phoenix traces, success criteria, artifacts, reports, and approval-gated accepted methods.
- NeMo Guardrails/OWASP-style prompt-injection and tool-use hardening as a complete integrated layer.
- Backup, restore, export, deletion, retention, and diagnostics bundle workflows.
- Product-level end-to-end smoke tests that start from a clean local stack and verify the normal user path through data add, processing, retrieval, report, feedback, and outcome.

## 6. Unsupported Capability Claims To Avoid

- Do not claim IGY6 is a full adaptive-intelligence system yet.
- Do not claim binary PDF, image, audio, or video parsing is complete.
- Do not claim browser/router/web collection is implemented or safe beyond planned/scaffolded posture.
- Do not claim the system can ingest arbitrary local PC data.
- Do not claim external accounts, websites, router pages, or browser sessions are collected without explicit scoped collector work.
- Do not claim graph memory performs full relationship reasoning or correlation discovery.
- Do not claim advanced forecasting, NeuralForecast, or scikit-learn modeling is active product behavior.
- Do not claim self-improvement can optimize methods and update production behavior.
- Do not claim MLflow, Phoenix, Optuna, DSPy, or NeMo Guardrails are fully integrated product workflows merely because services/configuration or planned records exist.
- Do not claim local LLM answers are available by default; deterministic evidence fallback is the reliable default.
- Do not claim recommendations are automatically executed.
- Do not claim the UI is fully guided for normal users; some route controls remain advanced and ID-driven.
- Do not claim every registered source type has a complete collector.
- Do not claim missing evidence means the underlying real-world information does not exist.
- Do not describe FastAPI, Python/Celery, or Celery beat as active runtime services.

## 7. Product Risks And Blockers

- Basic product use depends on users successfully navigating source/permission/approval IDs; this is a usability blocker.
- The current strongest ingestion path is UTF-8 text, limiting usefulness for common documents and screenshots.
- Some web API proxy error messages still refer to FastAPI, which can confuse runtime troubleshooting even though the active backend is Rust.
- Graph and pattern surfaces may look more complete than their reasoning depth actually is.
- Prediction/recommendation records exist before automatic prediction/recommendation workflows are complete, creating claim-risk if UI copy overstates capability.
- Self-improvement metadata exists without a real experiment runner, which risks implying autonomous improvement that is not implemented.
- External model policy is designed as local-first, but a full guardrail layer and trace/evaluation workflow are still missing.
- Backup/restore and deletion/export are not complete, which blocks long-term personal use with important data.
- End-to-end product workflow tests are not broad enough to prove normal-user completion from Add Data through Results and review.
- MLflow and Phoenix services run as infrastructure but are not yet integrated deeply enough to justify product claims around experiment tracking or tracing.
- Advanced controls expose low-level IDs and raw JSON, which can lead users into unsupported paths.

## 8. Ordered Next-DIFF Plan

1. Fix visible runtime wording drift and stale FastAPI proxy messages.
2. Build a guided manual text ingestion flow that hides raw IDs for normal users.
3. Add an end-to-end normal-user text ingestion smoke test.
4. Improve Work and error recovery UX around worker pipeline failures.
5. Complete evidence answer UX with citations, source trails, feedback controls, and persisted answer records.
6. Add report workflow UX for creating, rendering, reviewing, and exporting basic reports.
7. Harden source trust/noisy/sensitive/disabled management and policy effects.
8. Complete conversation history and user observation ingestion workflows.
9. Expand local project collection UX and safety validation.
10. Implement graph lineage explanation UX and entity/claim extraction foundations.
11. Expand baseline pattern detection to the required pattern categories.
12. Implement prediction/recommendation generation MVP with evidence, confidence, and disproof criteria.
13. Add outcome learning summaries that surface prior failed/successful advice.
14. Build self-improvement queue-to-experiment MVP with MLflow artifacts and approval-gated accepted methods.
15. Add backup/restore/export/delete hardening.

## 9. DIFF Count Estimate

Basic usable product: about 8 to 12 additional DIFFs. This means a user can reliably add UTF-8 text, watch processing, ask evidence-backed questions, review citations, generate a basic report, record feedback/outcomes, and recover from common errors without using raw IDs for the main path.

Solid local MVP: about 22 to 35 additional DIFFs. This adds guided source management, conversation/user observation ingestion, safer local project collection, richer evidence explorer, graph lineage inspection, complete baseline patterns, prediction/recommendation MVP, outcome learning summaries, report export, backup/restore, diagnostics, and meaningful product-level smoke tests.

Full adaptive-intelligence product: about 70 to 120 additional DIFFs. This covers all required source types, browser/router collectors, media parsing, graph extraction/reasoning, pattern/correlation/drift/anomaly detection, forecasting and ML baselines, self-improvement experiments with Optuna/MLflow/Phoenix, approval-gated method registry, advanced guardrails, long-term operations, exports/deletion, and hardening.

These are rough counts because the DIFF process intentionally keeps changes small and verifiable. Counts should be revised after each 10-DIFF block.

## 10. Recommended Next 15 DIFFs

### DIFF-179: Runtime Wording Drift And Proxy Error Cleanup

Scope: UI/API hygiene only. Replace stale FastAPI wording in Next.js API proxy error paths and any visible current-runtime copy that conflicts with Rust-only runtime truth. Do not change route behavior or backend contracts. Verification should include `npm --prefix apps/web run build`, `git diff --check`, and a scoped search proving no active UI/proxy text describes FastAPI as the live backend.

### DIFF-180: Guided Manual Text Source And Upload Flow

Scope: make the normal Add Data path create a manual text source, permission, approval when required, and upload without forcing the user into Advanced IDs. Keep binary parsing explicitly unsupported. Verification should use a mocked or running-stack safe test path and UI build.

### DIFF-181: End-To-End Manual Text Product Smoke

Scope: add a non-destructive or isolated smoke test for the normal manual UTF-8 path from source creation through upload, worker processing, evidence creation, vector upsert, and retrieval. It must not touch private runtime data by default and should use synthetic data.

### DIFF-182: Work Failure Recovery And Pipeline Status UX

Scope: improve user-facing Work status around queued/running/failed/completed tasks, including clear failure messages, related source/artifact/document links where available, and safe retry guidance. Avoid changing worker semantics unless a separate runtime DIFF scopes it.

### DIFF-183: Evidence Answer Review UX

Scope: make the Results answer experience show facts, assumptions, uncertainty, missing information, citations, and source trails in normal-user UI, with Useful, Wrong, Verified, Incomplete, Show Evidence, Record Outcome, and Send To Self-Improvement controls wired to existing routes where possible.

### DIFF-184: Persisted Answer And Chat Session Records

Scope: add persistent answer/chat records only if the data model already supports an appropriate table or a scoped migration is approved in the DIFF. The goal is to review prior answers, feedback, evidence used, and whether later outcomes contradicted the answer.

### DIFF-185: Basic Report Workflow UX And Export

Scope: move report create/render/review/export into a normal Results workflow. Reports should preserve evidence boundaries and avoid unsupported intelligence claims. Export should stay local and content-addressed.

### DIFF-186: Source Trust And Sensitivity Management

Scope: add normal-user controls for marking sources trusted, noisy, sensitive, disabled, or enabled, and document/verify the policy effects. Keep historical evidence immutable and represent changes as new records/audit events.

### DIFF-187: Conversation History And User Observation Intake

Scope: implement guided intake for user observations and explicitly authorized conversation history exports. Preserve prior intent, corrections, decisions, and source sensitivity. Do not scrape live chat accounts or browser state.

### DIFF-188: Local Project Collection Guided Flow

Scope: make local project collection usable without raw JSON by guiding source location, scoped paths, dry-run preview, approval, and collection. Keep path traversal protections and binary limitations visible.

### DIFF-189: Graph Lineage Explanation MVP

Scope: expand graph sync/retrieval into a user-facing explanation of how sources, artifacts, documents, chunks, evidence, reports, and analysis records are connected. This is lineage explanation, not full graph reasoning.

### DIFF-190: Entity And Claim Extraction Foundation

Scope: add deterministic or local-only entity/claim extraction from normalized text into structured records and graph relationships. Keep extraction confidence explicit and do not treat extracted claims as verified facts.

### DIFF-191: Required Pattern Detector Expansion

Scope: expand pattern detection toward recurrence, missing-information gaps, cross-source agreement, cross-source conflict, temporal association, drift, and anomaly candidates. Each pattern must cite evidence, confidence, missing information, and review status.

### DIFF-192: Prediction And Recommendation MVP

Scope: create evidence-backed prediction and recommendation generation for narrow supported cases. Each output must include evidence, confidence, uncertainty, expected result, disproof criteria, risk, approval requirement, and outcome target.

### DIFF-193: Outcome Learning Summary

Scope: summarize prior outcomes and feedback so the system can surface repeated failed advice, successful methods, and weak retrieval/reporting areas. This should inform future review and improvement records without silently changing production behavior.

## 11. Verification Commands Used

Read-only inspection commands used before writing:

```bash
git status --short
git branch --show-current
git log --oneline --decorate -5
git diff --stat
rg --files docs/diffs
rg --files docs/agents
rg --files docs/plans
sed -n '1,260p' AGENTS.md
sed -n '1,260p' docs/agents/CODEX_PROMPT_BASELINE.md
sed -n '1,260p' docs/BRANCH_POLICY.md
sed -n '1,260p' docs/diffs/DIFF-178-product-completion-roadmap-gap-audit.md
sed -n '1,260p' README.md
sed -n '261,520p' README.md
sed -n '1,620p' docs/ui/README.md
sed -n '1,1460p' configs/rust-cutover-manifest.json
sed -n '1,260p' infra/docker-compose.yml
find apps/web/src/app/api -maxdepth 4 -type f | sort
sed -n '1,260p' apps/web/src/app/page.tsx
sed -n '260,620p' apps/web/src/app/page.tsx
sed -n '1,220p' apps/web/src/app/api/agent/intent/route.ts
sed -n '1,220p' apps/web/src/app/api/agent/capabilities/route.ts
sed -n '1,220p' apps/web/src/app/api/chat/retrieval-preview/route.ts
sed -n '1,220p' apps/web/src/app/api/approvals/route.ts
sed -n '1,220p' apps/web/src/app/api/settings/env/route.ts
sed -n '1,220p' apps/web/src/app/api/settings/env/verify/route.ts
sed -n '1,220p' apps/web/src/app/api/settings/env/apply/route.ts
find crates/igy6-gateway crates/igy6-worker crates/igy6-agent-api crates/igy6-evidence-answer crates/igy6-llm crates/igy6-artifacts crates/igy6-normalization crates/igy6-chunking crates/igy6-vector-memory -maxdepth 3 -type f | sort
sed -n '1,260p' crates/igy6-gateway/src/lib.rs
sed -n '1,260p' crates/igy6-worker/src/lib.rs
sed -n '1,260p' crates/igy6-agent-api/src/lib.rs
sed -n '1,260p' crates/igy6-evidence-answer/src/lib.rs
sed -n '1,240p' crates/igy6-llm/src/lib.rs
sed -n '1,220p' crates/igy6-artifacts/src/lib.rs
sed -n '1,220p' crates/igy6-normalization/src/lib.rs
sed -n '1,240p' crates/igy6-chunking/src/lib.rs
sed -n '1,280p' crates/igy6-vector-memory/src/lib.rs
sed -n '1,260p' docs/diffs/DIFF_PROCESS.md
sed -n '1,220p' docs/diffs/DIFF_TEMPLATE.md
sed -n '1,260p' docs/plans/IGY6_FULL_PROJECT_COMPLETION_PLAN.md
sed -n '1,240p' docs/runtime/PROCESSING_STATUS.md
rg "fn handle|match \\(|route|RUST_NATIVE_ROUTES|agent/intent|evidence-answer|detect-baseline|lineage|settings/env" crates/igy6-gateway/src/lib.rs
rg "Daemon|execute|Qdrant|Neo4j|CollectionNormalization|DocumentChunking|ChunkVectorUpsert|shutdown|claim" crates/igy6-worker/src/lib.rs
rg "Status:|## Result|complete|Status:" docs/diffs/DIFF-17*.md docs/diffs/DIFF-16*.md docs/diffs/DIFF-15*.md docs/diffs/DIFF-14*.md docs/diffs/DIFF-13*.md docs/diffs/DIFF-12*.md docs/diffs/DIFF-11*.md docs/diffs/DIFF-10*.md
```

Required final verification commands are recorded in DIFF-178 after they are run.

## 12. Open Questions Needing Owner Decision

- Should the next product priority be guided manual text usability, broader source types, or evidence-answer/chat polish?
- Should dev-only planning documents under `docs/plans/` remain dev-only, or should a sanitized product roadmap be created separately for `main`?
- Which source type should become the first non-text/manual source: conversation history, local project folder, PC diagnostics export, public web page, or router/network export?
- Should binary PDF/image OCR be prioritized before graph/prediction features because it affects everyday ingestion value?
- What is the minimum acceptable local MVP: evidence-backed text search/reporting, or must predictions and recommendations be present?
- Should Phoenix tracing be integrated before self-improvement experiments, or can the first experiment runner use MLflow-only tracking?
- What local LLM hardware target and default model should be supported first, if any?
- How strict should external model policy remain for non-sensitive public sources?
- What backup/restore guarantees are required before the user stores important personal records?
- Should source trust changes affect retrieval ranking immediately, or only show warnings until a later ranking DIFF?
- Should failed advice learning change future answer behavior automatically after approval, or only display warnings and improvement suggestions?
- What report formats are required first: markdown, HTML, PDF export, or zipped evidence bundle?
- How much graph reasoning is needed for MVP: lineage explanation only, or entity/event correlation queries?
- Should DIFF cadence stay very small, or can related UI/backend changes be grouped for workflow completion once tests are stronger?
