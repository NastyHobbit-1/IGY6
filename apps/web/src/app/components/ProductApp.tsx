import type {
  HealthResponse,
  SourceRecord,
  CollectionRunRecord,
  RawArtifactRecord,
  NormalizedDocumentRecord,
  ChunkRecord,
  EvidenceItemRecord,
  VectorCollectionStatus,
  RuntimeLogsResponse,
  WorkItemRecord,
  ApprovalRecord,
  AuditEventRecord,
  EnvSettingsResponse,
  AgentCapabilitiesResponse,
  ApiResult,
} from "./types";
import { getJson } from "./api";
import { formatDate, excerpt } from "./helpers";
import { settingValue } from "./llm-helpers";
import { ClientScript } from "@/lib/use-dom-script";
import { UnifiedChatHub } from "./UnifiedChatHub";
import { ChatRetrievalPreview } from "./ChatRetrievalPreview";
import { AddInformationFlow } from "./product/AddInformationFlow";
import { SettingsPanel } from "./SettingsPanel";
import { StackRestartPanel } from "./StackRestartPanel";
import { UserSecurityPanel } from "./UserSecurityPanel";
import { PipelineOperationsPanel } from "./PipelineOperationsPanel";
import { PRODUCT_NAV_SCRIPT } from "./product/nav-script";
import {
  PRODUCT_SECTIONS,
  plainHealthStatus,
  plainServiceName,
  summarizeRuntime,
  visibleHealthChecks,
  dedupeHealthChecksByLabel,
  searchIndexStatus,
  workActivityLabel,
  workStatusPlain,
} from "./product/helpers";
import { buildEvidenceCardModel } from "./product/evidence-display";
import { buildAttentionItems } from "./product/attention";
import { EvidenceCard } from "./product/EvidenceCard";
import { ImageGallery, buildGalleryImageItems } from "./product/ImageGallery";
import { AttentionPanel } from "./product/AttentionPanel";
import { TroubleshootingLogsPanel } from "./product/TroubleshootingLogsPanel";
import { EmptyState } from "./ui/EmptyState";
import { StatusPill } from "./ui/StatusPill";

export async function ProductApp() {
  const [
    health,
    sources,
    collectionRuns,
    artifacts,
    documents,
    chunks,
    evidenceItems,
    vectorCollection,
    workItems,
    approvals,
    auditEvents,
    envSettings,
    agentCapabilities,
    runtimeLogs,
  ] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", []),
    getJson<NormalizedDocumentRecord[]>("/evidence/documents", []),
    getJson<ChunkRecord[]>("/evidence/chunks", []),
    getJson<EvidenceItemRecord[]>("/evidence/items", []),
    getJson<VectorCollectionStatus>("/memory/vector/chunks", { collection_name: "unknown", exists: false }),
    getJson<WorkItemRecord[]>("/work-items", []),
    getJson<ApprovalRecord[]>("/approvals", []),
    getJson<AuditEventRecord[]>("/audit-events", []),
    getJson<EnvSettingsResponse>("/settings/env", {
      file_status: { path: "unknown", backup_dir: "unknown", exists: false, writable: false, unknown_key_count: 0, output_format: "unknown" },
      groups: [],
      settings: [],
      unmanaged: [],
      warnings: [],
    }),
    getJson<AgentCapabilitiesResponse>("/agent/capabilities", {
      actions: [],
      runtime: { repo_root: "unknown", docker_cli_available: false, docker_compose_available: false, docker_socket_available: false, docker_host_configured: false, docker_control_available: false, docker_socket_path: null, reason: "Unavailable" },
      policy: { local_first: true, hosted_ai_enabled: false, external_model_policy: "blocked_by_default", arbitrary_command_execution: false, prompt_injection_filter: "unavailable", approval_required_for_system_changing: true, blocked_request_classes: [] },
    }),
    getJson<RuntimeLogsResponse>("/ops/runtime-logs?limit=120", {
      limit: 120,
      startup_log: { path: "ops/startup.log", exists: false, lines: [] },
      error_log: { path: "ops/error.log", exists: false, lines: [] },
    }),
  ]);

  const serviceChecks = dedupeHealthChecksByLabel(visibleHealthChecks(health.data.checks ?? {}));
  const searchIndex = searchIndexStatus(vectorCollection.data);
  const pendingApprovals = approvals.data.filter((a) => a.status === "pending");
  const activeWork = workItems.data.filter((w) => ["queued", "running", "pending_intent_verification"].includes(w.status));
  const failedWork = workItems.data.filter((w) => w.status === "failed");
  const recentEvidence = evidenceItems.data.slice(0, 24);
  const recentWork = workItems.data.slice(0, 10);
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentAudit = auditEvents.data.slice(0, 5);
  const sourceById = new Map(sources.data.map((s) => [s.id, s]));
  const artifactById = new Map(artifacts.data.map((a) => [a.id, a]));
  const artifactByDocumentId = new Map<string, RawArtifactRecord>();
  for (const doc of documents.data) {
    if (doc.raw_artifact_id) {
      const artifact = artifactById.get(doc.raw_artifact_id);
      if (artifact) artifactByDocumentId.set(doc.id, artifact);
    }
  }
  const llmEnabled = (settingValue(envSettings.data, "LLM_PROVIDER", "none") || "none") === "ollama";
  const llmModel = settingValue(envSettings.data, "OLLAMA_MODEL", "") || "not selected";
  const runtimeSummary = summarizeRuntime(health.data);
  const searchReachable = Boolean(vectorCollection.data.detail?.tcp_reachable);
  const attentionItems = buildAttentionItems({
    pendingApprovals,
    failedWork,
    searchIndexState: searchIndex.state,
    searchHint: searchReachable
      ? "Search memory is still being prepared automatically."
      : "Search memory is not reachable. Check that supporting services are running.",
  });
  const galleryItems = buildGalleryImageItems(artifacts.data, sourceById);

  const workItemsResult: ApiResult<WorkItemRecord[]> = workItems;
  const collectionRunsResult: ApiResult<CollectionRunRecord[]> = collectionRuns;
  const vectorResult: ApiResult<VectorCollectionStatus> = vectorCollection;
  const artifactsResult: ApiResult<RawArtifactRecord[]> = artifacts;
  const runtimeLogsResult: RuntimeLogsResponse = {
    ...runtimeLogs.data,
    error: runtimeLogs.error,
  };

  return (
    <div
      className="igy6-app"
      data-product-app
      data-ui-mode="simple"
      data-search-state={searchIndex.state}
      data-search-reachable={searchReachable ? "true" : "false"}
    >
      <input className="tabInput" id="tab-results" name="main-dashboard-tab" type="radio" defaultChecked hidden />
      <input className="tabInput" id="tab-add-data" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-work" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-settings" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-advanced" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-home" name="main-dashboard-tab" type="radio" hidden />

      <aside className="productSidebar" aria-label="IGY6 navigation">
        <div className="productBrand">
          <div className="productBrandMark">IG</div>
          <div>
            <strong>IGY6</strong>
            <span>Local evidence workspace</span>
          </div>
        </div>

        <div className="productModeToggle" role="group" aria-label="Display mode">
          <button type="button" className="productModeButton productModeActive" data-ui-mode="simple" aria-pressed="true">
            Simple
          </button>
          <button type="button" className="productModeButton" data-ui-mode="full" aria-pressed="false">
            Full
          </button>
        </div>

        <nav className="productNav" aria-label="Main sections">
          {PRODUCT_SECTIONS.map((section) => (
            <button
              key={section.id}
              type="button"
              className="productNavButton"
              data-product-nav={section.id}
              aria-current={section.id === "command" ? "page" : "false"}
            >
              <span>{section.label}</span>
              <small>{section.hint}</small>
            </button>
          ))}
        </nav>
        <div className="productSidebarStatus">
          <StatusPill state={health.data.status} />
          <span className="uiBreakLong">{runtimeSummary}</span>
          <span className="productHint uiBreakLong" data-product-status-banner aria-live="polite" />
          <AttentionPanel items={attentionItems} />
        </div>
      </aside>

      <main className="productMain" data-product-main>
        <section className="productSection productSectionActive" data-product-section="command" data-tab-panel="results" id="assistant">
          <header className="productHero">
            <div>
              <p className="productEyebrow">Command Center</p>
              <h1>Ask IGY6</h1>
              <p className="productLead">
                Ask questions over your local evidence, check status, open sections, or add information. IGY6 only claims what is actually wired.
              </p>
            </div>
            <div className="productHeroStatus">
              <article><span>Sources</span><strong>{sources.data.length}</strong></article>
              <article><span>Evidence</span><strong>{evidenceItems.data.length}</strong></article>
              <article><span>Active work</span><strong>{activeWork.length}</strong></article>
              <article><span>Search</span><strong data-search-status-label>{searchIndex.label}</strong></article>
            </div>
          </header>

          <div className="productQuickActions" aria-label="Quick actions">
            <button type="button" className="productChip" data-product-go="add" data-product-scroll="uploads-collection">Add information</button>
            <button type="button" className="productChip" data-product-go="evidence" data-product-scroll="evidence-panel">Browse evidence</button>
            <button type="button" className="productChip" data-product-go="gallery" data-product-scroll="image-gallery">Browse images</button>
            <button type="button" className="productChip" data-product-go="activity" data-product-scroll="work-processing">See activity</button>
            <button type="button" className="productChip" data-product-go="system">System health</button>
            {pendingApprovals.length > 0 ? (
              <button type="button" className="productChip productChipWarn" data-product-go="activity" data-product-scroll="approvals-panel">
                {pendingApprovals.length} approval(s)
              </button>
            ) : null}
          </div>

          <div className="productChatStage chatFirstShell">
            <UnifiedChatHub
              sourceCount={sources.data.length}
              evidenceCount={evidenceItems.data.length}
              chunkCount={chunks.data.length}
              workItemCount={workItems.data.length}
              pendingApprovals={pendingApprovals.length}
              vectorReady={vectorCollection.data.exists}
              llmEnabled={llmEnabled}
              llmModel={llmModel}
            />
          </div>

          <details className="productAdvanced">
            <summary>What IGY6 found (search details)</summary>
            <ChatRetrievalPreview />
          </details>

          {recentAudit.length > 0 ? (
            <section className="productCardGrid" aria-label="Recent activity">
              <h3>Recent activity</h3>
              {recentAudit.map((event) => (
                <article className="productCard uiOverflowSafe" key={event.id}>
                  <strong className="uiClampTitle">{event.event_type.replace(/_/g, " ")}</strong>
                  <span className="uiBreakLong">{event.decision ?? "recorded"} · {formatDate(event.created_at)}</span>
                </article>
              ))}
            </section>
          ) : null}
        </section>

        <section className="productSection" data-product-section="add" data-tab-panel="add-data" hidden id="data-knowledge">
          <AddInformationFlow sources={sources} approvals={approvals} />
        </section>

        <section className="productSection" data-product-section="evidence" data-tab-panel="results" hidden id="evidence-panel">
          <header className="productPanelHeader">
            <div>
              <p className="productEyebrow">Evidence</p>
              <h2>What IGY6 saved</h2>
            </div>
            <StatusPill state={evidenceItems.data.length > 0 ? "ready" : "empty"} />
          </header>
          <p className="productLead">
            Readable evidence cards from your sources. Each card shows the content type first, with technical IDs under Details in Full mode.
          </p>
          {recentEvidence.length > 0 ? (
            <div className="productCardGrid">
              {recentEvidence.map((item) => {
                const source = item.source_id ? sourceById.get(item.source_id) : undefined;
                const artifact = item.document_id ? artifactByDocumentId.get(item.document_id) : undefined;
                const model = buildEvidenceCardModel(item, source, artifact);
                return <EvidenceCard key={item.id} model={model} />;
              })}
            </div>
          ) : (
            <EmptyState label="No evidence yet. Add information, then check Activity while processing finishes." />
          )}
          {documents.data.length > 0 ? (
            <details className="productAdvanced" id="memory-panel">
              <summary>Documents ({documents.data.length})</summary>
              <ul className="productPlainList">
                {documents.data.slice(0, 8).map((doc) => (
                  <li key={doc.id} className="uiBreakLong">{doc.title ?? "Untitled document"}</li>
                ))}
              </ul>
            </details>
          ) : null}
        </section>

        <section className="productSection" data-product-section="gallery" data-tab-panel="results" hidden id="image-gallery">
          <header className="productPanelHeader">
            <div>
              <p className="productEyebrow">Images</p>
              <h2>Image gallery</h2>
            </div>
            <StatusPill state={galleryItems.length > 0 ? "ready" : "empty"} />
          </header>
          <p className="productLead">
            Browse saved images in one place. Search, sort, and filter without digging through generic evidence lists.
          </p>
          <ImageGallery items={galleryItems} error={artifacts.error} />
        </section>

        <section className="productSection" data-product-section="activity" data-tab-panel="work" hidden id="work-processing">
          <header className="productPanelHeader">
            <div>
              <p className="productEyebrow">Activity</p>
              <h2>What IGY6 is doing</h2>
            </div>
          </header>
          <PipelineOperationsPanel
            workItems={workItemsResult}
            collectionRuns={collectionRunsResult}
            vectorCollection={vectorResult}
          />
          <div className="productCardGrid" id="activity-work">
            {recentWork.length > 0 ? recentWork.map((item) => {
              const status = workStatusPlain(item);
              return (
                <article className="productActivityCard uiOverflowSafe" key={item.id}>
                  <div className="productEvidenceHead">
                    <h3 className="uiClampTitle">{workActivityLabel(item)}</h3>
                    <StatusPill state={status.state} />
                  </div>
                  <p className="uiClampSummary">{status.next}</p>
                  <span className="productMuted">Updated {formatDate(item.updated_at ?? item.created_at)}</span>
                  {item.status === "failed" ? (
                    <button type="button" className="productPrimaryButton" data-retry-failed-work data-work-item-id={item.id}>
                      Retry
                    </button>
                  ) : null}
                  <details className="productAdvanced">
                    <summary>Technical details</summary>
                    <pre className="uiBreakLong">{JSON.stringify({ id: item.id, work_type: item.work_type, status: item.status, error_message: item.error_message }, null, 2)}</pre>
                  </details>
                </article>
              );
            }) : <EmptyState label="No work recorded yet." />}
          </div>
          <section className="productSubsection" id="approvals-panel">
            <h3>Approvals</h3>
            {pendingApprovals.length > 0 ? (
              <div className="productCardGrid">
                {pendingApprovals.map((approval) => (
                  <article className="productCard uiOverflowSafe" key={approval.id}>
                    <strong className="uiClampTitle">{approval.request_type.replace(/_/g, " ")}</strong>
                    <span>Needs your decision</span>
                    <button type="button" className="productGhostButton" data-product-go="settings" data-product-scroll="safety-audit">Review in Settings</button>
                  </article>
                ))}
              </div>
            ) : <p className="productHint">No approvals waiting.</p>}
          </section>
          {recentRuns.length > 0 ? (
            <section className="productSubsection">
              <h3>Recent collections</h3>
              <div className="productCardGrid">
                {recentRuns.map((run) => {
                  const summary = run.summary_json ?? {};
                  const aiSummary = typeof summary.ai_review_summary === "string" ? summary.ai_review_summary : null;
                  return (
                    <article className="productCard uiOverflowSafe" key={run.id}>
                      <strong>{run.status}</strong>
                      <span>{formatDate(run.created_at)}</span>
                      {aiSummary ? <p className="uiClampSummary">{excerpt(aiSummary, 160)}</p> : null}
                    </article>
                  );
                })}
              </div>
            </section>
          ) : null}
        </section>

        <section className="productSection" data-product-section="system" data-tab-panel="advanced" hidden id="advanced-diagnostics">
          <header className="productPanelHeader">
            <div>
              <p className="productEyebrow">System</p>
              <h2>Health and services</h2>
            </div>
            <StatusPill state={health.data.status} />
          </header>
          <p className="productLead uiBreakLong">{runtimeSummary}</p>
          <div className="productServiceGrid" id="search-status">
            <article className="productServiceCard uiOverflowSafe">
              <span>Web UI</span>
              <strong>Running</strong>
              <StatusPill state="ok" />
            </article>
            {serviceChecks.map(([name, check]) => {
              const plain = plainHealthStatus(check.status);
              return (
                <article className="productServiceCard uiOverflowSafe" key={name}>
                  <span>{plainServiceName(name)}</span>
                  <strong>{plain.label}</strong>
                  <StatusPill state={plain.state} />
                  {check.detail ? <small className="uiBreakLong">{check.detail}</small> : null}
                </article>
              );
            })}
            <article className="productServiceCard uiOverflowSafe">
              <span>Search</span>
              <strong data-search-status-label>{searchIndex.label}</strong>
              <StatusPill state={searchIndex.state} />
              {searchIndex.state !== "ok" ? (
                <button type="button" className="productGhostButton" data-product-vector-ensure>
                  {searchReachable ? "Repair search" : "Check services first"}
                </button>
              ) : null}
            </article>
            <article className="productServiceCard uiOverflowSafe">
              <span>Local model</span>
              <strong>{llmEnabled ? `Running (${llmModel})` : "Not configured"}</strong>
              <StatusPill state={llmEnabled ? "ok" : "unknown"} />
            </article>
          </div>
          <TroubleshootingLogsPanel logs={runtimeLogsResult} />
          <details className="productAdvanced">
            <summary>Advanced diagnostics</summary>
            <pre className="uiBreakLong">{JSON.stringify({ health: health.data, vector: vectorCollection.data, capabilities: agentCapabilities.data.runtime }, null, 2)}</pre>
          </details>
        </section>

        <section className="productSection" data-product-section="settings" data-tab-panel="settings" hidden id="settings">
          <header className="productPanelHeader">
            <div>
              <p className="productEyebrow">Settings</p>
              <h2>Configuration and privacy</h2>
            </div>
          </header>
          <div className="productSettingsStack" id="safety-audit">
            <StackRestartPanel />
            <SettingsPanel envSettings={envSettings} />
            <UserSecurityPanel />
          </div>
          <details className="productAdvanced" id="user-security">
            <summary>Advanced: raw environment and audit JSON</summary>
            <pre className="uiBreakLong">{JSON.stringify({ settings: envSettings.data.file_status, approvals: approvals.data.slice(0, 5), artifacts: artifactsResult.data.length }, null, 2)}</pre>
          </details>
        </section>
      </main>

      <ClientScript script={PRODUCT_NAV_SCRIPT} />
    </div>
  );
}