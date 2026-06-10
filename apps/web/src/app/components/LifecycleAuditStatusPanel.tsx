import type { SourceRecord, RawArtifactRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, EvidenceAnswerRecord, ClaimRecord, VectorCollectionStatus, GraphSchemaStatus, PatternRecord, HypothesisRecord, PredictionRecord, RecommendationRecord, WorkItemRecord, AgentTaskPlanRecord, ApprovalRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ExperimentRecord, ReportRecord, EnvSettingsResponse, ApiResult } from "./types";
import { StatusPill } from "./ui/StatusPill";

export function LifecycleAuditStatusPanel({
  sources,
  approvals,
  artifacts,
  documents,
  chunks,
  evidenceItems,
  evidenceAnswers,
  claims,
  feedback,
  outcomes,
  workItems,
  taskPlans,
  reports,
  patterns,
  hypotheses,
  predictions,
  recommendations,
  improvements,
  experiments,
  envSettings,
  vectorCollection,
  graphSchema
}: {
  sources: ApiResult<SourceRecord[]>;
  approvals: ApiResult<ApprovalRecord[]>;
  artifacts: ApiResult<RawArtifactRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  claims: ApiResult<ClaimRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  patterns: ApiResult<PatternRecord[]>;
  hypotheses: ApiResult<HypothesisRecord[]>;
  predictions: ApiResult<PredictionRecord[]>;
  recommendations: ApiResult<RecommendationRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
  experiments: ApiResult<ExperimentRecord[]>;
  envSettings: ApiResult<EnvSettingsResponse>;
  vectorCollection: ApiResult<VectorCollectionStatus>;
  graphSchema: ApiResult<GraphSchemaStatus>;
}) {
  const envHas = (key: string) => envSettings.data.settings.find((setting) => setting.key === key)?.has_value ?? false;
  const lifecycleLoadErrors = [
    { label: "sources", error: sources.error },
    { label: "artifacts", error: artifacts.error },
    { label: "documents", error: documents.error },
    { label: "chunks", error: chunks.error },
    { label: "evidence items", error: evidenceItems.error },
    { label: "evidence answers", error: evidenceAnswers.error },
    { label: "reports", error: reports.error },
    { label: "settings env", error: envSettings.error },
    { label: "vector store", error: vectorCollection.error },
    { label: "graph schema", error: graphSchema.error }
  ].filter((item) => item.error);
  const qdrantReachable = vectorCollection.data.detail?.tcp_reachable === true;
  const qdrantState = vectorCollection.data.exists
    ? "collection-visible"
    : qdrantReachable
      ? "reachable-unverified"
      : vectorCollection.error
        ? "unreachable"
        : "not-visible";
  const graphProbe = graphSchema.data.constraints[0];
  const graphReachable = graphProbe?.tcp_reachable === true;
  const graphState = graphReachable
    ? "reachable-read-only"
    : graphSchema.data.constraints.length > 0
      ? "schema-visible"
      : "schema-not-visible";
  const dataClasses = [
    { label: "sources", count: sources.data.length, backup: "metadata export MVP", export: "metadata", restore: "dry-run validation only", deletion: "future explicit DIFF" },
    { label: "permissions/approvals", count: approvals.data.length, backup: "metadata export MVP", export: "audit metadata", restore: "dry-run validation only", deletion: "restricted" },
    { label: "raw artifacts", count: artifacts.data.length, backup: "not in MVP", export: "metadata only", restore: "future", deletion: "dangerous" },
    { label: "documents/chunks", count: documents.data.length + chunks.data.length, backup: "metadata export MVP", export: "content excluded", restore: "dry-run validation only", deletion: "dangerous" },
    { label: "evidence/claims/answers", count: evidenceItems.data.length + claims.data.length + evidenceAnswers.data.length, backup: "metadata export MVP", export: "content excluded", restore: "dry-run validation only", deletion: "dangerous" },
    { label: "feedback/outcomes", count: feedback.data.length + outcomes.data.length, backup: "metadata export MVP", export: "review metadata", restore: "dry-run validation only", deletion: "restricted" },
    { label: "work/task records", count: workItems.data.length + taskPlans.data.length, backup: "metadata export MVP", export: "metadata", restore: "dry-run validation only", deletion: "restricted" },
    { label: "reports", count: reports.data.length, backup: "metadata export MVP", export: "metadata; raw markdown excluded", restore: "dry-run validation only", deletion: "future explicit DIFF" },
    { label: "patterns/predictions/recommendations", count: patterns.data.length + hypotheses.data.length + predictions.data.length + recommendations.data.length, backup: "metadata export MVP", export: "analysis metadata", restore: "dry-run validation only", deletion: "restricted" },
    { label: "improvements/experiments", count: improvements.data.length + experiments.data.length, backup: "metadata export MVP", export: "metadata", restore: "dry-run validation only", deletion: "restricted" }
  ];
  const lifecycleReadiness = [
    { label: "IGY6_DATA_ROOT", state: envHas("IGY6_DATA_ROOT") ? "configured" : "not reported", detail: "Root for runtime data. Values are not printed here." },
    { label: "ARTIFACT_STORE_PATH", state: envHas("ARTIFACT_STORE_PATH") ? "configured" : "not reported", detail: "Raw/generated artifact storage; raw inclusion needs owner selection." },
    { label: "EXPORT_STORE_PATH", state: envHas("EXPORT_STORE_PATH") ? "configured" : "not reported", detail: "Reserved local export path; current report export uses markdown artifacts." },
    { label: "ENV_BACKUP_DIR", state: envHas("ENV_BACKUP_DIR") ? "configured" : "not reported", detail: ".env backup location for settings writes; .env is excluded from product exports." },
    {
      label: "Qdrant",
      state: qdrantState,
      detail: vectorCollection.data.exists
        ? `Collection ${vectorCollection.data.collection_name} is visible.`
        : qdrantReachable
          ? `${vectorCollection.data.collection_name} is reachable; collection is created on first embedding run.`
          : "Vector store is not reachable from the API container."
    },
    {
      label: "Neo4j",
      state: graphState,
      detail: graphReachable
        ? "Bolt endpoint is reachable; full constraint inventory is read-only in this build."
        : "Graph store reachability could not be confirmed."
    }
  ];

  return (
    <section className="panel lifecycleAudit" data-lifecycle-audit-status>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Data lifecycle</p>
          <h2>Backup, Restore, Export, And Delete Audit</h2>
        </div>
        <StatusPill state="non-destructive-audit" />
      </div>
      <div className="guidedManualNotice">
        <strong>Audit only.</strong>
        <span>This panel maps data classes and lifecycle boundaries. It does not delete, restore, create full backup archives, dump runtime data, print secrets, or modify `.env`.</span>
      </div>
      {lifecycleLoadErrors.length > 0 ? (
        <div className="settingsWarnings">
          <strong>Some lifecycle inputs could not be loaded; audit counts may be incomplete.</strong>
          {lifecycleLoadErrors.map((item) => (
            <span key={item.label}>{item.label}: {item.error}</span>
          ))}
        </div>
      ) : null}
      <section className="metrics compact" aria-label="Lifecycle store status">
        <article><span>Data root</span><strong>{envHas("IGY6_DATA_ROOT") ? "Set" : "Unknown"}</strong></article>
        <article><span>Artifacts</span><strong>{artifacts.data.length}</strong></article>
        <article><span>Reports</span><strong>{reports.data.filter((report) => report.artifact_path).length}</strong></article>
        <article><span>Vector store</span><strong>{vectorCollection.data.exists ? "Visible" : qdrantReachable ? "Reachable" : "Unknown"}</strong></article>
        <article><span>Graph schema</span><strong>{graphSchema.data.constraints.length}</strong></article>
      </section>
      <section className="quad">
        <div>
          <div className="subHeader"><h3>Data Classes</h3></div>
          <div className="stack">
            {dataClasses.map((item) => (
              <article className="item evidenceItem" key={item.label}>
                <div>
                  <strong>{item.label}</strong>
                  <span>Backup/export: {item.backup} · Export detail: {item.export}</span>
                  <span>Restore: {item.restore} · Delete: {item.deletion}</span>
                </div>
                <div><StatusPill state={`${item.count}-records`} /></div>
              </article>
            ))}
          </div>
        </div>
        <div>
          <div className="subHeader"><h3>Store Readiness</h3></div>
          <div className="stack">
            {lifecycleReadiness.map((item) => (
              <article className="item evidenceItem" key={item.label}>
                <div><strong>{item.label}</strong><span>{item.detail}</span></div>
                <div><StatusPill state={item.state} /></div>
              </article>
            ))}
          </div>
        </div>
        <div>
          <div className="subHeader"><h3>Exclusions</h3></div>
          <div className="stack">
            <article className="item evidenceItem"><div><strong>Secrets and `.env`</strong><span>Excluded from product exports; settings backups are separate and controlled.</span></div><StatusPill state="excluded" /></article>
            <article className="item evidenceItem"><div><strong>Raw private artifacts</strong><span>Include only in future owner-selected backup/export flows with explicit warnings.</span></div><StatusPill state="sensitive" /></article>
            <article className="item evidenceItem"><div><strong>Runtime databases</strong><span>PostgreSQL, Qdrant, Neo4j, MLflow, Phoenix, and Redis need service-specific future procedures.</span></div><StatusPill state="future-diff" /></article>
          </div>
        </div>
        <div>
          <div className="subHeader"><h3>Dangerous Future Work</h3></div>
          <div className="stack">
            <article className="item evidenceItem"><div><strong>Destructive delete</strong><span>Requires explicit future DIFF, confirmation, audit event, and dependency review.</span></div><StatusPill state="not-implemented" /></article>
            <article className="item evidenceItem"><div><strong>Restore</strong><span>Current restore support is dry-run validation only; writing runtime records still needs a future explicit DIFF.</span></div><StatusPill state="dry-run-only" /></article>
            <article className="item evidenceItem"><div><strong>Full backup archive</strong><span>Metadata export exists; full backup archives for raw artifacts and service stores still need secret exclusion and raw-artifact policy.</span></div><StatusPill state="future-diff" /></article>
          </div>
        </div>
      </section>
    </section>
  );
}

