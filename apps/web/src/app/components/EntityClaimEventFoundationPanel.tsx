import type { SourceRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, ClaimRecord, ApiResult } from "./types";
import { formatDate, excerpt, uniqueStringValues, shortRecordId, jsonString, evidenceReviewState } from "./helpers";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function EntityClaimEventFoundationPanel({
  evidenceItems,
  claims,
  sources,
  documents,
  chunks
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  claims: ApiResult<ClaimRecord[]>;
  sources: ApiResult<SourceRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
}) {
  type RelationshipReviewCandidate = {
    id: string;
    relationType: string;
    subject: string;
    object: string;
    provenanceText: string;
    reviewStatus: string;
    supportCount: number;
    confidence: number | null;
  };
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const documentById = new Map(documents.data.map((document) => [document.id, document]));
  const chunkById = new Map(chunks.data.map((chunk) => [chunk.id, chunk]));
  const claimsByEvidence = new Map<string, ClaimRecord[]>();
  claims.data.forEach((claim) => {
    (claim.evidence_ids ?? []).forEach((id) => {
      const current = claimsByEvidence.get(id) ?? [];
      current.push(claim);
      claimsByEvidence.set(id, current);
    });
  });

  const entityCandidates = evidenceItems.data.flatMap((item) => {
    const phrases = uniqueStringValues(item.statement.match(/\b[A-Z][A-Za-z0-9-]{2,}(?:\s+[A-Z][A-Za-z0-9-]{2,}){0,2}\b/g) ?? [], 4);
    return phrases.map((phrase) => ({ phrase, item }));
  }).slice(0, 8);
  const claimCandidates = evidenceItems.data
    .filter((item) => !claimsByEvidence.has(item.id))
    .slice(0, 8)
    .map((item) => ({ item, text: excerpt(item.statement, 160) }));
  const eventCandidates = evidenceItems.data
    .filter((item) => {
      const observedAt = jsonString(item.metadata_json?.observed_at) ?? jsonString(item.metadata_json?.decided_at);
      return Boolean(observedAt || /\b(20\d{2}-\d{2}-\d{2}|yesterday|today|tomorrow|incident|meeting|release|decision)\b/i.test(item.statement));
    })
    .slice(0, 8);
  const provenance = (item: EvidenceItemRecord): string => {
    const source = item.source_id ? sourceById.get(item.source_id) : null;
    const document = item.document_id ? documentById.get(item.document_id) : null;
    const chunk = item.chunk_id ? chunkById.get(item.chunk_id) : null;
    return [
      source ? `source ${source.name}` : "source not linked",
      document ? `document ${document.title ?? shortRecordId(document.id)}` : "document not linked",
      chunk ? `chunk ${chunk.chunk_index}` : "chunk not linked",
      `evidence ${shortRecordId(item.id)}`
    ].join(" > ");
  };
  const linkedClaims = claims.data.filter((claim) => (claim.evidence_ids ?? []).length > 0).slice(0, 8);
  const evidenceRelationshipCandidates = evidenceItems.data.flatMap((item) => {
    const rows: RelationshipReviewCandidate[] = [];
    if (item.source_id) {
      rows.push({
        id: `${item.id}:source`,
        relationType: "evidence_observed_from_source",
        subject: `evidence ${shortRecordId(item.id)}`,
        object: `source ${sourceById.get(item.source_id)?.name ?? shortRecordId(item.source_id)}`,
        provenanceText: provenance(item),
        reviewStatus: evidenceReviewState(item),
        supportCount: 1,
        confidence: item.confidence
      });
    }
    if (item.document_id) {
      rows.push({
        id: `${item.id}:document`,
        relationType: "evidence_extracted_from_document",
        subject: `evidence ${shortRecordId(item.id)}`,
        object: `document ${documentById.get(item.document_id)?.title ?? shortRecordId(item.document_id)}`,
        provenanceText: provenance(item),
        reviewStatus: evidenceReviewState(item),
        supportCount: 1,
        confidence: item.confidence
      });
    }
    if (item.chunk_id) {
      rows.push({
        id: `${item.id}:chunk`,
        relationType: "evidence_supported_by_chunk",
        subject: `evidence ${shortRecordId(item.id)}`,
        object: `chunk ${chunkById.get(item.chunk_id)?.chunk_index ?? shortRecordId(item.chunk_id)}`,
        provenanceText: provenance(item),
        reviewStatus: evidenceReviewState(item),
        supportCount: 1,
        confidence: item.confidence
      });
    }
    return rows;
  });
  const claimRelationshipCandidates = linkedClaims.flatMap((claim) => (claim.evidence_ids ?? []).map((evidenceId) => {
    const item = evidenceItems.data.find((candidate) => candidate.id === evidenceId);
    return {
      id: `${claim.id}:${evidenceId}`,
      relationType: "claim_supported_by_evidence",
      subject: `claim ${shortRecordId(claim.id)}`,
      object: `evidence ${shortRecordId(evidenceId)}`,
      provenanceText: item ? provenance(item) : `evidence ${shortRecordId(evidenceId)} not loaded`,
      reviewStatus: claim.status,
      supportCount: claim.evidence_ids?.length ?? 0,
      confidence: claim.confidence
    };
  }));
  const relationshipCandidates = [...evidenceRelationshipCandidates, ...claimRelationshipCandidates].slice(0, 10);

  return (
    <section className="panel entityClaimEventFoundation" data-entity-claim-event-foundation>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Structured memory foundation</p>
          <h2>Entity, Claim, Event, And Relationship Review</h2>
        </div>
        <StatusPill state="relationship-foundation" />
      </div>
      <div className="guidedManualNotice">
        <strong>Evidence-tied review only.</strong>
        <span>This surface derives conservative review candidates from local text evidence, lineage links, and existing claim records. It does not mutate evidence, resolve identities, run hosted AI, claim correlation discovery, or claim full graph reasoning.</span>
      </div>
      {[evidenceItems.error, claims.error, sources.error, documents.error, chunks.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some structured review inputs could not be loaded; candidates may be incomplete.</p>
      ) : null}
      <section className="metrics compact" aria-label="Structured review counts">
        <article><span>Evidence items</span><strong>{evidenceItems.data.length}</strong></article>
        <article><span>Stored claims</span><strong>{claims.data.length}</strong></article>
        <article><span>Entity candidates</span><strong>{entityCandidates.length}</strong></article>
        <article><span>Claim candidates</span><strong>{claimCandidates.length}</strong></article>
        <article><span>Event candidates</span><strong>{eventCandidates.length}</strong></article>
        <article><span>Relationship candidates</span><strong>{relationshipCandidates.length}</strong></article>
      </section>
      <section className="panelInset">
        <div className="subHeader"><h3>Relationship Candidates With Provenance</h3></div>
        <div className="stack">
          {relationshipCandidates.map((relationship) => (
            <article className="item evidenceItem" key={relationship.id}>
              <div>
                <strong>{relationship.relationType}</strong>
                <span>{relationship.subject} to {relationship.object}</span>
                <span>{relationship.provenanceText}</span>
                <span>Review status: {relationship.reviewStatus}</span>
              </div>
              <div>
                <StatusPill state="review-only" />
                <span>support {relationship.supportCount}</span>
                <span>{relationship.confidence === null ? "unscored" : `${relationship.confidence}% confidence`}</span>
              </div>
            </article>
          ))}
        </div>
        {relationshipCandidates.length === 0 ? <EmptyState label="No relationship candidates are available from loaded evidence and claims yet." /> : null}
      </section>
      <section className="quad">
        <div>
          <div className="subHeader"><h3>Entity Candidates</h3></div>
          <div className="stack">
            {entityCandidates.map(({ phrase, item }, index) => (
              <article className="item evidenceItem" key={`${item.id}:entity:${phrase}:${index}`}>
                <div>
                  <strong>{phrase}</strong>
                  <span>{provenance(item)}</span>
                  <span>Unverified: capitalization is only a review hint, not entity resolution.</span>
                </div>
                <div><StatusPill state="needs-review" /><span>{item.confidence === null ? "unscored evidence" : `${item.confidence}% evidence`}</span></div>
              </article>
            ))}
          </div>
          {entityCandidates.length === 0 ? <EmptyState label="No simple entity review candidates found in loaded evidence." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Claim Candidates</h3></div>
          <div className="stack">
            {claimCandidates.map(({ item, text }) => (
              <article className="item evidenceItem" key={`${item.id}:claim`}>
                <div>
                  <strong>{item.evidence_type}</strong>
                  <span>{text}</span>
                  <span>{provenance(item)}</span>
                  <span>Next: review before any future claim-create workflow stores it.</span>
                </div>
                <div><StatusPill state="review-only" /><span>{evidenceReviewState(item)}</span></div>
              </article>
            ))}
          </div>
          {claimCandidates.length === 0 ? <EmptyState label="No unclaimed evidence candidates found." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Event Candidates</h3></div>
          <div className="stack">
            {eventCandidates.map((item) => (
              <article className="item evidenceItem" key={`${item.id}:event`}>
                <div>
                  <strong>{jsonString(item.metadata_json?.observed_at) ?? jsonString(item.metadata_json?.decided_at) ?? "date needs review"}</strong>
                  <span>{excerpt(item.statement, 140)}</span>
                  <span>{provenance(item)}</span>
                  <span>Unverified: event timing and meaning require owner review.</span>
                </div>
                <div><StatusPill state="needs-review" /><span>{formatDate(item.created_at)}</span></div>
              </article>
            ))}
          </div>
          {eventCandidates.length === 0 ? <EmptyState label="No event review candidates found in loaded evidence." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Stored Claims With Provenance</h3>{claims.error ? <span className="errorText">{claims.error}</span> : null}</div>
          <div className="stack">
            {linkedClaims.map((claim) => (
              <article className="item evidenceItem" key={claim.id}>
                <div>
                  <strong>{claim.claim_type}</strong>
                  <span>{excerpt(claim.claim_text, 140)}</span>
                  <span>Evidence: {(claim.evidence_ids ?? []).slice(0, 4).map(shortRecordId).join(", ") || "not linked"}</span>
                  <span>Metadata: {claim.metadata_json ? "available" : "none"}</span>
                </div>
                <div><StatusPill state={claim.status} /><span>{claim.confidence === null ? "unscored" : `${claim.confidence}%`}</span></div>
              </article>
            ))}
          </div>
          {linkedClaims.length === 0 ? <EmptyState label="No stored claims with evidence provenance loaded yet." /> : null}
        </div>
      </section>
      <p className="messageMeta">Current gateway support exposes claim reads and relational lineage review. Entity/event persistence, relationship persistence, Neo4j sync actions, and claim creation remain future scoped work. Safe next action: review candidates against the source/evidence detail panels before relying on them.</p>
    </section>
  );
}

