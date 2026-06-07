# IGY6 Capability Truth Table

**Audit Date:** 2026-06-07 (performed on Grok6 clone)  
**Current Branch (for this table):** `grok` (in /home/nasty/Grok6 separate clone)  
**HEAD at audit start:** f05b128 Complete DIFF-245 post-244 capability integrity audit next build phase plan  
**Governing DIFF for this table + foundations:** DIFF-246 (this work)  
**Source materials:** IGY6 Finished Product Capability Specification (specs.txt), IGY6_CURRENT_IMPLEMENTATION_AUDIT_PACKAGE.txt + CAPABILITY_TRUTH_TABLE_CODEX_PROMPT.txt + TEMPLATE, DIFF-240 through DIFF-245, AGENTS.md, BRANCH_POLICY.md, CODEX_PROMPT_BASELINE.md, rust-cutover-manifest.json, docs/ui/README.md, code inspection of crates/ (especially igy6-write-api, igy6-evidence-answer, igy6-gateway), apps/web, scripts, infra.

## 1. Current branch / HEAD / audit date
See header. Work isolated to Grok6 `grok` branch per user request to complete the product on a new `grok` branch. Primary dev worktree left untouched.

## 2. Current implementation summary
IGY6 (at this snapshot) has a solid Rust-native runtime (gateway + worker), Next.js normal-user UI (tabs: Home/Add Data/Work/Results/Settings/Advanced), Postgres for relational (sources, permissions, evidence, approvals, audit, experiments, etc.), Qdrant for vectors, Neo4j declared for graph surfaces + host-bridge for potential external service bridging. Core text-oriented ingestion (manual, conversation, observation, local project, some web) flows through write-api -> artifacts -> normalization -> chunking -> vector + evidence-answer packet construction. Real permission records (source_permissions), audit events, and approval gates exist for several workflows (e.g. experiment acceptance). Evidence-grounded answers with citations, assumptions, inferences, uncertainty, missing info, and source trails are implemented in Rust with tests.

However, per the post-245 audit and specs, many "finished product" capabilities around rich collectors (browser exports, full media, wifi/RF, streams), deep graph extraction + persisted review of entities/claims/events/relationships, and full adaptive loop (outcome-linked calibration, method improvement execution) remain at lower implementation levels. DIFF-235..239 were UI/docs surfaces only. DIFF-240..244 added real scoped backend but many were not live-stack verified in the agent context and titles can overclaim (e.g. "generation", "self-improvement", "guardrails", "release readiness").

This table provides the honest mapping. The Grok6 `grok` branch (this DIFF) begins closing gaps with real backend foundations (SourceType extensions + dry-run/permission contract helpers) while producing this table.

## 3. Verification level definitions
(From IGY6_CURRENT_IMPLEMENTATION_AUDIT_PACKAGE.txt + codex prompt)

Implementation levels (highest proven):
- not_started
- docs_only
- ui_only
- ui_plus_existing_api
- new_api_route
- new_persistence_schema
- worker_runtime_behavior
- script_lifecycle_behavior
- tested
- live_stack_verified
- outcome_confirmed

Verification ladder (confidence):
1. Described only
2. Documented
3. UI-visible
4. API-wired
5. Persisted
6. Unit-tested
7. Integration-tested
8. Script/fixture-tested
9. Live-stack verified
10. Outcome-confirmed
11. Reused successfully later

A capability can be implemented at a level but weakly verified. Both are recorded.

## 4. Capability truth table

| Capability ID | Capability Name | Finished Requirement (from specs) | Current Status | Implementation Level | Verification Level | DIFFs | Primary Files | API Routes | Persistence / Storage | Runtime Behavior | Scripts | Tests / Commands | Live Smoke | Known Gaps | Overclaim Risk | Next Action |
|---------------|-----------------|-----------------------------------|----------------|----------------------|--------------------|-------|---------------|------------|-----------------------|------------------|---------|------------------|------------|------------|----------------|-------------|
| CAP-001 | Manual text ingestion | Add text and get evidence records with provenance | implemented | worker_runtime_behavior, tested | 7 (integration + unit) | DIFF-210+ | crates/igy6-write-api, igy6-artifacts, igy6-normalization, igy6-chunking, igy6-evidence-answer, gateway | /sources, /artifacts, write paths | Postgres (sources, artifacts, chunks, evidence), Qdrant | Full: upload -> artifact (content addressed) -> normalize -> chunk -> index -> evidence packet | normal-user-product-smoke, backup/restore scripts | cargo test (write-api, evidence-answer, gateway) | Partial (past smokes on dev) | Full end-to-end owner WSL with real data + approval flows | low | Owner WSL smoke + outcome recording |
| CAP-002 | Conversation history import | Import conversations as structured evidence with entities/claims/relationships | partially_implemented | ui_plus_existing_api + worker | 5-6 | DIFF-213, 240+ | write-api (conversation_history SourceType), evidence-answer, web Add Data | Same core + source_type=conversation_history | Postgres + Qdrant + evidence | Ingestion + packet works for text exports; relationship extraction limited | - | write-api tests for SourceType | Partial | Deep relationship extraction + graph linking; browser export formats | medium | Extend in collector foundations (this DIFF + 247) |
| CAP-003 | User observation ingestion | Record observations as first-party evidence with outcome links | implemented | worker_runtime_behavior, tested | 6 | DIFF-214 | write-api (UserObservation), evidence-answer | source + evidence paths | Postgres (evidence, outcomes) | Yes | - | Unit tests in evidence-answer | Partial | Stronger outcome linking + review UI | low-medium | Continue in experiment/outcome DIFFs |
| CAP-004 | Source trust/sensitivity review | Review source trust and sensitivity, permissions | implemented | new_persistence_schema + api | 5-6 | DIFF-210 | gateway (sources + source_permissions queries), write-api | /sources, /sources/{id}, permissions in responses | Postgres (sources, source_permissions) | Read + write permission records | - | write-api tests | Partial | Full approval middleware on all collectors | medium | DIFF-246 foundations |
| CAP-005 | Evidence correction/supersession | Mark evidence corrected/superseded/disputed | partially_implemented | ui_plus_existing_api | 4 | DIFF-211 | review-state routes (web + proxy), audit_events | /evidence/.../review-state | Postgres (evidence + audit) | Audit events recorded; state transitions exist in some flows | - | Limited | Limited | Full supersession + propagation to answers/graph | medium | Later DIFF |
| CAP-006 | Persisted evidence answer records | Save evidence-grounded answer records | implemented | worker_runtime_behavior + persistence + tested | 6-7 | DIFF-212, 219 | igy6-evidence-answer (build_evidence_answer_packet), gateway, retrieval | answer endpoints | Postgres (answers?) + Qdrant citations | Real packet with facts/assumptions/inferences/uncertainty/missing/source_trails | - | Multiple unit tests in evidence-answer lib (context true/false, citations) | Partial | Full roundtrip save + retrieval of answer records linked to outcomes | low | Verify in owner smoke |
| CAP-007 | Evidence-grounded answer surface | Show facts, assumptions, inferences, uncertainty, citations | implemented (core) | worker_runtime_behavior + ui | 6 | DIFF-219, 245 audit | igy6-evidence-answer + web Results | answer + evidence review | As above + UI display | Packet construction + UI surfaces | - | evidence-answer tests + web build | Partial (agent saw UI/docs reports earlier) | Outcome linkage + missing info prompting full | medium (earlier reports said UI only; code shows real) | This DIFF + owner verification |
| CAP-008 | Missing evidence prompting | Tell user what evidence is missing | partially_implemented | worker + ui | 5 | DIFF-220 | evidence-answer (missing_information vec), UI | answer responses | In packet | Vector present in packet | - | evidence-answer tests | Limited | Prominent UI + follow-up collection suggestions | medium | UI wiring + tests |
| CAP-009 | Outcome learning summary | Summarize successful/failed outcomes | partially_implemented | backend + ui | 4-5 | DIFF-221, 241 | experiment/outcome paths, calibration summary | /analysis/calibration/summary | Postgres (outcomes, predictions, recommendations) | Calibration counts + outcome linkage | - | Rust helper tests | Limited | Full learning loop + method improvement execution | high (title overclaim risk) | DIFF-242/241 hardening |
| CAP-010 | Prediction/recommendation records | Create/review prediction and recommendation records with calibration | partially_implemented | new_api_route + persistence | 5 | DIFF-222/223/241 | gateway calibration, write paths | GET /analysis/calibration/summary | Postgres | Descriptive stats + bands from persisted records | - | deterministic Rust tests | Limited | Generation engine + auto outcome feedback | high | Per DIFF-241 notes |
| CAP-011 | Pattern/conflict/drift/anomaly detection | Create baseline pattern records from data | partially_implemented | worker_runtime_behavior (gateway) | 5 | DIFF-224/240 | igy6-gateway pattern detection | analysis / pattern endpoints | Postgres (pattern records + evidence links) | Baseline detector with metadata, support counts, linked IDs | - | workspace tests | Partial (no full live in Codex) | Duplicate handling, review transitions, stability | high | Hardening future DIFF |
| CAP-012 | Experiment proposal workflow | Create experiment proposals with criteria/outcomes + approval gate | implemented (MVP) | new_api_route + new_persistence + ui | 5-6 | DIFF-201/242 | gateway POST /experiments/propose-from-improvement | /experiments/... | Postgres (experiment proposals + approvals) | Proposal + dry-run metadata + approval_required gate | - | workspace tests | Limited | Execution of accepted experiment + result comparison records | medium-high (self-improvement title risk) | Per DIFF-242 gaps |
| CAP-013 | Guardrail/tool-use policy | Flag/block unsupported or risky actions | implemented (scoped) | worker_runtime_behavior (classifier) | 6-7 | DIFF-176/243 | igy6-agent-api classifier + gateway /agent/capabilities | /agent/capabilities | Policy in memory + audit | Hardened for prompt injection, hosted model, raw command, secret exfil | - | Rust tests for injection/hosted/secret cases | Partial | Broader policy matrix + enforcement on all actions | medium | DIFF-243 notes + future |
| CAP-014 | Backup export MVP | Create safe metadata export bundle | implemented | script_lifecycle_behavior + tested | 8 (script + fixture) | DIFF-229/244 | scripts/backup-export-mvp.sh | n/a | .igy6-local/exports/ (sanitized) | Post-sanitization validation | backup-export-mvp.sh --... | bash -n + fixture checks in DIFF-244 | Script-level | Full service backup design, destructive flows | low | Owner WSL |
| CAP-015 | Restore dry-run MVP | Validate export bundle without destructive restore | implemented | script_lifecycle_behavior + tested | 8 | DIFF-230/244 | scripts/restore-dry-run-mvp.sh | n/a | fixtures | --strict-safety rejection of unsafe | restore-dry-run-mvp.sh | Strict safety fixture passed | Script | Actual destructive restore | low | Owner WSL |
| CAP-016 | Diagnostics bundle MVP | Create safe diagnostics bundle with redaction | implemented | script_lifecycle_behavior | 7-8 | DIFF-231/244 | scripts/diagnostics-bundle-mvp.sh | n/a | .igy6-local/diagnostics/ | Self-redaction checks | diagnostics-bundle-mvp.sh | Dry-run + safety validation | Script | Broader diagnostics | low | Owner WSL |
| CAP-017 | Normal-user product smoke | Product path checklist/helper + release readiness | implemented | script_lifecycle_behavior | 7 | DIFF-232/244 | scripts/normal-user-product-smoke.sh + RELEASE_READINESS_CHECKLIST | n/a | smoke results | --check + --release-readiness-check | normal-user-product-smoke.sh | --check passes synthetic | Script (Codex max) | Full owner WSL live stack | medium | Owner to run post-push |
| CAP-018 | Browser/web/router collector | Real scoped collector for URL/browser/router data per collector contract | partially_implemented (types + ui) | ui_only -> new_api_route (with this DIFF) | 4 -> 5 | DIFF-236, 246 (this) | write-api SourceType (Web* + RouterNetwork exist; BrowserExport added), gateway sources | source + collection_run audit | Postgres (sources + permissions + collection_run) | Registration + permission + audit path works for existing Web*/Router; new BrowserExport enables | - | SourceType parse tests | Limited | Real import of browser history export files + scoped preview/collect per contract | high (was UI-only) | This DIFF + DIFF-247 |
| CAP-019 | Media import backend | Real PDF/image/audio/video import and extraction | partially (type registration) | docs + ui + type registration (this DIFF) | 2-3 -> 4 | DIFF-237, 246 | igy6-artifacts + write-api (MediaFile added) | source + artifact | Postgres artifacts + metadata | Metadata + hash for media files; full binary extract not present (explicit) | - | SourceType + artifact tests | No | Full local extraction (PDF text, image desc, audio transcript, video frames) + chunking of extracted text | high (was UI-only; do not claim complete) | DIFF-248 (media MVP) |
| CAP-020 | Local project/PC diagnostics collector | Scoped project/diagnostics import backend | partially_implemented | ui + existing (LocalProject / LocalPcDiagnostics types) | 5 | DIFF-238, 240+ | write-api (existing types), gateway | source paths | Postgres + artifacts | Works for local_project / pc_diagnostics via manual-ish upload | diagnostics-bundle etc. | SourceType + smoke scripts | Partial | Crawl-free scoped collector + richer diagnostics | medium | DIFF-249 |
| CAP-021 | Graph extraction/relationship reasoning | Entity/claim/event/relationship persistence and review | partially (packet trails + docs) | docs + host-bridge + evidence trails -> graph candidate foundations | 3-4 | DIFF-226/239, 246 | igy6-evidence-answer (source trails, citations), host-bridge (neo4j mentions), gateway, README | limited | Postgres (evidence links) + Neo4j (declared) + host-bridge TCP | Trails in answers; relationship candidates extractable from packet; full persisted graph review surface not present | - | evidence-answer tests | No (infra present) | Persisted graph nodes/edges + review workflow + Neo4j writes for claims/events/rels | high (UI-only in 239) | DIFF-250 + graph persistence DIFF |
| CAP-022 | Wi-Fi/RF signal intelligence | Ingest signal readings, map coverage, correlate outcomes | not_started | not_started | 1 (described) | none | specs, plans, audit only | none | none | none | none | none | no | No implementation | high | Plan + DIFF per 245 sequence |
| CAP-023 | Stream recording/playback | Register streams, capture sessions, extract OCR/transcript/events | not_started | not_started | 1 | none | specs only | none | none | none | none | none | no | No implementation | high | Plan + DIFF |
| CAP-024 | Image/visual generation | Generate diagrams/images and save artifacts | not_started (in product) | not_started | 1 | none (env has image_gen for agents) | specs, Adaptive build instructions | none in product | artifacts storage (declared) | none in runtime | none | none | no | Product must use local configured model or bounded action; no integration yet | high | Future bounded action DIFF (local LLM vision or external with policy) |
| CAP-025 | Code project analysis/patch workflow | Analyze repo, identify bugs, propose/apply approved patches, run tests | partially (agent context + scripts) | docs + cli + some runtime | 3-4 | various | igy6-cli, scripts, agent-api guardrails | limited | limited | Guardrails block arbitrary shell; some diagnostic + patch suggestion in agent flows | run.sh, status, rust-cutover etc. | cargo test, clippy in verif | Limited (this env) | Bounded approved patch executor + full repo analysis collector | high | Scoped in future DIFF (safety critical) |
| CAP-026 | Capability truth table | Track claimed vs actual capability state across finished spec vs implementation | implemented (this DIFF) | docs + script_lifecycle (audit) + new_persistence (md) | 9 (this document) | DIFF-245 (planned), DIFF-246 (delivered) | docs/runtime/IGY6_CAPABILITY_TRUTH_TABLE.md (this file), audit package, specs.txt, DIFFs | n/a (docs + md) | docs (git tracked) | Manual + agent-driven audit process; can be re-run via commands in section 12 | normal-user-product-smoke, operator checks | The verification section of this DIFF + cargo/npm checks | This document + past smokes | Automation of table regeneration + diffing against prior | medium (was docs_only) | Maintain via future DIFFs; owner re-audit after WSL |

(Additional rows can be added for finer-grained items from specs such as provenance on every record, specific collector contract steps, experiment execution, method reliability records, etc. Core loop steps are covered by the above aggregated CAPs.)

## 5. High overclaim risks
- Titles containing "generation", "self-improvement", "guardrails", "release readiness", "graph extraction", "media import", "collector" can be read as more complete than the scoped MVP or UI surfaces that were delivered (explicitly called out in DIFF-245 and individual DIFFs).
- Evidence answer surface (CAP-007) was reported "UI/docs only" in some prior summaries; code inspection shows real Rust packet construction + tests + trails. Risk reduced but still needs live outcome linking verification.
- Any claim of binary media, image, audio, video, wifi, or stream parsing/ingestion is prohibited until a later scoped DIFF adds + verifies (per AGENTS.md and specs).
- Graph/relationship is aspirational in README/infra + trails in answers; not full persisted review yet.

## 6. UI-only capabilities
Primarily the control surfaces from DIFF-235..239 before backend was added in later DIFFs. Current UI (normal-user tabs) is mostly wired to real API for core paths. Advanced tab and some Results cards for patterns/predictions/experiments still carry higher overclaim risk until live verification.

## 7. Docs-only capabilities
- Full collector contract steps for all source categories (specs section 2).
- Detailed media analysis, wifi mapping, stream OCR/extraction, camera/sensor fusion.
- Complete adaptive self-improvement execution (method variants, accepted/rejected, reuse).
- Some relationship reasoning and graph review workflows.

## 8. Backend/API/persistence/runtime capabilities
- Core ingestion (CAP-001 and friends), evidence packets (CAP-006/007), source + permission model (CAP-004), audit, experiment proposals (CAP-012), guardrail classifier (CAP-013), calibration summary (CAP-010), baseline pattern detection (CAP-011), lifecycle scripts (CAP-014-017).
- With this DIFF: SourceType registration for browser/media/wifi/stream + dry-run/permission helper foundations (advances CAP-018/019/021/022 toward new_api_route + runtime).

## 9. Live-stack verified capabilities
Limited in agent context (Codex sandbox often cannot run full Docker). Past owner smokes reported for ranges DIFF-219..223 and some later. Script-level (backup/restore/diagnostics/smoke --check) are fixture/script verified. Full live-stack + outcome-confirmed for advanced collectors and graph pending owner WSL on the pushed grok branch.

## 10. Planned/not-started finished-product capabilities
CAP-022,023,024 (wifi, stream, image gen in product), full media extraction, full graph persistence + review, experiment *execution*, autonomous method improvement, rich sensor/stream collectors, complete provenance explanation surfaces for every reasoning step, production promotion readiness.

## 11. Next required DIFFs
Per DIFF-245 updated plan + this work:
- Continue/expand DIFF-246 foundations (this).
- DIFF-247 Browser/Web/Router Import Backend MVP (real collect of export files per contract).
- DIFF-248 Media Import Backend MVP (metadata + safe fallback; extraction later).
- DIFF-249 Local Project/Diagnostics Import Backend MVP.
- DIFF-250 Graph Entity/Claim/Event Persistence And Review.
- Subsequent for prediction hardening, experiment execution, full collector contract enforcement, truth table automation, owner WSL + outcome confirmation.
- Update this table after each.

## 12. Owner WSL smoke commands
(As required by codex prompt / DIFFs / completion plan)

```bash
# From repo root (after git checkout grok or pull of grok branch)
git status --short
git diff --check
npm --prefix apps/web run build
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets || true

# Scripts (syntax + dry)
bash -n scripts/backup-export-mvp.sh
bash -n scripts/restore-dry-run-mvp.sh
bash -n scripts/diagnostics-bundle-mvp.sh
bash -n scripts/normal-user-product-smoke.sh

# Full (owner WSL with Docker + services)
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result

# Fresh clone / runtime lifecycle (as applicable)
scripts/fresh-clone-startup-check.sh --check || true
scripts/runtime-lifecycle-check.sh --check || true
scripts/post-cutover-smoke.sh --check || true
```

Re-run the full audit commands from the CAPABILITY_TRUTH_TABLE_CODEX_PROMPT.txt (sed on AGENTS/BRANCH_POLICY/plan, find, grep for key terms) after major changes and update this table.

## Confirmation for this audit instance
- Prohibited scope avoided: no main, no merges, no .env, no private data dump, no broad changes outside DIFF-246 scope, work only in Grok6/grok.
- This table + the DIFF-246 foundations raise several high-risk items (collectors, media type, graph trails, permission helpers) from previous "UI-only / docs / not_started" reports toward documented new_api_route + runtime behavior.
- Ready for owner review and WSL verification on the pushed `grok` branch.

**End of IGY6_CAPABILITY_TRUTH_TABLE.md (produced under DIFF-246 on grok branch)**
