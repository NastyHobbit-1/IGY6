import type { EvidenceItemRecord, EvidenceAnswerRecord, WorkItemRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ReportRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function EvidenceFeedbackWorkflow({
  evidenceItems,
  evidenceAnswers,
  reports,
  workItems,
  feedback,
  outcomes,
  improvements
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
}) {
  const browserApiBaseUrl = "/api";
  const feedbackTargets = [
    ...evidenceItems.data.slice(0, 6).map((item) => ({
      type: "evidence_item",
      id: item.id,
      label: `Evidence item ${item.id}`
    })),
    ...evidenceAnswers.data.slice(0, 4).map((answer) => ({
      type: "evidence_answer",
      id: answer.id,
      label: `Answer record ${answer.id}`
    })),
    ...reports.data.slice(0, 3).map((report) => ({
      type: "report",
      id: report.id,
      label: `Report ${report.id}`
    })),
    ...workItems.data.slice(0, 3).map((workItem) => ({
      type: "work_item",
      id: workItem.id,
      label: `Work item ${workItem.id}`
    }))
  ];
  const outcomeTargets = [
    ...reports.data.slice(0, 4).map((report) => ({
      type: "report",
      id: report.id,
      label: `Report ${report.id}`
    })),
    ...workItems.data.slice(0, 4).map((workItem) => ({
      type: "work_item",
      id: workItem.id,
      label: `Work item ${workItem.id}`
    }))
  ];
  const defaultEvidenceId = evidenceItems.data[0]?.id ?? "";
  const improvementFeedbackLabels = new Set(["wrong", "not_useful", "incomplete", "rejected"]);
  const improvementOutcomeStatuses = new Set(["wrong", "not_useful", "partial", "inconclusive"]);
  const feedbackSignals = feedback.data.filter((event) => event.target_type !== "source" && improvementFeedbackLabels.has(event.label));
  const outcomeSignals = outcomes.data.filter((outcome) => improvementOutcomeStatuses.has(outcome.outcome_status));
  const reviewSignals = [
    ...feedbackSignals.slice(0, 6).map((event) => ({
      kind: "feedback",
      id: event.id,
      targetType: event.target_type,
      targetId: event.target_id,
      label: event.label,
      note: event.note ?? "",
      existingImprovement: improvements.data.find((item) => item.metadata_json?.feedback_id === event.id)
    })),
    ...outcomeSignals.slice(0, 6).map((outcome) => ({
      kind: "outcome",
      id: outcome.id,
      targetType: outcome.target_type,
      targetId: outcome.target_id,
      label: outcome.outcome_status,
      note: outcome.summary ?? "",
      existingImprovement: improvements.data.find((item) => item.metadata_json?.outcome_id === outcome.id)
    }))
  ];
  const script = `
(() => {
  const root = document.querySelector("[data-evidence-feedback-workflow]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
	  const result = root.querySelector("[data-evidence-feedback-result]");
	  const improvementResult = root.querySelector("[data-improvement-review-result]");
	  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
	  const selected = (name) => {
    const option = root.querySelector("[name='" + name + "']")?.selectedOptions?.[0];
    return {
      id: option?.value || "",
      type: option?.getAttribute("data-target-type") || ""
	    };
	  };
	  const selectedSignal = () => {
	    const option = root.querySelector("[name='improvement_signal']")?.selectedOptions?.[0];
	    return {
	      id: option?.value || "",
	      kind: option?.getAttribute("data-signal-kind") || "",
	      targetType: option?.getAttribute("data-target-type") || "",
	      targetId: option?.getAttribute("data-target-id") || "",
	      label: option?.getAttribute("data-signal-label") || ""
	    };
	  };
	  const targetAreaFor = (targetType) => {
	    if (targetType === "document") return "parsing";
	    if (targetType === "evidence_item") return "retrieval";
	    if (targetType === "prediction") return "prediction";
	    if (targetType === "report") return "reporting";
	    if (targetType === "work_item") return "safety";
	    return "reasoning";
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
      details.setAttribute("data-feedback-outcome-status", "");
      [
        ["record", payload.id],
        ["target", (payload.target_type || "") + " " + (payload.target_id || "")],
        ["label", payload.label || payload.outcome_status || "recorded"]
      ].forEach(([label, detail]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = detail || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
	    }
	  };
	  const showImprovement = (state, message, payload) => {
	    if (!improvementResult) return;
	    improvementResult.innerHTML = "";
	    const title = document.createElement("strong");
	    title.textContent = state;
	    const body = document.createElement("span");
	    body.textContent = message;
	    improvementResult.append(title, body);
	    if (payload) {
	      const details = document.createElement("dl");
	      details.setAttribute("data-improvement-review-status", "");
	      [
	        ["improvement", payload.id],
	        ["target area", payload.target_area],
	        ["status", payload.status],
	        ["priority", payload.priority]
	      ].forEach(([label, detail]) => {
	        const term = document.createElement("dt");
	        term.textContent = label;
	        const description = document.createElement("dd");
	        description.textContent = detail || "not returned";
	        details.append(term, description);
	      });
	      improvementResult.appendChild(details);
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
  root.querySelector("[data-feedback-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const target = selected("feedback_target");
    try {
      const payload = await postJson("/feedback", {
        target_type: target.type,
        target_id: target.id,
        label: value("feedback_label"),
        note: value("feedback_note") || null,
        metadata_json: { created_from: "results_feedback_outcome_capture" }
      });
      show("Feedback recorded", "IGY6 persisted the review feedback.", payload);
    } catch (error) {
      show("Feedback failed", String(error));
    }
  });
	  root.querySelector("[data-outcome-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const target = selected("outcome_target");
    const evidenceIds = value("outcome_evidence_ids")
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
    try {
      const payload = await postJson("/outcomes", {
        target_type: target.type,
        target_id: target.id,
        outcome_status: value("outcome_status"),
        summary: value("outcome_summary") || null,
        evidence_ids: evidenceIds,
        metadata_json: { created_from: "results_feedback_outcome_capture" }
      });
      show("Outcome recorded", "IGY6 persisted the outcome and updated the supported target.", payload);
    } catch (error) {
	      show("Outcome failed", String(error));
	    }
	  });
	  root.querySelector("[data-improvement-review-form]")?.addEventListener("submit", async (event) => {
	    event.preventDefault();
	    const signal = selectedSignal();
	    if (!signal.id) {
	      showImprovement("No review signal", "Record weak feedback or an outcome before creating an improvement proposal.");
	      return;
	    }
	    try {
	      const payload = await postJson("/improvements", {
	        target_area: targetAreaFor(signal.targetType),
	        objective: value("improvement_objective") || ("Review " + signal.kind + " " + signal.label + " for " + signal.targetType + " " + signal.targetId + "."),
	        priority: value("improvement_priority") || "normal",
	        proposed_by_actor_id: "local-owner",
	        metadata_json: {
	          created_from: "results_improvement_review",
	          signal_kind: signal.kind,
	          signal_label: signal.label,
	          target_type: signal.targetType,
	          target_id: signal.targetId,
	          feedback_id: signal.kind === "feedback" ? signal.id : null,
	          outcome_id: signal.kind === "outcome" ? signal.id : null
	        }
	      });
	      showImprovement("Improvement proposed", "IGY6 created review metadata only. No method changed and no experiment ran.", payload);
	    } catch (error) {
	      showImprovement("Improvement proposal failed", String(error));
	    }
	  });
	})();
	`;

  return (
    <section
      className="guidedManualText"
      data-evidence-feedback-workflow
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="guidedManualNotice">
        <strong>Review outcome capture</strong>
        <span>
          Record feedback on retrieved evidence, saved answer records, or a supported report/work item outcome. Outcomes for answer records are not supported by the current outcome API.
        </span>
      </div>
      <form className="guidedManualForm" data-feedback-form>
        <label>
          <span>Feedback target</span>
          <select name="feedback_target" disabled={feedbackTargets.length === 0}>
            {feedbackTargets.map((target) => (
              <option key={`${target.type}:${target.id}`} value={target.id} data-target-type={target.type}>{target.label}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Feedback label</span>
          <select name="feedback_label" defaultValue="useful">
            <option value="useful">useful</option>
            <option value="verified">verified</option>
            <option value="incomplete">incomplete</option>
            <option value="wrong">wrong</option>
            <option value="not_useful">not_useful</option>
          </select>
        </label>
        <label>
          <span>Feedback note</span>
          <textarea name="feedback_note" rows={2} placeholder="Optional review note." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={feedbackTargets.length === 0}>Record feedback</button>
          <span>{feedbackTargets.length > 0 ? "Targets come from current evidence, saved answer records, reports, and work items." : "No supported feedback target is available yet."}</span>
        </div>
      </form>
      <form className="guidedManualForm" data-outcome-form>
        <label>
          <span>Outcome target</span>
          <select name="outcome_target" disabled={outcomeTargets.length === 0}>
            {outcomeTargets.map((target) => (
              <option key={`${target.type}:${target.id}`} value={target.id} data-target-type={target.type}>{target.label}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Outcome status</span>
          <select name="outcome_status" defaultValue="useful">
            <option value="useful">useful</option>
            <option value="correct">correct</option>
            <option value="partial">partial</option>
            <option value="wrong">wrong</option>
            <option value="not_useful">not_useful</option>
            <option value="inconclusive">inconclusive</option>
          </select>
        </label>
        <label>
          <span>Evidence ids</span>
          <input name="outcome_evidence_ids" defaultValue={defaultEvidenceId} placeholder="Optional comma-separated evidence ids." />
        </label>
        <label>
          <span>Outcome summary</span>
          <textarea name="outcome_summary" rows={2} placeholder="Optional outcome summary." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={outcomeTargets.length === 0}>Record outcome</button>
          <span>{outcomeTargets.length > 0 ? "Outcomes are only offered for API-supported targets." : "No supported report or work item target is available yet."}</span>
        </div>
      </form>
	      <div className="guidedManualResult" data-evidence-feedback-result>
	        <strong>{feedback.data.length + outcomes.data.length > 0 ? "Review records exist" : "No review record selected"}</strong>
	        <span>{feedback.data.length + outcomes.data.length > 0 ? "Recent feedback and outcomes remain visible in Safety & Audit." : "Record feedback after reviewing retrieved evidence, or record an outcome when a supported target exists."}</span>
	      </div>
	      <section className="improvementReview" data-improvement-review>
	        <div className="guidedManualNotice">
	          <strong>Improvement review</strong>
	          <span>Weak feedback and unresolved outcomes can become proposed improvement items. This is review metadata only; IGY6 does not change methods or run experiments here.</span>
	        </div>
	        <div className="stack">
	          {reviewSignals.slice(0, 6).map((signal) => (
	            <article className="item evidenceItem" key={`${signal.kind}:${signal.id}`} data-improvement-signal>
	              <div>
	                <strong>{signal.kind} · {signal.label}</strong>
	                <span>{signal.targetType} {signal.targetId}</span>
	                <span>{signal.note || "No note recorded."}</span>
	              </div>
	              <div>
	                <StatusPill state={signal.existingImprovement ? "improvement-exists" : "needs-review"} />
	                <span>{signal.existingImprovement?.id ?? "No linked improvement item yet"}</span>
	              </div>
	            </article>
	          ))}
	        </div>
	        {reviewSignals.length === 0 ? <EmptyState label="No weak feedback or unresolved outcome signals are available yet." /> : null}
	        <form className="guidedManualForm" data-improvement-review-form>
	          <label>
	            <span>Review signal</span>
	            <select name="improvement_signal" disabled={reviewSignals.length === 0}>
	              {reviewSignals.map((signal) => (
	                <option
	                  key={`${signal.kind}:${signal.id}`}
	                  value={signal.id}
	                  data-signal-kind={signal.kind}
	                  data-target-type={signal.targetType}
	                  data-target-id={signal.targetId}
	                  data-signal-label={signal.label}
	                >
	                  {signal.kind} {signal.label} · {signal.targetType} {signal.targetId}
	                </option>
	              ))}
	            </select>
	          </label>
	          <label>
	            <span>Improvement objective</span>
	            <textarea name="improvement_objective" rows={2} placeholder="Review why this feedback/outcome was weak and define what should improve." />
	          </label>
	          <label>
	            <span>Priority</span>
	            <select name="improvement_priority" defaultValue="normal">
	              <option value="low">low</option>
	              <option value="normal">normal</option>
	              <option value="high">high</option>
	              <option value="urgent">urgent</option>
	            </select>
	          </label>
	          <div className="guidedManualActions">
	            <button type="submit" disabled={reviewSignals.length === 0}>Propose improvement item</button>
	            <span>{reviewSignals.length > 0 ? "Uses existing /improvements persistence." : "Record a weak signal before proposing improvement work."}</span>
	          </div>
	        </form>
	        <div className="guidedManualResult" data-improvement-review-result>
	          <strong>{improvements.data.length > 0 ? "Improvement items exist" : "No improvement item selected"}</strong>
	          <span>{improvements.data.length > 0 ? "Existing improvement records are listed in Method Review." : "Create a proposal only from a real review signal."}</span>
	        </div>
	      </section>
	      <ClientScript script={script} />
	    </section>
  );
}
