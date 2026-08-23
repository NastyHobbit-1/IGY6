import type { SourceRecord, CollectionRunRecord, RawArtifactRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, EvidenceAnswerRecord, GraphSchemaStatus, AgentTaskPlanRecord, ReportRecord, ApiResult } from "./types";
import { uniqueStringValues, shortRecordId, evidenceReviewState, metadataMentionsId } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function GraphLineageExplanationPanel({
  sources,
  collectionRuns,
  artifacts,
  documents,
  chunks,
  evidenceItems,
  evidenceAnswers,
  reports,
  taskPlans,
  graphSchema
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  artifacts: ApiResult<RawArtifactRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  graphSchema: ApiResult<GraphSchemaStatus>;
}) {
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const artifactById = new Map(artifacts.data.map((artifact) => [artifact.id, artifact]));
  const lineageRows = sources.data.slice(0, 8).map((source) => {
    const sourceRuns = collectionRuns.data.filter((run) => run.source_id === source.id);
    const sourceArtifacts = artifacts.data.filter((artifact) => artifact.source_id === source.id || sourceRuns.some((run) => run.id === artifact.collection_run_id));
    const sourceDocuments = documents.data.filter((document) => document.source_id === source.id || (document.raw_artifact_id ? sourceArtifacts.some((artifact) => artifact.id === document.raw_artifact_id) : false));
    const documentIds = new Set(sourceDocuments.map((document) => document.id));
    const sourceChunks = chunks.data.filter((chunk) => documentIds.has(chunk.document_id));
    const chunkIds = new Set(sourceChunks.map((chunk) => chunk.id));
    const sourceEvidence = evidenceItems.data.filter((item) => item.source_id === source.id || (item.document_id ? documentIds.has(item.document_id) : false) || (item.chunk_id ? chunkIds.has(item.chunk_id) : false));
    const evidenceIds = new Set(sourceEvidence.map((item) => item.id));
    const sourceAnswers = evidenceAnswers.data.filter((answer) => (answer.source_ids ?? []).includes(source.id) || (answer.evidence_item_ids ?? []).some((id) => evidenceIds.has(id)) || (answer.document_ids ?? []).some((id) => documentIds.has(id)) || (answer.chunk_ids ?? []).some((id) => chunkIds.has(id)));
    const sourceReports = reports.data.filter((report) => metadataMentionsId(report.metadata_json, source.id) || sourceEvidence.some((item) => metadataMentionsId(report.metadata_json, item.id)));
    const sourceTaskPlans = taskPlans.data.filter((plan) => metadataMentionsId(plan.metadata_json, source.id) || sourceEvidence.some((item) => metadataMentionsId(plan.metadata_json, item.id)));
    const correctionStates = uniqueStringValues(sourceEvidence.map(evidenceReviewState), 6);
    const firstDocument = sourceDocuments[0];
    const firstArtifact = firstDocument?.raw_artifact_id ? artifactById.get(firstDocument.raw_artifact_id) : sourceArtifacts[0];
    const firstChunk = firstDocument ? chunks.data.find((chunk) => chunk.document_id === firstDocument.id) : sourceChunks[0];
    const firstEvidence = firstChunk ? sourceEvidence.find((item) => item.chunk_id === firstChunk.id) : sourceEvidence[0];
    const trail = [
      `source ${source.name}`,
      firstArtifact ? `artifact ${shortRecordId(firstArtifact.id)} (${firstArtifact.mime_type ?? "unknown type"})` : "artifact not linked",
      firstDocument ? `document ${firstDocument.title ?? shortRecordId(firstDocument.id)}` : "document not linked",
      firstChunk ? `chunk ${firstChunk.chunk_index}` : "chunk not linked",
      firstEvidence ? `evidence ${shortRecordId(firstEvidence.id)}` : "evidence not linked",
      sourceAnswers.length > 0 ? `${sourceAnswers.length} answer record(s)` : "no linked answers",
      sourceReports.length > 0 ? `${sourceReports.length} report(s)` : "no linked reports",
      sourceTaskPlans.length > 0 ? `${sourceTaskPlans.length} task plan(s)` : "no linked task plans"
    ];
    const safeNextAction = sourceEvidence.length > 0
      ? "Open evidence detail or Ask over evidence to inspect citations before relying on this lineage."
      : sourceDocuments.length > 0
        ? "Open Work to confirm chunk/evidence generation completed."
        : "Add or process supported local text before expecting downstream evidence.";
    return {
      source,
      sourceRuns,
      sourceArtifacts,
      sourceDocuments,
      sourceChunks,
      sourceEvidence,
      sourceAnswers,
      sourceReports,
      sourceTaskPlans,
      correctionStates,
      trail,
      safeNextAction
    };
  });

  return (
    <section className="panel graphLineageExplanation" data-graph-lineage-explanation>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Graph and lineage</p>
          <h2>Lineage Explanation</h2>
        </div>
        <StatusPill state={graphSchema.data.constraints.length > 0 ? "neo4j-schema-visible" : "relational-fallback"} />
      </div>
      <div className="guidedManualNotice">
        <strong>{graphSchema.data.constraints.length > 0 ? "Neo4j schema foundation is visible." : "Using relational lineage fallback."}</strong>
        <span>This view explains why local records are connected from source to artifact to document to chunk to evidence to answer/report/task. It does not claim full graph reasoning, correlation discovery, or secret/raw data export.</span>
      </div>
      {[sources.error, collectionRuns.error, artifacts.error, documents.error, chunks.error, evidenceItems.error, evidenceAnswers.error, reports.error, taskPlans.error, graphSchema.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some lineage records could not be loaded; shown lineage may be incomplete.</p>
      ) : null}
      <section className="metrics compact" aria-label="Lineage record counts">
        <article><span>Sources</span><strong>{sources.data.length}</strong></article>
        <article><span>Artifacts</span><strong>{artifacts.data.length}</strong></article>
        <article><span>Documents</span><strong>{documents.data.length}</strong></article>
        <article><span>Chunks</span><strong>{chunks.data.length}</strong></article>
        <article><span>Evidence</span><strong>{evidenceItems.data.length}</strong></article>
        <article><span>Graph constraints</span><strong>{graphSchema.data.constraints.length}</strong></article>
      </section>
      <div className="stack">
        {lineageRows.map((row) => (
          <article className="item evidenceItem" key={row.source.id}>
            <div>
              <strong>{row.source.name}</strong>
              <span>{row.trail.join(" > ")}</span>
              <span>Correction/supersession state: {row.correctionStates.length > 0 ? row.correctionStates.join(", ") : "not reviewed"}</span>
              <span>Next: {row.safeNextAction}</span>
            </div>
            <div>
              <StatusPill state={row.source.trust_level || "review_needed"} />
              <StatusPill state={row.source.sensitivity || "internal"} />
              <StatusPill state={row.source.enabled ? "enabled" : "disabled"} />
              <span>{row.sourceArtifacts.length} artifacts · {row.sourceDocuments.length} documents · {row.sourceChunks.length} chunks · {row.sourceEvidence.length} evidence</span>
            </div>
          </article>
        ))}
      </div>
      {lineageRows.length === 0 ? <EmptyState label="No sources are available for lineage explanation yet." /> : null}
      <section className="guidedManualActions" data-graph-lineage-ops data-api-base-url="/api">
        <button type="button" data-graph-ensure-schema>Ensure Neo4j schema</button>
        <button type="button" data-graph-sync-lineage>Sync lineage to graph</button>
        <span data-graph-lineage-ops-result>Run when vector/graph memory needs schema or lineage refresh.</span>
      </section>
      <ClientScript script={`
(() => {
  const root = document.querySelector("[data-graph-lineage-ops]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const result = root.querySelector("[data-graph-lineage-ops-result]");
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body || {}) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  const run = async (label, path) => {
    if (result) result.textContent = label + " running...";
    try {
      const payload = await postJson(path, {});
      if (result) result.textContent = label + " complete: " + JSON.stringify(payload);
    } catch (error) {
      if (result) result.textContent = label + " failed: " + (error instanceof Error ? error.message : "Unknown error");
    }
  };
  root.querySelector("[data-graph-ensure-schema]")?.addEventListener("click", () => run("Ensure Neo4j schema", "/memory/graph/schema/ensure"));
  root.querySelector("[data-graph-sync-lineage]")?.addEventListener("click", () => run("Sync lineage", "/memory/graph/lineage/sync"));
})();
`} />
    </section>
  );
}

