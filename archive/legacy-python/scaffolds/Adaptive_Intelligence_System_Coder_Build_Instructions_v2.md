# Adaptive Intelligence System

## Comprehensive Coder Build Instructions - Version 2.0

Date: April 26, 2026 Purpose: Single all-in-one build instruction
document focused on function, capabilities, end goal, stack, and
component mapping.

# 0. Non-Negotiable End Product Goal

Build a private, local-first adaptive intelligence and decision-support
system that ingests authorized information, remembers it, links it,
finds patterns and correlations, makes predictions and recommendations,
checks outcomes, learns from feedback, and improves its own methods
through controlled experiments.

The system must be evidence-backed. Every conclusion must connect to
source evidence or be labeled as an assumption, hypothesis, estimate, or
unsupported statement.

The system must be safe by default. Read-only collection and analysis
are allowed. Changes to files, PC settings, router settings, accounts,
repositories, websites, or external services require explicit approval.

# 1. Product Definition

The finished product is not just a chatbot, RAG demo, dashboard, task
queue, or experiment tracker. It is a learning intelligence layer over
the user's authorized information.

The user should be able to ask:

- What does the system know?
- What has it learned?
- What changed recently?
- What patterns exist?
- What evidence supports this conclusion?
- What predictions did it make before?
- Were those predictions right?
- What advice worked or failed before?
- What information is missing?
- What should be done next?

## 1.1 Terminology and Roles

To avoid ambiguity in later phases, this document uses the following
definitions:

- **Source**: Any approved location of data (e.g., file upload, project
  repository, diagnostic export, authorized website). Each source has a
  unique identifier, a human‑friendly name, a `source_type`, connection
  parameters (e.g., file path or URL), a permission scope, and a
  sensitivity classification.
- **Evidence**: Normalized atomic facts extracted from raw artifacts.
  Evidence items are immutable and must cite their origin.
- **Claim**: A statement extracted or produced by the system, supported
  by one or more evidence items.
- **Pattern**: A detected relationship such as recurrence, temporal
  association, configuration drift, cross‑source agreement or conflict,
  failed‑advice recurrence, successful‑method recurrence, anomaly, or
  missing‑information gap.
- **Hypothesis**: A possible explanation that links claims or patterns,
  including both supporting and missing evidence.
- **Prediction**: A testable statement derived from a hypothesis that
  includes a conclusion, cited evidence, confidence, uncertainty,
  expected result, what would prove it wrong, and is later marked
  confirmed, disconfirmed, partially supported or inconclusive.
- **Recommendation**: A suggested action based on observations and
  interpretations. Recommendations must include the observation,
  interpretation, suggested action, risk, approval requirement, expected
  result, and safety or rollback note.
- **Outcome**: What actually happened after a recommendation was
  followed or a prediction was made.
- **Report**: A decision‑ready document summarizing sources, evidence,
  methods tried, metrics, findings, confidence, uncertainty,
  recommendations, and next actions.

User roles:

- **Owner/Administrator**: Controls source registration, permission
  scopes, approval of sensitive actions and method changes, and has full
  access to evidence, patterns, predictions, reports, and
  self‑improvement settings.

- **Analyst/User**: Ingests data within allowed scopes, asks questions,
  reviews evidence and patterns, records outcomes, provides feedback,
  and can propose improvements but cannot change system settings without
  approval.

- **Approver**: A role (which may coincide with the Owner) responsible
  for reviewing and approving or rejecting sensitive actions such as
  file changes, network modifications, external service calls, or method
  promotions.

- Single-user first implementation:

- 

- The first implementation may run in single-user mode where the
  Owner/Administrator, Analyst/User, and Approver are the same person.
  Even in single-user mode, the internal permission model, approval
  records, audit events, source scopes, and safety gates must still
  exist.

- 

- Single-user mode should reduce UI complexity, not remove safety
  controls. The system must still require explicit approval before
  sensitive or system-changing actions.

The system must:

- Ingest authorized information from local files, project repositories,
  PC diagnostics, router/network exports, websites, screenshots, manual
  notes, and user feedback.
- Normalize incoming information into searchable evidence.
- Store semantic memory in Qdrant.
- Store relationship memory in Neo4j.
- Detect patterns, correlations, conflicts, drift, recurring failures,
  and anomalies.
- Generate hypotheses, predictions, and recommendations with confidence
  and evidence.
- Track whether predictions and advice were right, wrong, useful, not
  useful, partial, or inconclusive.
- Improve methods for parsing, retrieval, graph extraction, prediction,
  confidence scoring, reporting, and reasoning through controlled
  experiments.
- Expose the entire system through a chat and review UI with evidence
  cards, confidence labels, and user controls.

# 2. Required Functions

The following functions are required. They may be implemented as
separate modules, services, or packages, but they must all exist and
connect through shared data models.

## F01 - Source Registry and Permission Policy

Purpose: Register every source and define what the system is allowed to
do with it.

Must store:

- Source name.
- Source type.
- Source location or connection method.
- Access scope.
- Sensitivity level.
- Allowed operations.
- External model policy.
- Enabled/disabled state.
- Audit history.

Primary stack item: PostgreSQL.

Example source record:

    {
      "id": "src-1234",
      "name": "Production Logs",
      "source_type": "manual_upload",
      "location": "/data/logs/",
      "scope": {"paths": ["/var/log/app/"], "operations": ["read"]},
      "sensitivity": "internal",
      "allowed_operations": ["read"],
      "external_model_policy": "no_external_models",
      "enabled": true
    }

Allowed values for `source_type` should come from an enumeration, for
example: `manual_upload`, `local_project`, `local_pc_diagnostics`,
`web_public`, `web_authorized_account`, `router_network`,
`user_observation`.

Each source must have a globally unique ID, and its permission policy
must specify what paths or domains are allowed, which operations
(`read`, `list`, `query`) are permitted, and any approval requirements.

## F02 - Data Ingestion and Connectors

Purpose: Collect authorized data from approved sources.

Required source types:

- Manual uploads.
- Local files and project repositories.
- PC diagnostic exports or approved read-only commands.
- Websites and documentation.
- Authorized web pages under the user's account.
- Router/network exports, screenshots, or read-only admin pages.
- Manual user observations and outcome notes.
- Conversation history and prior chat exports, when explicitly provided
  or authorized by the user.

Primary stack items: Celery workers, custom collectors, Playwright, safe
document readers.

Conversation history source:

The system should support a conversation_history source type for
user-approved exported chats, prior project discussions, coder handoff
documents, correction history, and earlier decision context.

This source type is important because the system must preserve user
intent over time, avoid repeating misunderstandings, and connect new
requests to prior project direction.

## F03 - Normalization and Sensitivity Classification

Purpose: Convert raw artifacts into usable evidence.

Must create:

- Normalized documents.
- Text chunks.
- Metadata.
- Observations.
- Sensitivity labels.
- Source links.
- Hashes and timestamps.

Primary stack items: Python workers and PostgreSQL.

## F04 - Evidence Ledger

Purpose: Store immutable evidence links so every answer can be traced
backward.

Must connect:

- Source.
- Collection run.
- Raw artifact.
- Normalized document.
- Chunk.
- Observation.
- Claim.
- Pattern.
- Hypothesis.
- Prediction.
- Recommendation.
- Report.
- Outcome.

Primary stack item: PostgreSQL.

## F05 - Vector Memory

Purpose: Store embeddings for semantic retrieval.

Must support:

- Semantic search.
- Metadata filtering.
- Similar prior case lookup.
- Retrieval of chunks, observations, reports, and reusable findings.

Primary stack item: Qdrant.

## F06 - Graph Memory

Purpose: Store relationships between information.

Must store:

- Entities.
- Events.
- Observations.
- Claims.
- Patterns.
- Hypotheses.
- Predictions.
- Recommendations.
- Outcomes.
- Methods.
- Reports.

Primary stack item: Neo4j.

## F07 - Evidence-Backed Chat

Purpose: Let the user ask questions and discuss what the system knows.

Must retrieve from:

- PostgreSQL metadata and evidence ledger.
- Qdrant vector memory.
- Neo4j graph memory.
- Previous reports.
- Prior predictions.
- Outcomes and feedback.

Primary stack items: Next.js UI, FastAPI, LangGraph, NeMo Guardrails.

## F08 - Pattern and Correlation Engine

Purpose: Detect useful relationships that may not be obvious.

Must detect:

- Repeated events.
- Temporal associations.
- Configuration drift.
- Cross-source agreement.
- Cross-source conflict.
- Anomalies.
- Failed-advice recurrence.
- Successful-method recurrence.
- Missing-information gaps.

Primary stack items: Neo4j, PostgreSQL, scikit-learn.

## F09 - Prediction and Advice Engine

Purpose: Produce testable predictions and recommendations.

Every prediction must include:

- Conclusion.
- Evidence used.
- Confidence.
- Uncertainty.
- Expected result.
- What would prove it wrong.
- Later outcome when known.

Every recommendation must include:

- Observation.
- Interpretation.
- Suggested action.
- Risk.
- Approval requirement.
- Expected result.
- Safety or rollback note.

Primary stack items: scikit-learn, StatsForecast, NeuralForecast,
PostgreSQL, Neo4j.

## F10 - Feedback and Outcome Tracking

Purpose: Let the system learn from what happened after advice or
predictions.

Must record:

- User feedback.
- Whether the advice was followed.
- What happened afterward.
- Whether the prediction was correct.
- Whether the recommendation was useful.
- What should change next time.

Primary stack items: PostgreSQL and Neo4j.

## F11 - Self-Improvement Queue

Purpose: Convert weak spots and improvement ideas into structured
experiment work.

Inputs include:

- Weak answers.
- Wrong predictions.
- Not-useful recommendations.
- Parser failures.
- Poor retrieval results.
- Bad confidence scoring.
- Missed patterns.
- User improvement ideas.
- Promising candidate methods.

Primary stack items: PostgreSQL and Celery.

## F12 - Experiment Runner

Purpose: Test better methods and compare them to the current baseline.

Must support:

- Candidate generation.
- Trial execution.
- Metric tracking.
- Baseline comparison.
- Abandoning weak branches.
- Preserving failures.
- Promoting promising methods.
- Producing experiment reports.

Primary stack items: Optuna, MLflow, Celery, artifact store.

## F13 - Report Generator

Purpose: Produce readable decision-ready reports, not raw logs.

Reports must include:

- Original request or trigger.
- Sources used.
- Evidence and conflicts.
- Methods tried.
- Metrics.
- Findings.
- Confidence and uncertainty.
- Recommendations.
- Next actions.
- Items needing review.

Primary stack items: FastAPI/report service, artifact store, PostgreSQL.

## F14 - Safety and Approval Gate

Purpose: Prevent unsafe, destructive, external, or sensitive actions
without approval.

Must block or require approval for:

- File changes.
- PC setting changes.
- Router setting changes.
- Account actions.
- Repository writes.
- External service actions.
- Browser form submissions.
- Sensitive data export.
- External model use with sensitive content.

Primary stack items: NeMo Guardrails, OWASP LLM controls, policy
middleware, audit events.

Approval flow:

- When a component attempts an operation that is on the blocked list
  (file changes, network modifications, etc.), it must create an
  approval request record in PostgreSQL and enqueue it for review.
- The UI must display pending approvals to users with the **Approver**
  role; they can approve or reject the request. Approvals and rejections
  must be audited.
- On approval, the operation is executed by the worker; on rejection,
  the operation is canceled and an appropriate message is returned to
  the requester.
- Approval logic must be enforced centrally (e.g., via middleware) so
  that connectors or models cannot bypass it.

## F15 - Admin and Review UI

Purpose: Let the user inspect and control the system.

Must expose:

- Sources.
- Permissions.
- Evidence.
- Graph relationships.
- Patterns.
- Predictions.
- Outcomes.
- Feedback.
- Work queue.
- Self-improvement experiments.
- Reports.
- Safety settings.

Primary stack item: Next.js UI.

# 3. Required User Capabilities

The user must be able to:

- Add an authorized source.
- Define source scope and permissions.
- Run a source dry-run before collection.
- Inspect what was collected.
- Ask questions over stored evidence.
- Inspect the source trail behind answers.
- Review detected patterns.
- Verify or reject a pattern.
- Receive predictions and advice with confidence.
- Record whether advice or predictions were correct or useful.
- Mark sources as trusted, noisy, sensitive, or disabled.
- Send weak results or promising ideas to self-improvement.
- Compare experiment results.
- Approve or reject proposed method changes.
- Export reports for coders or future context.

# 4. Baseline Stack and Exact Responsibility Map

This is the default implementation stack. Do not replace a major
component without documenting why, what replaces it, migration risk, and
security impact.

## 4.1 Next.js + React + TypeScript

Role: Frontend UI.

What it does:

- Chat interface.
- Evidence explorer.
- Source manager.
- Graph and pattern views.
- Prediction and outcome review.
- Self-improvement dashboard.
- Work queue.
- Reports.
- Settings and safety controls.

How it connects:

- Calls FastAPI only.
- Does not directly access databases, local files, Qdrant, Neo4j, Redis,
  MLflow, or Phoenix.

## 4.2 Python 3.12+ + FastAPI

Role: API backend.

What it does:

- Typed HTTP API.
- Auth/session handling.
- Source registration.
- Retrieval planner.
- Chat endpoint.
- Approval endpoint.
- Prediction/advice endpoint.
- Feedback/outcome endpoint.
- Report endpoint.
- Work queue endpoint.
- Self-improvement control endpoint.

How it connects:

- Reads/writes PostgreSQL.
- Queries Qdrant.
- Queries Neo4j.
- Enqueues Celery jobs through Redis.
- Sends traces to Phoenix.
- Reads experiment status from MLflow.
- Runs guardrail and policy checks.

## 4.3 PostgreSQL + SQLAlchemy + Alembic

Role: Relational control plane and audit store.

What it stores:

- Sources and permissions.
- Collection runs.
- Raw artifact metadata.
- Normalized documents.
- Chunks.
- Evidence items.
- Claims.
- Patterns.
- Hypotheses.
- Predictions.
- Recommendations.
- Outcomes.
- Feedback events.
- Work items.
- Improvement items.
- Experiment metadata.
- Reports.
- Approvals.
- Audit events.

How it connects:

- Source of truth for system state.
- Referenced by API, workers, reports, UI, and self-improvement engine.

## 4.4 Local Content-Addressed Artifact Store

Role: Raw and generated artifact storage.

What it stores:

- Uploaded files.
- Screenshots.
- Logs.
- Router exports.
- Collected web pages.
- Generated reports.
- Experiment artifacts.
- Model outputs.

How it connects:

- PostgreSQL stores metadata and file path/hash.
- Workers read/write artifacts.
- Reports link back to artifacts.

## 4.5 Qdrant

Role: Vector memory.

What it does:

- Stores embeddings.
- Performs semantic search.
- Finds similar prior cases.
- Retrieves relevant chunks and observations.
- Supports metadata filters.

How it connects:

- Workers upsert embeddings.
- FastAPI retrieval planner queries Qdrant during chat, reports, and
  pattern review.

## 4.6 Neo4j

Role: Graph memory.

What it does:

- Stores entities, events, observations, claims, relationships,
  patterns, hypotheses, predictions, recommendations, outcomes, methods,
  and reports.
- Enables relationship traversal.
- Supports evidence lineage.
- Supports correlation discovery.
- Supports explanation of why items are connected.

How it connects:

- Workers upsert nodes/relationships.
- API queries graph for reasoning, source trail, pattern detection, and
  reports.

## 4.7 Redis + Celery + Celery Beat

Role: Background execution and scheduling.

What it does:

- Ingestion jobs.
- Normalization jobs.
- Embedding jobs.
- Graph extraction jobs.
- Pattern scans.
- Report generation.
- Self-improvement experiments.
- Scheduled rechecks.

How it connects:

- FastAPI creates work items and enqueues jobs.
- Workers execute jobs and update PostgreSQL.

## 4.8 LlamaIndex Readers + Custom Collectors

Role: Ingestion helpers.

What they do:

- Use safe document readers for standard formats.
- Use custom collectors for PC, router, project, source-code, and
  sensitive sources.
- Convert collected input into RawArtifact and NormalizedDocument
  records.

How they connect:

- Run inside Celery workers.
- Write artifact metadata to PostgreSQL.
- Store raw files in artifact store.

## 4.9 Playwright

Role: Browser automation for authorized read-only collection.

What it does:

- Reads approved websites.
- Captures rendered pages.
- Captures screenshots.
- Reads router/admin pages when no export/API exists.

How it connects:

- Runs in collector sandbox.
- Requires explicit domain/source scope.
- Defaults to read-only.
- Browser actions that submit, edit, delete, buy, send, change, or save
  require approval.

## 4.10 LangGraph

Role: Controlled reasoning workflow.

What it does:

- Multi-step retrieval and reasoning.
- Clarification paths.
- Human approval checkpoints.
- Retry paths.
- Self-improvement handoffs.

How it connects:

- Runs inside API/worker service.
- Uses PostgreSQL, Qdrant, Neo4j, guardrails, and policy checks.
- Cannot bypass approval policy.

## 4.11 scikit-learn

Role: Baseline machine learning.

What it does:

- Classification.
- Clustering.
- Anomaly detection.
- Regression.
- Confidence calibration.
- Recommendation feature scoring.

How it connects:

- Uses feature tables created from evidence, patterns, feedback, and
  outcomes.
- Writes metrics and model outputs to MLflow/artifact store.

## 4.12 StatsForecast

Role: Classical time-series forecasting.

What it does:

- Forecasts recurring metrics.
- Forecasts counts, durations, failures, timings, and repeated signals.
- Provides inspectable baseline forecasts.

How it connects:

- Uses time-series feature tables.
- Establishes baseline before using neural forecasting.

## 4.13 NeuralForecast

Role: Advanced forecasting.

What it does:

- Neural time-series models when enough clean historical data exists.
- More complex prediction after simple baselines are insufficient.

How it connects:

- Phase 2 component.
- Must be compared against StatsForecast baseline.

## 4.14 Optuna

Role: Optimization engine.

What it does:

- Searches thresholds.
- Tunes ranking weights.
- Tunes parser settings.
- Tunes model parameters.
- Compares candidate method choices.

How it connects:

- Used by self-improvement runner.
- Trial results linked to MLflow.

## 4.15 MLflow

Role: Experiment tracking.

What it does:

- Tracks runs.
- Tracks parameters.
- Tracks metrics.
- Stores artifacts.
- Compares methods.
- Supports reproducibility.
- Tracks model/method versions.

How it connects:

- Required for self-improvement experiments.
- Reports link to MLflow run IDs.

## 4.16 Arize Phoenix Self-Hosted

Role: LLM and retrieval observability.

What it does:

- Traces prompts.
- Traces retrieval context.
- Traces tool calls.
- Tracks latency.
- Supports evaluation and debugging.

How it connects:

- FastAPI sends chat/retrieval traces.
- Self-improvement uses traces to find weak spots.

## 4.17 DSPy

Role: Prompt and reasoning pipeline optimization.

What it does:

- Metric-driven optimization of prompts and reasoning modules.
- Improves prompt/retrieval/pipeline behavior after stable evaluation
  datasets exist.

How it connects:

- Phase 2 component.
- Must log results to MLflow.
- Must not change production prompts without approval.

## 4.18 NeMo Guardrails + OWASP LLM Controls

Role: Safety and policy enforcement.

What they do:

- Input checks.
- Output checks.
- Prompt-injection resistance.
- Tool-use restrictions.
- Sensitive-data handling.
- Approval rules.
- External model restrictions.

How they connect:

- Wrap chat, retrieval, tool calls, browser actions, and generated
  actions.
- Policy decisions are stored in audit_events.

## 4.19 Docker Compose

Role: Local deployment.

What it does:

- Runs UI.
- Runs API.
- Runs workers.
- Runs Redis.
- Runs PostgreSQL.
- Runs Qdrant.
- Runs Neo4j.
- Runs MLflow.
- Runs Phoenix.

How it connects:

- Default local-first deployment.
- Must support backup and restore of volumes.

# 5. How Components Connect

    AUTHORIZED SOURCES
      manual uploads, screenshots, notes, logs
      allowed local folders and project repositories
      approved PC diagnostics or exports
      approved websites and router/admin pages
      user feedback and outcome notes
            |
            v
    SOURCE REGISTRY + PERMISSION POLICY [PostgreSQL]
            |
            v
    COLLECTORS / CONNECTORS [Celery + custom collectors + Playwright + safe readers]
            |
            v
    RAW ARTIFACT STORE [local content-addressed files]
            |
            v
    NORMALIZATION + SENSITIVITY CLASSIFICATION [Python workers]
            |
            v
    EVIDENCE LEDGER [PostgreSQL]
            |
            +--> EMBEDDINGS --> VECTOR MEMORY [Qdrant]
            |
            +--> ENTITIES / CLAIMS / EVENTS / RELATIONSHIPS --> GRAPH MEMORY [Neo4j]
            |
            +--> FEATURES / LABELS / OUTCOMES --> ML TABLES [PostgreSQL + artifacts]

    USER CHAT / REVIEW UI [Next.js]
            |
            v
    API BACKEND [FastAPI]
            +--> retrieval planner queries PostgreSQL + Qdrant + Neo4j
            +--> LangGraph controls reasoning workflow
            +--> guardrails enforce policy and safety checks
            +--> prediction/advice service creates evidence-backed outputs
            +--> feedback/outcome service updates memory and graph
            +--> Phoenix records traces
            |
            v
    SELF-IMPROVEMENT ENGINE
            +--> work items in PostgreSQL / Celery
            +--> optimization with Optuna
            +--> run tracking with MLflow
            +--> proposed method changes require approval
            |
            v
    REPORTS + ACCEPTED METHODS
            reports shown in UI; accepted methods update configurable system behavior

# 6. Repository Structure

    adaptive-intelligence/
      apps/
        web/                         # Next.js + React + TypeScript UI
      services/
        api/                         # FastAPI backend
        worker/                      # Celery jobs: ingestion, normalization, embeddings, graph, reports
        collectors/                  # Source connectors and collector sandbox code
        ml/                          # scikit-learn, StatsForecast, NeuralForecast, feature builders
        self_improvement/            # Optuna/MLflow experiment runner and method registry
        reports/                     # Report templates and renderers
      packages/
        schemas/                     # Shared Pydantic/TypeScript schemas
        policy/                      # Approval, sensitivity, and safety policy definitions
      infra/
        docker-compose.yml
        migrations/                  # Alembic migrations
        neo4j/                       # Neo4j constraints/indexes
        qdrant/                      # Qdrant collection configs
      configs/
        sources/                     # Source templates and connector configs
        guardrails/                  # NeMo Guardrails and safety configs
        evals/                       # Evaluation datasets and scoring configs
      storage/
        artifacts/                   # Content-addressed raw and generated artifacts
        exports/                     # User-exportable reports/bundles
      docs/
        architecture.md
        security-policy.md
        api.md
        user-guide.md
        operations.md

# 7. Core Data Model

## 7.1 PostgreSQL Required Tables

- sources: registered data source with type, name, owner, trust level,
  and enabled state.
- source_permissions: scope, allowed operations, sensitivity rules,
  external model policy, approval requirement.
- collection_runs: one execution of a connector, status, errors, counts,
  and timestamps.
- raw_artifacts: hash, path, source ID, MIME type, capture time,
  collector version.
- normalized_documents: clean text, title, document type, language,
  metadata, artifact link.
- chunks: chunk text, location, source document link, embedding status.
- evidence_items: atomic source-backed facts, observations, and claim
  references.
- claims: statements extracted or made by the system with evidence
  links.
- patterns: detected pattern records with type, confidence, evidence,
  and status.
- hypotheses: possible explanations with evidence, missing evidence, and
  confidence.
- predictions: testable predictions with expected result and disproof
  condition.
- recommendations: suggested actions with risk, approval requirement,
  and expected result.
- outcomes: what happened after a prediction, recommendation, or task.
- feedback_events: user labels such as useful, wrong, verified, noisy,
  trusted, rejected.
- work_items: queue records for tasks, ingestion, reports, experiments,
  review items.
- improvement_items: self-improvement targets and experiment plans.
- experiment_runs: links to MLflow run IDs, Optuna studies, metrics,
  artifacts.
- reports: generated summaries and decision documents.
- approvals: user approvals or denials for sensitive actions.
- audit_events: every important action, policy decision, source access,
  and change.

## 7.2 Neo4j Required Nodes

- Entity.
- Event.
- Observation.
- Claim.
- Pattern.
- Hypothesis.
- Prediction.
- Recommendation.
- Outcome.
- Method.
- Report.

## 7.3 Neo4j Required Relationships

- SUPPORTED_BY.
- CONTRADICTED_BY.
- OCCURRED_AFTER.
- POSSIBLY_CAUSED_BY.
- OBSERVED_IN.
- MENTIONS.
- USES_METHOD.
- PRODUCED.
- CONFIRMED_BY.
- DISCONFIRMED_BY.
- IMPROVES.
- FAILED_IN.

# 8. Source and Connector Rules

Every source must be registered before collection. Every connector must
support dry-run, permission validation, collection, normalization,
sensitivity classification, and cleanup.

Required source types:

- manual_upload: files, screenshots, pasted logs, JSON, CSV, PDF, DOCX,
  Markdown, and text.
- local_project: allowed project folders and repositories.
- local_pc_diagnostics: approved diagnostic exports or safe read-only
  command outputs.
- web_public: public docs, release notes, references, and pages.
- web_authorized_account: sites under the user's account with explicit
  approval.
- router_network: router exports, screenshots, admin UI pages,
  DHCP/DNS/log data.
- user_observation: manual notes, outcomes, corrections, and “this
  started after...” statements.
- conversation_history: user-approved exported chats, prior project
  discussions, coder handoff documents, correction history, and earlier
  decision context.

Connector contract:

- name.
- version.
- source_type.
- validate_scope(source_permission).
- dry_run(source, permission).
- collect(source, permission, run_context).
- normalize(raw_artifact).
- classify_sensitivity(normalized_document).
- extract_metadata(raw_artifact, normalized_document).
- cleanup(run_context).

Example Python interface for connectors:

    from typing import List, Protocol

    class Connector(Protocol):
        name: str
        version: str
        source_type: str

        def validate_scope(self, permission: "SourcePermission") -> None:
            """Validate that the source permissions are within the connector's allowed scope."""

        def dry_run(self, source: "Source", permission: "SourcePermission") -> "DryRunResult":
            """Return a summary of what would be collected without performing the collection."""

        def collect(self, source: "Source", permission: "SourcePermission", run_context: "CollectionRun") -> List["RawArtifact"]:
            """Collect raw artifacts from the source and return them."""

        def normalize(self, raw: "RawArtifact") -> "NormalizedDocument":
            """Convert a raw artifact into a normalized document."""

        def classify_sensitivity(self, doc: "NormalizedDocument") -> "SensitivityLabel":
            """Assign a sensitivity label to a normalized document."""

        def extract_metadata(self, raw: "RawArtifact", doc: "NormalizedDocument") -> "Metadata":
            """Extract additional metadata from the raw artifact and normalized document."""

        def cleanup(self, run_context: "CollectionRun") -> None:
            """Perform any necessary cleanup after a collection run."""

Each connector method should throw descriptive exceptions on failure and
log progress for observability. The example above uses type hints for
clarity; implementers should replace the quoted types with actual
classes or Pydantic models.

# 9. Chat and Reasoning Flow

Intent verification requirement:

Before creating a work item, running an experiment, executing a
collector, making a prediction, recommending a sensitive action, or
starting any major task, the system must summarize its understanding of
the user's intent and allow the user to verify, correct, abandon, or
revise that interpretation.

This verification should include:

· Original user request.

· System interpretation.

· Proposed work type.

· Sources likely to be used.

· Expected output.

· Safety or approval requirements.

· Known assumptions.

· Missing information.

If the task is low-risk and purely informational, the system may proceed
after showing its interpretation. If the task touches files, PC state,
router/network state, accounts, repositories, websites, external
services, source permissions, or production method changes, explicit
approval is required.

1.  User sends message.
2.  FastAPI creates chat_request record.
3.  Guardrails check prompt injection, unsafe request, source scope, and
    sensitive data.
4.  Intent classifier identifies question, source command, prediction
    request, feedback, report request, approval action, or improvement
    request.
5.  Retrieval planner queries PostgreSQL metadata, Qdrant semantic
    memory, Neo4j graph relationships, previous reports, and prior
    outcomes.
6.  Evidence ranker boosts recent, verified, trusted, and direct
    evidence; lowers noisy, contradicted, outdated, or rejected
    evidence.
7.  LangGraph workflow separates facts, assumptions, inferences,
    correlations, hypotheses, predictions, and advice.
8.  Answer generator produces response with evidence trail, confidence,
    uncertainty, conflicts, and next action.
9.  Output guardrails check unsafe instructions, leakage, unsupported
    claims, and missing approval requirements.
10. UI displays answer, evidence cards, confidence, and controls for
    useful, wrong, verified, noisy, record outcome, and send to
    self-improvement.

# 10. Pattern, Correlation, Prediction, and Advice Engine

Required pattern types:

- Recurrence: same or similar observation appears repeatedly.
- Temporal association: Event B repeatedly follows Event A within a time
  window.
- Configuration drift: current state differs from prior baseline.
- Cross-source agreement: independent sources support the same
  conclusion.
- Cross-source conflict: sources disagree.
- Failed-advice recurrence: advice failed before in a similar context.
- Successful-method recurrence: method repeatedly helped.
- Anomaly: observation deviates from baseline.
- Missing-information gap: confidence is blocked by absent evidence.

Example:

Suppose the system ingests weekly disk‑usage logs and deployment events.
It observes that when disk usage exceeds 90 % within 24 hours of a
deployment, error rates rise. This would be captured as a **temporal
association** pattern: “High disk usage → increased errors within 24 h
after deployment.” Similarly, if multiple sources report configuration
parameters drifting from baseline values over time, that is a
**configuration drift** pattern.

Prediction lifecycle: Candidate hypothesis -\> evidence gathered -\>
weak/plausible/strong -\> prediction created -\> outcome recorded -\>
confirmed/disconfirmed/partially supported/inconclusive -\> memory
updated.

Every prediction must include:

- Conclusion.
- Evidence used.
- Confidence.
- Uncertainty.
- Expected result.
- What would prove it wrong.
- Later outcome when known.

# 11. Self-Improvement System

The self-improvement system improves the methods used by the main
system. It does not replace the main system's memory, source ingestion,
chat, prediction tracking, or user-facing review layer.

Inputs:

- Weak answers.
- Wrong predictions.
- Not-useful recommendations.
- Parser failures.
- Poor retrieval.
- Bad confidence scoring.
- Missed patterns.
- User improvement ideas.
- Candidate methods.

Experiment process: 1. Verify intent. 2. Define success criteria. 3.
Generate candidate methods. 4. Run trials. 5. Compare to baseline. 6.
Save metrics and artifacts. 7. Abandon weak branches. 8. Report results.
9. Propose accepted method changes. 10. Require approval before
production behavior changes.

Tools:

- Celery for execution.
- Optuna for search and optimization.
- MLflow for run tracking and artifacts.
- Phoenix for trace analysis.
- DSPy in phase 2 for prompt/pipeline optimization.

## 11.1 Evaluation and Test Datasets

Define baseline evaluation tasks to verify each phase before full
deployment.

- **Classification baseline:** Use scikit‑learn to classify whether log
  entries are “error” or “normal” based on a small labeled CSV. This
  dataset can be stored under `storage/eval/classification_sample.csv`.
- **Time‑series baseline:** Use StatsForecast to predict CPU or memory
  usage from synthetic metrics in `storage/eval/time_series_sample.csv`
  and compare NeuralForecast models against these baselines.
- **Retrieval baseline:** Populate Qdrant with a handful of simple
  documents and verify that vector search returns expected matches given
  a query.

These datasets should be versioned and used both for initial testing and
later self‑improvement experiments. Metrics derived from these tasks
should be logged via MLflow so that new methods can be compared against
the baseline.

Advanced ML usage rule:

NeuralForecast and DSPy must not be used merely because they are
available. They should only be introduced when simpler baselines are
insufficient and measurable evaluation data exists.

Required baseline order:

1\. Use simple rules or direct evidence when sufficient.

2\. Use scikit-learn baselines for classification, clustering, anomaly
detection, and regression.

3\. Use StatsForecast for classical time-series forecasting before
neural forecasting.

4\. Use NeuralForecast only when enough clean historical data exists and
it beats the simpler baseline.

5\. Use DSPy only when there is a stable evaluation set and prompt or
reasoning pipeline changes can be measured.

Any advanced method must be compared against the current baseline and
logged through MLflow before it can be proposed for production use.

# 12. UI Pages and Controls

Required UI areas:

- Chat: ask questions and discuss findings.
- Sources: add, dry-run, enable, disable, delete where practical, review
  scope, mark trust/noise.
- Evidence Explorer: inspect documents, chunks, claims, observations,
  and source trail.
- Graph/Patterns: review entities, events, relationships, patterns, and
  hypotheses.
- Predictions/Advice: review expected outcomes and recommendations.
- Work Queue: track queued/running/blocked/completed/failed work.
- Self-Improvement: review experiments and method changes.
- Reports: read and export decision-ready reports.
- Settings/Safety: manage models, budgets, permissions, policies,
  external model use.

Required controls:

- Useful.
- Wrong.
- Verified.
- Incomplete.
- Show evidence.
- Record outcome.
- Send to self-improvement.
- Add source.
- Run dry-run.
- Enable/disable source.
- Mark trusted.
- Mark noisy.
- Approve action.
- Reject action.
- Pause.
- Resume.
- Cancel.
- Retry.
- Archive.
- Export report.

# 13. Safety and Privacy Rules

- Read-only default for all collectors and browser automation.
- Every source requires scope, sensitivity label, allowed operations,
  and audit trail.
- Credentials, cookies, tokens, passwords, API keys, and session data
  must be masked and excluded from model calls.
- Prompt-injection defense must treat web pages, files, logs, and notes
  as untrusted input.
- Any write, destructive, external, account, router, firewall, system,
  or repository change requires explicit approval.
- External model use is blocked for personal/system-sensitive data
  unless explicitly allowed by policy and approved.
- Browser automation must be domain-scoped and read-only unless a
  specific approved action exists.
- All source access, tool calls, policy decisions, approvals, denials,
  reports, and method changes must be audited.
- Accepted method changes must be versioned and reversible where
  practical.

## 13A. Logging and Observability

Reliable logging and observability are required to debug and improve the
system. The API and workers must emit structured logs including at
minimum a timestamp, service/module name, severity, and a correlation ID
that links related events across services. Phoenix is used to trace chat
and retrieval events; logs related to pattern detection, prediction, and
self‑improvement runs should also be sent to Phoenix.

Error handling conventions:

- Any uncaught exception in a collector or background job must mark the
  associated work item as “failed” with an error message stored in
  PostgreSQL. The full stack trace should be captured in the artifact
  store for later analysis.
- Connector and normalization functions should throw descriptive
  exceptions when invalid input is encountered; these errors must be
  propagated to the UI.
- Health checks should be implemented for each service and exposed
  through the API to support monitoring and alerting.

# 14. Reports

Reports must be decision-ready, not raw logs.

Every report must include:

- Title and report type.
- Original request or trigger.
- Verified intent or inferred purpose.
- Sources used and source quality.
- Relevant evidence and conflicts.
- Methods tried or reasoning path used.
- Metrics if any.
- Facts, assumptions, hypotheses, predictions, and recommendations
  separated clearly.
- Confidence and uncertainty.
- Outcome status if known.
- Next actions and items needing review.
- Links to artifacts, runs, graph nodes, and evidence records.

# 15. Build Phases

## Phase 0 - Project Skeleton

Build:

- Monorepo.
- Docker Compose.
- FastAPI.
- Next.js.
- PostgreSQL.
- Redis/Celery.
- Qdrant.
- Neo4j.
- MLflow.
- Phoenix.

Done when:

- All services run locally.
- Health checks pass.
- Migrations apply.

## Phase 1 - Source and Evidence Core

Build:

- Source registry.
- Permission model.
- Manual upload connector.
- Local project connector.
- Artifact store.
- Normalization.
- Evidence ledger.

Done when:

- User can ingest files/folders and inspect evidence.

## Phase 2 - Vector and Graph Memory

Build:

- Chunking.
- Embeddings.
- Qdrant upserts/search.
- Entity/claim extraction.
- Neo4j graph upserts/traversal.

Done when:

- User can semantic-search and inspect relationships.

## Phase 3 - Evidence-Backed Chat

Build:

- Retrieval planner.
- LangGraph reasoning flow.
- Source cards.
- Confidence labels.
- Feedback buttons.
- Phoenix traces.

Done when:

- Answers cite evidence and accept feedback.

## Phase 4 - Patterns and Predictions

Build:

- Recurrence detection.
- Temporal association detection.
- Drift detection.
- Conflict detection.
- Anomaly detection.
- Prediction/advice records.
- Outcome tracking.

Done when:

- System creates testable predictions and records outcomes.

## Phase 5 - Self-Improvement MVP

Build:

- Improvement queue.
- Experiment runner.
- Optuna/MLflow tracking.
- Method registry.
- Experiment reports.

Done when:

- System compares methods and proposes improvements.

## Phase 6 - Browser and Router Collectors

Build:

- Playwright read-only collection for approved websites/router pages.
- Screenshot capture.
- Page text extraction.
- Domain scope enforcement.

Done when:

- Browser collection is scoped, auditable, and approval-controlled.

## Phase 7 - Advanced ML and Optimization

Build:

- StatsForecast models.
- NeuralForecast models where justified.
- scikit-learn models.
- DSPy experiments.
- Evaluation datasets.

Done when:

- Predictions improve against measurable baselines.

## Phase 8 - Hardening and Export

Build:

- Security review.
- Data deletion/export.
- Report export.
- Backup/restore.
- Performance tests.
- Documentation.

Done when:

- System is ready for long-term local use.

# 16. Acceptance Criteria

A coder can consider the build aligned only when:

- User can add an authorized source and see its exact permission scope.
- User can ingest files, notes, logs, project folders, web pages,
  router/network exports, and manual observations.
- System stores raw artifacts, normalized documents, chunks, evidence
  records, vector embeddings, and graph relationships.
- User can ask a chat question and receive an evidence-backed answer
  with confidence and uncertainty.
- User can inspect source evidence behind a claim, prediction,
  recommendation, report, or pattern.
- System detects recurrence, drift, source conflict, temporal
  association, and anomaly patterns.
- System creates testable predictions and records whether they were
  correct, wrong, partial, or inconclusive.
- System remembers failed advice and does not repeat it as if new.
- User can mark sources trusted/noisy and answers
  useful/wrong/verified/incomplete.
- System can send weak spots or improvement goals into the
  self-improvement queue.
- Self-improvement runner tests multiple candidate methods, compares
  metrics, and produces a report.
- No system-changing action occurs without explicit approval.
- Every sensitive source access and important decision is auditable.
- Reports are exportable and readable without digging through raw logs.
- All services start through Docker Compose and can be backed
  up/restored.

# 17. Final Coder Summary

Build a local-first adaptive intelligence system. It should take in
authorized information from files, PC diagnostics, project repositories,
notes, websites, router/network data, screenshots, and manual
observations. It should store the raw evidence, normalize it, embed it
into vector memory, connect it into a graph, find patterns and
correlations, create hypotheses, make predictions or recommendations,
track whether they were right, and adjust future reasoning. It must
include a chat interface where the user can ask what the system knows,
what it learned, why it believes something, what evidence supports it,
what changed, and what should happen next. It must include a
self-improvement engine that tests better methods for parsing,
retrieval, scoring, prediction, reporting, and reasoning. It must be
read-only and permissioned by default, with approval required before any
change to files, PC settings, router settings, accounts, repositories,
websites, or external services.

# 18. Operational Considerations

Although the system is designed for local-first operation, deployment
and maintenance require additional guidance:

- **Environment variables and secrets:** Define environment variables
  for database URLs, authentication tokens, Qdrant and Neo4j
  credentials, MLflow tracking URI, and Phoenix configuration. Keep
  secrets in an `.env` file or secret manager; do not hard‑code them.
- **Data volumes:** Use Docker volumes to persist PostgreSQL, Qdrant,
  Neo4j, MLflow, and Phoenix data. Provide scripts to back up and
  restore these volumes.
- **Hardware requirements:** Minimum recommended hardware for local
  development is 16 GB RAM and 4 CPU cores. For production use, adjust
  resource allocations based on data volume and expected query
  throughput.
- **Backup and restore:** Document how to back up each component’s data
  and how to restore from a backup. Regular backups should be automated
  via cron or Celery beat jobs.
- **Network access and firewalls:** Restrict service ports to localhost
  unless remote access is explicitly required and secured via SSH
  tunnels or VPN.
- **Update and migration:** When upgrading database schemas via Alembic
  or changing Neo4j/Qdrant structures, provide migration scripts and
  ensure that versioning is reflected in the repository.

These considerations ensure that the coder sets up a robust,
reproducible local deployment that can be upgraded and maintained over
time.

------------------------------------------------------------------------
