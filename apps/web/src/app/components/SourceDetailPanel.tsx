import type { SourceRecord, CollectionRunRecord, RawArtifactRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, FeedbackRecord, OutcomeRecord, ApiResult } from "./types";
import { formatDate, formatBytes, excerpt, evidenceReviewState } from "./helpers";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function SourceDetailPanel({
  sources,
  collectionRuns,
  artifacts,
  documents,
  chunks,
  evidenceItems,
  feedback,
  outcomes
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  artifacts: ApiResult<RawArtifactRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
}) {
  const recentSources = sources.data.slice(0, 8);
  const detailFor = (source: SourceRecord) => {
    const sourceRuns = collectionRuns.data.filter((run) => run.source_id === source.id);
    const sourceArtifacts = artifacts.data.filter((artifact) => artifact.source_id === source.id || sourceRuns.some((run) => run.id === artifact.collection_run_id));
    const sourceDocuments = documents.data.filter((document) => document.source_id === source.id || sourceArtifacts.some((artifact) => artifact.id === document.raw_artifact_id));
    const documentIds = new Set(sourceDocuments.map((document) => document.id));
    const sourceChunks = chunks.data.filter((chunk) => documentIds.has(chunk.document_id));
    const chunkIds = new Set(sourceChunks.map((chunk) => chunk.id));
    const sourceEvidence = evidenceItems.data.filter((item) => item.source_id === source.id || (item.document_id ? documentIds.has(item.document_id) : false) || (item.chunk_id ? chunkIds.has(item.chunk_id) : false));
    const sourceFeedback = feedback.data.filter((event) => event.target_type === "source" && event.target_id === source.id);
    const sourceOutcomes = outcomes.data.filter((event) => event.target_type === "source" && event.target_id === source.id);
    const correctionEvidence = sourceEvidence.filter((item) => evidenceReviewState(item) !== "unreviewed");
    const safeNextAction = !source.enabled
      ? "Source is disabled. Review trust/sensitivity before future collection."
      : sourceEvidence.length > 0
        ? "Open Chat to inspect linked evidence or ask over evidence."
        : sourceRuns.length > 0
          ? "Open Work to check processing before expecting evidence."
          : "Use Guided Upload, Conversation History Import, or User Observation Ingestion if this source type is supported.";
    return {
      sourceRuns,
      sourceArtifacts,
      sourceDocuments,
      sourceChunks,
      sourceEvidence,
      sourceFeedback,
      sourceOutcomes,
      correctionEvidence,
      safeNextAction
    };
  };

  return (
    <section className="guidedManualText sourceDetailPanel" data-source-detail-panel>
      <div className="guidedManualNotice">
        <strong>Source detail.</strong>
        <span>Inspect source lineage and review state. This panel shows metadata and evidence previews only; it does not dump raw artifact contents or claim new policy enforcement.</span>
      </div>
      {[sources.error, collectionRuns.error, artifacts.error, documents.error, chunks.error, evidenceItems.error, feedback.error, outcomes.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some source detail records could not be loaded; shown detail may be incomplete.</p>
      ) : null}
      {recentSources.map((source) => {
        const detail = detailFor(source);
        return (
          <details className="advancedPanel sourceDetailCard" key={source.id}>
            <summary>{source.name} · {source.source_type} · {source.enabled ? "enabled" : "disabled"}</summary>
            <div className="sourceDetailGrid">
              <article className="item evidenceItem">
                <div>
                  <strong>{source.name}</strong>
                  <span>{source.source_type} · {source.location ?? "no location recorded"}</span>
                </div>
                <div>
                  <StatusPill state={source.trust_level || "review_needed"} />
                  <StatusPill state={source.sensitivity || "internal"} />
                  <StatusPill state={source.enabled ? "enabled" : "disabled"} />
                </div>
              </article>
              <dl className="workStatusIds">
                <dt>source id</dt><dd>{source.id}</dd>
                <dt>label</dt><dd>{source.name}</dd>
                <dt>type</dt><dd>{source.source_type}</dd>
                <dt>trust</dt><dd>{source.trust_level || "review_needed"}</dd>
                <dt>sensitivity</dt><dd>{source.sensitivity || "internal"}</dd>
                <dt>state</dt><dd>{source.enabled ? "enabled" : "disabled"}</dd>
                <dt>permissions</dt><dd>{source.permissions?.length ?? 0}</dd>
                <dt>collection runs</dt><dd>{detail.sourceRuns.length}</dd>
                <dt>artifacts</dt><dd>{detail.sourceArtifacts.length}</dd>
                <dt>documents</dt><dd>{detail.sourceDocuments.length}</dd>
                <dt>chunks</dt><dd>{detail.sourceChunks.length}</dd>
                <dt>evidence</dt><dd>{detail.sourceEvidence.length}</dd>
                <dt>feedback</dt><dd>{detail.sourceFeedback.length}</dd>
                <dt>outcomes</dt><dd>{detail.sourceOutcomes.length > 0 ? detail.sourceOutcomes.length : "not linked or unsupported"}</dd>
                <dt>corrections</dt><dd>{detail.correctionEvidence.length}</dd>
                <dt>next action</dt><dd>{detail.safeNextAction}</dd>
              </dl>
              <section className="quad">
                <div>
                  <h4>Permissions</h4>
                  <div className="stack">
                    {(source.permissions ?? []).slice(0, 4).map((permission) => (
                      <article className="item evidenceItem" key={permission.id}>
                        <div><strong>{permission.external_model_policy}</strong><span>{permission.allowed_operations.join(", ") || "no operations recorded"}</span></div>
                        <div><StatusPill state={permission.approval_required ? "approval-required" : "immediate"} /><span>{permission.id}</span></div>
                      </article>
                    ))}
                  </div>
                  {(source.permissions ?? []).length === 0 ? <EmptyState label="No permissions linked to this source." /> : null}
                </div>
                <div>
                  <h4>Collection Runs</h4>
                  <div className="stack">
                    {detail.sourceRuns.slice(0, 4).map((run) => (
                      <article className="item evidenceItem" key={run.id}>
                        <div><strong>{run.status}</strong><span>{run.dry_run ? "dry run" : "collection"} · {formatDate(run.created_at)}</span></div>
                        <div><StatusPill state={run.status} /><span>{run.id}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.sourceRuns.length === 0 ? <EmptyState label="No collection runs linked to this source." /> : null}
                </div>
                <div>
                  <h4>Artifacts</h4>
                  <div className="stack">
                    {detail.sourceArtifacts.slice(0, 4).map((artifact) => (
                      <article className="item evidenceItem" key={artifact.id}>
                        <div><strong>{artifact.mime_type ?? "unknown mime"}</strong><span>{formatBytes(artifact.size_bytes)} · {formatDate(artifact.created_at)}</span></div>
                        <div><StatusPill state="metadata-only" /><span>{artifact.id}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.sourceArtifacts.length === 0 ? <EmptyState label="No raw artifact metadata linked to this source." /> : null}
                </div>
                <div>
                  <h4>Documents And Chunks</h4>
                  <div className="stack">
                    {detail.sourceDocuments.slice(0, 4).map((document) => (
                      <article className="item evidenceItem" key={document.id}>
                        <div><strong>{document.title ?? document.document_type}</strong><span>{document.sensitivity} · {formatDate(document.created_at)}</span></div>
                        <div><StatusPill state={document.document_type} /><span>{detail.sourceChunks.filter((chunk) => chunk.document_id === document.id).length} chunk(s)</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.sourceDocuments.length === 0 ? <EmptyState label="No documents or chunks linked to this source." /> : null}
                </div>
              </section>
              <section>
                <h4>Evidence, Reviews, Feedback, And Outcomes</h4>
                <div className="stack">
                  {detail.sourceEvidence.slice(0, 6).map((item) => (
                    <article className="item evidenceItem" key={item.id}>
                      <div><strong>{excerpt(item.statement, 140)}</strong><span>{item.evidence_type} · {item.confidence ?? "unknown confidence"}</span></div>
                      <div><StatusPill state={evidenceReviewState(item)} /><span>{item.id}</span></div>
                    </article>
                  ))}
                </div>
                {detail.sourceEvidence.length === 0 ? <EmptyState label="No evidence linked to this source yet." /> : null}
                <p className="messageMeta">
                  Feedback linked directly to this source: {detail.sourceFeedback.length}. Outcomes linked directly to this source: {detail.sourceOutcomes.length}. Evidence review indicators are additive metadata; superseded evidence remains visible.
                </p>
              </section>
            </div>
          </details>
        );
      })}
      {recentSources.length === 0 ? <EmptyState label="No sources are available for detail review yet." /> : null}
    </section>
  );
}

