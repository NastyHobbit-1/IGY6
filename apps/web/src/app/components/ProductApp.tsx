import type {
  HealthResponse,
  SourceRecord,
  CollectionRunRecord,
  RawArtifactRecord,
  NormalizedDocumentRecord,
  ChunkRecord,
  EvidenceItemRecord,
  VectorCollectionStatus,
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
  workActivityLabel,
  workStatusPlain,
  evidenceCardTitle,
  evidenceWhyItMatters,
} from "./product/helpers";
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
  ]);

  const checks = health.data.checks ?? {};
  const pendingApprovals = approvals.data.filter((a) => a.status === "pending");
  const activeWork = workItems.data.filter((w) => ["queued", "running", "pending_intent_verification"].includes(w.status));
  const failedWork = workItems.data.filter((w) => w.status === "failed");
  const recentEvidence = evidenceItems.data.slice(0, 12);
  const recentWork = workItems.data.slice(0, 10);
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentAudit = auditEvents.data.slice(0, 5);
  const sourceById = new Map(sources.data.map((s) => [s.id, s]));
  const llmEnabled = (settingValue(envSettings.data, "LLM_PROVIDER", "none") || "none") === "ollama";
  const llmModel = settingValue(envSettings.data, "OLLAMA_MODEL", "") || "not selected";
  const runtimeSummary = summarizeRuntime(health.data);
  const attentionCount = pendingApprovals.length + failedWork.length + (vectorCollection.data.exists ? 0 : 1);

  const workItemsResult: ApiResult<WorkItemRecord[]> = workItems;
  const collectionRunsResult: ApiResult<CollectionRunRecord[]> = collectionRuns;
  const vectorResult: ApiResult<VectorCollectionStatus> = vectorCollection;

  return (
    <div className="igy6-app" data-product-app>
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
          <span>{runtimeSummary}</span>
          {attentionCount > 0 ? <span className="productAttention">{attentionCount} item(s) need attention</span> : null}
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
              <article><span>Search index</span><strong>{vectorCollection.data.exists ? "Ready" : "Needs setup"}</strong></article>
            </div>
          </header>

          <div className="productQuickActions" aria-label="Quick actions">
            <button type="button" className="productChip" data-product-go="add" data-product-scroll="uploads-collection">Add information</button>
            <button type="button" className="productChip" data-product-go="evidence" data-product-scroll="evidence-panel">Browse evidence</button>
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
                <article className="productCard" key={event.id}>
                  <strong>{event.event_type.replace(/_/g, " ")}</strong>
                  <span>{event.decision ?? "recorded"} · {formatDate(event.created_at)}</span>
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
            Readable evidence cards from your sources. Open details for excerpts; technical IDs stay in Advanced.
          </p>
          {recentEvidence.length > 0 ? (
            <div className="productCardGrid">
              {recentEvidence.map((item) => {
                const source = item.source_id ? sourceById.get(item.source_id) : undefined;
                return (
                  <article className="productEvidenceCard" key={item.id}>
                    <div className="productEvidenceHead">
                      <h3>{evidenceCardTitle(item, source)}</h3>
                      <StatusPill state={item.confidence && item.confidence > 0.7 ? "high" : "recorded"} />
                    </div>
                    <p>{excerpt(item.statement, 180)}</p>
                    <dl className="productMeta">
                      <div><dt>Source</dt><dd>{source?.name ?? "Local source"}</dd></div>
                      <div><dt>Why it matters</dt><dd>{evidenceWhyItMatters(item)}</dd></div>
                      <div><dt>Status</dt><dd>Saved locally</dd></div>
                    </dl>
                    <details className="productAdvanced">
                      <summary>Technical details</summary>
                      <pre>{JSON.stringify({ id: item.id, document_id: item.document_id, chunk_id: item.chunk_id, type: item.evidence_type }, null, 2)}</pre>
                    </details>
                  </article>
                );
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
                  <li key={doc.id}>{doc.title ?? "Untitled document"}</li>
                ))}
              </ul>
            </details>
          ) : null}
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
          <div className="productCardGrid" id="data-search">
            {recentWork.length > 0 ? recentWork.map((item) => {
              const status = workStatusPlain(item);
              return (
                <article className="productActivityCard" key={item.id}>
                  <div className="productEvidenceHead">
                    <h3>{workActivityLabel(item)}</h3>
                    <StatusPill state={status.state} />
                  </div>
                  <p>{status.next}</p>
                  <span className="productMuted">Updated {formatDate(item.updated_at ?? item.created_at)}</span>
                  <details className="productAdvanced">
                    <summary>Technical details</summary>
                    <pre>{JSON.stringify({ id: item.id, work_type: item.work_type, status: item.status, error_message: item.error_message }, null, 2)}</pre>
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
                  <article className="productCard" key={approval.id}>
                    <strong>{approval.request_type.replace(/_/g, " ")}</strong>
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
                    <article className="productCard" key={run.id}>
                      <strong>{run.status}</strong>
                      <span>{formatDate(run.created_at)}</span>
                      {aiSummary ? <p>{excerpt(aiSummary, 160)}</p> : null}
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
          <p className="productLead">{runtimeSummary}</p>
          <div className="productServiceGrid">
            <article className="productServiceCard">
              <span>Web UI</span>
              <strong>Running</strong>
              <StatusPill state="ok" />
            </article>
            {Object.entries(checks).map(([name, check]) => {
              const plain = plainHealthStatus(check.status);
              return (
                <article className="productServiceCard" key={name}>
                  <span>{plainServiceName(name)}</span>
                  <strong>{plain.label}</strong>
                  <StatusPill state={plain.state} />
                  {check.detail ? <small>{check.detail}</small> : null}
                </article>
              );
            })}
            <article className="productServiceCard">
              <span>Search index</span>
              <strong>{vectorCollection.data.exists ? "Running" : "Needs attention"}</strong>
              <StatusPill state={vectorCollection.data.exists ? "ok" : "warning"} />
              {!vectorCollection.data.exists ? (
                <button type="button" className="productGhostButton" data-product-go="activity" data-product-scroll="data-search">Fix in Activity</button>
              ) : null}
            </article>
            <article className="productServiceCard">
              <span>Local model</span>
              <strong>{llmEnabled ? `Running (${llmModel})` : "Not configured"}</strong>
              <StatusPill state={llmEnabled ? "ok" : "unknown"} />
            </article>
          </div>
          <details className="productAdvanced">
            <summary>Advanced diagnostics</summary>
            <pre>{JSON.stringify({ health: health.data, vector: vectorCollection.data, capabilities: agentCapabilities.data.runtime }, null, 2)}</pre>
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
            <pre>{JSON.stringify({ settings: envSettings.data.file_status, approvals: approvals.data.slice(0, 5), artifacts: artifacts.data.length }, null, 2)}</pre>
          </details>
        </section>
      </main>

      <ClientScript script={PRODUCT_NAV_SCRIPT} />
    </div>
  );
}