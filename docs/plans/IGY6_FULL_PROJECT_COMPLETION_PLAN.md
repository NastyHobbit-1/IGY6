# IGY6 Full Project Completion Plan

## Status

IGY6 has completed the Rust-only application API and worker runtime cutover (see DIFF-103, DIFF-140+ series and rust-migration/ artifacts).

**Track 2 — End-to-End Product Workflows: COMPLETED**
**Track 3 — UI Completion: COMPLETED**

Current runtime posture:

- Rust gateway API is active.
- Rust worker daemon is active.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.
- Runtime/private data remains outside the repo under `IGY6_DATA_ROOT`.
- Remaining non-Rust components are expected supporting/product components:
  - Next.js web
  - PostgreSQL
  - Redis (archived in active Compose per post-cutover)
  - Qdrant
  - Neo4j
  - MLflow
  - Phoenix

This plan covers post-cutover project completion, not Rust migration. All work on the `grok` development branch (per active branch policy for this clone; see root AGENTS.md and recent DIFF-24x/247 activity). No merges to main performed.

See also:
- `docs/runtime/IGY6_CAPABILITY_TRUTH_TABLE.md` (CAP-026, delivered under DIFF-246)
- `docs/diffs/DIFF-247-ENHANCEMENTS-2-10.md` (master tracker for enhancements 2-10, completed)
- `docs/diffs/DIFF-245*` and `DIFF-246*` (post-audit foundations and truth table)
- Recent `grok` commits: Mermaid architecture diagram (README), CLI clap refactor, license/README modernization, web component splits, and the full DIFF-240..247 chain.

### Track 2 — End-to-End Product Workflows

**Status: COMPLETED** (via chained DIFF-210+ / 24x series on grok + DIFF-247)

Delivered (key mappings; see individual DIFFs and truth table for details):

- Core text-oriented ingestion + evidence pipeline: manual upload, conversation history, user observations, local project, web sources (DIFF-210+, 213-214, 240+). Full flow: source -> artifact (content-addressed, kind detection via infer + pdf-extract for PDF text) -> normalize -> chunk -> vector (Qdrant) -> evidence-answer packet (facts/assumptions/inferences/uncertainty/missing info + citations/source trails) in `igy6-evidence-answer`.
- Agent/task/planner workflows: guided intake, approval-to-action, persisted plans, evidence-aware suggestions, outcome review, plan-to-work-item (many DIFF-197 to 212+).
- Self-improvement / experiment loop MVP: DIFF-242.
- Guardrails / tool-use / external model policy: DIFF-243.
- Data lifecycle hardening, permission/audit records, dry-run/approval helpers, SourceType extensions + collector contract foundations: DIFF-244, 246 (truth table + backend MVP).
- Graph: lineage persistence + review surface foundations (Neo4j schema, sync ops, GraphLineageExplanationPanel wired to full chain: sources/artifacts/docs/chunks/evidence/answers/reports). Earlier DIFF-032/033/054/062 series.
- Evidence answers, reports, feedback/outcome capture, source/evidence history, review UX: covered across 185-212+ DIFFs + retrieval/evidence-answer crates.
- Capability audit + honest truth table (CAP-026): DIFF-245 (planning/audit) + DIFF-246 (foundations + table). Many capabilities at "worker_runtime_behavior + tested" or "new_api + persistence" for text paths; explicit gaps documented for deep multimodal (no OCR/vision/audio transcription claimed), full browser exports, rich graph entity review, etc. (per product goal and baseline: text-oriented strongest path).
- Operator scripts, smoke verification, post-cutover audits, fresh-clone/runtime-lifecycle checks: extensive DIFF-122/123/168-195 series + scripts (ui-smoke, operator-smoke, post-cutover-*).

Criteria met per DIFFs: Real backend/API/persistence/runtime behavior for scoped text + media-metadata workflows; provenance/audit/approval gates present; evidence-grounded outputs; no overclaims on binary deep parsing.

### Track 3 — UI Completion

**Status: COMPLETED** (via DIFF-172/173/121+ and grok branch polish)

- Tabbed normal-user dashboard (Home / Add Data / Work / Results / Settings / Advanced) with honest empty states, no fake demo data.
- Collector flows, Media Library (full-res images/videos from source with original bytes viewer), Results/Evidence/Graph lineage views, Work status, Reports, User & Security (password + optional TOTP), Advanced diagnostics.
- README + `docs/ui/README.md` + architecture Mermaid diagram (integrated under DIFF-247 #9 from side-branch review) + WORKING.md alignment for grok branch.
- Dynamic ports, clear local URLs, password gate on protected actions.
- Rust API proxy routes kept aligned; plain CSS (no Tailwind per branch rules).
- Smoke coverage for UI interactions (DIFF-122 etc.).

Criteria met: No dead buttons/misleading text in normal flows (per smoke DIFFs); Rust API mapping verified in UI; `npm --prefix apps/web run build` passes; UI guide and README updated for actual grok runtime (text + collected media focus).

Planned DIFFs completed via governance and enhancements (see DIFF-247 for 2-10 mapping and deferrals of remaining specialized items like full #2 multimodal deep ingest, #8 veteran templates, #10 dedicated perf to future one-at-a-time DIFFs e.g. 248+).

Done when criteria met: No dead buttons, no misleading text, Rust API mapping verified, UI build passes. (Met; further polish can be new DIFFs.)

**Note on branch/DIFF governance:** All described completion work performed on `grok` (active dev branch for this repo per current state and root AGENTS.md). DIFF-247 closed the enhancements 2-10 master tracker. No merges performed. Future product work continues via small verifiable DIFFs on grok. See baseline for Codex verification limits (non-Docker checks preferred in some envs).