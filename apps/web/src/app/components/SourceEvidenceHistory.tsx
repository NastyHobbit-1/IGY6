import type { SourceRecord, CollectionRunRecord, RawArtifactRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, ApiResult } from "./types";
import { EmptyState } from "./ui/EmptyState";

export function SourceEvidenceHistory({
  sources,
  collectionRuns,
  artifacts,
  documents,
  chunks,
  evidenceItems
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  artifacts: ApiResult<RawArtifactRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const artifactsByRun = new Map<string, RawArtifactRecord[]>();
  artifacts.data.forEach((artifact) => {
    const key = artifact.collection_run_id ?? "";
    if (!key) return;
    artifactsByRun.set(key, [...(artifactsByRun.get(key) ?? []), artifact]);
  });
  const documentsByArtifact = new Map<string, NormalizedDocumentRecord[]>();
  documents.data.forEach((document) => {
    const key = document.raw_artifact_id ?? "";
    if (!key) return;
    documentsByArtifact.set(key, [...(documentsByArtifact.get(key) ?? []), document]);
  });
  const chunksByDocument = new Map<string, ChunkRecord[]>();
  chunks.data.forEach((chunk) => {
    chunksByDocument.set(chunk.document_id, [...(chunksByDocument.get(chunk.document_id) ?? []), chunk]);
  });
  const evidenceByChunk = new Map<string, EvidenceItemRecord[]>();
  const evidenceByDocument = new Map<string, EvidenceItemRecord[]>();
  evidenceItems.data.forEach((item) => {
    if (item.chunk_id) {
      evidenceByChunk.set(item.chunk_id, [...(evidenceByChunk.get(item.chunk_id) ?? []), item]);
    }
    if (item.document_id) {
      evidenceByDocument.set(item.document_id, [...(evidenceByDocument.get(item.document_id) ?? []), item]);
    }
  });

  const histories = collectionRuns.data.slice(0, 5).map((run) => {
    const runArtifacts = artifactsByRun.get(run.id) ?? [];
    const runDocuments = runArtifacts.flatMap((artifact) => documentsByArtifact.get(artifact.id) ?? []);
    const runChunks = runDocuments.flatMap((document) => chunksByDocument.get(document.id) ?? []);
    const chunkEvidence = runChunks.flatMap((chunk) => evidenceByChunk.get(chunk.id) ?? []);
    const documentEvidence = runDocuments.flatMap((document) => evidenceByDocument.get(document.id) ?? []);
    const uniqueEvidence = [...new Map([...chunkEvidence, ...documentEvidence].map((item) => [item.id, item])).values()];
    return {
      run,
      source: run.source_id ? sourceById.get(run.source_id) : undefined,
      artifacts: runArtifacts,
      documents: runDocuments,
      chunks: runChunks,
      evidence: uniqueEvidence
    };
  });

  return (
    <section className="guidedManualText sourceHistory" data-source-evidence-history>
      <div className="guidedManualNotice">
        <strong>Source and evidence history</strong>
        <span>Recent processing lineage by identifier only. Raw uploaded text and artifact files are not displayed here.</span>
      </div>
      <div className="stack">
        {histories.map((history) => (
          <article className="item evidenceItem" key={history.run.id} data-source-history-item>
            <div>
              <strong>{history.source?.name ?? "Unknown source"}</strong>
              <span>{history.source?.source_type ?? "no source type"} · run {history.run.id}</span>
            </div>
            <dl>
              <dt>source</dt><dd>{history.run.source_id ?? "not linked"}</dd>
              <dt>status</dt><dd>{history.run.status}</dd>
              <dt>artifact</dt><dd>{history.artifacts[0]?.id ?? "none recorded"}</dd>
              <dt>document</dt><dd>{history.documents[0]?.id ?? "none recorded"}</dd>
              <dt>chunks</dt><dd>{history.chunks.length}</dd>
              <dt>evidence</dt><dd>{history.evidence.length}</dd>
            </dl>
          </article>
        ))}
      </div>
      {histories.length === 0 ? <EmptyState label="No source/evidence history is available yet." /> : null}
    </section>
  );
}

