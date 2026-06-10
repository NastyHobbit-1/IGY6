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
import { settingValue } from "./llm-helpers";
import { AddInformationFlow } from "./product/AddInformationFlow";
import { buildAttentionItems } from "./product/attention";
import { buildGalleryImageItems } from "./product/ImageGallery";
import { searchIndexStatus, summarizeRuntime } from "./product/helpers";
import { AppShell } from "./shell/AppShell";
import { CommandCenterSection } from "./product/sections/CommandCenterSection";
import { EvidenceSection } from "./product/sections/EvidenceSection";
import { GallerySection } from "./product/sections/GallerySection";
import { ActivitySection } from "./product/sections/ActivitySection";
import { SystemSection } from "./product/sections/SystemSection";
import { SettingsSection } from "./product/sections/SettingsSection";

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

  const searchIndex = searchIndexStatus(vectorCollection.data);
  const pendingApprovals = approvals.data.filter((a) => a.status === "pending");
  const activeWork = workItems.data.filter((w) => ["queued", "running", "pending_intent_verification"].includes(w.status));
  const failedWork = workItems.data.filter((w) => w.status === "failed");
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
  const searchReachable = Boolean(vectorCollection.data.detail?.tcp_reachable);
  const attentionItems = buildAttentionItems({
    pendingApprovals,
    failedWork,
    searchIndexState: searchIndex.state,
    searchHint: searchReachable
      ? "Search memory is still being prepared automatically."
      : "Search memory is not reachable. Check System for service status.",
    searchEnsureError: vectorCollection.error ?? null,
  });
  const galleryItems = buildGalleryImageItems(artifacts.data, sourceById);
  const runtimeLogsResult: RuntimeLogsResponse = {
    ...runtimeLogs.data,
    error: runtimeLogs.error,
  };

  return (
    <AppShell
      healthStatus={health.data.status}
      runtimeSummary={summarizeRuntime(health.data)}
      attentionItems={attentionItems}
      searchState={searchIndex.state}
      searchReachable={searchReachable}
    >
      <CommandCenterSection
        sourcesCount={sources.data.length}
        evidenceCount={evidenceItems.data.length}
        activeWorkCount={activeWork.length}
        searchLabel={searchIndex.label}
        pendingApprovals={pendingApprovals.length}
        chunkCount={chunks.data.length}
        workItemCount={workItems.data.length}
        vectorReady={vectorCollection.data.exists}
        llmEnabled={llmEnabled}
        llmModel={llmModel}
        recentAudit={recentAudit}
      />

      <section className="productSection" data-product-section="add" data-tab-panel="add-data" hidden id="data-knowledge">
        <AddInformationFlow sources={sources} approvals={approvals} />
      </section>

      <EvidenceSection
        evidenceItems={evidenceItems.data}
        documents={documents.data}
        sourceById={sourceById}
        artifactByDocumentId={artifactByDocumentId}
      />

      <GallerySection items={galleryItems} error={artifacts.error} />

      <ActivitySection
        workItems={workItems}
        collectionRuns={collectionRuns}
        vectorCollection={vectorCollection}
        recentWork={recentWork}
        pendingApprovals={pendingApprovals}
        recentRuns={recentRuns}
      />

      <SystemSection
        health={health.data}
        vectorCollection={vectorCollection.data}
        llmEnabled={llmEnabled}
        llmModel={llmModel}
        runtimeLogs={runtimeLogsResult}
        agentCapabilities={agentCapabilities.data}
      />

      <SettingsSection
        envSettings={envSettings}
        approvals={approvals.data}
        artifactsCount={artifacts.data.length}
      />
    </AppShell>
  );
}