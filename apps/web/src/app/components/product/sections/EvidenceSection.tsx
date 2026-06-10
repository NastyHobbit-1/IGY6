import type { EvidenceItemRecord, NormalizedDocumentRecord, RawArtifactRecord, SourceRecord } from "../../types";
import { buildEvidenceCardModel } from "../evidence-display";
import { EvidenceCard } from "../EvidenceCard";
import { SectionHeader } from "../../shell/SectionHeader";
import { EmptyState } from "../../ui/EmptyState";
import { DetailDrawer } from "../../ui/DetailDrawer";

export function EvidenceSection({
  evidenceItems,
  documents,
  sourceById,
  artifactByDocumentId,
}: {
  evidenceItems: EvidenceItemRecord[];
  documents: NormalizedDocumentRecord[];
  sourceById: Map<string, SourceRecord>;
  artifactByDocumentId: Map<string, RawArtifactRecord>;
}) {
  const nonImageEvidence = evidenceItems.filter((item) => {
    const source = item.source_id ? sourceById.get(item.source_id) : undefined;
    const artifact = item.document_id ? artifactByDocumentId.get(item.document_id) : undefined;
    const model = buildEvidenceCardModel(item, source, artifact);
    return model.contentType !== "Image";
  });

  return (
    <section className="productSection" data-product-section="evidence" data-tab-panel="results" hidden id="evidence-panel">
      <SectionHeader
        eyebrow="Evidence"
        title="What IGY6 saved"
        lead="Readable evidence cards from your sources. Technical IDs are under Details in Full mode."
        statusState={nonImageEvidence.length > 0 ? "ready" : "empty"}
      />
      {nonImageEvidence.length > 0 ? (
        <div className="productCardGrid">
          {nonImageEvidence.map((item) => {
            const source = item.source_id ? sourceById.get(item.source_id) : undefined;
            const artifact = item.document_id ? artifactByDocumentId.get(item.document_id) : undefined;
            const model = buildEvidenceCardModel(item, source, artifact);
            return <EvidenceCard key={item.id} model={model} />;
          })}
        </div>
      ) : (
        <EmptyState label="No evidence yet. Add information, then check Activity while processing finishes." />
      )}
      {documents.length > 0 ? (
        <DetailDrawer summary={`Documents (${documents.length})`} id="memory-panel">
          <ul className="productPlainList">
            {documents.slice(0, 12).map((doc) => (
              <li key={doc.id} className="uiBreakLong">{doc.title ?? "Untitled document"}</li>
            ))}
          </ul>
        </DetailDrawer>
      ) : null}
    </section>
  );
}