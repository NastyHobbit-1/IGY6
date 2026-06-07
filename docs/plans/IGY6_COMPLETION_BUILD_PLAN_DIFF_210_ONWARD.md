# IGY6 Completion Build Plan From DIFF-210 Onward

Status: Active build plan for `dev`  
Starting point: DIFF-210  
Branch: `dev`  
Promotion branch: `main` only by explicit owner instruction

## 1. Purpose

This document defines the build path from DIFF-210 toward the intended IGY6
product. It is a completion target, not a claim that the current product is
complete.

IGY6 is not just a chatbot and not just a RAG demo. The product goal is a private, local-first adaptive intelligence system that can ingest trusted information, turn it into traceable evidence, reason over that evidence, plan safe work, ask for approval, execute bounded supported actions, record outcomes, and improve its methods through controlled user-approved feedback loops.

The build must continue through small, verifiable DIFFs. Each DIFF must add real product value, reduce a known gap, or harden an existing user-facing workflow. Avoid spending more DIFFs on smoke tooling unless a product change requires a verification update.

## 2. Branch Rules

Work happens on `dev`.

Private/dev/build instruction files stay on `dev`.

Do not remove from `dev`:

- `.codex`
- `AGENTS.md`
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md`
- `docs/agents/`
- `docs/plans/`

`main` remains public/runtime-clean.

Do not merge `dev` into `main`.

Do not cherry-pick broad dev commits into `main`.

Promotion to `main` must be selective and owner-approved.

Only promote public/runtime-safe files.

DIFF records, private build prompts, internal plans, and dev-only files do not promote to `main` unless the owner explicitly says so.

## 3. Codex Local Environment Rule

Codex local may run in a sandbox that strips Docker group access and remaps `/var/run/docker.sock` to `nobody:nogroup`.

Therefore, Codex should not be required to run full Docker smoke.

Codex may run:

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- `cargo fmt --all --check`
- `cargo test --workspace`
- script syntax checks
- non-Docker checks

The owner runs full runtime verification in normal WSL:

```bash
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result
```

A DIFF may record that full smoke was skipped in Codex because of the Codex sandbox. It must not claim full runtime verification unless the owner-provided WSL smoke result exists.

## 4. Current Verified Baseline

As of the DIFF-209 completion point, the project has a real working foundation:

- Rust API gateway
- Rust worker daemon
- Next.js web UI
- PostgreSQL
- Redis
- Qdrant
- Neo4j
- MLflow
- Phoenix
- manual UTF-8 text upload path
- source/permission/approval records
- raw artifact records
- normalization work items
- document/chunk/evidence creation
- retrieval preview
- Results review UI
- work status UI
- report workflow UI
- feedback/outcome capture
- source/evidence history UI
- improvement/experiment review metadata UI
- operator smoke script
- smoke result recording
- smoke result viewer
- persisted agent task plans
- approval-gated report-generation plan-to-work behavior
- persisted evidence-check summaries on task plans

The project is real. It is not scaffolding-only. But it is not yet the full adaptive-intelligence product.

## 5. Completion Standard

A feature is not complete merely because a DIFF says “Complete.”

For every DIFF, classify what was actually added:

1. Docs only
2. UI only
3. UI plus existing API wiring
4. New API route
5. New persistence/schema
6. New worker/runtime behavior
7. Script/lifecycle behavior
8. Live-stack verified behavior

Do not describe a larger feature as complete unless these are true:

- backend persistence exists where required
- API behavior is implemented
- UI wiring exists
- unsupported states are honest
- tests/build pass
- runtime verification passes, or the DIFF clearly records that runtime verification is pending owner WSL smoke

Avoid false-complete language.

Use:

- “review metadata exists”
- “proposal creation exists”
- “bounded work item creation exists for report-generation plans”
- “runtime smoke pending local WSL verification”

Do not use:

- “self-improvement is complete”
- “full AI task handling is complete”
- “autonomous action execution exists”
- “all source types are supported”
- “graph reasoning is complete”
- “forecasting is complete”

## 6. DIFF Process Rules

Each DIFF must:

- have one clear purpose
- create one `docs/diffs/DIFF-###-name.md`
- inspect relevant prior DIFFs
- inspect relevant runtime/UI/API files
- avoid broad refactors
- avoid unrelated cleanup
- preserve private/dev files
- use synthetic test data only
- avoid secret printing
- avoid `.env` dumps
- avoid runtime/private data dumps
- avoid arbitrary command execution
- avoid fake buttons
- avoid dead controls
- honestly show unsupported states
- run required verification
- commit separately

After each DIFF:

```bash
git status --short
git diff --check
git diff --name-status
npm --prefix apps/web run build
```

If Rust changed:

```bash
cargo fmt --all --check
cargo test --workspace
```

Always verify private/dev files remain tracked:

```bash
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort
```

Always scan stale statuses:

```bash
grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true
```

## 7. Product Priority From DIFF-210 Forward

The next work must build IGY6 itself, not more smoke tooling.

Current biggest product gaps:

1. Source trust and sensitivity management
2. Evidence correction and supersession
3. Persisted evidence-answer/chat records
4. Conversation history import
5. User observation ingestion
6. Source onboarding and source management polish
7. Local LLM provider status/routing UX
8. Evidence-backed answer generation UX
9. Prediction/recommendation MVP
10. Outcome learning summaries
11. Self-improvement proposal-to-experiment workflow
12. Backup/restore/export/delete
13. Graph lineage/entity/claim extraction
14. Pattern expansion
15. Source/media/collector expansion
16. Graph, pattern, prediction, and improvement expansion
17. Guardrails, lifecycle hardening, and release readiness
18. Main/public promotion pipeline, deferred until explicit owner instruction

## 8. Ordered DIFF Plan

### DIFF-210 — Source Trust And Sensitivity Management UX

Purpose: Add normal-user UX for marking sources as trusted, noisy, sensitive, disabled, or review-needed.

Scope:

- inspect source records, feedback side effects, source permission policy, evidence records
- expose source trust/sensitivity state in UI
- allow safe updates only if backend support exists
- otherwise add minimal backend route
- show how source state affects evidence trust if implemented
- do not delete sources
- do not hide evidence silently
- do not overclaim policy enforcement

Verification:

- web build
- Rust tests if backend changed
- synthetic source update if route exists
- local smoke by owner if runtime behavior changed

### DIFF-211 — Evidence Correction And Supersession UX

Purpose: Allow users to correct evidence without deleting history.

Scope:

- inspect evidence item schema and claim/source lineage
- add correction/supersession UX
- preserve immutable original evidence
- create correction record or superseding evidence link
- show corrected/superseded state in Results
- do not mutate raw artifacts
- do not delete original records

### DIFF-212 — Persisted Evidence Answer / Chat Session Records

Purpose: Persist evidence-backed answers and review history.

Scope:

- inspect evidence answer packet routes
- inspect existing chat/retrieval/session data
- add answer/session record persistence if absent
- store safe summary, citations, evidence IDs, missing-info state, uncertainty
- do not store secrets or raw private data unnecessarily
- show answer history in Results
- connect feedback/outcomes to answer records

### DIFF-213 — Conversation History Import MVP

Purpose: Add a controlled path for importing prior conversation/history text as evidence.

Scope:

- source type: `conversation_history`
- manual paste/upload of UTF-8 conversation text only
- preserve user intent, corrections, decisions, and context where possible
- create source/artifact/document/chunk/evidence records
- show limitations
- no account scraping
- no browser extraction
- no external connector

### DIFF-214 — User Observation Ingestion MVP

Purpose: Add first-party user observation records as trusted local evidence.

Scope:

- source type: `user_observation`
- normal-user form for “I observed / I know / I decided”
- timestamps
- optional tags
- optional related task/source/evidence link
- process into evidence where appropriate
- show observation history

### DIFF-215 — Guided Source Onboarding Completion

Purpose: Reduce ID friction in Add Data.

Scope:

- guided source creation
- permission creation
- approval creation where required
- upload path
- clear source state
- no Advanced tab required for normal manual/conversation/user-observation paths

### DIFF-216 — Source Detail Page / Panel

Purpose: Add a normal-user source detail view.

Scope:

- source metadata
- trust/sensitivity state
- permissions
- collection runs
- artifacts
- documents
- chunks
- evidence
- feedback/outcomes
- corrections/supersessions

### DIFF-217 — Evidence Detail Page / Panel

Purpose: Add a normal-user evidence detail view.

Scope:

- evidence content preview
- source trail
- document/chunk lineage
- trust state
- correction/supersession state
- feedback/outcome links
- related task plans
- related reports

### DIFF-218 — Local LLM Provider Status And Routing UX

Purpose: Make local AI status clear.

Scope:

- Ollama/local provider status
- selected model
- enabled/disabled state
- deterministic fallback
- unavailable model guidance
- no claim that hosted AI is used by default
- no hidden external data transfer

### DIFF-219 — Evidence-Grounded Answer Generation MVP

Purpose: Move from retrieval preview to user-facing evidence-grounded answers.

Scope:

- use existing evidence answer packet behavior
- show facts, assumptions, inferences, uncertainty, missing info, citations
- optional local LLM only if enabled and sufficiently evidenced
- deterministic fallback when model unavailable
- persist answer if DIFF-212 is complete

### DIFF-220 — Missing Evidence Prompting

Purpose: When evidence is weak, guide the user on what to add next.

Scope:

- missing-info prompts
- suggested source type
- suggested upload/observation/conversation import
- no unsupported conclusions
- no claim missing evidence means real-world absence

### DIFF-221 — Outcome Learning Summary MVP

Purpose: Start turning outcomes into user-visible learning summaries.

Scope:

- inspect feedback/outcome/improvement records
- show repeated failed advice
- show repeated successful methods
- create improvement candidate from outcome clusters if supported
- no autonomous method change

### DIFF-222 — Prediction / Recommendation Record Creation MVP

Purpose: Add controlled prediction/recommendation creation from evidence.

Scope:

- evidence-linked prediction/recommendation records
- confidence/uncertainty
- disproof criteria
- expected result
- review status
- no automatic execution
- no forecasting engine claim

### DIFF-223 — Prediction / Recommendation Outcome Review

Purpose: Close loop from prediction/recommendation to outcome.

Scope:

- mark correct/wrong/partial/inconclusive
- link evidence and task plan
- create improvement item if outcome failed
- update review surfaces

### DIFF-224 — Baseline Pattern Expansion MVP

Purpose: Expand pattern detection beyond basic records.

Scope:

- recurrence
- missing-information gap
- cross-source agreement
- cross-source conflict
- failed-advice recurrence
- successful-method recurrence
- do not claim advanced statistical validation unless implemented

### DIFF-225 — Graph Lineage Explanation UX

Purpose: Explain why records are connected.

Scope:

- source → artifact → document → chunk → evidence → answer/report/task
- Neo4j lineage if available
- fallback relational lineage if graph missing
- no claim of full graph reasoning

### DIFF-226 — Entity / Claim / Event Extraction Foundation

Purpose: Begin structured graph memory.

Scope:

- extract basic entities/claims/events from text evidence
- store records with provenance
- link to chunks/evidence
- review before trust where appropriate
- no broad NLP overclaim

### DIFF-227 — Report Template And Export MVP

Purpose: Make reports useful outside the UI.

Scope:

- report templates
- local markdown export
- evidence appendix
- citation list
- do not add PDF unless verified
- content-addressed local artifact if supported

### DIFF-228 — Backup / Restore / Export / Delete Audit

Purpose: Plan and implement the first safe data lifecycle controls.

Scope:

- audit current data storage
- define backup/export/delete boundaries
- implement smallest safe export if practical
- do not delete runtime data without explicit owner-approved DIFF

### DIFF-229 — Backup Export MVP

Purpose: Create local backup/export bundle for safe records.

Scope:

- metadata export
- evidence/source/report/task records
- no secrets
- no `.env`
- no raw private paths
- optional artifact inclusion only if explicit and safe

### DIFF-230 — Restore Dry-Run MVP

Purpose: Validate backup bundle without restoring destructively.

Scope:

- inspect backup
- validate schema/version
- report what would restore
- no destructive writes

### DIFF-231 — Diagnostics Bundle MVP

Purpose: Create a safe support bundle.

Scope:

- version info
- branch/HEAD
- route health
- service status
- recent non-secret smoke summary
- no secrets
- no `.env`
- no raw data dumps

### DIFF-232 — Normal-User End-To-End Product Smoke

Purpose: Verify Add Data → Work → Results → Answer → Report → Feedback → Outcome.

Scope:

- owner-run WSL path
- synthetic data
- scripted or documented
- no new smoke features unless needed
- product path only

### DIFF-233 — Product Claims Audit

Purpose: Prevent overclaiming before MVP.

Scope:

- README
- UI copy
- docs/ui
- roadmap
- unsupported capability warnings
- remove/soften claims not implemented

### DIFF-234 — Defer Promotion And Consolidate Next Build Phase

Purpose: Defer promotion and replace the old promotion sequence with the next
consolidated product-build phase.

Scope:

- update this build plan only
- record owner instruction that promotion is deferred
- do not perform public promotion audit
- do not create promotion branches
- do not touch `main`
- define DIFF-235 through DIFF-244 as the next 10 product-build DIFFs

### DIFF-235 — Source Expansion And Connector Contract Foundation

Purpose: Define and implement the foundation for additional source types and
connector behavior.

Scope:

- source policy
- scope validation
- dry-run requirements
- connector metadata
- sensitivity classification
- cleanup posture
- audit boundaries
- do not scrape accounts or browsers directly unless separately implemented in
  later DIFFs

### DIFF-236 — Browser / Web / Router Collector MVP

Purpose: Add controlled browser/web/router collection MVPs with explicit user
scope, dry-run preview, read-only default, audit events, and no hidden
collection.

Scope:

- browser/web/router collector contracts and safe UI/API paths where scoped
- explicit user scope and dry-run preview before collection
- read-only default behavior
- audit events for collection attempts and decisions
- no cookies, tokens, credentials, or private account data unless a later
  explicitly scoped permission model supports it
- no hidden collection

### DIFF-237 — PDF / Image / Audio / Video Import MVP

Purpose: Add media import foundations for common document/media types.

Scope:

- PDF text extraction only where local, safe, and verified
- image OCR only where local, safe, and verified
- audio transcription only where local, safe, and verified
- video transcription only where local, safe, and verified
- honest unsupported states for formats, codecs, file sizes, and missing local
  tools
- no hosted AI or hidden external transfer

### DIFF-238 — Local Project And PC Diagnostics Collector Hardening

Purpose: Improve local project collection and add authorized PC diagnostics
import.

Scope:

- explicit user-selected paths/files
- dry-run preview
- path traversal and scope boundaries
- no arbitrary filesystem crawling
- diagnostics import as user-authorized records
- safe redaction and audit posture

### DIFF-239 — Graph Extraction And Relationship Reasoning Foundation

Purpose: Expand entity, claim, event, and relationship extraction with
provenance and review status.

Scope:

- entity, claim, event, and relationship extraction improvements
- provenance links to source/artifact/document/chunk/evidence
- review status and trust posture
- graph/lineage explanation improvements
- no claim of full graph reasoning

### DIFF-240 — Pattern / Conflict / Drift / Anomaly Expansion

Purpose: Expand pattern detection to recurrence, missing-information gaps,
cross-source agreement/conflict, configuration drift, anomaly signals,
failed-advice recurrence, and successful-method recurrence.

Scope:

- recurrence
- missing-information gaps
- cross-source agreement and conflict
- configuration drift
- anomaly signals
- failed-advice recurrence
- successful-method recurrence
- do not claim advanced statistical validation unless implemented

### DIFF-241 — Prediction / Recommendation Generation And Calibration MVP

Purpose: Improve prediction/recommendation generation, evidence links,
uncertainty, confidence, disproof criteria, outcome tracking, and calibration
review.

Scope:

- evidence-linked prediction/recommendation generation
- uncertainty and confidence display
- disproof criteria and expected result capture
- outcome tracking
- calibration review
- no automatic recommendation execution

### DIFF-242 — Self-Improvement Experiment Workflow MVP

Purpose: Implement the controlled self-improvement workflow from improvement
item to experiment proposal, run/dry-run metadata, success criteria, result
comparison, and approval-gated accepted method.

Scope:

- improvement item to experiment proposal flow
- run/dry-run metadata
- success criteria
- result comparison
- approval-gated accepted method records
- no autonomous self-modification

### DIFF-243 — Guardrails / Tool-Use / External-Model Policy Hardening

Purpose: Add stronger prompt-injection, tool-use, approval, external-model, and
hosted-service policy checks.

Scope:

- prompt-injection checks
- tool-use guardrails
- approval hardening
- external-model and hosted-service policy checks
- sensitive/system-changing actions remain explicit and auditable

### DIFF-244 — Data Lifecycle Hardening And Release Readiness

Purpose: Continue backup/export/restore/delete/retention diagnostics,
normal-user product smoke, release readiness checks, and claim audits.

Scope:

- backup/export/restore/delete/retention diagnostics
- normal-user product smoke hardening
- release readiness checks
- product claims audits
- promotion can be reconsidered after this DIFF, but only by explicit owner
  instruction
- no promotion, branch switch, merge, cherry-pick, push, or public-file
  promotion unless the owner explicitly requests a later promotion DIFF

### DIFF-245 — Post-244 Capability Integrity Audit And Next Build Phase Plan

Purpose: Audit the actual capability state after DIFF-240 through DIFF-244,
prevent false-complete drift, and define the next product-build sequence.

Scope:

- classify DIFF-240 through DIFF-244 by actual behavior added
- distinguish backend/API/script behavior from UI/docs-only surfaces
- record live-stack verification still pending owner WSL smoke
- confirm DIFF-235 through DIFF-239 were UI/docs-only control surfaces
- extend the active plan with DIFF-246 through DIFF-255
- no runtime implementation, branch promotion, merge, cherry-pick, push, or
  main work

### DIFF-246 — Real Connector Permission And Dry-Run Runtime

Purpose: Convert connector policy/status surfaces into a real bounded
connector permission and dry-run runtime foundation.

Scope:

- inspect source permissions, approvals, audit events, collection runs, and
  DIFF-235 connector contract wording
- add or harden backend/API support for connector scope validation, dry-run
  requests, sensitivity classification metadata, and audit records
- require explicit user scope and approval posture for connector-backed source
  types
- keep future collectors disabled unless their backend path exists
- no browser/account scraping, credential collection, hidden collection, or
  external service transfer

### DIFF-247 — Browser/Web/Router Import Backend MVP

Purpose: Add the safest backend MVP for browser, web, and router import flows.

Scope:

- prefer manual paste/upload of browser page text exports, web page text, and
  router diagnostic text unless safe bounded fetch support already exists
- create backend/API dry-run and import records with explicit scope,
  exclusions, sensitivity posture, approval state, and audit events
- preserve source/artifact/document/chunk/evidence lineage for imported text
  where the pipeline supports it
- no cookies, tokens, credentials, browser profile reads, account scraping,
  website crawling, network scanning, router writes, or hidden external
  requests

### DIFF-248 — Media Import Backend MVP

Purpose: Add local, safe backend support for media import metadata and the
smallest verified extraction path.

Scope:

- inspect current artifact handling, normalization, chunking, evidence
  creation, file type checks, and DIFF-237 UI status surface
- implement bounded metadata persistence and one verified local extraction path
  where safe, preferably PDF text extraction if a local dependency/path already
  exists
- mark image OCR, audio transcription, and video transcription unsupported or
  planned unless local safe tooling is implemented and tested
- enforce file size/type bounds and preserve lineage
- no hosted OCR/transcription APIs, hidden transfer, unbounded binary parsing,
  or raw media dumps

### DIFF-249 — Local Project/Diagnostics Import Backend MVP

Purpose: Add backend support for authorized local project manifests and PC
diagnostics imports without arbitrary filesystem crawling.

Scope:

- inspect local_project source behavior, diagnostics scripts, source
  permissions, approvals, dry-runs, and DIFF-238 UI surface
- support explicit user-provided manifests/diagnostic exports and bounded
  dry-run/import records
- redact private paths and secret-shaped fields before persistence where
  practical
- preserve lineage when imported into the evidence pipeline
- no live system probing, command execution, arbitrary path reads, recursive
  crawling, `.env`, SSH key, browser profile, token, cookie, or credential
  collection

### DIFF-250 — Graph Entity/Claim/Event Persistence And Review

Purpose: Move graph/entity/claim/event/relationship review beyond UI-only
candidate rows into scoped persisted records.

Scope:

- inspect current claim records, entity/event candidates, relationship review
  surfaces, Neo4j posture, evidence lineage, and DIFF-239 outcome
- add minimal persistence/API behavior for entity, claim, event, or
  relationship review records with provenance and review status
- require source/evidence/chunk/document links where available
- use Neo4j only if the current sync path is safe and verified; otherwise use
  relational persistence and state that full graph reasoning remains incomplete
- no hosted AI, correlation discovery claim, raw evidence mutation, or full
  graph reasoning claim

### DIFF-251 — Pattern Detection Persistence And Review Hardening

Purpose: Harden DIFF-240 baseline pattern detection into safer persisted
review workflow behavior.

Scope:

- inspect persisted pattern records, detector metadata, review status, linked
  evidence/outcome/source IDs, and UI review flow
- improve duplicate handling, detector keys, review-state transitions, and
  evidence/source support summaries
- add tests for validation and persistence paths where gaps exist
- keep categories baseline and descriptive only
- no advanced statistical validation, forecasting, causality, or automatic
  behavior changes

### DIFF-252 — Prediction/Recommendation Persistence And Outcome Calibration Hardening

Purpose: Harden prediction/recommendation persistence, outcome linkage, and
descriptive calibration review.

Scope:

- inspect DIFF-241 calibration summary, DIFF-222/223 creation/outcome behavior,
  outcome records, evidence links, and UI review flow
- improve validation of confidence, uncertainty, expected result, disproof
  criteria, timeframe, outcome status, and calibration status
- strengthen record/outcome linkage and descriptive confidence-band summaries
- add tests for backend validation and calibration helper behavior
- no forecasting engine claim, advanced calibration claim, or automatic
  recommendation execution

### DIFF-253 — Self-Improvement Experiment Persistence And Approval Hardening

Purpose: Harden the DIFF-242 experiment proposal workflow and approval-gated
accepted-method path.

Scope:

- inspect improvement items, experiment proposal records, approvals, accepted
  method metadata, outcome records, and result comparison fields
- improve proposal validation, status transitions, accepted/rejected/deferred
  handling, and approval lookup behavior
- add tests for rejection paths and accepted-method approval requirements
- keep experiment execution manual or dry-run unless a later DIFF implements a
  bounded approved executor
- no autonomous self-modification, self-editing, method auto-promotion, MLflow
  run creation, Optuna study creation, Phoenix trace workflow, or hosted AI

### DIFF-254 — Guardrail Policy Test Matrix And Enforcement Hardening

Purpose: Expand DIFF-243 guardrails into a clearer policy matrix with stronger
tests and enforcement.

Scope:

- inspect agent intent classification, action registry, approvals, policy
  crate, local LLM routing, and UI safety posture
- add a test matrix for prompt injection, hosted/external model requests,
  secret exfiltration, raw commands, dangerous action wording, approval
  requirements, and unsupported actions
- improve backend policy response details and enforcement where gaps are found
- ensure sensitive/system-changing actions stay explicit and auditable
- no new dangerous tools, shell execution, hosted AI enablement, `.env` edits,
  or hidden external transfer

### DIFF-255 — Release Readiness Runtime Verification And Gap Closure

Purpose: Use the owner-run smoke posture and Codex-safe checks to close the
highest-risk lifecycle/release-readiness gaps without promotion.

Scope:

- inspect DIFF-244 release-readiness checklist, lifecycle scripts, diagnostics,
  product smoke, and owner-provided WSL smoke result if available
- close scoped gaps in export validation, restore dry-run validation,
  diagnostics redaction, product smoke checklist, claims audit, or release
  readiness docs
- record what is live-stack verified versus Codex-only verified
- do not promote, touch `main`, merge, cherry-pick, push, delete runtime data,
  restore runtime data, or create unsafe backup archives

## 9. Promotion Deferral

Promotion is deferred until explicit owner instruction.

The old promotion sequence is removed from the active ordered plan and deferred:

- DIFF-234 — Public Promotion Candidate Audit
- DIFF-235 — Selective Public Promotion Dry Run
- DIFF-236 — Main Promotion

Future promotion audit, selective promotion dry-run, and main promotion work
will be rescheduled later only if the owner explicitly requests it.

Promotion rules remain:

- do not merge `dev` into `main`;
- do not cherry-pick broad dev commits into `main`;
- do not touch `main` during product-build DIFFs;
- do not create promotion branches without explicit owner instruction;
- selectively promote only necessary public/runtime-safe files later, when the
  owner requests it;
- keep private/dev/build instruction files on `dev`.

## 10. Larger Post-MVP Blocks

After DIFF-245, promotion remains deferred until explicit owner instruction.
Product expansion continues through DIFF-246 through DIFF-255 unless the owner
requests a different product build sequence. Do not start promotion audit,
promotion dry-run, main promotion, public-file promotion, promotion branches,
main work, merges, cherry-picks, or pushes without explicit owner instruction.

DIFF-235 through DIFF-239 were UI/docs-only control surfaces. They established
visible source/connector, browser/web/router, media, local project/diagnostics,
and graph review posture, but they did not add backend collector runtime,
media parsing runtime, filesystem diagnostics collection, or persisted graph
relationship extraction behavior.

DIFF-240 through DIFF-244 added some real backend/API/script behavior:

- DIFF-240 added Rust gateway baseline pattern detection expansion.
- DIFF-241 added Rust gateway calibration summary read behavior.
- DIFF-242 added Rust gateway experiment proposal persistence workflow.
- DIFF-243 added Rust agent/gateway policy hardening.
- DIFF-244 added lifecycle/release-readiness script behavior.

These behaviors passed Codex-safe build/test/script checks, but full live-stack
verification remains pending owner WSL smoke unless a later DIFF records an
owner-provided successful result.

DIFF-246 through DIFF-255 should prefer real backend/API/script/persistence or
runtime behavior over UI-only surfaces. UI-only DIFFs are allowed only when the
DIFF record proves backend work is unsafe, too broad for the DIFF, or already
complete and adequately verified.

### Source Expansion

- PDF text extraction
- image OCR
- audio transcription
- video transcription
- local project collector hardening
- web/browser collector MVP
- router collector MVP
- PC diagnostics import

### Reasoning Expansion

- persisted chat threads
- evidence disagreement display
- multi-step investigation
- contradiction detection
- confidence calibration
- source quality weighting

### Self-Improvement Expansion

- improvement queue
- experiment design
- experiment dry-run
- MLflow artifact recording
- Phoenix trace recording
- Optuna trial support
- method comparison
- approval-gated accepted method registry
- rollback method state

### Prediction/Recommendation Expansion

- baseline forecasting
- recommendation ranking
- disproof tracking
- calibration dashboard
- outcome-based method scoring

### Security/Guardrails Expansion

- tool-use guardrails
- prompt injection scanning
- external model policy enforcement
- approval middleware
- action sandbox policy
- audit event coverage

## 11. Realistic DIFF Count From DIFF-210

These counts assume small, reviewable, verifiable DIFFs.

Basic usable product from DIFF-210:

- about 12 to 20 more DIFFs

Solid local MVP from DIFF-210:

- about 35 to 55 more DIFFs

Full original adaptive-intelligence product from DIFF-210:

- about 90 to 150 more DIFFs

DIFF-235 through DIFF-244 intentionally consolidated one major product phase.
DIFF-246 through DIFF-255 define the next product-build phase. Each must still
stay scoped, verifiable, and honest about unsupported states.

## 12. Prompt Template For Future Product DIFFs

Use this template for each future DIFF.

```text
You are working in the IGY6 repo on branch dev.

Do not create smoke-tooling-only work unless this DIFF explicitly requires verification tooling.
This DIFF must improve the actual IGY6 product workflow.

Codex-local environment rule:
Do not run full Docker smoke from Codex. Codex local command sandbox strips docker group access and remaps /var/run/docker.sock to nobody:nogroup. Run npm build, cargo fmt, cargo test, and non-Docker checks only. The owner will run full operator smoke in normal WSL after the DIFF.

Branch policy:
- Work happens on dev.
- Do not remove anything from dev.
- Do not remove private/dev/build files.
- Do not touch main.
- Do not merge.
- Do not cherry-pick.
- Do not push.

Goal:
Create and complete DIFF-###: <title>.

Create:
- docs/diffs/DIFF-###-<slug>.md

Purpose:
<one clear product purpose>

Required inspection:
git status --short
git branch --show-current
git log --oneline --decorate -30
git branch -vv
git diff --name-status
git diff --check
sed -n '1,360p' AGENTS.md
sed -n '1,320p' docs/BRANCH_POLICY.md
<prior DIFF inspections>
<relevant file inspections>
git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort

Required work:
1. <specific product task>
2. <specific product task>
3. <specific product task>

Safety:
- no secrets
- no .env dump
- no runtime/private data dump
- no arbitrary command execution
- no fake controls
- unsupported states must be honest

Verification:
git status --short
git diff --check
git diff --name-status
npm --prefix apps/web run build

If Rust changed:
cargo fmt --all --check
cargo test --workspace

Do not run full Docker smoke from Codex.

Owner-run local WSL verification after commit:
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result

Commit:
git add -A
git commit -m "Complete DIFF-### <short title>"

Final response must include:
- DIFF created
- branch and HEAD before work
- files inspected
- product changes made
- API/backend changes, if any
- unsupported states handled
- verification run
- files changed
- commit hash
- confirmation no main work, merge, cherry-pick, push, fake controls, arbitrary command execution, .env edit, or runtime/private data dump
```

## 13. Immediate Next Action

The next DIFF is:

```text
DIFF-246 — Real Connector Permission And Dry-Run Runtime
```

**Update (DIFF-246 on grok branch):** DIFF-246 was started on the isolated `grok` branch in the Grok6 clone (/home/nasty/Grok6) to deliver:
- The official IGY6_CAPABILITY_TRUTH_TABLE.md (CAP-026 closure, honest classifications from post-245 audit + code inspection + specs).
- Backend foundations: extended SourceType (BrowserExport, MediaFile, WifiSignal, StreamCapture) + supports_dry_run_preview / requires_explicit_approval helpers in igy6-write-api (real new_api_route + contract helper behavior per collector contract in specs).
- This advances CAP-018/019/021/022/026 from prior UI-only / not_started / high-overclaim toward documented runtime + tested.

See docs/diffs/DIFF-246-grok6-capability-truth-table-and-backend-mvp-foundations.md and docs/runtime/IGY6_CAPABILITY_TRUTH_TABLE.md.

The `grok` branch carries the "complete grok6 product" work per the provided audit handoff package + specs. Push of `grok` (new branch) requested by owner; no main, no merge, no cherry-pick, no promotion.

Do not start promotion DIFFs. Do not perform Public Promotion Candidate Audit,
Selective Public Promotion Dry Run, or Main Promotion unless the owner gives a
later explicit instruction.

Build the product. (Grok6/grok work is additional completion vehicle; primary dev remains on dev per policy.)
