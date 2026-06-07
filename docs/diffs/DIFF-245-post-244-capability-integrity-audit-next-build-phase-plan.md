# DIFF-245 - Post-244 Capability Integrity Audit And Next Build Phase Plan

Status: Complete

## Branch And Baseline

- Active branch before work: `dev`
- HEAD before work: `148d1a0 Complete DIFF-244 data lifecycle hardening release readiness`
- `dev` ahead/behind `origin/dev` before work: even with `origin/dev`
- Controlling plan: `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Scope

DIFF-245 audits the actual capability state after DIFF-240 through DIFF-244
and updates the active build plan with the next product-build phase. It is
planning/audit only.

This DIFF does not implement runtime features, edit runtime app code, edit Rust
code, edit scripts, touch `main`, merge, cherry-pick, push, promote files,
create promotion branches, edit `.env`, or dump runtime/private data.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `docs/diffs/DIFF-240-pattern-conflict-drift-anomaly-expansion.md`
- `docs/diffs/DIFF-241-prediction-recommendation-generation-calibration-mvp.md`
- `docs/diffs/DIFF-242-self-improvement-experiment-workflow-mvp.md`
- `docs/diffs/DIFF-243-guardrails-tool-use-external-model-policy-hardening.md`
- `docs/diffs/DIFF-244-data-lifecycle-hardening-release-readiness.md`
- `README.md`
- `docs/ui/README.md`
- DIFF-240 commit file list:
  `8808989d22cdf7b2edba93c973aed887e2ecdeda`
- DIFF-241 commit file list:
  `705115c79ff364ffde505e10e4b18844a9bbf2b2`
- DIFF-242 commit file list:
  `b10403f799f14753b425f0e6bea0286c63e4457a`
- DIFF-243 commit file list:
  `f06ad5b6ec63040a35ef341e7528fbf5b3ba6f8b`
- DIFF-244 commit file list:
  `148d1a0c8af6fd40b8de40d76678684c230323bb`
- Focused implementation scan for pattern, calibration, experiment,
  guardrail/policy, backup, restore, diagnostics, and release readiness terms.

## Capability Classification

### DIFF-240 - Pattern / Conflict / Drift / Anomaly Expansion

Classification:

- UI plus existing API wiring
- worker/runtime behavior through Rust gateway analysis behavior
- not live-stack verified in Codex

Actually implemented:

- Expanded Rust gateway baseline pattern detection to include outcome metadata
  and additional baseline categories.
- Added detector metadata, support/evidence counts, linked source/outcome IDs
  where available, review status, and unverified notes.
- Updated Results UI and UI guide.

Completion claim accuracy:

- Accurate as a baseline detector expansion.
- Not a complete pattern-intelligence system.

Overclaim risks:

- Could be overread as statistical anomaly detection or causality if detached
  from the DIFF notes. The DIFF and UI docs explicitly say it is baseline only.

Tests:

- Rust workspace tests passed for the changed gateway code.
- No live-stack detector run was performed in Codex.

Remaining gaps:

- Harder duplicate handling, persisted review transitions, detector key
  stability, and owner WSL live-stack verification remain future work.

### DIFF-241 - Prediction / Recommendation Generation And Calibration MVP

Classification:

- new API route
- UI plus existing API wiring
- Rust backend read/calibration-summary behavior
- not live-stack verified in Codex

Actually implemented:

- Added `GET /analysis/calibration/summary`.
- Reads persisted prediction/recommendation and outcome records and returns
  descriptive counts, outcome counts, evidence-linked totals, and confidence
  bands.
- Added deterministic Rust helper tests and UI display.

Completion claim accuracy:

- Accurate for descriptive calibration summary behavior.
- The title includes generation, but this DIFF mostly improved read/calibration
  behavior over existing record creation rather than adding a new forecasting
  generation engine.

Overclaim risks:

- "Generation" and "calibration" can sound broader than the implementation.
  DIFF and UI docs correctly state there is no forecasting engine, automatic
  recommendation execution, or advanced calibration statistics.

Tests:

- Rust helper and workspace tests passed.
- Web build passed.
- No owner WSL live-stack verification was performed in Codex.

Remaining gaps:

- Stronger validation, outcome linkage, calibration review workflows, and
  live-stack verification remain future work.

### DIFF-242 - Self-Improvement Experiment Workflow MVP

Classification:

- new API route
- new persistence workflow behavior
- UI plus API wiring
- not live-stack verified in Codex

Actually implemented:

- Added `POST /experiments/propose-from-improvement`.
- Persists planned experiment proposal records from improvement items with
  proposal scope, dry-run metadata, success criteria, result comparison plan,
  review status, and accepted-method metadata.
- Added approval gate for accepted experiment status requiring an approved
  `experiment_acceptance` approval record.
- Updated Results UI and UI guide.

Completion claim accuracy:

- Accurate for proposal/dry-run/review persistence workflow.
- Not experiment execution and not autonomous self-improvement.

Overclaim risks:

- "Self-improvement" can imply autonomous method changes. DIFF and UI docs
  explicitly reject that claim.

Tests:

- Rust workspace tests passed for the changed gateway code.
- Web build passed.
- No owner WSL live-stack verification was performed in Codex.

Remaining gaps:

- Stronger status transition tests, accepted-method approval edge cases,
  result comparison records, and any bounded approved experiment executor
  remain future work.

### DIFF-243 - Guardrails / Tool-Use / External-Model Policy Hardening

Classification:

- Rust backend policy/classifier behavior
- UI plus existing API wiring
- not live-stack verified in Codex

Actually implemented:

- Hardened `igy6-agent-api` classifier behavior for prompt injection,
  hosted/external model requests, provider-name requests, raw command surfaces,
  and secret-dump/exfiltration wording.
- Blocked unsafe request-understanding before action matching.
- Extended `/agent/capabilities` with local-first, hosted-AI, external-model,
  arbitrary-command, prompt-injection, approval, and blocked request-class
  policy posture.
- Updated Settings safety UI and UI guide.

Completion claim accuracy:

- Accurate for scoped classifier and policy-posture hardening.
- Not complete guardrails or a complete action sandbox.

Overclaim risks:

- "Guardrails" can sound complete. DIFF and UI docs keep this scoped to
  current classifier/capabilities behavior.

Tests:

- Rust tests cover prompt injection, hosted model requests, and secret dumps.
- Cargo workspace tests and web build passed.
- No owner WSL live-stack verification was performed in Codex.

Remaining gaps:

- Broader policy matrix, approval middleware edge cases, action registry
  enforcement hardening, and live-stack verification remain future work.

### DIFF-244 - Data Lifecycle Hardening And Release Readiness

Classification:

- script/lifecycle behavior
- docs
- not live-stack verified in Codex

Actually implemented:

- Backup export performs post-sanitization safety validation before writing.
- Restore dry-run supports `--strict-safety`.
- Diagnostics bundle performs self-redaction checks before writing or dry-run
  summary.
- Normal-user smoke helper gained `--release-readiness-check`.
- Added synthetic unsafe fixture and release-readiness checklist.

Completion claim accuracy:

- Accurate for lifecycle script hardening and release-readiness checks.
- Not a complete backup system, destructive restore, destructive delete,
  retention system, production readiness, or promotion readiness.

Overclaim risks:

- "Release readiness" can imply production readiness. The DIFF and checklist
  state that promotion remains deferred and owner WSL smoke is still required.

Tests:

- Script syntax checks passed.
- Safe fixture passed restore strict safety.
- Unsafe fixture was rejected as expected.
- Diagnostics dry-run passed safety validation.
- Release-readiness check passed.
- Web build passed.

Remaining gaps:

- Owner WSL live-stack smoke, full service backup design, destructive restore,
  delete/retention enforcement, and promotion planning remain future work.

## Cross-DIFF Integrity Findings

- DIFF-235 through DIFF-239 were UI/docs-only control surfaces. They should not
  be treated as backend collector, media parser, diagnostics collector, graph
  extraction, or relationship reasoning runtime.
- DIFF-240 through DIFF-244 did add real backend/API/script behavior, but none
  was live-stack verified in Codex.
- The build plan still pointed the immediate next action at DIFF-235 after
  DIFF-244. This DIFF corrects that plan drift.
- No false-complete claim requiring code correction was found in the DIFF-240
  through DIFF-244 records. The main overclaim risks are title-level ambiguity:
  "generation", "self-improvement", "guardrails", and "release readiness" can
  sound broader than the implemented scoped behavior when read without the DIFF
  notes.

## Remaining Gaps After DIFF-244

- Real connector permission and dry-run runtime foundation.
- Browser/web/router backend import behavior for safe manual or bounded
  sources.
- Media import backend support and verified local extraction where safe.
- Local project and PC diagnostics backend import behavior without crawling.
- Persisted graph entity/claim/event/relationship review records.
- Pattern persistence/review hardening beyond baseline candidate creation.
- Prediction/recommendation validation and outcome calibration hardening.
- Experiment proposal status and approval hardening.
- Guardrail policy test matrix and enforcement hardening.
- Owner WSL live-stack verification and release-readiness gap closure.

## Build Plan Updates

Updated `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md` to:

- add `Script/lifecycle behavior` to the completion classification list;
- add DIFF-245 through DIFF-255 to the active ordered plan;
- state that promotion remains deferred until explicit owner instruction;
- state that DIFF-235 through DIFF-239 were UI/docs-only control surfaces;
- state that DIFF-240 through DIFF-244 added scoped backend/API/script behavior
  subject to owner WSL smoke;
- require DIFF-246 through DIFF-255 to prefer real backend/API/script,
  persistence, or runtime behavior over UI-only surfaces unless the DIFF record
  proves backend work is unsafe, too broad, or already complete;
- update the immediate next action to DIFF-246.

## New DIFF-246 Through DIFF-255 Sequence

1. DIFF-246 - Real Connector Permission And Dry-Run Runtime
2. DIFF-247 - Browser/Web/Router Import Backend MVP
3. DIFF-248 - Media Import Backend MVP
4. DIFF-249 - Local Project/Diagnostics Import Backend MVP
5. DIFF-250 - Graph Entity/Claim/Event Persistence And Review
6. DIFF-251 - Pattern Detection Persistence And Review Hardening
7. DIFF-252 - Prediction/Recommendation Persistence And Outcome Calibration
   Hardening
8. DIFF-253 - Self-Improvement Experiment Persistence And Approval Hardening
9. DIFF-254 - Guardrail Policy Test Matrix And Enforcement Hardening
10. DIFF-255 - Release Readiness Runtime Verification And Gap Closure

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

No Rust, TypeScript, script, or runtime files changed, so cargo, npm, script
syntax, and Docker/live-stack checks were not required for this planning-only
DIFF.

## Files Changed

- `docs/diffs/DIFF-245-post-244-capability-integrity-audit-next-build-phase-plan.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Verification Summary

- Planning/audit docs only.
- Private/dev files remained tracked.
- Stale status scan still reports older out-of-scope draft/template/status
  strings; DIFF-245 itself is complete.

## Scope Confirmation

- Promotion remains deferred.
- No runtime code, Rust code, app code, or script behavior was changed.
- No hosted AI call, hidden external transfer, browser/account scraping,
  credential/cookie/token collection, fake control, arbitrary command execution
  behavior, destructive delete, destructive restore, unsafe backup archive,
  `.env` edit, runtime/private data dump, main work, merge, cherry-pick, push,
  promotion, promotion branch creation, or private/dev file removal was
  performed.
