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
7. Live-stack verified behavior

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

After DIFF-244, promotion may be reconsidered only by explicit owner
instruction. Product expansion should continue through scoped DIFFs unless the
owner requests a promotion audit.

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

DIFF-235 through DIFF-244 intentionally consolidate the next major product
areas into 10 larger build DIFFs. Each must still stay scoped, verifiable, and
honest about unsupported states.

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
DIFF-235 — Source Expansion And Connector Contract Foundation
```

Do not start promotion DIFFs. Do not perform Public Promotion Candidate Audit,
Selective Public Promotion Dry Run, or Main Promotion unless the owner gives a
later explicit instruction.

Build the product.
