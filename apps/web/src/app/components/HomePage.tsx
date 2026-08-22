import type { HealthResponse, SourceRecord, CollectionRunRecord, RawArtifactRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, EvidenceAnswerRecord, ClaimRecord, VectorCollectionStatus, GraphSchemaStatus, PatternRecord, HypothesisRecord, PredictionRecord, RecommendationRecord, CalibrationSummary, WorkItemRecord, AgentTaskPlanRecord, ApprovalRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ExperimentRecord, ReportRecord, AuditEventRecord, EnvSettingsResponse, AgentCapabilitiesResponse } from "./types";
import { MINIMAL_UI_TOGGLE_SCRIPT, WORKSPACE_HASH_ROUTER_SCRIPT, RUNTIME_POSTURE, USER_READINESS } from "./constants";
import { formatDate, formatBytes, excerpt, evidenceReviewState, workItemRelatedIds, workItemGuidance, workItemDispatchVisibility } from "./helpers";
import { settingValue } from "./llm-helpers";
import { getJson } from "./api";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";
import { TermHelp } from "./ui/TermHelp";
import { HelpHeading } from "./ui/HelpHeading";
import { AgentCommandPanel } from "./AgentCommandPanel";
import { AgentTaskHistoryReview } from "./AgentTaskHistoryReview";
import { BaselinePatternExpansionPanel } from "./BaselinePatternExpansionPanel";
import { BasicReportWorkflow } from "./BasicReportWorkflow";
import { BrowserWebRouterCollectorMvp } from "./BrowserWebRouterCollectorMvp";
import { BypassIntelPanel } from "./BypassIntelPanel";
import { ChatRetrievalPreview } from "./ChatRetrievalPreview";
import { ChatWebFetchDock } from "./ChatWebFetchDock";
import { ConnectorContractStatusPanel } from "./ConnectorContractStatusPanel";
import { ConversationHistoryImport } from "./ConversationHistoryImport";
import { EntityClaimEventFoundationPanel } from "./EntityClaimEventFoundationPanel";
import { EvidenceAnswerHistory } from "./EvidenceAnswerHistory";
import { EvidenceCorrectionSupersessionWorkflow } from "./EvidenceCorrectionSupersessionWorkflow";
import { EvidenceDetailPanel } from "./EvidenceDetailPanel";
import { EvidenceFeedbackWorkflow } from "./EvidenceFeedbackWorkflow";
import { GraphLineageExplanationPanel } from "./GraphLineageExplanationPanel";
import { GuidedManualTextUpload } from "./GuidedManualTextUpload";
import { ImprovementExperimentReview } from "./ImprovementExperimentReview";
import { LifecycleAuditStatusPanel } from "./LifecycleAuditStatusPanel";
import { LocalLlmStatusPanel } from "./LocalLlmStatusPanel";
import { LocalProjectPcDiagnosticsHardeningPanel } from "./LocalProjectPcDiagnosticsHardeningPanel";
import { MediaImportMvp } from "./MediaImportMvp";
import { MinimalWorkspacePanel } from "./MinimalWorkspacePanel";
import { MissingEvidencePromptPanel } from "./MissingEvidencePromptPanel";
import { MvpActionConsole } from "./MvpActionConsole";
import { OnboardingJourney } from "./OnboardingJourney";
import { OutcomeLearningSummary } from "./OutcomeLearningSummary";
import { PipelineOperationsPanel } from "./PipelineOperationsPanel";
import { PredictionRecommendationCreator } from "./PredictionRecommendationCreator";
import { PredictionRecommendationOutcomeReview } from "./PredictionRecommendationOutcomeReview";
import { SettingsHubNav } from "./SettingsHubNav";
import { SettingsPanel } from "./SettingsPanel";
import { SourceCollectionApprovalReview } from "./SourceCollectionApprovalReview";
import { SourceDetailPanel } from "./SourceDetailPanel";
import { SourceEvidenceHistory } from "./SourceEvidenceHistory";
import { SourceTrustSensitivityManagement } from "./SourceTrustSensitivityManagement";
import { TroubleshootingLogsPanel } from "./TroubleshootingLogsPanel";
import { UnifiedChatHub } from "./UnifiedChatHub";
import { UserObservationIngestion } from "./UserObservationIngestion";
import { UserSecurityPanel } from "./UserSecurityPanel";

export async function HomePage() {
  const [
    health,
    sources,
    collectionRuns,
    artifacts,
    documents,
    chunks,
    evidenceItems,
    evidenceAnswers,
    claims,
    vectorCollection,
    graphSchema,
    patterns,
    hypotheses,
    predictions,
    recommendations,
    calibrationSummary,
    workItems,
    approvals,
    feedback,
    outcomes,
    improvements,
    experiments,
    reports,
    agentTaskPlans,
    auditEvents,
    envSettings,
    agentCapabilities
  ] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", []),
    getJson<NormalizedDocumentRecord[]>("/evidence/documents", []),
    getJson<ChunkRecord[]>("/evidence/chunks", []),
    getJson<EvidenceItemRecord[]>("/evidence/items", []),
    getJson<EvidenceAnswerRecord[]>("/evidence-answers", []),
    getJson<ClaimRecord[]>("/evidence/claims", []),
    getJson<VectorCollectionStatus>("/memory/vector/chunks", { collection_name: "unknown", exists: false }),
    getJson<GraphSchemaStatus>("/memory/graph/schema", { constraints: [] }),
    getJson<PatternRecord[]>("/analysis/patterns", []),
    getJson<HypothesisRecord[]>("/analysis/hypotheses", []),
    getJson<PredictionRecord[]>("/analysis/predictions", []),
    getJson<RecommendationRecord[]>("/analysis/recommendations", []),
    getJson<CalibrationSummary>("/analysis/calibration/summary", {
      schema_version: "prediction_recommendation_calibration_summary.v1",
      record_counts: {
        predictions: 0,
        recommendations: 0,
        total: 0,
        evidence_linked: 0,
        with_outcome: 0
      },
      outcome_counts: {
        correct: 0,
        wrong: 0,
        partial: 0,
        useful: 0,
        not_useful: 0,
        inconclusive: 0,
        total: 0
      },
      by_kind: {
        prediction: { records: 0, outcomes: 0 },
        recommendation: { records: 0, outcomes: 0 }
      },
      confidence_bands: {},
      calibration_status: "unavailable",
      limitations: [],
      forecasting_engine: false,
      auto_execute_recommendations: false,
      advanced_calibration: false
    }),
    getJson<WorkItemRecord[]>("/work-items", []),
    getJson<ApprovalRecord[]>("/approvals", []),
    getJson<FeedbackRecord[]>("/feedback", []),
    getJson<OutcomeRecord[]>("/outcomes", []),
    getJson<ImprovementRecord[]>("/improvements", []),
    getJson<ExperimentRecord[]>("/experiments", []),
    getJson<ReportRecord[]>("/reports", []),
    getJson<AgentTaskPlanRecord[]>("/agent/task-plans", []),
    getJson<AuditEventRecord[]>("/audit-events", []),
    getJson<EnvSettingsResponse>("/settings/env", {
      file_status: {
        path: "unknown",
        backup_dir: "unknown",
        exists: false,
        writable: false,
        unknown_key_count: 0,
        output_format: "unknown"
      },
      groups: [],
      settings: [],
      unmanaged: [],
      warnings: []
    }),
    getJson<AgentCapabilitiesResponse>("/agent/capabilities", {
      actions: [],
      runtime: {
        repo_root: "unknown",
        docker_cli_available: false,
        docker_compose_available: false,
        docker_socket_available: false,
        docker_host_configured: false,
        docker_control_available: false,
        docker_socket_path: null,
        reason: "Agent capabilities were unavailable."
      },
      policy: {
        local_first: true,
        hosted_ai_enabled: false,
        external_model_policy: "blocked_by_default",
        arbitrary_command_execution: false,
        prompt_injection_filter: "unavailable",
        approval_required_for_system_changing: true,
        blocked_request_classes: []
      }
    })
  ]);

  const checks = health.data.checks ?? {};
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentArtifacts = artifacts.data.slice(0, 6);
  const recentDocuments = documents.data.slice(0, 5);
  const recentChunks = chunks.data.slice(0, 5);
  const recentEvidence = evidenceItems.data.slice(0, 5);
  const recentClaims = claims.data.slice(0, 5);
  const recentPatterns = patterns.data.slice(0, 4);
  const recentHypotheses = hypotheses.data.slice(0, 4);
  const recentPredictions = predictions.data.slice(0, 4);
  const recentRecommendations = recommendations.data.slice(0, 4);
  const recentWorkItems = workItems.data.slice(0, 8);
  const recentApprovals = approvals.data.slice(0, 4);
  const recentFeedback = feedback.data.slice(0, 4);
  const recentOutcomes = outcomes.data.slice(0, 4);
  const recentReports = reports.data.slice(0, 4);
  const recentAuditEvents = auditEvents.data.slice(0, 4);
  const pendingApprovals = approvals.data.filter((approval) => approval.status === "pending");
  const approvedApprovals = approvals.data.filter((approval) => approval.status === "approved");
  const rejectedApprovals = approvals.data.filter((approval) => ["denied", "rejected"].includes(approval.status));
  const queuedWorkItems = workItems.data.filter((item) => item.status === "queued");
  const runningWorkItems = workItems.data.filter((item) => item.status === "running");
  const completedWorkItems = workItems.data.filter((item) => item.status === "completed");
  const failedWorkItems = workItems.data.filter((item) => item.status === "failed");
  const blockedActions = agentCapabilities.data.actions.filter((action) => !action.executable_in_api_runtime);
  const approvalRequiredActions = agentCapabilities.data.actions.filter((action) => action.approval_required);
  const policyPosture = agentCapabilities.data.policy;
  const blockedRequestClasses = policyPosture?.blocked_request_classes ?? [];

  const queuedWorkCount = workItems.data.filter((item) => item.status === "queued" || item.status === "pending_intent_verification" || item.status === "running").length;

  return (
    <main className="consoleShell chatFirstShell" data-minimal-ui-root data-minimal-ui-active="false">
      <aside className="leftSidebar" aria-label="IGY6 navigation">
        <div className="brandBlock">
          <div className="brandMark">IG</div>
          <div>
            <strong>IGY6</strong>
            <span>Chat-first evidence workspace</span>
          </div>
        </div>

        <div className="sidebarActions">
          <label className="sidebarButton primary" htmlFor="tab-results">Open chat</label>
          <button type="button" className="sidebarButton" data-minimal-ui-toggle aria-pressed="false">Simple mode</button>
        </div>

        <p className="sidebarHint" data-minimal-ui-hint>Say what you want in chat — I&apos;ll ask plain questions when I&apos;m not sure what you mean.</p>

        <nav className="navSection compactNav" aria-label="Workspace views">
          <label htmlFor="tab-results">Chat</label>
          <label htmlFor="tab-add-data">Data</label>
          <label htmlFor="tab-work">Work</label>
          <label htmlFor="tab-settings">Settings</label>
          <label htmlFor="tab-advanced">More</label>
        </nav>

        <section className="sidebarList" aria-label="Recent work">
          <div className="sidebarHeading">
            <span>Recent work</span>
            <StatusPill state="rust-worker" />
          </div>
          {recentWorkItems.map((workItem) => (
            <article className="miniRecord" key={workItem.id}>
              <strong>{workItem.work_type}</strong>
              <span>{workItem.status} · {formatDate(workItem.created_at)}</span>
            </article>
          ))}
          {recentWorkItems.length === 0 ? <EmptyState label="No work items yet." /> : null}
        </section>

        <footer className="localFooter">
          <StatusPill state="local-first" />
          <span>Read-only by default · No external model</span>
        </footer>
      </aside>

      <section className="mainConsole">
        <header className="topBar compactTopBar responsiveToolbar">
          <div>
            <p className="eyebrow">IGY6</p>
            <h1>Local Evidence Workspace</h1>
          </div>
          <div className="topStatus responsiveStatusRow">
            <button type="button" className="simpleModeToggle" data-minimal-ui-toggle aria-pressed="false">Simple mode</button>
            <StatusPill state="local-first" />
            <StatusPill state={health.data.status} />
          </div>
        </header>

        <section className="productTabs compactTabs" aria-label="Main dashboard tabs">
          <input className="tabInput" id="tab-add-data" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-work" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-results" name="main-dashboard-tab" type="radio" defaultChecked />
          <input className="tabInput" id="tab-settings" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-advanced" name="main-dashboard-tab" type="radio" />
          <nav className="tabList" aria-label="Main dashboard">
            <label role="tab" htmlFor="tab-results">Chat</label>
            <label role="tab" htmlFor="tab-add-data">Data</label>
            <label role="tab" htmlFor="tab-work">Work</label>
            <label role="tab" htmlFor="tab-settings">Settings</label>
            <label role="tab" htmlFor="tab-advanced">More</label>
          </nav>
        </section>

        <section className="panel workflowHero tabContent" id="home" data-tab-panel="results">
          <div className="panelHeader">
            <div>
              <p className="eyebrow">Home</p>
              <h2>System Ready</h2>
            </div>
            <StatusPill state={health.data.status} />
          </div>
          <section className="readinessStrip" aria-label="Current readiness">
            {USER_READINESS.map((item) => (
              <article key={item.label}>
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <StatusPill state={item.state} />
              </article>
            ))}
          </section>
          <p className="readinessSummary">System ready. Background worker ready. {pendingApprovals.length > 0 ? "Review pending approvals before sensitive collection." : "No approval needs attention right now."}</p>
          <section className="metrics compact" aria-label="Home overview">
            <article><span>Service readiness</span><strong>{Object.keys(checks).length ? `${Object.values(checks).filter((check) => check.status === "ok").length}/${Object.keys(checks).length}` : "Unknown"}</strong></article>
            <article><span>Recent data activity</span><strong>{recentRuns.length + recentArtifacts.length}</strong></article>
            <article><span>Recent work</span><strong>{recentWorkItems.length}</strong></article>
            <article><span>Pending approvals</span><strong>{pendingApprovals.length}</strong></article>
            <article><span>Recent audit events</span><strong>{recentAuditEvents.length}</strong></article>
          </section>
          <div className="primaryWorkflowGrid" aria-label="Primary workflows">
            <article>
              <span>1</span>
              <h3>Add data</h3>
              <p>Create a scoped source and upload approved UTF-8 text such as notes, logs, or exports.</p>
              <label htmlFor="tab-add-data">Open Add Data</label>
            </article>
            <article>
              <span>2</span>
              <h3>Check processing</h3>
              <p>See what is waiting, running, completed, or needs attention.</p>
              <label htmlFor="tab-work">Open Work</label>
            </article>
            <article>
              <span>3</span>
              <h3>Ask with evidence</h3>
              <p>{sources.data.length === 0 ? "Add a data source first." : evidenceItems.data.length === 0 ? "Add approved text and check processing." : "Ask a question over local evidence."}</p>
              <label htmlFor="tab-results">Open Chat</label>
            </article>
          </div>
        </section>

        <section className="chatStage workflowSection tabContent" id="assistant" data-tab-panel="results">
          <MinimalWorkspacePanel
            sourceCount={sources.data.length}
            evidenceCount={evidenceItems.data.length}
            queuedWorkCount={queuedWorkCount}
            pendingApprovals={pendingApprovals.length}
          />
          <OnboardingJourney
            sourceCount={sources.data.length}
            evidenceCount={evidenceItems.data.length}
            chunkCount={chunks.data.length}
            llmEnabled={(settingValue(envSettings.data, "LLM_PROVIDER", "none") || "none") === "ollama"}
            llmModel={settingValue(envSettings.data, "OLLAMA_MODEL", "") || "not selected"}
          />
          <ChatWebFetchDock />
          <UnifiedChatHub
            sourceCount={sources.data.length}
            evidenceCount={evidenceItems.data.length}
            chunkCount={chunks.data.length}
            workItemCount={workItems.data.length}
            pendingApprovals={pendingApprovals.length}
            vectorReady={vectorCollection.data.exists}
            llmEnabled={(settingValue(envSettings.data, "LLM_PROVIDER", "none") || "none") === "ollama"}
            llmModel={settingValue(envSettings.data, "OLLAMA_MODEL", "") || "not selected"}
          />

          <section className="chatResultsDock" aria-label="Evidence and action results">
            <ChatRetrievalPreview />
          </section>

          <details className="chatContextPanels">
            <summary>Context panels (LLM status, evidence gaps, history)</summary>
            <LocalLlmStatusPanel envSettings={envSettings} context="assistant" />
            <MissingEvidencePromptPanel
              evidenceItems={evidenceItems}
              chunks={chunks}
              sources={sources}
              evidenceAnswers={evidenceAnswers}
              taskPlans={agentTaskPlans}
            />
            <EvidenceAnswerHistory evidenceAnswers={evidenceAnswers} feedback={feedback} />
            <AgentCommandPanel capabilities={agentCapabilities} approvals={approvals} taskPlans={agentTaskPlans} />
            <AgentTaskHistoryReview
              taskPlans={agentTaskPlans}
              workItems={workItems}
              approvals={approvals}
              feedback={feedback}
              outcomes={outcomes}
              improvements={improvements}
            />
          </details>
        </section>

        <section className="panel diagnosticsPanel tabContent" id="advanced-diagnostics" data-tab-panel="advanced">
          <div className="panelHeader">
            <div>
              <p className="eyebrow">Advanced</p>
              <h2>Diagnostics</h2>
            </div>
            <StatusPill state={health.data.status} />
          </div>
          <section className="runtimePosture" aria-label="Technical runtime posture">
            {RUNTIME_POSTURE.map((item) => (
              <article key={item.label}>
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <StatusPill state={item.state} />
              </article>
            ))}
          </section>
          <section className="split">
            <article className="panelInset">
              <h3>Service readiness</h3>
              <div className="checkList">
                {Object.entries(checks).map(([name, check]) => (
                  <article className="checkRow" key={name}>
                    <span>{name}</span>
                    <StatusPill state={check.status} />
                  </article>
                ))}
                {Object.keys(checks).length === 0 ? <EmptyState label="No readiness details returned." /> : null}
              </div>
            </article>
            <article className="panelInset">
              <h3>Recent audit</h3>
              <div className="stack">
                {recentAuditEvents.map((event) => (
                  <article className="miniRecord" key={event.id}>
                    <strong>{event.event_type}</strong>
                    <span>{event.decision ?? "recorded"} · {event.actor_id}</span>
                  </article>
                ))}
              </div>
              {recentAuditEvents.length === 0 ? <EmptyState label="No audit events yet." /> : null}
            </article>
          </section>
        </section>

        <section className="panel toolConsole tabContent" aria-label="Advanced route console" data-tab-panel="advanced">
          <details>
            <summary>
              <span>
                <strong>Advanced Route Console</strong>
                <em>Existing API-backed controls · no new workflow behavior</em>
              </span>
              <StatusPill state="advanced" />
            </summary>
            <MvpActionConsole />
          </details>
        </section>

        <section className="workspaceGrid" aria-label="IGY6 workflow records">
          <section className="panel workflowSection tabContent" id="data-knowledge" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data</p>
                <h2>Bring In Authorized Information</h2>
              </div>
              <StatusPill state="local-first" />
            </div>
            <div className="lifecycleFlow" aria-label="Data lifecycle">
              {["Source", "Upload / Collection", "Raw Artifact", "Document", "Chunks", "Evidence", "Memory", "Analysis / Chat Retrieval"].map((step) => (
                <span key={step}>{step}</span>
              ))}
            </div>
            <section className="workflowTabs" aria-label="Add data steps">
              <a href="#data-overview">Overview</a>
              <a href="#sources-panel">Sources</a>
              <a href="#uploads-collection">Uploads</a>
              <a href="#browser-web-router-import">Web fetch</a>
              <a href="#chat-web-fetch">Chat web fetch</a>
            </section>
            <div className="quickStartGrid" id="data-overview">
              <article>
                <h3>Normal PC user examples</h3>
                <p>Upload warranty notes, router troubleshooting notes, a bill note, or a folder inventory and ask what changed, what expires, or what looks duplicated.</p>
              </article>
              <article>
                <h3>Project examples</h3>
                <p>Upload build notes, verification summaries, or project logs and ask what failed, what changed, or what needs review.</p>
              </article>
            </div>
          </section>

          <section className="panel tabContent" id="sources-panel" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data / Sources</p>
                <h2><HelpHeading term="source">Where Your Data Comes From</HelpHeading></h2>
              </div>
              {sources.error ? <span className="errorText">{sources.error}</span> : <StatusPill state={`${sources.data.length}-sources`} />}
            </div>
            <div className="fieldGuide">
              <article><strong>Source name</strong><span>Everyday: "Router Troubleshooting Notes" · Coder: "IGY6 Build Logs"</span></article>
              <article><strong>Source type</strong><span>Use "manual_upload" for generic pasted text, "conversation_history" for prior conversation/history imports, or "user_observation" for first-party observations, decisions, preferences, corrections, and notes.</span></article>
              <article><strong>Location</strong><span>Everyday: "local notes folder" · Coder: "local repo logs"</span></article>
              <article><strong>Sensitivity</strong><span>Everyday: "private" · Coder: "internal"</span></article>
              <article><strong>Allowed operations</strong><span>Everyday: "read, collect" · Coder: "read, collect, dry_run"</span></article>
            </div>
            <div className="table compactTable">
              {sources.data.map((source) => (
                <div className="row" key={source.id}>
                  <strong>{source.name}</strong>
                  <span><TermHelp term="sourceType" label={source.source_type} /></span>
                  <span>{source.sensitivity}</span>
                  <span>{source.permissions?.length ?? 0} <TermHelp term="sourcePermission" label="permissions" /></span>
                  <StatusPill state={source.enabled ? "enabled" : "disabled"} />
                </div>
              ))}
            </div>
            {sources.data.length === 0 ? <EmptyState label="No sources registered yet." /> : null}
            <ConnectorContractStatusPanel />
            <SourceTrustSensitivityManagement
              sources={sources}
              collectionRuns={collectionRuns}
              documents={documents}
              evidenceItems={evidenceItems}
            />
            <SourceDetailPanel
              sources={sources}
              collectionRuns={collectionRuns}
              artifacts={artifacts}
              documents={documents}
              chunks={chunks}
              evidenceItems={evidenceItems}
              feedback={feedback}
              outcomes={outcomes}
            />
            <details className="advancedPanel">
              <summary>Advanced: source IDs, permission IDs, and raw source data</summary>
              <pre>{JSON.stringify(sources.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

          <section className="panel tabContent" id="uploads-collection" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data / Uploads</p>
                <h2><HelpHeading term="manualUpload">Guided Upload</HelpHeading></h2>
              </div>
              <StatusPill state="approval-aware" />
            </div>
            <GuidedManualTextUpload sources={sources} approvals={approvals} />
            <ol className="workflowSteps">
              <li><strong>Step 1: Select or create source.</strong><span>Use a manual_upload source for notes/logs, a conversation_history source for prior chat/history text, or a user_observation source for owner-provided context.</span></li>
              <li><strong>Step 2: Check approval status.</strong><span>Source permissions show whether approval is required before collection.</span></li>
              <li><strong>Step 3: Request approval if required.</strong><span>Everyday reason: "Allow IGY6 to process this uploaded troubleshooting note." Coder reason: "Approve processing this local build log for evidence extraction."</span></li>
              <li><strong>Step 4: Upload text or a safe file extract.</strong><span>Current manual upload works best with UTF-8 text.</span></li>
              <li><strong>Step 5: Review created records.</strong><span>Check collection run, raw artifact, and work item status.</span></li>
              <li><strong>Step 6: Next action.</strong><span>Check processing, view evidence, or ask Assistant a question.</span></li>
            </ol>
            <ConversationHistoryImport sources={sources} approvals={approvals} />
            <UserObservationIngestion sources={sources} approvals={approvals} />
            <BrowserWebRouterCollectorMvp />
            <MediaImportMvp />
            <LocalProjectPcDiagnosticsHardeningPanel sources={sources} approvals={approvals} />
            <div className="subHeader"><h3><HelpHeading term="collectionRun">Collection Runs</HelpHeading></h3>{collectionRuns.error ? <span className="errorText">{collectionRuns.error}</span> : null}</div>
            <div className="stack">
              {recentRuns.map((run) => (
                <article className="item evidenceItem" key={run.id}>
                  <div><strong>{run.status}</strong><span>{run.dry_run ? "dry run" : "collection"} · requested by {run.requested_by_actor_id}</span></div>
                  <div><span>{formatDate(run.created_at)}</span><StatusPill state={run.dry_run ? "dry-run" : "collected"} /></div>
                </article>
              ))}
            </div>
            {recentRuns.length === 0 ? <EmptyState label="No collection runs recorded yet." /> : null}
            <details className="advancedPanel">
              <summary>Advanced: raw artifact IDs, collection run IDs, and upload JSON</summary>
              <pre>{JSON.stringify({ collection_runs: collectionRuns.data.slice(0, 10), raw_artifacts: artifacts.data.slice(0, 10) }, null, 2)}</pre>
            </details>
          </section>

          <section className="panel tabContent" id="evidence-panel" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2><HelpHeading term="evidenceItem">Evidence And Documents</HelpHeading></h2>
              </div>
              {[documents.error, chunks.error, evidenceItems.error, claims.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some evidence endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Evidence totals">
              <article><span><TermHelp term="collectionRun" label="Collection runs" /></span><strong>{collectionRuns.data.length}</strong></article>
              <article><span><TermHelp term="rawArtifact" label="Raw artifacts" /></span><strong>{artifacts.data.length}</strong></article>
              <article><span><TermHelp term="normalizedDocument" label="Documents" /></span><strong>{documents.data.length}</strong></article>
              <article><span><TermHelp term="chunk" label="Chunks" /></span><strong>{chunks.data.length}</strong></article>
              <article><span><TermHelp term="evidenceItem" label="Evidence" /></span><strong>{evidenceItems.data.length}</strong></article>
              <article><span><TermHelp term="claim" label="Claims" /></span><strong>{claims.data.length}</strong></article>
            </section>
            <GraphLineageExplanationPanel
              sources={sources}
              collectionRuns={collectionRuns}
              artifacts={artifacts}
              documents={documents}
              chunks={chunks}
              evidenceItems={evidenceItems}
              evidenceAnswers={evidenceAnswers}
              reports={reports}
              taskPlans={agentTaskPlans}
              graphSchema={graphSchema}
            />
            <EntityClaimEventFoundationPanel
              evidenceItems={evidenceItems}
              claims={claims}
              sources={sources}
              documents={documents}
              chunks={chunks}
            />
            <EvidenceCorrectionSupersessionWorkflow evidenceItems={evidenceItems} />
            <EvidenceDetailPanel
              evidenceItems={evidenceItems}
              sources={sources}
              documents={documents}
              chunks={chunks}
              evidenceAnswers={evidenceAnswers}
              taskPlans={agentTaskPlans}
              reports={reports}
              feedback={feedback}
              outcomes={outcomes}
            />
            <SourceEvidenceHistory
              sources={sources}
              collectionRuns={collectionRuns}
              artifacts={artifacts}
              documents={documents}
              chunks={chunks}
              evidenceItems={evidenceItems}
            />
            <section className="quad">
              <div>
                <div className="subHeader"><h3><HelpHeading term="collectionRun">Collection Runs</HelpHeading></h3>{collectionRuns.error ? <span className="errorText">{collectionRuns.error}</span> : null}</div>
                <div className="stack">
                  {recentRuns.map((run) => (
                    <article className="item evidenceItem" key={run.id}>
                      <div><strong>{run.status}</strong><span>{run.dry_run ? "dry run" : "collection"} · requested by {run.requested_by_actor_id}</span></div>
                      <div><span>{formatDate(run.created_at)}</span><StatusPill state={run.source_id ? "source-linked" : "no-source"} /></div>
                    </article>
                  ))}
                </div>
                {recentRuns.length === 0 ? <EmptyState label="No collection runs recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="rawArtifact">Raw Artifacts</HelpHeading></h3>{artifacts.error ? <span className="errorText">{artifacts.error}</span> : null}</div>
                <div className="stack">
                  {recentArtifacts.map((artifact) => (
                    <article className="item evidenceItem" key={artifact.id}>
                      <div><strong>{formatBytes(artifact.size_bytes)}</strong><span>{artifact.mime_type ?? "unknown type"}</span></div>
                      <div><span>{formatDate(artifact.created_at)}</span><StatusPill state={artifact.collection_run_id ? "run-linked" : "no-run"} /></div>
                    </article>
                  ))}
                </div>
                {recentArtifacts.length === 0 ? <EmptyState label="No raw artifacts recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="normalizedDocument">Documents</HelpHeading></h3>{documents.error ? <span className="errorText">{documents.error}</span> : null}</div>
                <div className="stack">
                  {recentDocuments.map((document) => (
                    <article className="item evidenceItem" key={document.id}>
                      <div><strong>{document.title ?? "Untitled document"}</strong><span>{document.document_type} · {document.sensitivity}</span></div>
                      <div><span>{formatDate(document.created_at)}</span><StatusPill state={document.source_id ? "source-linked" : "no-source"} /></div>
                    </article>
                  ))}
                </div>
                {recentDocuments.length === 0 ? <EmptyState label="No normalized documents recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="chunk">Chunks</HelpHeading></h3>{chunks.error ? <span className="errorText">{chunks.error}</span> : null}</div>
                <div className="stack">
                  {recentChunks.map((chunk) => (
                    <article className="item evidenceItem" key={chunk.id}>
                      <div><strong>Chunk {chunk.chunk_index}</strong><span>Normalized document chunk</span></div>
                      <div><StatusPill state={chunk.embedding_status} /><span>index {chunk.chunk_index}</span></div>
                    </article>
                  ))}
                </div>
                {recentChunks.length === 0 ? <EmptyState label="No chunks generated yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="evidenceItem">Evidence Items</HelpHeading></h3>{evidenceItems.error ? <span className="errorText">{evidenceItems.error}</span> : null}</div>
                <div className="stack">
                  {recentEvidence.map((item) => (
                    <article className="item evidenceItem" key={item.id}>
                      <div><strong>{item.evidence_type}</strong><span>{excerpt(item.statement)}</span></div>
                      <div>
                        <span>{item.confidence === null ? "unscored" : `${item.confidence}%`}</span>
                        <StatusPill state={item.chunk_id ? "chunk-linked" : "no-chunk"} />
                        <StatusPill state={evidenceReviewState(item)} />
                      </div>
                    </article>
                  ))}
                </div>
                {recentEvidence.length === 0 ? <EmptyState label="No evidence items recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="claim">Claims</HelpHeading></h3>{claims.error ? <span className="errorText">{claims.error}</span> : null}</div>
                <div className="stack">
                  {recentClaims.map((claim) => (
                    <article className="item evidenceItem" key={claim.id}>
                      <div><strong>{claim.claim_type}</strong><span>{excerpt(claim.claim_text)}</span></div>
                      <div><StatusPill state={claim.status} /><span>{claim.confidence === null ? "unscored" : `${claim.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentClaims.length === 0 ? <EmptyState label="No claims recorded yet." /> : null}
              </div>
            </section>
          </section>

          <section className="panel tabContent" id="memory-panel" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2><HelpHeading term="vectorMemory">Search Memory And Findings</HelpHeading></h2>
              </div>
              {[vectorCollection.error, graphSchema.error, patterns.error, hypotheses.error, predictions.error, recommendations.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some memory or analysis endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Memory and analysis totals">
              <article><span><TermHelp term="vectorMemory" label="Vector collection" /></span><strong>{vectorCollection.data.exists ? "Ready" : "Missing"}</strong></article>
              <article><span><TermHelp term="graphMemory" label="Graph constraints" /></span><strong>{graphSchema.data.constraints.length}</strong></article>
              <article><span><TermHelp term="pattern" label="Patterns" /></span><strong>{patterns.data.length}</strong></article>
              <article><span><TermHelp term="recommendation" label="Recommendations" /></span><strong>{recommendations.data.length}</strong></article>
            </section>
            <section className="split">
              <article className="item evidenceItem">
                <div><strong><TermHelp term="qdrant" label={vectorCollection.data.collection_name} /></strong><span>Configured chunk collection</span></div>
                <div><StatusPill state={vectorCollection.data.exists ? "enabled" : "missing"} /></div>
              </article>
              <article className="item evidenceItem">
                <div><strong><TermHelp term="neo4j" label={`${graphSchema.data.constraints.length} constraints`} /></strong><span>Graph schema inspection only</span></div>
                <div><StatusPill state={graphSchema.error ? "error" : "ok"} /></div>
              </article>
            </section>
            <PredictionRecommendationCreator
              evidenceItems={evidenceItems}
              evidenceAnswers={evidenceAnswers}
              reports={reports}
              taskPlans={agentTaskPlans}
            />
            <PredictionRecommendationOutcomeReview
              predictions={predictions}
              recommendations={recommendations}
              evidenceAnswers={evidenceAnswers}
              reports={reports}
              taskPlans={agentTaskPlans}
              feedback={feedback}
              outcomes={outcomes}
              improvements={improvements}
              calibrationSummary={calibrationSummary}
            />
            <BaselinePatternExpansionPanel
              patterns={patterns}
              evidenceItems={evidenceItems}
              sources={sources}
              evidenceAnswers={evidenceAnswers}
              outcomes={outcomes}
              feedback={feedback}
            />
            <section className="quad analysisGrid" id="analysis-panel">
              <div>
                <div className="subHeader"><h3><HelpHeading term="pattern">Patterns</HelpHeading></h3>{patterns.error ? <span className="errorText">{patterns.error}</span> : null}</div>
                <div className="stack">
                  {recentPatterns.map((pattern) => (
                    <article className="item evidenceItem" key={pattern.id}>
                      <div><strong>{pattern.pattern_type}</strong><span>{excerpt(pattern.summary)}</span></div>
                      <div><StatusPill state={pattern.status} /><span>{pattern.confidence === null ? "unscored" : `${pattern.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentPatterns.length === 0 ? <EmptyState label="No patterns recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="hypothesis">Hypotheses</HelpHeading></h3>{hypotheses.error ? <span className="errorText">{hypotheses.error}</span> : null}</div>
                <div className="stack">
                  {recentHypotheses.map((hypothesis) => (
                    <article className="item evidenceItem" key={hypothesis.id}>
                      <div><strong>Hypothesis</strong><span>{excerpt(hypothesis.hypothesis_text)}</span></div>
                      <div><StatusPill state={hypothesis.status} /><span>{hypothesis.confidence === null ? "unscored" : `${hypothesis.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentHypotheses.length === 0 ? <EmptyState label="No hypotheses recorded yet." /> : null}
                <form className="guidedManualForm" data-hypothesis-create-form data-api-base-url="/api">
                  <label><span>New hypothesis</span><textarea name="hypothesis_text" rows={2} placeholder="Describe a testable hypothesis grounded in local evidence." /></label>
                  <label><span>Supporting evidence ids</span><input name="hypothesis_evidence_ids" placeholder="evidence-id-1, evidence-id-2" /></label>
                  <button type="submit">Record hypothesis</button>
                </form>
                <p className="actionHint" data-hypothesis-create-result>Record a hypothesis without reloading the page.</p>
                <ClientScript script={`
(() => {
  const form = document.querySelector("[data-hypothesis-create-form]");
  if (!form || form.getAttribute("data-wired") === "true") return;
  form.setAttribute("data-wired", "true");
  const apiBaseUrl = form.getAttribute("data-api-base-url");
  const result = document.querySelector("[data-hypothesis-create-result]");
  const show = (message) => { if (result) result.textContent = message; };
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = form.querySelector("[name='hypothesis_text']")?.value?.trim() || "";
    const evidenceIds = (form.querySelector("[name='hypothesis_evidence_ids']")?.value || "").split(",").map((item) => item.trim()).filter(Boolean);
    if (!text) return;
    show("Recording hypothesis...");
    try {
      const response = await fetch(apiBaseUrl + "/analysis/hypotheses", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ hypothesis_text: text, supporting_evidence_ids: evidenceIds })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(JSON.stringify(payload));
      const id = payload.id ? " id " + payload.id : "";
      show("Hypothesis recorded" + id + ". Refresh the browser when you want the list redrawn.");
      form.reset();
    } catch (error) {
      show(error instanceof Error ? error.message : "Hypothesis create failed");
    }
  });
})();
`} />
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="prediction">Predictions</HelpHeading></h3>{predictions.error ? <span className="errorText">{predictions.error}</span> : null}</div>
                <div className="stack">
                  {recentPredictions.map((prediction) => (
                    <article className="item evidenceItem" key={prediction.id}>
                      <div><strong>{excerpt(prediction.prediction_text, 80)}</strong><span>{excerpt(prediction.expected_result, 90)}</span></div>
                      <div><StatusPill state={prediction.status} /><span>{prediction.confidence === null ? "unscored" : `${prediction.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentPredictions.length === 0 ? <EmptyState label="No predictions recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="recommendation">Recommendations</HelpHeading></h3>{recommendations.error ? <span className="errorText">{recommendations.error}</span> : null}</div>
                <div className="stack">
                  {recentRecommendations.map((recommendation) => (
                    <article className="item evidenceItem" key={recommendation.id}>
                      <div><strong>{recommendation.risk_level}</strong><span>{excerpt(recommendation.recommendation_text)}</span></div>
                      <div><StatusPill state={recommendation.status} /><span>{recommendation.approval_required ? "approval" : "no approval"}</span></div>
                    </article>
                  ))}
                </div>
                {recentRecommendations.length === 0 ? <EmptyState label="No recommendations recorded yet." /> : null}
              </div>
            </section>
            <ImprovementExperimentReview improvements={improvements} experiments={experiments} />
            <PipelineOperationsPanel workItems={workItems} />
          </section>

          <section className="panel workflowSection tabContent" id="work-processing" data-tab-panel="work">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Work</p>
                <h2><HelpHeading term="workItem">Processing Status</HelpHeading></h2>
              </div>
              {[workItems.error, approvals.error, feedback.error, outcomes.error, reports.error, auditEvents.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some review or operations endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Work and processing totals">
              <article><span>Queued</span><strong>{queuedWorkItems.length}</strong></article>
              <article><span>Running</span><strong>{runningWorkItems.length}</strong></article>
              <article><span>Completed</span><strong>{completedWorkItems.length}</strong></article>
              <article><span>Failed</span><strong>{failedWorkItems.length}</strong></article>
            </section>
            <div className="lifecycleFlow" aria-label="Processing pipeline">
              {["Raw Artifact", "Normalized Document", "Chunks", "Evidence", "Vector Memory", "Graph Memory"].map((step) => (
                <span key={step}>{step}</span>
              ))}
            </div>
            <p className="agentRuntimeReason">Background processing is ready. Supported queued work stays behind system checks and does not run arbitrary user input.</p>
            <section className="quad analysisGrid">
              <div>
                <div className="subHeader"><h3><HelpHeading term="workItem">Work Items</HelpHeading></h3>{workItems.error ? <span className="errorText">{workItems.error}</span> : null}</div>
                <div className="stack">
                  {recentWorkItems.map((workItem) => {
                    const guidance = workItemGuidance(workItem);
                    const relatedIds = workItemRelatedIds(workItem);
                    const dispatchVisibility = workItemDispatchVisibility(workItem);
                    return (
                      <article className="item evidenceItem workStatusItem" key={workItem.id} data-work-status-item>
                        <div>
                          <strong>{workItem.work_type}</strong>
                          <span>Work item: {workItem.id}</span>
                          <span>{guidance.outcome}</span>
                          <dl className="workStatusIds" aria-label={`Dispatch visibility for ${workItem.id}`} data-work-dispatch-visibility>
                            {dispatchVisibility.map((detail) => (
                              <div key={`${workItem.id}-dispatch-${detail.label}`}>
                                <dt>{detail.label}</dt>
                                <dd>{detail.value}</dd>
                              </div>
                            ))}
                          </dl>
                          {relatedIds.length > 0 ? (
                            <dl className="workStatusIds" aria-label={`Related records for ${workItem.id}`}>
                              {relatedIds.map((related) => (
                                <div key={`${workItem.id}-${related.label}`}>
                                  <dt>{related.label}</dt>
                                  <dd>{related.values.slice(0, 3).join(", ")}{related.values.length > 3 ? ` +${related.values.length - 3} more` : ""}</dd>
                                </div>
                              ))}
                            </dl>
                          ) : null}
                        </div>
                        <div>
                          <StatusPill state={workItem.status} />
                          <span>created {formatDate(workItem.created_at)}</span>
                          <span>updated {formatDate(workItem.updated_at ?? workItem.created_at)}</span>
                          <span>{guidance.next}</span>
                        </div>
                      </article>
                    );
                  })}
                </div>
                {recentWorkItems.length === 0 ? <EmptyState label="No work items recorded yet." /> : null}
              </div>
            </section>
            <p className="actionHint">Use Pipeline operations in Results → Memory for search, vector ensure, and one-click dispatch on queued work items.</p>
            <details className="advancedPanel">
              <summary>Advanced: dispatch controls, work item IDs, and raw queue JSON</summary>
              <p>Route: POST /work-items/:work_item_id/dispatch. Pipeline operations panel also exposes dispatch for queued items.</p>
              <pre>{JSON.stringify(workItems.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

          <SettingsHubNav />
          <BypassIntelPanel />
          <SettingsPanel envSettings={envSettings} />
          <UserSecurityPanel />
          <TroubleshootingLogsPanel />

          <section className="panel workflowSection tabContent" id="safety-audit" data-tab-panel="settings">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Settings</p>
                <h2>Safety, Approvals, And Policy</h2>
              </div>
              <StatusPill state="approval-gated" />
            </div>
            <section className="workflowTabs" aria-label="Safety and audit panels">
              <a href="#safety-overview">Overview</a>
              <a href="#approvals">Approvals</a>
              <a href="#audit-log">Audit Log</a>
              <a href="#safety-rules">Safety Rules</a>
              <a href="#safety-advanced">Advanced</a>
            </section>
            <section className="metrics compact" id="safety-overview" aria-label="Safety overview">
              <article><span>Pending approvals</span><strong>{pendingApprovals.length}</strong></article>
              <article><span>Blocked actions</span><strong>{blockedActions.length}</strong></article>
              <article><span>Approval-required actions</span><strong>{approvalRequiredActions.length}</strong></article>
              <article><span>External model policy</span><strong>{policyPosture?.external_model_policy ?? "blocked"}</strong></article>
              <article><span>Hosted AI</span><strong>{policyPosture?.hosted_ai_enabled ? "enabled" : "blocked"}</strong></article>
              <article><span>Prompt injection filter</span><strong>{policyPosture?.prompt_injection_filter ?? "unknown"}</strong></article>
            </section>
            <LifecycleAuditStatusPanel
              sources={sources}
              approvals={approvals}
              artifacts={artifacts}
              documents={documents}
              chunks={chunks}
              evidenceItems={evidenceItems}
              evidenceAnswers={evidenceAnswers}
              claims={claims}
              feedback={feedback}
              outcomes={outcomes}
              workItems={workItems}
              taskPlans={agentTaskPlans}
              reports={reports}
              patterns={patterns}
              hypotheses={hypotheses}
              predictions={predictions}
              recommendations={recommendations}
              improvements={improvements}
              experiments={experiments}
              envSettings={envSettings}
              vectorCollection={vectorCollection}
              graphSchema={graphSchema}
            />
            <section className="quad analysisGrid">
              <div id="approvals">
                <div className="subHeader"><h3><HelpHeading term="approval">Approvals</HelpHeading></h3>{approvals.error ? <span className="errorText">{approvals.error}</span> : null}</div>
                <div className="messageMeta">
                  <StatusPill state={`${pendingApprovals.length}-pending`} />
                  <StatusPill state={`${approvedApprovals.length}-approved`} />
                  <StatusPill state={`${rejectedApprovals.length}-rejected`} />
                </div>
                <div className="stack">
                  {recentApprovals.map((approval) => (
                    <article className="item evidenceItem" key={approval.id}>
                      <div><strong>{approval.request_type}</strong><span>{approval.decision_reason ?? `requested by ${approval.requested_by_actor_id}`}</span></div>
                      <div><StatusPill state={approval.status} /><span>{approval.decided_by_actor_id ?? "undecided"}</span></div>
                    </article>
                  ))}
                </div>
                {recentApprovals.length === 0 ? <EmptyState label="No approvals recorded yet." /> : null}
                <SourceCollectionApprovalReview approvals={approvals} />
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="feedback">Feedback</HelpHeading></h3>{feedback.error ? <span className="errorText">{feedback.error}</span> : null}</div>
                <div className="stack">
                  {recentFeedback.map((event) => (
                    <article className="item evidenceItem" key={event.id}>
                      <div><strong>{event.label}</strong><span>{event.note ?? `${event.target_type} feedback`}</span></div>
                      <div><span>{event.actor_id}</span><span>{formatDate(event.created_at)}</span></div>
                    </article>
                  ))}
                </div>
                {recentFeedback.length === 0 ? <EmptyState label="No feedback recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="outcome">Outcomes</HelpHeading></h3>{outcomes.error ? <span className="errorText">{outcomes.error}</span> : null}</div>
                <div className="stack">
                  {recentOutcomes.map((outcome) => (
                    <article className="item evidenceItem" key={outcome.id}>
                      <div><strong>{outcome.target_type}</strong><span>{outcome.summary ?? "Outcome recorded"}</span></div>
                      <div><StatusPill state={outcome.outcome_status} /><span>{formatDate(outcome.created_at)}</span></div>
                    </article>
                  ))}
                </div>
                {recentOutcomes.length === 0 ? <EmptyState label="No outcomes recorded yet." /> : null}
              </div>

              <div id="safety-rules">
                <div className="subHeader"><h3>Safety Rules</h3></div>
                <div className="stack">
                  <article className="item evidenceItem"><div><strong>Approval-required default</strong><span>System-changing actions require explicit local approval.</span></div><StatusPill state="enabled" /></article>
                  <article className="item evidenceItem"><div><strong>Allowed operation classes</strong><span>Read-only checks, retrieval preview, approved stack controls, approved collection.</span></div><StatusPill state="bounded" /></article>
                  <article className="item evidenceItem"><div><strong>Tool-use policy</strong><span>Raw shell command, user-provided argv, and arbitrary command execution remain unsupported.</span></div><StatusPill state={policyPosture?.arbitrary_command_execution ? "unsafe" : "blocked"} /></article>
                  <article className="item evidenceItem"><div><strong>External model policy</strong><span>Local-first evidence workflows do not send data to external or hosted AI models by default.</span></div><StatusPill state={policyPosture?.hosted_ai_enabled ? "review" : "blocked"} /></article>
                  <article className="item evidenceItem"><div><strong>Blocked request classes</strong><span>{blockedRequestClasses.length > 0 ? blockedRequestClasses.join(", ") : "Unavailable from capabilities endpoint."}</span></div><StatusPill state="blocked" /></article>
                  <article className="item evidenceItem"><div><strong>Runtime capability</strong><span>{agentCapabilities.data.runtime.reason ?? "Capability status is reported by the system runtime."}</span></div><StatusPill state={agentCapabilities.data.runtime.docker_control_available ? "runtime-ready" : "runtime-blocked"} /></article>
                </div>
              </div>

              <div id="audit-log">
                <div className="subHeader"><h3><HelpHeading term="auditEvent">Audit Log</HelpHeading></h3>{auditEvents.error ? <span className="errorText">{auditEvents.error}</span> : null}</div>
                <div className="fieldGuide">
                  <article><strong>Filters</strong><span>Actor, event type, source, work item, approval, and action filters are preserved as advanced audit controls.</span></article>
                </div>
                <div className="stack">
                  {recentAuditEvents.map((event) => (
                    <article className="item evidenceItem" key={event.id}>
                      <div><strong>{event.event_type}</strong><span>{event.resource_type ?? "resource"} event</span></div>
                      <div><StatusPill state={event.decision ?? "recorded"} /><span>{event.actor_id}</span></div>
                    </article>
                  ))}
                </div>
                {recentAuditEvents.length === 0 ? <EmptyState label="No audit events recorded yet." /> : null}
              </div>
            </section>
            <details className="advancedPanel" id="safety-advanced">
              <summary>Advanced: approval IDs, audit JSON, route filters, and raw safety records</summary>
              <pre>{JSON.stringify({ approvals: approvals.data.slice(0, 10), audit_events: auditEvents.data.slice(0, 10), actions: agentCapabilities.data.actions }, null, 2)}</pre>
            </details>
          </section>

          <section className="panel workflowSection tabContent" id="reports" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2>Reports</h2>
              </div>
              {reports.error ? <span className="errorText">{reports.error}</span> : <StatusPill state={`${reports.data.length}-reports`} />}
            </div>
            <div className="fieldGuide">
              <article><strong>Report reason</strong><span>Everyday: "Create a summary of this uploaded bill." · Project: "Summarize the latest verification notes."</span></article>
            </div>
            <OutcomeLearningSummary
              feedback={feedback}
              outcomes={outcomes}
              improvements={improvements}
              evidenceAnswers={evidenceAnswers}
              reports={reports}
              taskPlans={agentTaskPlans}
              workItems={workItems}
              predictions={predictions}
              recommendations={recommendations}
            />
            <EvidenceFeedbackWorkflow evidenceItems={evidenceItems} evidenceAnswers={evidenceAnswers} reports={reports} workItems={workItems} feedback={feedback} outcomes={outcomes} improvements={improvements} />
            <BasicReportWorkflow
              reports={reports}
              evidenceItems={evidenceItems}
              evidenceAnswers={evidenceAnswers}
              evidenceCount={evidenceItems.data.length}
              documentCount={documents.data.length}
              chunkCount={chunks.data.length}
            />
            <section className="quad analysisGrid">
              <div>
                <div className="subHeader"><h3>Reports</h3>{reports.error ? <span className="errorText">{reports.error}</span> : null}</div>
                <div className="stack">
                  {recentReports.map((report) => (
                    <article className="item evidenceItem" key={report.id}>
                      <div><strong>{report.title}</strong><span>{report.report_type} · {report.id}</span></div>
                      <div><StatusPill state={report.status} /><span>{report.artifact_path ? "markdown artifact ready" : "metadata only"}</span></div>
                    </article>
                  ))}
                </div>
                {recentReports.length === 0 ? <EmptyState label="No reports recorded yet." /> : null}
              </div>
            </section>
            <details className="advancedPanel">
              <summary>Advanced: report render route, report IDs, output JSON, and export details</summary>
              <p>Use Advanced Route Console above for report create/render. Routes: POST /reports and POST /reports/:report_id/render.</p>
              <pre>{JSON.stringify(reports.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

        </section>
      </section>

      <aside className="rightContext" aria-label="IGY6 context">
        <section className="contextCard">
          <div className="panelHeader">
            <h2>Context</h2>
            <StatusPill state={health.data.status} />
          </div>
          {health.error ? <span className="errorText">{health.error}</span> : null}
          <div className="contextStats">
            <article><span><TermHelp term="source" label="Sources" /></span><strong>{sources.data.length}</strong></article>
            <article><span><TermHelp term="evidenceItem" label="Evidence" /></span><strong>{evidenceItems.data.length}</strong></article>
            <article><span><TermHelp term="workItem" label="Work queue" /></span><strong>{workItems.data.length}</strong></article>
            <article><span><TermHelp term="approval" label="Approvals" /></span><strong>{approvals.data.length}</strong></article>
          </div>
        </section>

        <section className="contextCard">
          <h2>Service Readiness</h2>
          <div className="checkList">
            {Object.entries(checks).map(([name, check]) => (
              <article className="checkRow" key={name}>
                <span>{name}</span>
                <StatusPill state={check.status} />
              </article>
            ))}
            {Object.keys(checks).length === 0 ? <EmptyState label="No readiness details returned." /> : null}
          </div>
        </section>

        <section className="contextCard">
          <h2><HelpHeading term="source">Recent Sources</HelpHeading></h2>
          <div className="stack">
            {sources.data.slice(0, 4).map((source) => (
              <article className="miniRecord" key={source.id}>
                <strong>{source.name}</strong>
                <span>{source.source_type} · {source.trust_level}</span>
              </article>
            ))}
          </div>
          {sources.data.length === 0 ? <EmptyState label="No sources yet." /> : null}
        </section>

        <section className="contextCard">
          <h2><HelpHeading term="auditEvent">Recent Audit</HelpHeading></h2>
          <div className="stack">
            {recentAuditEvents.map((event) => (
              <article className="miniRecord" key={event.id}>
                <strong>{event.event_type}</strong>
                <span>{event.decision ?? "recorded"} · {event.actor_id}</span>
              </article>
            ))}
          </div>
          {recentAuditEvents.length === 0 ? <EmptyState label="No audit events yet." /> : null}
        </section>

        <section className="contextCard">
          <h2>Method Review</h2>
          <div className="stack">
            <article className="miniRecord">
              <strong><TermHelp term="improvementItem" label="Improvement Item" /></strong>
              <span>Proposed tuning target; not execution.</span>
            </article>
            <article className="miniRecord">
              <strong><TermHelp term="experimentRun" label="Experiment Run" /></strong>
              <span>Experiment metadata; not active MLflow/Optuna execution.</span>
            </article>
          </div>
        </section>

        <section className="contextCard reminderCard">
          <h2>Uncertainty</h2>
          <p>Retrieval only reflects sources that have been registered, collected, normalized, chunked, and embedded. Missing or disabled sources are not evidence.</p>
        </section>
      </aside>
      <ClientScript script={WORKSPACE_HASH_ROUTER_SCRIPT} />
      <ClientScript script={MINIMAL_UI_TOGGLE_SCRIPT} />
    </main>
  );
}
