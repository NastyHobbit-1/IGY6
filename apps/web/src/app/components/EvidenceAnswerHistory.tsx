import type { EvidenceAnswerRecord, FeedbackRecord, ApiResult } from "./types";
import { formatDate } from "./helpers";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function EvidenceAnswerHistory({
  evidenceAnswers,
  feedback
}: {
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
}) {
  const recentAnswers = evidenceAnswers.data.slice(0, 8);
  const feedbackForAnswer = (answerId: string) => feedback.data.filter((event) => event.target_type === "evidence_answer" && event.target_id === answerId);

  return (
    <section className="guidedManualText" data-evidence-answer-history>
      <div className="guidedManualNotice">
        <strong>Saved evidence answer records</strong>
        <span>Saved answer records preserve the original retrieval review and evidence identifiers. They do not rewrite evidence, hide superseded evidence, or create full chat memory.</span>
      </div>
      {evidenceAnswers.error ? <p className="errorText">Saved answer records could not be loaded: {evidenceAnswers.error}</p> : null}
      <div className="stack">
        {recentAnswers.map((answer) => {
          const evidenceIds = answer.evidence_item_ids ?? [];
          const labels = answer.safe_labels ?? [];
          const linkedFeedback = feedbackForAnswer(answer.id);
          return (
            <article className="item evidenceItem" key={answer.id}>
              <div>
                <strong>{answer.user_question}</strong>
                <span>{answer.answer_text ?? "Saved answer record without answer text."}</span>
                <span className="messageMeta">Evidence IDs: {evidenceIds.length > 0 ? evidenceIds.slice(0, 5).join(", ") : "none recorded"}</span>
                <span className="messageMeta">Trail: {labels.length > 0 ? labels.slice(0, 6).join(" · ") : "no safe trail labels recorded"}</span>
                <span className="messageMeta">Original evidence, documents, chunks, sources, and raw artifacts remain preserved.</span>
              </div>
              <div>
                <StatusPill state={answer.answer_status} />
                <span>{answer.retrieval_mode} · {answer.retrieval_count} hit(s)</span>
                <span>{answer.local_model_status ?? "local model status not recorded"}</span>
                <span>{formatDate(answer.created_at)}</span>
                <span>{linkedFeedback.length > 0 ? `${linkedFeedback.length} feedback record(s)` : "Feedback can target this answer record."}</span>
              </div>
            </article>
          );
        })}
      </div>
      {recentAnswers.length === 0 ? <EmptyState label="No saved answer records yet. Ask over evidence, then save the answer record." /> : null}
      <p className="messageMeta">Outcomes are not offered for answer records yet because the outcome API only validates reports, work items, predictions, recommendations, hypotheses, and patterns.</p>
    </section>
  );
}

