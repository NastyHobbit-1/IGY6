# IGY6 Dev Build Plan

## Purpose

This document defines the working build plan for the IGY6 `dev` branch.

The purpose of `dev` is to preserve the full project-building context, agent instructions, private build notes, detailed implementation direction, and future product roadmap while keeping `main` clean and product/runtime-facing.

`main` is the public/product branch.

`dev` is the tracked development branch where build guidance, agent prompts, detailed planning, and implementation coordination may exist.

## Current Project Vision

IGY6 is intended to become a private, local-first, evidence-backed adaptive intelligence and decision-support system.

The long-term system should:

- ingest authorized information;
- remember it;
- link related facts, sources, claims, events, and outcomes;
- normalize raw data into evidence;
- search and retrieve evidence;
- answer questions with source-backed support;
- identify patterns and correlations;
- generate hypotheses;
- make predictions and recommendations;
- track outcomes;
- learn from feedback;
- run controlled experiments;
- compare baseline and candidate methods;
- improve its own methods only through auditable, approval-aware workflows.

IGY6 must not become an unsafe black-box automation tool.

Every conclusion should either connect to evidence or be clearly labeled as an assumption, hypothesis, estimate, unsupported statement, or insufficient-evidence result.

## Branch Policy

### main

`main` is product/runtime-facing.

`main` should contain:

- source code;
- runtime scripts;
- user/operator documentation;
- configuration templates;
- tests;
- product-facing plans;
- locked DIFF history;
- archived legacy code needed for history or rollback.

`main` should not contain:

- private build prompts;
- Codex prompts;
- local agent instruction prompts;
- personal coordination notes;
- private strategy documents;
- `.codex`;
- root `AGENTS.md`;
- build-only instruction documents.

### dev

`dev` is the full working development branch.

`dev` may contain:

- everything allowed on `main`;
- root `AGENTS.md`;
- `.codex`;
- build instruction documents;
- `docs/agents/`;
- detailed project planning;
- Codex prompts;
- agent role instructions;
- experimental implementation direction;
- temporary coordination notes;
- full build strategy.

`dev` is allowed to be pushed to GitHub because the owner explicitly chose to track it remotely so it does not disappear by accident.

## Remote Tracking Policy

The `dev` branch should track `origin/dev`.

Expected branch posture:

    main -> origin/main
    dev  -> origin/dev

`main` must remain clean and product-facing.

`dev` may contain build instructions and private working material.

Do not merge `dev` into `main` blindly.

To promote changes from `dev` to `main`, copy only explicitly selected product/runtime files.

Preferred promotion pattern:

    git checkout main
    git pull origin main
    git checkout dev -- <specific product file path>
    git status --short
    git diff --check
    git commit -m "<scoped product change>"
    git push origin main

Never promote all of `docs/agents/` to `main`.

Never promote `.codex`, root `AGENTS.md`, or private build instruction files to `main`.

## Current Verified Runtime Posture

The project has completed the Rust runtime cutover.

Current active runtime:

- Rust API gateway is active.
- Rust worker daemon is active.
- Next.js web UI is active.
- PostgreSQL is used for relational state and audit/evidence control-plane data.
- Qdrant is used for vector memory.
- Neo4j remains supporting graph infrastructure.
- Redis, MLflow, and Phoenix remain supporting services.
- Legacy FastAPI is archived and inactive.
- Legacy Python/Celery worker is archived and inactive.
- Celery beat is inactive.

Current verified application posture:

- Rust-only application API/worker runtime is claimed.
- Runtime data belongs outside the repository under `IGY6_DATA_ROOT`.
- `main` has been stripped of build-agent instruction files.
- `dev` preserves build-agent instruction files.
- Route parity has passed with no web-required FastAPI fallback.
- Rust worker can process core text pipeline work types.
- The web UI is tabbed and simplified for normal users.

## Current Strengths

The project currently has solid foundations in these areas:

- Rust API gateway.
- Rust worker daemon.
- Docker Compose local runtime.
- Simple run/stop/restart/status scripts.
- Runtime lifecycle validation scripts.
- Post-cutover runtime audit script.
- Fresh-clone startup validation script.
- Post-cutover smoke suite.
- Text-oriented ingestion pipeline.
- Normalization.
- Chunking.
- Evidence-oriented records.
- Qdrant vector upsert path.
- Agent API/capability/intent surfaces.
- Local LLM routing configuration.
- Evidence-answer fallback behavior.
- Approval/audit posture.
- Normal-user tabbed UI.
- Archived legacy Python code for rollback/history only.

## Current Limits

The project is not yet the full original adaptive intelligence system.

Current known limits:

- UTF-8 text workflows are the strongest supported path.
- Binary PDF/image/audio/video parsing is not fully claimed.
- Deep graph reasoning is not complete.
- Pattern/correlation engine is not complete.
- Prediction engine is not complete.
- Recommendation engine is not complete.
- Outcome learning is not complete.
- Experiment runner/self-improvement loop is not complete.
- Some UI flows are still scaffolds or partial.
- Some advanced reports depend on existing records and later implementation.
- The system does not yet fully answer “what has it learned?” across all evidence, outcomes, and experiments.

## Development Philosophy From Here

The Rust cutover is complete. The next phase is product completion.

Build from the user journey outward:

1. Understand the user request.
2. Decide whether it is a question, task, source add, approval, report, feedback, correction, or system action.
3. Clarify when necessary.
4. Convert valid requests into visible plans or work items.
5. Execute supported work through the worker.
6. Show status and evidence.
7. Produce reviewable results.
8. Capture user feedback and outcomes.
9. Convert failures or weak results into improvement items.
10. Run controlled experiments.
11. Promote better methods only after review and approval.

The project should avoid adding invisible magic.

Every major operation should be:

- visible;
- auditable;
- explainable;
- reversible where possible;
- tied to evidence or explicit user approval.

## Major Completion Tracks

The remaining full-project work is organized into tracks.

Each track should be implemented through scoped DIFFs.

### Track 1: Product Documentation and Repo Hygiene

Goal:

Make the repo understandable to users, operators, and future builders without exposing private build instructions on `main`.

Tasks:

- Keep `README.md` product-facing.
- Keep UI README accurate.
- Keep runtime docs current.
- Keep `main` stripped of build prompts.
- Keep `dev` complete and tracked.
- Document what is complete, partial, planned, and unsupported.
- Keep historical DIFFs locked.
- Keep runtime/private data out of the repo.

Done when:

- A new user can understand what IGY6 is.
- A new user can start it.
- A new user can identify what is supported now.
- A builder can understand the next development track.
- `main` has no private build-agent files.

### Track 2: Request Understanding

Goal:

IGY6 must understand what the user is asking before it acts.

The system must classify incoming requests into categories such as:

- answer a question from evidence;
- add a source;
- upload/add data;
- check work status;
- create a report;
- request an action;
- request a system-changing operation;
- give feedback;
- record an outcome;
- correct a prior answer;
- ask for diagnostics;
- ask for project/runtime status;
- ask for an experiment or improvement.

Required behavior:

- Identify the user intent.
- Identify required evidence.
- Identify missing details.
- Identify whether the request is safe.
- Identify whether approval is required.
- Identify whether a work item should be created.
- Identify whether clarification is needed.
- Present a plain-language request summary before doing risky or long-running work.

Deliverables:

- Stronger request classifier.
- Request summary object.
- Clarification-needed state.
- Request-to-plan preview.
- Request-to-work-item mapping.
- UI surface for “IGY6 understood this as...”.
- Tests for safe/unsafe/missing-info requests.

Done when:

- The UI can show what IGY6 thinks the user asked.
- The user can confirm or correct it.
- Unsupported or ambiguous requests do not silently become work.
- The system does not create unsafe work without approval.

### Track 3: Work Item System

Goal:

Work items should become the operational spine of IGY6.

A work item should record:

- what was requested;
- why it was requested;
- who/what created it;
- required source/evidence;
- current status;
- queued/running/completed/failed state;
- worker attempts;
- error messages;
- audit events;
- created artifacts;
- created evidence;
- chained work;
- result summary;
- approval state;
- final user acceptance/rejection.

Needed capabilities:

- Work item list UI.
- Work item detail page.
- Chained work visibility.
- Retry failed work.
- Cancel queued work.
- Mark result accepted/rejected.
- Explain why a work item failed.
- Link work item to evidence/report/output.
- Show worker execution history.

Done when:

- A normal user can see what IGY6 is doing.
- A failed job is understandable.
- A completed job links to useful outputs.
- Chained work is visible.
- Retry/cancel behavior is explicit and auditable.

### Track 4: Add Data and Source Management

Goal:

Make adding information easy and safe.

Supported source flow should include:

- source name;
- source type;
- permission scope;
- sensitivity;
- allowed operations;
- dry-run/preview;
- collection result;
- processing status;
- output evidence.

Near-term focus:

- Manual text upload.
- Manual notes.
- Local project source records.
- Basic file metadata.
- Source trust/sensitivity labels.

Later focus:

- PDFs.
- Images/screenshots.
- Audio/video.
- Browser/web collectors.
- Local diagnostic exports.
- Repo analysis.
- Router/network exports.

Done when:

- A user can add a supported source without understanding internal tables.
- The UI explains what will happen.
- Unsupported types are clearly labeled unsupported or planned.
- Source data creates traceable evidence.

### Track 5: Evidence and Answer Workflow

Goal:

IGY6 should answer questions with evidence and show its work.

Required behavior:

- Retrieve relevant chunks/evidence.
- Build an evidence packet.
- Generate or fallback to an evidence answer.
- Show citations/source trails.
- Separate facts from assumptions.
- Show insufficient-evidence results when needed.
- Allow user feedback on answer quality.
- Allow conversion of weak answers into improvement items.

Done when:

- A user can ask a question over stored evidence.
- The answer shows support.
- Weak or unsupported answers are clearly labeled.
- Feedback can be recorded.

### Track 6: Graph Memory and Relationship Views

Goal:

IGY6 should link information, not just search it.

Graph memory should represent:

- entities;
- observations;
- claims;
- evidence items;
- events;
- sources;
- reports;
- recommendations;
- predictions;
- outcomes;
- methods.

Relationship types should include:

- supported by;
- contradicted by;
- observed in;
- mentions;
- related to;
- occurred after;
- caused by / possibly caused by;
- confirmed by;
- disconfirmed by;
- produced;
- used method.

Done when:

- The system can show why records are connected.
- A user can inspect relationships.
- Graph data supports pattern detection and reports.

### Track 7: Pattern and Correlation Engine

Goal:

IGY6 should find useful relationships the user may not notice.

Pattern types:

- repeated event;
- temporal association;
- configuration drift;
- cross-source agreement;
- cross-source conflict;
- anomaly;
- recurring failure;
- recurring successful method;
- missing-information gap;
- repeated user correction;
- repeated unsupported answer type.

Pattern records should include:

- pattern type;
- evidence used;
- confidence;
- explanation;
- uncertainty;
- review status;
- user feedback;
- related work items;
- related reports.

Done when:

- IGY6 can propose patterns.
- Patterns are reviewable, not blindly accepted.
- Patterns cite supporting evidence.
- False/weak patterns can be rejected.

### Track 8: Prediction and Recommendation Engine

Goal:

IGY6 should make testable predictions and careful recommendations.

Predictions must include:

- conclusion;
- evidence used;
- confidence;
- uncertainty;
- expected result;
- what would prove it wrong;
- review status;
- later outcome.

Recommendations must include:

- observation;
- interpretation;
- suggested action;
- risk;
- approval requirement;
- expected result;
- rollback/safety note;
- outcome follow-up.

Done when:

- The system can create cautious, evidence-backed recommendations.
- Risk and approval requirements are visible.
- The system tracks whether recommendations worked.

### Track 9: Feedback and Outcome Learning

Goal:

IGY6 should learn from what happened.

Feedback should capture:

- useful/not useful;
- correct/incorrect;
- too vague;
- unsupported;
- missing evidence;
- bad retrieval;
- unsafe suggestion;
- good recommendation;
- failed recommendation;
- successful method.

Outcome records should capture:

- what action was taken;
- what happened;
- whether prediction was confirmed;
- whether recommendation helped;
- what changed afterward;
- what should be done differently next time.

Done when:

- User feedback changes future behavior through explicit records.
- Outcomes can confirm/disconfirm predictions.
- Repeated failures become improvement items.

### Track 10: Experimentation and Self-Improvement

Goal:

IGY6 should improve through controlled experiments, not guesswork.

Experiment flow:

1. Detect weak result or improvement opportunity.
2. Create improvement item.
3. Define baseline method.
4. Define candidate method.
5. Define test data.
6. Define metric.
7. Run experiment.
8. Compare baseline vs candidate.
9. Produce experiment report.
10. Require approval before promoting a method.
11. Track post-promotion outcomes.

Experiment records should include:

- hypothesis;
- baseline method;
- candidate method;
- dataset/evidence scope;
- metric;
- result;
- confidence;
- risk;
- approval status;
- promotion status.

Done when:

- The system can test a better method.
- It can explain whether the candidate improved results.
- It does not silently change production behavior.
- Promotion is approval-gated.

### Track 11: Reports and Exports

Goal:

IGY6 should produce readable decision-ready reports.

Reports should include:

- request or trigger;
- sources used;
- evidence summary;
- conflicts;
- patterns found;
- predictions/recommendations;
- confidence and uncertainty;
- actions taken;
- outcomes;
- next actions;
- unsupported areas;
- appendices/source trails.

Report types:

- evidence summary;
- source review;
- work item result;
- pattern report;
- prediction report;
- recommendation report;
- experiment report;
- project status report.

Done when:

- Reports are useful to a human.
- Reports link back to evidence.
- Reports can be exported or reviewed later.

### Track 12: UI/UX Completion

Goal:

The app should feel usable by a normal person, not like a developer console.

UI principles:

- Default screens should be simple.
- Technical details should live in Advanced.
- Empty states should explain what to do next.
- Actions should use normal language.
- Work status should be visible.
- Evidence should be understandable.
- Risk/approval should be obvious.
- Nothing important should be hidden.

Needed UI improvements:

- Add Data flow polish.
- Work item detail page.
- Evidence result cards.
- Answer with evidence view.
- Source detail page.
- Pattern review page.
- Recommendation review page.
- Experiment review page.
- Report viewer.
- Settings/safety clarity.
- Better error and empty states.

Done when:

- A user can complete the main workflow without needing developer knowledge.

### Track 13: Safety and Approval Hardening

Goal:

Prevent unsafe or unwanted actions.

Actions requiring approval:

- file writes;
- system setting changes;
- network/router changes;
- repository writes;
- external service calls;
- browser form submissions;
- sensitive data export;
- method promotion;
- destructive operations.

Required features:

- approval request records;
- approval UI;
- audit trail;
- rejection path;
- retry after approval;
- clear risk explanation;
- rollback note where applicable.

Done when:

- Risky actions cannot bypass the approval path.
- The user can understand and approve/reject actions.

### Track 14: End-to-End Product Hardening

Goal:

Make IGY6 reliable as a product.

Required checks:

- fresh clone startup;
- runtime smoke;
- lifecycle check;
- UI build;
- Rust tests;
- route parity;
- Docker Compose config;
- no private data committed;
- no build-agent files on `main`;
- startup/shutdown/restart behavior documented;
- main user journey verified.

Done when:

- A fresh user can start the system and complete the supported workflow.
- Known unsupported features are honestly labeled.
- Runtime validation is repeatable.

## Recommended DIFF Roadmap

The exact DIFF number should continue from the current repo state.

Suggested sequence:

### Phase 1: Product Alignment

- README/product vision alignment.
- Product docs and UI docs consistency.
- Current capability matrix.
- Dev/main branch tracking and policy verification.

### Phase 2: Request Understanding

- Intent/request classifier hardening.
- Clarification-needed workflow.
- Request summary and user correction.
- Request-to-plan preview.

### Phase 3: Work Items

- User-created work items from request plans.
- Work item list/detail UI.
- Retry/cancel/failure recovery.
- Chained work visibility.
- Work result review.

### Phase 4: Evidence Workflow

- Add Data flow polish.
- End-to-end text ingestion user journey.
- Evidence result cards.
- Evidence answer UI.
- Feedback on answers.

### Phase 5: Reports

- Report creation flow.
- Report viewer.
- Report export/review.
- Work item to report linkage.

### Phase 6: Graph and Patterns

- Graph relationship model verification.
- Relationship UI.
- Pattern detection MVP.
- Pattern review and feedback.

### Phase 7: Recommendations and Predictions

- Recommendation model MVP.
- Recommendation review UI.
- Prediction model MVP.
- Outcome capture for predictions and recommendations.

### Phase 8: Experimentation

- Improvement item model/UI.
- Experiment plan model.
- Experiment runner MVP.
- Baseline vs candidate comparison.
- Experiment report.
- Approval-gated method promotion.

### Phase 9: Final Product Hardening

- Full normal-user journey.
- Safety/approval hardening.
- Runtime validation expansion.
- Error/empty-state polish.
- Final docs pass.
- Versioned milestone release.

## Estimated Remaining DIFF Count

To reach a solid usable IGY6 product:

    15 to 22 focused DIFFs

To reach the full original adaptive intelligence vision:

    30 to 45 focused DIFFs

This estimate assumes each DIFF remains scoped, verified, and non-destructive.

## Immediate Next DIFF Candidates

Recommended next candidates:

1. Request understanding and clarification flow.
2. Work item detail and retry/failure UI.
3. Add Data end-to-end text workflow.
4. Evidence answer result polish.
5. Improvement item model and UI.

Best next DIFF:

    Request understanding and clarification flow

Reason:

The system must understand what the user wants before it creates work items, runs experiments, or makes recommendations.

## Verification Standards

Every implementation DIFF should run relevant checks.

Baseline checks:

    git status --short
    git diff --check
    python3 -m json.tool configs/rust-cutover-manifest.json
    cargo fmt --all --check
    cargo clippy --workspace --all-targets
    cargo test --workspace
    docker compose -f infra/docker-compose.yml --env-file .env.example config
    npm --prefix apps/web run build

Runtime posture checks:

    python3 scripts/post-cutover-runtime-audit.py
    scripts/post-cutover-smoke.sh --check
    scripts/fresh-clone-startup-check.sh --check
    scripts/runtime-lifecycle-check.sh --check
    scripts/rust-cutover.sh --check

UI checks when web UI changes:

    npm --prefix apps/web run build
    npm --prefix apps/web run test:ui-smoke

Do not run destructive commands during verification.

Do not touch runtime/private data unless the active DIFF explicitly allows it.

## Dev Branch Operating Rules

Before beginning work:

    git checkout dev
    git pull origin dev
    git merge origin/main
    git status --short

Before pushing dev:

    git status --short
    git log --oneline --decorate -8

Push dev:

    git push origin dev

If dev does not yet track origin/dev:

    git push -u origin dev

Do not push private runtime data.

Do not commit `target/`, `.env`, runtime artifacts, credentials, tokens, cookies, storage folders, or Docker volume data.

## Promotion Rules From Dev to Main

Never merge all of `dev` into `main`.

To promote product/runtime changes:

    git checkout main
    git pull origin main
    git checkout dev -- <specific file 1> <specific file 2>
    git status --short
    git diff --check
    git commit -m "<scoped product change>"
    git push origin main

Allowed to promote:

- product code;
- runtime scripts;
- user/operator docs;
- tests;
- config templates;
- public product plans.

Do not promote:

- `.codex`;
- root `AGENTS.md`;
- private build instruction documents;
- Codex prompt files;
- `docs/agents/` private prompts;
- runtime/private data.

## Definition of Done for Full Project Completion

IGY6 can be considered complete for the original vision when:

- A user can add authorized data.
- The system can process it into evidence.
- The system can answer questions with evidence.
- The system can show what it knows and what it does not know.
- The system can create and track work items.
- The system can detect useful patterns.
- The system can make cautious predictions and recommendations.
- The system can track outcomes.
- The system can learn from feedback.
- The system can run controlled experiments.
- The system can compare methods.
- The system can propose improvements.
- The system requires approval before risky actions or method promotion.
- The UI supports the normal workflow without developer knowledge.
- Runtime validation passes from a fresh clone.
- `main` remains product-facing.
- `dev` preserves full build context and agent instructions.
