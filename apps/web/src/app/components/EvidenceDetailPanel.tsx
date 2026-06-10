import type { SourceRecord, NormalizedDocumentRecord, ChunkRecord, EvidenceItemRecord, EvidenceAnswerRecord, AgentTaskPlanRecord, FeedbackRecord, OutcomeRecord, ReportRecord, ApiResult } from "./types";
import { formatDate, excerpt, evidenceReviewState, evidenceReviewNote } from "./helpers";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function EvidenceDetailPanel({
  evidenceItems,
  sources,
  documents,
  chunks,
  evidenceAnswers,
  taskPlans,
  reports,
  feedback,
  outcomes
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  sources: ApiResult<SourceRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
}) {
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const documentById = new Map(documents.data.map((document) => [document.id, document]));
  const chunkById = new Map(chunks.data.map((chunk) => [chunk.id, chunk]));
  const metadataMentions = (metadata: Record<string, unknown> | null | undefined, id: string): boolean => {
    if (!metadata) {
      return false;
    }
    try {
      return JSON.stringify(metadata).includes(id);
    } catch {
      return false;
    }
  };
  const reviewRecord = (item: EvidenceItemRecord): Record<string, unknown> | null => {
    const reviewState = item.metadata_json?.review_state;
    return reviewState && typeof reviewState === "object" ? reviewState as Record<string, unknown> : null;
  };
  const detailFor = (item: EvidenceItemRecord) => {
    const source = item.source_id ? sourceById.get(item.source_id) ?? null : null;
    const document = item.document_id ? documentById.get(item.document_id) ?? null : null;
    const chunk = item.chunk_id ? chunkById.get(item.chunk_id) ?? null : null;
    const directFeedback = feedback.data.filter((event) => event.target_type === "evidence_item" && event.target_id === item.id);
    const directOutcomes = outcomes.data.filter((event) => event.target_type === "evidence_item" && event.target_id === item.id);
    const relatedAnswers = evidenceAnswers.data.filter((answer) => (answer.evidence_item_ids ?? []).includes(item.id));
    const relatedTaskPlans = taskPlans.data.filter((plan) => metadataMentions(plan.metadata_json, item.id));
    const relatedReports = reports.data.filter((report) => metadataMentions(report.metadata_json, item.id));
    const review = reviewRecord(item);
    const supersedingEvidenceId = typeof review?.superseding_evidence_item_id === "string" ? review.superseding_evidence_item_id : null;
    const safeNextAction = evidenceReviewState(item) === "superseded"
      ? "Inspect the superseding evidence link before relying on this evidence."
      : directFeedback.length === 0
        ? "Add feedback if this evidence is useful, weak, wrong, or incomplete."
        : relatedAnswers.length > 0
          ? "Review related saved answer records and citations before using this evidence in a decision."
          : "Use Ask over evidence or create a report after confirming the source trail.";
    return {
      source,
      document,
      chunk,
      directFeedback,
      directOutcomes,
      relatedAnswers,
      relatedTaskPlans,
      relatedReports,
      supersedingEvidenceId,
      safeNextAction
    };
  };
  const recentEvidence = evidenceItems.data.slice(0, 10);

  return (
    <section className="guidedManualText evidenceDetailPanel" data-evidence-detail-panel>
      <div className="guidedManualNotice">
        <strong>Evidence detail.</strong>
        <span>Inspect evidence preview, source trail, lineage, review state, feedback, outcomes, answers, task plans, and reports. This view is read-only and keeps long raw text bounded.</span>
      </div>
      {[evidenceItems.error, sources.error, documents.error, chunks.error, evidenceAnswers.error, taskPlans.error, reports.error, feedback.error, outcomes.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some evidence detail records could not be loaded; shown detail may be incomplete.</p>
      ) : null}
      {recentEvidence.map((item) => {
        const detail = detailFor(item);
        const metadataSensitivity = typeof item.metadata_json?.sensitivity === "string" ? item.metadata_json.sensitivity : "sensitivity-unknown";
        return (
          <details className="advancedPanel evidenceDetailCard" key={item.id}>
            <summary>{item.evidence_type} · {evidenceReviewState(item)} · {excerpt(item.statement, 72)}</summary>
            <div className="sourceDetailGrid">
              <article className="item evidenceItem">
                <div>
                  <strong>{excerpt(item.statement, 260)}</strong>
                  <span>{item.evidence_type} · confidence {item.confidence ?? "unknown"} · created {formatDate(item.created_at)}</span>
                </div>
                <div>
                  <StatusPill state={evidenceReviewState(item)} />
                  <StatusPill state={detail.source?.trust_level || "source-trust-unknown"} />
                  <StatusPill state={detail.source?.sensitivity || metadataSensitivity} />
                </div>
              </article>
              <dl className="workStatusIds">
                <dt>evidence id</dt><dd>{item.id}</dd>
                <dt>source</dt><dd>{detail.source ? `${detail.source.name} · ${detail.source.id}` : item.source_id ?? "not linked"}</dd>
                <dt>document</dt><dd>{detail.document ? `${detail.document.title ?? detail.document.document_type} · ${detail.document.id}` : item.document_id ?? "not linked"}</dd>
                <dt>chunk</dt><dd>{detail.chunk ? `index ${detail.chunk.chunk_index} · ${detail.chunk.id}` : item.chunk_id ?? "not linked"}</dd>
                <dt>source trust</dt><dd>{detail.source?.trust_level ?? "not available"}</dd>
                <dt>source sensitivity</dt><dd>{detail.source?.sensitivity ?? "not available"}</dd>
                <dt>review note</dt><dd>{evidenceReviewNote(item) ?? "not recorded"}</dd>
                <dt>superseding evidence</dt><dd>{detail.supersedingEvidenceId ?? "not linked"}</dd>
                <dt>feedback</dt><dd>{detail.directFeedback.length}</dd>
                <dt>outcomes</dt><dd>{detail.directOutcomes.length > 0 ? detail.directOutcomes.length : "not linked or unsupported"}</dd>
                <dt>saved answers</dt><dd>{detail.relatedAnswers.length}</dd>
                <dt>task plans</dt><dd>{detail.relatedTaskPlans.length > 0 ? detail.relatedTaskPlans.length : "not linked by metadata"}</dd>
                <dt>reports</dt><dd>{detail.relatedReports.length > 0 ? detail.relatedReports.length : "not linked by metadata"}</dd>
                <dt>next action</dt><dd>{detail.safeNextAction}</dd>
              </dl>
              <section className="quad">
                <div>
                  <h4>Feedback</h4>
                  <div className="stack">
                    {detail.directFeedback.slice(0, 4).map((event) => (
                      <article className="item evidenceItem" key={event.id}>
                        <div><strong>{event.label}</strong><span>{event.note ?? "no note"}</span></div>
                        <div><StatusPill state={event.actor_id} /><span>{formatDate(event.created_at)}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.directFeedback.length === 0 ? <EmptyState label="No feedback linked to this evidence." /> : null}
                </div>
                <div>
                  <h4>Outcomes</h4>
                  <div className="stack">
                    {detail.directOutcomes.slice(0, 4).map((event) => (
                      <article className="item evidenceItem" key={event.id}>
                        <div><strong>{event.outcome_status}</strong><span>{event.summary ?? "Outcome recorded"}</span></div>
                        <div><StatusPill state={event.target_type} /><span>{formatDate(event.created_at)}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.directOutcomes.length === 0 ? <EmptyState label="No direct evidence outcome links are available." /> : null}
                </div>
                <div>
                  <h4>Saved Answers</h4>
                  <div className="stack">
                    {detail.relatedAnswers.slice(0, 4).map((answer) => (
                      <article className="item evidenceItem" key={answer.id}>
                        <div><strong>{excerpt(answer.user_question, 90)}</strong><span>{answer.answer_status} · {answer.retrieval_count} hit(s)</span></div>
                        <div><StatusPill state={answer.local_model_status ?? "local-model-not-recorded"} /><span>{answer.id}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.relatedAnswers.length === 0 ? <EmptyState label="No saved answer records cite this evidence yet." /> : null}
                </div>
                <div>
                  <h4>Plans And Reports</h4>
                  <div className="stack">
                    {detail.relatedTaskPlans.slice(0, 2).map((plan) => (
                      <article className="item evidenceItem" key={plan.id}>
                        <div><strong>{excerpt(plan.user_request_summary, 90)}</strong><span>{plan.intent_category}</span></div>
                        <div><StatusPill state={plan.status} /><span>{plan.id}</span></div>
                      </article>
                    ))}
                    {detail.relatedReports.slice(0, 2).map((report) => (
                      <article className="item evidenceItem" key={report.id}>
                        <div><strong>{report.title}</strong><span>{report.report_type}</span></div>
                        <div><StatusPill state={report.status} /><span>{report.id}</span></div>
                      </article>
                    ))}
                  </div>
                  {detail.relatedTaskPlans.length === 0 && detail.relatedReports.length === 0 ? <EmptyState label="No task plan or report metadata links were found." /> : null}
                </div>
              </section>
            </div>
          </details>
        );
      })}
      {recentEvidence.length === 0 ? <EmptyState label="No evidence items are available for detail review yet." /> : null}
    </section>
  );
}

