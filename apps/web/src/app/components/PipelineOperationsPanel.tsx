import type { WorkItemRecord, CollectionRunRecord, ApiResult, VectorCollectionStatus } from "./types";
import { ClientScript } from "@/lib/use-dom-script";
import { workActivityLabel, workStatusPlain } from "./product/helpers";
import { ActionButton } from "./ui/ActionButton";

export function PipelineOperationsPanel({
  workItems,
  collectionRuns,
}: {
  workItems: ApiResult<WorkItemRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  vectorCollection?: ApiResult<VectorCollectionStatus>;
}) {
  const recentSummaries = collectionRuns.data
    .filter((run) => run.summary_json && typeof run.summary_json === "object")
    .slice(0, 4);
  const actionableWorkItems = workItems.data
    .filter((item) =>
      item.status === "failed"
      || (item.status === "pending_intent_verification" && item.work_type === "collection_normalization"))
    .slice(0, 8);

  const script = `
(() => {
  const root = document.querySelector("[data-pipeline-operations]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const result = root.querySelector("[data-pipeline-ops-result]");
  const show = (message) => { if (result) result.textContent = message; };
  const formatError = (error) => {
    const raw = error instanceof Error ? error.message : String(error);
    try {
      const payload = JSON.parse(raw);
      if (typeof payload?.detail === "string") return payload.detail;
    } catch {
      /* not JSON */
    }
    return raw;
  };
  const postJson = async (path, body) => {
    const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body || {}) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  const queueWorkItem = async (workItemId, button, successMessage) => {
    if (!workItemId) return;
    if (button) button.disabled = true;
    show("Queueing work...");
    try {
      await postJson("/api/work-items/" + encodeURIComponent(workItemId) + "/status", { status: "queued", actor_id: "local-owner" });
      show(successMessage);
      if (button) button.textContent = "Queued";
    } catch (error) {
      show(formatError(error));
      if (button) button.disabled = false;
    }
  };
  root.querySelectorAll("[data-retry-failed-work]").forEach((button) => {
    button.addEventListener("click", () => {
      queueWorkItem(
        button.getAttribute("data-work-item-id"),
        button,
        "Work re-queued. Refresh the page when you want updated activity cards.",
      );
    });
  });
  root.querySelectorAll("[data-approve-pending-work]").forEach((button) => {
    button.addEventListener("click", () => {
      queueWorkItem(
        button.getAttribute("data-work-item-id"),
        button,
        "Work approved and queued. Processing continues in the background — refresh when you want updated status.",
      );
    });
  });
  root.querySelector("[data-retrieval-search]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const message = root.querySelector("[name='pipeline_search_message']")?.value?.trim() || "";
    const limit = Number(root.querySelector("[name='pipeline_search_limit']")?.value || 5);
    if (!message) return;
    show("Searching...");
    try {
      const payload = await postJson("/api/chat/retrieval-preview", { message, limit });
      show("What IGY6 found: " + JSON.stringify(payload));
    } catch (error) {
      show(formatError(error));
    }
  });
})();
`;

  return (
    <section className="panelInset pipelineOperations uiOverflowSafe" id="data-search" data-pipeline-operations>
      <div className="subHeader"><h3>Processing status</h3></div>
      <p className="actionHint">
        New information is prepared automatically. Search memory is prepared in the background — open System if search needs attention.
      </p>
      <form data-retrieval-search className="productAdvanced" data-ui-full-only>
        <label><span>Try a search</span><input name="pipeline_search_message" placeholder="What did I upload today?" /></label>
        <label><span>Result limit</span><input name="pipeline_search_limit" type="number" min="1" max="20" defaultValue="5" /></label>
        <button type="submit">What IGY6 found</button>
      </form>
      {recentSummaries.length > 0 ? (
        <div className="stack">
          <p className="actionHint">Recent ingest summaries:</p>
          {recentSummaries.map((run) => {
            const summary = run.summary_json ?? {};
            const aiSummary = typeof summary.ai_review_summary === "string"
              ? summary.ai_review_summary
              : summary.ai_review_summary_pending
                ? "Summary is still being prepared."
                : null;
            return (
              <article className="item evidenceItem uiOverflowSafe" key={run.id}>
                <div>
                  <strong>{run.status}</strong>
                  <span className="uiBreakLong productAdvanced">{run.id}</span>
                  {aiSummary ? <span className="uiClampSummary">{aiSummary}</span> : null}
                </div>
              </article>
            );
          })}
        </div>
      ) : null}
      {actionableWorkItems.length > 0 ? (
        <div className="stack">
          {actionableWorkItems.map((item) => {
            const status = workStatusPlain(item);
            return (
              <article className="item evidenceItem uiOverflowSafe" key={item.id}>
                <div>
                  <strong className="uiClampTitle">{workActivityLabel(item)}</strong>
                  <span className="uiClampSummary">{status.next}</span>
                  <span className="productAdvanced uiBreakLong">{item.id}</span>
                </div>
                {item.status === "failed" ? (
                  <ActionButton variant="primary" data-retry-failed-work data-work-item-id={item.id}>
                    Retry
                  </ActionButton>
                ) : item.status === "pending_intent_verification" ? (
                  <ActionButton variant="primary" data-approve-pending-work data-work-item-id={item.id}>
                    Approve and queue
                  </ActionButton>
                ) : null}
              </article>
            );
          })}
        </div>
      ) : (
        <p className="actionHint">Processing is running automatically. No manual steps are required right now.</p>
      )}
      <p data-pipeline-ops-result className="productHint">Ready.</p>
      <ClientScript script={script} />
    </section>
  );
}