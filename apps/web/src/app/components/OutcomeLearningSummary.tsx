import type { EvidenceAnswerRecord, PredictionRecord, RecommendationRecord, WorkItemRecord, AgentTaskPlanRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ReportRecord, ApiResult } from "./types";
import { formatDate, excerpt } from "./helpers";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function OutcomeLearningSummary({
  feedback,
  outcomes,
  improvements,
  evidenceAnswers,
  reports,
  taskPlans,
  workItems,
  predictions,
  recommendations
}: {
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  predictions: ApiResult<PredictionRecord[]>;
  recommendations: ApiResult<RecommendationRecord[]>;
}) {
  const targetLabels = new Map<string, string>();
  evidenceAnswers.data.forEach((answer) => targetLabels.set(`evidence_answer:${answer.id}`, `Answer: ${excerpt(answer.user_question, 80)}`));
  reports.data.forEach((report) => targetLabels.set(`report:${report.id}`, `Report: ${report.title}`));
  taskPlans.data.forEach((plan) => targetLabels.set(`agent_task_plan:${plan.id}`, `Task plan: ${excerpt(plan.user_request_summary, 80)}`));
  workItems.data.forEach((workItem) => targetLabels.set(`work_item:${workItem.id}`, `Work item: ${workItem.work_type}`));
  predictions.data.forEach((prediction) => targetLabels.set(`prediction:${prediction.id}`, `Prediction: ${excerpt(prediction.prediction_text, 80)}`));
  recommendations.data.forEach((recommendation) => targetLabels.set(`recommendation:${recommendation.id}`, `Recommendation: ${excerpt(recommendation.recommendation_text, 80)}`));

  const negativeFeedbackLabels = new Set(["wrong", "not_useful", "incomplete", "rejected", "weak", "noisy"]);
  const positiveFeedbackLabels = new Set(["useful", "verified", "trusted"]);
  const negativeOutcomeStatuses = new Set(["wrong", "not_useful", "partial", "inconclusive"]);
  const positiveOutcomeStatuses = new Set(["correct", "useful"]);
  const negativeSignals = [
    ...feedback.data.filter((event) => negativeFeedbackLabels.has(event.label)).map((event) => ({
      kind: "feedback",
      id: event.id,
      targetType: event.target_type,
      targetId: event.target_id,
      label: event.label,
      note: event.note ?? "",
      createdAt: event.created_at,
      linkedImprovement: improvements.data.find((item) => item.metadata_json?.feedback_id === event.id)
    })),
    ...outcomes.data.filter((outcome) => negativeOutcomeStatuses.has(outcome.outcome_status)).map((outcome) => ({
      kind: "outcome",
      id: outcome.id,
      targetType: outcome.target_type,
      targetId: outcome.target_id,
      label: outcome.outcome_status,
      note: outcome.summary ?? "",
      createdAt: outcome.created_at,
      linkedImprovement: improvements.data.find((item) => item.metadata_json?.outcome_id === outcome.id)
    }))
  ];
  const positiveSignals = [
    ...feedback.data.filter((event) => positiveFeedbackLabels.has(event.label)).map((event) => ({
      kind: "feedback",
      id: event.id,
      targetType: event.target_type,
      targetId: event.target_id,
      label: event.label,
      note: event.note ?? "",
      createdAt: event.created_at
    })),
    ...outcomes.data.filter((outcome) => positiveOutcomeStatuses.has(outcome.outcome_status)).map((outcome) => ({
      kind: "outcome",
      id: outcome.id,
      targetType: outcome.target_type,
      targetId: outcome.target_id,
      label: outcome.outcome_status,
      note: outcome.summary ?? "",
      createdAt: outcome.created_at
    }))
  ];
  const repeated = (signals: Array<{ targetType: string; targetId: string; label: string }>, field: "target" | "label") => {
    const counts = new Map<string, number>();
    signals.forEach((signal) => {
      const key = field === "target" ? `${signal.targetType}:${signal.targetId}` : signal.label;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    });
    return Array.from(counts.entries())
      .filter(([, count]) => count > 1)
      .sort((a, b) => b[1] - a[1]);
  };
  const repeatedFailedTargets = repeated(negativeSignals, "target");
  const repeatedFailedLabels = repeated(negativeSignals, "label");
  const repeatedSuccessfulTargets = repeated(positiveSignals, "target");
  const repeatedSuccessfulLabels = repeated(positiveSignals, "label");
  const unlinkedNegativeSignals = negativeSignals.filter((signal) => !signal.linkedImprovement);
  const candidatePrompt = repeatedFailedTargets.length > 0 || repeatedFailedLabels.length > 0 || unlinkedNegativeSignals.length > 0
    ? "Candidate improvement is available: use the Improvement review form below for a visible weak feedback or unresolved outcome signal."
    : "No repeated negative pattern is visible yet. Keep recording outcomes before proposing an improvement candidate.";

  return (
    <section className="panel outcomeLearningPanel" data-outcome-learning-summary>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Learning summary</p>
          <h2>Outcome Learning Summary</h2>
        </div>
        <StatusPill state={negativeSignals.length > 0 ? "review-candidates" : "no-negative-pattern"} />
      </div>
      <div className="guidedManualNotice">
        <strong>Review patterns, do not auto-change behavior.</strong>
        <span>This summary groups recorded feedback and outcomes so you can spot repeated failures or useful methods. It does not change future reasoning, promote methods, run experiments, or claim autonomous self-improvement.</span>
      </div>
      {[feedback.error, outcomes.error, improvements.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some feedback, outcome, or improvement records could not be loaded.</p>
      ) : null}
      <section className="metrics compact" aria-label="Outcome learning metrics">
        <article><span>Negative signals</span><strong>{negativeSignals.length}</strong></article>
        <article><span>Positive signals</span><strong>{positiveSignals.length}</strong></article>
        <article><span>Repeated failed labels</span><strong>{repeatedFailedLabels.length}</strong></article>
        <article><span>Repeated successful labels</span><strong>{repeatedSuccessfulLabels.length}</strong></article>
      </section>
      <section className="split">
        <div>
          <div className="subHeader"><h3>Recent Negative Outcomes</h3></div>
          <div className="stack">
            {negativeSignals.slice(0, 5).map((signal) => (
              <article className="item evidenceItem" key={`${signal.kind}:${signal.id}`}>
                <div>
                  <strong>{signal.label}</strong>
                  <span>{targetLabels.get(`${signal.targetType}:${signal.targetId}`) ?? `${signal.targetType} ${signal.targetId}`}</span>
                  <span>{signal.note || "No note recorded."}</span>
                </div>
                <div>
                  <StatusPill state={signal.linkedImprovement ? "improvement-linked" : "candidate"} />
                  <span>{formatDate(signal.createdAt)}</span>
                </div>
              </article>
            ))}
          </div>
          {negativeSignals.length === 0 ? <EmptyState label="No negative feedback or unresolved outcomes recorded yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Recent Positive Outcomes</h3></div>
          <div className="stack">
            {positiveSignals.slice(0, 5).map((signal) => (
              <article className="item evidenceItem" key={`${signal.kind}:${signal.id}`}>
                <div>
                  <strong>{signal.label}</strong>
                  <span>{targetLabels.get(`${signal.targetType}:${signal.targetId}`) ?? `${signal.targetType} ${signal.targetId}`}</span>
                  <span>{signal.note || "No note recorded."}</span>
                </div>
                <div>
                  <StatusPill state="successful-signal" />
                  <span>{formatDate(signal.createdAt)}</span>
                </div>
              </article>
            ))}
          </div>
          {positiveSignals.length === 0 ? <EmptyState label="No positive feedback or successful outcomes recorded yet." /> : null}
        </div>
      </section>
      <section className="split">
        <div>
          <div className="subHeader"><h3>Repeated Failed Signals</h3></div>
          <div className="stack">
            {[...repeatedFailedLabels.map(([label, count]) => ({ label, count, kind: "label" })), ...repeatedFailedTargets.map(([label, count]) => ({ label: targetLabels.get(label) ?? label, count, kind: "target" }))].slice(0, 6).map((item) => (
              <article className="item evidenceItem" key={`${item.kind}:${item.label}`}>
                <div><strong>{item.label}</strong><span>{item.kind} repeated {item.count} time(s).</span></div>
                <div><StatusPill state="needs-review" /></div>
              </article>
            ))}
          </div>
          {repeatedFailedLabels.length + repeatedFailedTargets.length === 0 ? <EmptyState label="No repeated failed target or label detected yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Repeated Successful Signals</h3></div>
          <div className="stack">
            {[...repeatedSuccessfulLabels.map(([label, count]) => ({ label, count, kind: "label" })), ...repeatedSuccessfulTargets.map(([label, count]) => ({ label: targetLabels.get(label) ?? label, count, kind: "target" }))].slice(0, 6).map((item) => (
              <article className="item evidenceItem" key={`${item.kind}:${item.label}`}>
                <div><strong>{item.label}</strong><span>{item.kind} repeated {item.count} time(s).</span></div>
                <div><StatusPill state="keep-observing" /></div>
              </article>
            ))}
          </div>
          {repeatedSuccessfulLabels.length + repeatedSuccessfulTargets.length === 0 ? <EmptyState label="No repeated successful target or label detected yet." /> : null}
        </div>
      </section>
      <div className="guidedManualResult">
        <strong>Candidate improvement prompt</strong>
        <span>{candidatePrompt}</span>
      </div>
    </section>
  );
}

