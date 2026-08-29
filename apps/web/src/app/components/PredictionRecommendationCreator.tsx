import type { EvidenceItemRecord, EvidenceAnswerRecord, AgentTaskPlanRecord, ReportRecord, ApiResult } from "./types";
import { excerpt } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";

export function PredictionRecommendationCreator({
  evidenceItems,
  evidenceAnswers,
  reports,
  taskPlans
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
}) {
  const browserApiBaseUrl = "/api";
  const defaultEvidenceIds = evidenceItems.data.slice(0, 3).map((item) => item.id);
  const answerEvidenceIds = evidenceAnswers.data.flatMap((answer) => answer.evidence_item_ids ?? []).slice(0, 3);
  const suggestedEvidenceIds = defaultEvidenceIds.length > 0 ? defaultEvidenceIds : answerEvidenceIds;
  const contextOptions = [
    ...evidenceAnswers.data.slice(0, 4).map((answer) => ({
      value: `answer:${answer.id}`,
      label: `Answer: ${excerpt(answer.user_question, 80)}`
    })),
    ...reports.data.slice(0, 4).map((report) => ({
      value: `report:${report.id}`,
      label: `Report: ${report.title}`
    })),
    ...taskPlans.data.slice(0, 4).map((plan) => ({
      value: `task_plan:${plan.id}`,
      label: `Task: ${excerpt(plan.user_request_summary, 80)}`
    }))
  ];
  const script = `
(() => {
  const root = document.querySelector("[data-prediction-recommendation-creator]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const form = root.querySelector("[data-pr-create-form]");
  const result = root.querySelector("[data-pr-create-result]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const evidenceIds = () => value("evidence_ids").split(",").map((item) => item.trim()).filter(Boolean);
  const contextMetadata = () => {
    const context = value("context_link");
    const metadata = {
      created_from: "prediction_recommendation_creation_mvp",
      title: value("record_title") || null,
      uncertainty: value("uncertainty") || null,
      timeframe: value("timeframe") || null,
      disproof_criteria: value("disproof_criteria") || null,
      not_auto_executed: true,
      forecasting_engine_output: false,
      reviewable: true
    };
    if (context.includes(":")) {
      const parts = context.split(":");
      metadata.context_type = parts[0];
      metadata.context_id = parts.slice(1).join(":");
    }
    return metadata;
  };
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
      details.setAttribute("data-pr-create-status", "");
      [
        ["record", payload.id],
        ["status", payload.status],
        ["confidence", payload.confidence ?? "unknown"],
        ["evidence", Array.isArray(payload.evidence_ids) ? payload.evidence_ids.length : 0]
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
    const ids = evidenceIds();
    if (ids.length === 0) {
      show("Evidence required", "Add at least one existing evidence ID before creating a record.");
      return;
    }
    const confidence = Number(value("confidence"));
    const common = {
      evidence_ids: ids,
      confidence: Number.isFinite(confidence) ? confidence : undefined,
      status: value("review_status") || "proposed",
      actor_id: "local-owner",
      metadata_json: contextMetadata()
    };
    try {
      if (value("record_type") === "prediction") {
        if (!value("expected_result")) {
          show("Expected result required", "Prediction records need an expected result for later review.");
          return;
        }
        const payload = await postJson("/analysis/predictions", {
          ...common,
          prediction_text: value("record_summary") || value("record_title"),
          expected_result: value("expected_result"),
          disproof_condition: value("disproof_criteria") || null
        });
        show("Prediction created", "Prediction record was saved for later review and outcome tracking. It was not executed or treated as guaranteed truth.", payload);
        return;
      }
      const payload = await postJson("/analysis/recommendations", {
        ...common,
        recommendation_text: value("record_summary") || value("record_title"),
        risk_level: value("risk_level") || "unknown",
        approval_required: checked("approval_required"),
        expected_result: value("expected_result") || null
      });
      show("Recommendation created", "Recommendation record was saved for later review and outcome tracking. It was not executed automatically.", payload);
    } catch (error) {
      show("Record create failed", String(error));
    }
  });
})();
`;

  return (
    <section
      className="panel predictionRecommendationCreator"
      data-prediction-recommendation-creator
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Reviewable records</p>
          <h2>Prediction / Recommendation Creation</h2>
        </div>
        <StatusPill state={suggestedEvidenceIds.length > 0 ? "evidence-linked" : "evidence-required"} />
      </div>
      <div className="guidedManualNotice">
        <strong>Evidence-linked and reviewable.</strong>
        <span>These records are owner-created review records. They are not automatic execution, guaranteed truth, forecasting engine output, or autonomous reasoning.</span>
      </div>
      <form className="guidedManualForm" data-pr-create-form>
        <label>
          <span>Record type</span>
          <select name="record_type" defaultValue="prediction">
            <option value="prediction">prediction</option>
            <option value="recommendation">recommendation</option>
          </select>
        </label>
        <label>
          <span>Title</span>
          <input name="record_title" placeholder="Short title for review." />
        </label>
        <label>
          <span>Summary</span>
          <textarea name="record_summary" rows={2} placeholder="What is expected or recommended, bounded by cited evidence?" />
        </label>
        <label>
          <span>Evidence IDs</span>
          <input name="evidence_ids" defaultValue={suggestedEvidenceIds.join(",")} placeholder="Required comma-separated existing evidence ids." />
        </label>
        <label>
          <span>Context link</span>
          <select name="context_link" defaultValue="">
            <option value="">No answer/report/task context selected</option>
            {contextOptions.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Confidence</span>
          <input name="confidence" type="number" min="0" max="100" defaultValue="50" />
        </label>
        <label>
          <span>Uncertainty</span>
          <textarea name="uncertainty" rows={2} placeholder="What could make this wrong or incomplete?" />
        </label>
        <label>
          <span>Expected result</span>
          <textarea name="expected_result" rows={2} placeholder="What outcome should be reviewed later?" />
        </label>
        <label>
          <span>Disproof criteria</span>
          <textarea name="disproof_criteria" rows={2} placeholder="What evidence or outcome would disprove this?" />
        </label>
        <label>
          <span>Timeframe if known</span>
          <input name="timeframe" placeholder="Example: review after next billing cycle." />
        </label>
        <label>
          <span>Review status</span>
          <select name="review_status" defaultValue="proposed">
            <option value="proposed">proposed</option>
            <option value="open">open</option>
            <option value="needs_review">needs_review</option>
          </select>
        </label>
        <label>
          <span>Recommendation risk level</span>
          <select name="risk_level" defaultValue="unknown">
            <option value="unknown">unknown</option>
            <option value="low">low</option>
            <option value="medium">medium</option>
            <option value="high">high</option>
          </select>
        </label>
        <label className="checkLine">
          <input name="approval_required" type="checkbox" defaultChecked /> Recommendation requires approval before any future action.
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={suggestedEvidenceIds.length === 0}>Create review record</button>
          <span>{suggestedEvidenceIds.length > 0 ? "Creates a persisted prediction or recommendation record linked to existing evidence." : "Process evidence before creating a prediction or recommendation record."}</span>
        </div>
      </form>
      <div className="guidedManualResult" data-pr-create-result>
        <strong>{suggestedEvidenceIds.length > 0 ? "Ready for evidence-linked creation" : "Evidence required"}</strong>
        <span>Records are reviewable and outcome-trackable. Recommendations are not executed by this form.</span>
      </div>
      <ClientScript script={script} />
    </section>
  );
}
