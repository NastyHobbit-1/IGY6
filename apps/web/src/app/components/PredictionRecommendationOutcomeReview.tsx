import type { EvidenceAnswerRecord, PredictionRecord, RecommendationRecord, CalibrationSummary, AgentTaskPlanRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ReportRecord, ApiResult } from "./types";
import { formatDate, excerpt } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function PredictionRecommendationOutcomeReview({
  predictions,
  recommendations,
  evidenceAnswers,
  reports,
  taskPlans,
  feedback,
  outcomes,
  improvements,
  calibrationSummary
}: {
  predictions: ApiResult<PredictionRecord[]>;
  recommendations: ApiResult<RecommendationRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
  calibrationSummary: ApiResult<CalibrationSummary>;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const records = [
    ...predictions.data.slice(0, 8).map((prediction) => ({
      type: "prediction",
      id: prediction.id,
      title: excerpt(prediction.prediction_text, 100),
      detail: prediction.expected_result,
      status: prediction.status,
      evidenceIds: prediction.evidence_ids ?? [],
      metadata: prediction.metadata_json ?? {}
    })),
    ...recommendations.data.slice(0, 8).map((recommendation) => ({
      type: "recommendation",
      id: recommendation.id,
      title: excerpt(recommendation.recommendation_text, 100),
      detail: recommendation.expected_result ?? recommendation.risk_level,
      status: recommendation.status,
      evidenceIds: recommendation.evidence_ids ?? [],
      metadata: recommendation.metadata_json ?? {}
    }))
  ];
  const firstEvidenceIds = records[0]?.evidenceIds ?? [];
  const contextLabel = (metadata: Record<string, unknown>) => {
    const contextType = typeof metadata.context_type === "string" ? metadata.context_type : "";
    const contextId = typeof metadata.context_id === "string" ? metadata.context_id : "";
    if (!contextType || !contextId) return "No answer/report/task context recorded.";
    if (contextType === "answer") {
      const answer = evidenceAnswers.data.find((item) => item.id === contextId);
      return answer ? `Answer: ${excerpt(answer.user_question, 80)}` : `Answer ${contextId}`;
    }
    if (contextType === "report") {
      const report = reports.data.find((item) => item.id === contextId);
      return report ? `Report: ${report.title}` : `Report ${contextId}`;
    }
    if (contextType === "task_plan") {
      const plan = taskPlans.data.find((item) => item.id === contextId);
      return plan ? `Task: ${excerpt(plan.user_request_summary, 80)}` : `Task plan ${contextId}`;
    }
    return `${contextType} ${contextId}`;
  };
  const reviewRecords = records.map((record) => ({
    ...record,
    directFeedback: feedback.data.filter((event) => event.target_type === record.type && event.target_id === record.id),
    directOutcomes: outcomes.data.filter((outcome) => outcome.target_type === record.type && outcome.target_id === record.id),
    linkedImprovements: improvements.data.filter((item) => item.metadata_json?.target_type === record.type && item.metadata_json?.target_id === record.id)
  }));
  const script = `
(() => {
  const root = document.querySelector("[data-pr-outcome-review]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const form = root.querySelector("[data-pr-outcome-form]");
  const result = root.querySelector("[data-pr-outcome-result]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const selected = () => root.querySelector("[name='review_target']")?.selectedOptions?.[0];
  const evidenceIds = () => value("outcome_evidence_ids").split(",").map((item) => item.trim()).filter(Boolean);
  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-pr-outcome-status", "");
      [
        ["record", payload.id],
        ["status", payload.outcome_status || payload.status],
        ["target", payload.target_type ? payload.target_type + " " + payload.target_id : payload.target_area]
      ].forEach(([label, detail]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = String(detail ?? "not returned");
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const option = selected();
    if (!option) {
      show("No target", "Create a prediction or recommendation before recording an outcome.");
      return;
    }
    const targetType = option.getAttribute("data-target-type");
    const targetId = option.value;
    const outcomeStatus = value("outcome_status");
    const ids = evidenceIds();
    try {
      const outcome = await postJson("/outcomes", {
        target_type: targetType,
        target_id: targetId,
        outcome_status: outcomeStatus,
        summary: value("outcome_summary") || null,
        evidence_ids: ids,
        metadata_json: {
          created_from: "prediction_recommendation_outcome_review",
          improvement_candidate_requested: checked("create_improvement_candidate"),
          auto_executed_recommendation: false,
          auto_changed_future_behavior: false
        }
      });
      const improvementStatuses = new Set(["wrong", "not_useful", "partial", "inconclusive"]);
      if (checked("create_improvement_candidate") && improvementStatuses.has(outcomeStatus)) {
        const improvement = await postJson("/improvements", {
          target_area: targetType === "prediction" ? "prediction" : "reasoning",
          objective: value("improvement_objective") || ("Review " + outcomeStatus + " outcome for " + targetType + " " + targetId + "."),
          priority: "normal",
          proposed_by_actor_id: "local-owner",
          metadata_json: {
            created_from: "prediction_recommendation_outcome_review",
            target_type: targetType,
            target_id: targetId,
            outcome_id: outcome.id,
            outcome_status: outcomeStatus,
            auto_changed_future_behavior: false
          }
        });
        show("Outcome and improvement candidate recorded", "The outcome was saved and an improvement candidate was proposed for review. No future behavior was changed automatically.", improvement);
        return;
      }
      show("Outcome recorded", "The outcome was saved for review. No recommendation was executed and no future behavior was changed automatically.", outcome);
    } catch (error) {
      show("Outcome review failed", String(error));
    }
  });
})();
`;

  return (
    <section
      className="panel predictionRecommendationOutcomeReview"
      data-pr-outcome-review
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Review loop</p>
          <h2>Prediction / Recommendation Outcome Review</h2>
        </div>
        <StatusPill state={records.length > 0 ? "review-ready" : "no-records"} />
      </div>
      <div className="guidedManualNotice">
        <strong>Record outcomes explicitly.</strong>
        <span>Mark predictions and recommendations correct, wrong, partial, useful, not useful, or inconclusive. This does not execute recommendations, recalibrate a forecasting engine, or auto-change future recommendations.</span>
      </div>
      {[predictions.error, recommendations.error, feedback.error, outcomes.error, improvements.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some prediction, recommendation, feedback, outcome, or improvement records could not be loaded.</p>
      ) : null}
      <section className="metrics compact" aria-label="Prediction recommendation calibration summary">
        <article><span>Calibration status</span><strong>{calibrationSummary.data.calibration_status}</strong></article>
        <article><span>Records</span><strong>{calibrationSummary.data.record_counts.total}</strong></article>
        <article><span>Evidence-linked</span><strong>{calibrationSummary.data.record_counts.evidence_linked}</strong></article>
        <article><span>With outcomes</span><strong>{calibrationSummary.data.record_counts.with_outcome}</strong></article>
        <article><span>Correct/useful</span><strong>{calibrationSummary.data.outcome_counts.correct + calibrationSummary.data.outcome_counts.useful}</strong></article>
        <article><span>Wrong/not useful</span><strong>{calibrationSummary.data.outcome_counts.wrong + calibrationSummary.data.outcome_counts.not_useful}</strong></article>
      </section>
      {calibrationSummary.error ? (
        <p className="errorText">Calibration summary could not be loaded: {calibrationSummary.error}</p>
      ) : null}
      <div className="guidedManualNotice">
        <strong>Calibration is descriptive.</strong>
        <span>Summary counts come from persisted records and explicit outcomes. No forecasting engine is run, no recommendation is executed, and confidence bands are not advanced calibration statistics.</span>
      </div>
      <section className="split">
        <div>
          <div className="subHeader"><h3>Review Records</h3></div>
          <div className="stack">
            {reviewRecords.slice(0, 8).map((record) => (
              <article className="item evidenceItem" key={`${record.type}:${record.id}`}>
                <div>
                  <strong>{record.title}</strong>
                  <span>{record.detail || "No expected result recorded."}</span>
                  <span>{contextLabel(record.metadata)}</span>
                  <span>Evidence IDs: {record.evidenceIds.length > 0 ? record.evidenceIds.slice(0, 3).join(", ") : "none recorded"}</span>
                </div>
                <div>
                  <StatusPill state={record.type} />
                  <StatusPill state={record.status} />
                  <span>feedback {record.directFeedback.length} · outcomes {record.directOutcomes.length} · improvements {record.linkedImprovements.length}</span>
                </div>
              </article>
            ))}
          </div>
          {reviewRecords.length === 0 ? <EmptyState label="No prediction or recommendation records are available yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Existing Outcomes</h3></div>
          <div className="stack">
            {outcomes.data.filter((outcome) => outcome.target_type === "prediction" || outcome.target_type === "recommendation").slice(0, 8).map((outcome) => (
              <article className="item evidenceItem" key={outcome.id}>
                <div>
                  <strong>{outcome.outcome_status}</strong>
                  <span>{outcome.target_type} {outcome.target_id}</span>
                  <span>{outcome.summary ?? "No summary note recorded."}</span>
                </div>
                <div><StatusPill state="recorded" /><span>{formatDate(outcome.created_at)}</span></div>
              </article>
            ))}
          </div>
          {outcomes.data.filter((outcome) => outcome.target_type === "prediction" || outcome.target_type === "recommendation").length === 0 ? <EmptyState label="No prediction/recommendation outcomes recorded yet." /> : null}
        </div>
      </section>
      <form className="guidedManualForm" data-pr-outcome-form>
        <label>
          <span>Prediction/recommendation target</span>
          <select name="review_target" disabled={records.length === 0}>
            {records.map((record) => (
              <option key={`${record.type}:${record.id}`} value={record.id} data-target-type={record.type}>{record.type} · {record.title}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Outcome status</span>
          <select name="outcome_status" defaultValue="inconclusive">
            <option value="correct">correct</option>
            <option value="wrong">wrong</option>
            <option value="partial">partial</option>
            <option value="useful">useful</option>
            <option value="not_useful">not_useful</option>
            <option value="inconclusive">inconclusive</option>
          </select>
        </label>
        <label>
          <span>Evidence IDs</span>
          <input name="outcome_evidence_ids" defaultValue={firstEvidenceIds.join(",")} placeholder="Optional comma-separated evidence IDs." />
        </label>
        <label>
          <span>Summary note</span>
          <textarea name="outcome_summary" rows={2} placeholder="What happened, and what evidence supports the review?" />
        </label>
        <label className="checkLine">
          <input name="create_improvement_candidate" type="checkbox" /> If wrong, partial, not useful, or inconclusive, propose an improvement candidate for review.
        </label>
        <label>
          <span>Improvement objective</span>
          <textarea name="improvement_objective" rows={2} placeholder="Optional objective if creating an improvement candidate." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={records.length === 0}>Record outcome review</button>
          <span>{records.length > 0 ? "Uses existing outcome and improvement routes. No recommendation is executed." : "Create a prediction or recommendation record first."}</span>
        </div>
      </form>
      <div className="guidedManualResult" data-pr-outcome-result>
        <strong>{records.length > 0 ? "Review controls ready" : "No review target yet"}</strong>
        <span>Improvement candidates are proposed metadata only; IGY6 does not auto-change methods or recommendations.</span>
      </div>
      <ClientScript script={script} />
    </section>
  );
}

