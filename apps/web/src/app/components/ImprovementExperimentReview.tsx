import type { ImprovementRecord, ExperimentRecord, ApiResult } from "./types";
import { formatDate, excerpt } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";
import { HelpHeading } from "./ui/HelpHeading";

export function ImprovementExperimentReview({
  improvements,
  experiments
}: {
  improvements: ApiResult<ImprovementRecord[]>;
  experiments: ApiResult<ExperimentRecord[]>;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const recentImprovements = improvements.data.slice(0, 6);
  const recentExperiments = experiments.data.slice(0, 6);
  const script = `
(() => {
  const root = document.querySelector("[data-improvement-experiment-review]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const result = root.querySelector("[data-experiment-proposal-result]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const lines = (name) => value(name).split("\\n").map((item) => item.trim()).filter(Boolean);
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
      details.setAttribute("data-experiment-proposal-status", "");
      [
        ["experiment", payload.id],
        ["status", payload.status],
        ["improvement", payload.improvement_item_id || "not linked"],
        ["execution", "metadata only"]
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
  root.querySelectorAll("[data-experiment-status-button]").forEach((button) => {
    button.addEventListener("click", async () => {
      const experimentId = button.getAttribute("data-experiment-id");
      const status = button.getAttribute("data-experiment-status");
      if (!experimentId || !status) return;
      button.disabled = true;
      try {
        const payload = await postJson("/experiments/" + encodeURIComponent(experimentId) + "/status", {
          status,
          actor_id: "local-owner",
          status_note: "Updated from Improvement and experiment review panel"
        });
        show("Experiment status updated", "Status changed to " + status + ".", payload);
      } catch (error) {
        show("Experiment status failed", String(error));
      } finally {
        button.disabled = false;
      }
    });
  });
  root.querySelector("[data-experiment-proposal-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const improvementId = value("experiment_improvement_id");
    if (!improvementId) {
      show("No improvement selected", "Create or select an improvement item before proposing experiment metadata.");
      return;
    }
    try {
      const payload = await postJson("/experiments/propose-from-improvement", {
        improvement_item_id: improvementId,
        proposal_scope: value("experiment_scope"),
        success_criteria: lines("experiment_success_criteria"),
        dry_run_summary: value("experiment_dry_run_summary"),
        result_comparison_plan: value("experiment_result_comparison_plan"),
        actor_id: "local-owner"
      });
      show("Experiment proposal recorded", "IGY6 created controlled proposal and dry-run metadata only. No runner, MLflow/Optuna execution, or production method change ran.", payload);
    } catch (error) {
      show("Experiment proposal failed", String(error));
    }
  });
})();
`;

  return (
    <section
      className="guidedManualText improvementExperimentReview"
      data-improvement-experiment-review
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="guidedManualNotice">
        <strong>Improvement and experiment review</strong>
        <span>Review proposed improvements and planned experiment metadata. Accepted methods require approval; no autonomous method changes, MLflow/Optuna run, or Phoenix trace workflow is triggered here.</span>
      </div>
      {[improvements.error, experiments.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some improvement or experiment endpoints returned errors.</p>
      ) : null}
      <section className="split">
        <div>
          <div className="subHeader"><h3><HelpHeading term="improvementItem">Improvement Items</HelpHeading></h3></div>
          <div className="stack">
            {recentImprovements.map((item) => (
              <article className="item evidenceItem" key={item.id} data-improvement-review-item>
                <div>
                  <strong>{item.target_area}</strong>
                  <span>{excerpt(item.objective, 140)}</span>
                  <span>proposed by {item.proposed_by_actor_id}</span>
                </div>
                <div>
                  <StatusPill state={item.status} />
                  <span>{item.priority}</span>
                </div>
              </article>
            ))}
          </div>
          {recentImprovements.length === 0 ? <EmptyState label="No improvement items recorded yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3><HelpHeading term="experimentRun">Experiment Records</HelpHeading></h3></div>
          <div className="stack">
            {recentExperiments.map((experiment) => (
              <article className="item evidenceItem" key={experiment.id} data-experiment-review-item>
                <div>
                  <strong>{experiment.status}</strong>
                  <span>Improvement: {experiment.improvement_item_id ?? "not linked"}</span>
                  <span>MLflow: {experiment.mlflow_run_id ?? "not executed"}</span>
                  <span>Optuna: {experiment.optuna_study_name ?? "not executed"}</span>
                </div>
                <div>
                  <StatusPill state={experiment.status} />
                  <span>{formatDate(experiment.created_at)}</span>
                  {experiment.status === "planned" || experiment.status === "running" ? (
                    <div className="guidedManualActions">
                      {experiment.status === "planned" ? <button type="button" data-experiment-status-button data-experiment-id={experiment.id} data-experiment-status="running">Mark running</button> : null}
                      <button type="button" data-experiment-status-button data-experiment-id={experiment.id} data-experiment-status="completed">Mark completed</button>
                      <button type="button" data-experiment-status-button data-experiment-id={experiment.id} data-experiment-status="abandoned">Abandon</button>
                    </div>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
          {recentExperiments.length === 0 ? <EmptyState label="No experiment records recorded yet." /> : null}
        </div>
      </section>
      <form className="guidedManualForm" data-experiment-proposal-form>
        <label>
          <span>Improvement item</span>
          <select name="experiment_improvement_id" disabled={recentImprovements.length === 0}>
            {recentImprovements.map((item) => (
              <option key={item.id} value={item.id}>{item.target_area} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Review scope</span>
          <textarea name="experiment_scope" rows={2} required placeholder="Describe the bounded comparison or review question. No runner starts from this form." />
        </label>
        <label>
          <span>Success criteria</span>
          <textarea name="experiment_success_criteria" rows={3} required placeholder="One criterion per line, for example: fewer incomplete evidence answers after manual review" />
        </label>
        <label>
          <span>Dry-run summary</span>
          <textarea name="experiment_dry_run_summary" rows={2} required placeholder="What would be checked before execution, and what remains not run?" />
        </label>
        <label>
          <span>Result comparison plan</span>
          <textarea name="experiment_result_comparison_plan" rows={2} required placeholder="How results would be compared after an approved run." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={recentImprovements.length === 0}>Record proposal / dry-run metadata</button>
          <span>{recentImprovements.length > 0 ? "Creates a planned experiment proposal only; accepted methods remain approval-gated." : "Create or receive an improvement item before proposing an experiment record."}</span>
        </div>
      </form>
      <div className="guidedManualResult" data-experiment-proposal-result>
        <strong>{recentExperiments.length > 0 ? "Experiment metadata exists" : "No experiment proposal selected"}</strong>
        <span>{recentExperiments.length > 0 ? "Recent records are listed above for review." : "Use this only to record metadata for later review."}</span>
      </div>
      <ClientScript script={script} />
    </section>
  );
}

