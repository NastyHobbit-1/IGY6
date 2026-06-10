import type { WorkItemRecord, CollectionRunRecord, ApiResult, VectorCollectionStatus } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { workActivityLabel, workStatusPlain, searchIndexStatus } from "./product/helpers";
import { StatusPill } from "./ui/StatusPill";

export function PipelineOperationsPanel({
  workItems,
  collectionRuns,
  vectorCollection,
}: {
  workItems: ApiResult<WorkItemRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  vectorCollection: ApiResult<VectorCollectionStatus>;
}) {
  const recentSummaries = collectionRuns.data
    .filter((run) => run.summary_json && typeof run.summary_json === "object")
    .slice(0, 4);
  const actionableWorkItems = workItems.data
    .filter((item) =>
      item.status === "failed"
      || (item.status === "pending_intent_verification" && item.work_type === "collection_normalization"))
    .slice(0, 8);
  const searchIndex = searchIndexStatus(vectorCollection.data);
  const queuedJson = JSON.stringify(
    actionableWorkItems.map((item) => ({ id: item.id, work_type: item.work_type, status: item.status })),
  ).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-pipeline-operations]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const queued = JSON.parse(root.querySelector("[data-pipeline-queued-json]")?.textContent || "[]");
  const result = root.querySelector("[data-pipeline-ops-result]");
  const show = (message) => { if (result) result.textContent = message; };
  const postJson = async (path, body) => {
    const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body || {}) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  const autoQueueLegacy = async () => {
    const legacy = queued.filter((item) => item.status === "pending_intent_verification" && item.work_type === "collection_normalization");
    if (legacy.length === 0) return;
    const queueKey = "igy6-auto-queued-" + legacy.map((item) => item.id).sort().join(",");
    try {
      if (window.sessionStorage.getItem(queueKey) === "done") return;
    } catch {
      /* ignore storage errors */
    }
    show("Preparing " + legacy.length + " older collection job(s)...");
    for (const item of legacy) {
      try {
        await postJson("/api/work-items/" + encodeURIComponent(item.id) + "/status", { status: "queued", actor_id: "local-owner" });
      } catch (error) {
        show(String(error));
        return;
      }
    }
    try {
      window.sessionStorage.setItem(queueKey, "done");
    } catch {
      /* ignore storage errors */
    }
    show("Queued " + legacy.length + " job(s). Processing continues in the background — refresh when you want updated status.");
  };
  root.querySelector("[data-vector-ensure]")?.addEventListener("click", async () => {
    show("Preparing search...");
    try {
      await postJson("/api/memory/vector/chunks/ensure", {});
      show("Search is ready. Refresh the page if the status line still looks stale.");
    } catch (e) {
      show(String(e));
    }
  });
  root.querySelector("[data-retrieval-search]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const message = root.querySelector("[name='pipeline_search_message']")?.value?.trim() || "";
    const limit = Number(root.querySelector("[name='pipeline_search_limit']")?.value || 5);
    if (!message) return;
    show("Searching...");
    try { show("What IGY6 found: " + JSON.stringify(await postJson("/api/chat/retrieval-preview", { message, limit }))); } catch (e) { show(String(e)); }
  });
  root.querySelectorAll("[data-retry-failed-work]").forEach((button) => {
    button.addEventListener("click", async () => {
      const workItemId = button.getAttribute("data-work-item-id");
      if (!workItemId) return;
      button.disabled = true;
      show("Retrying work...");
      try {
        await postJson("/api/work-items/" + encodeURIComponent(workItemId) + "/status", { status: "queued", actor_id: "local-owner" });
        show("Work re-queued. Refresh the page when you want updated activity cards.");
      } catch (e) {
        show(String(e));
      } finally {
        button.disabled = false;
      }
    });
  });
  void autoQueueLegacy();
})();
`;
  return (
    <section className="panelInset pipelineOperations uiOverflowSafe" id="data-search" data-pipeline-operations>
      <div className="subHeader"><h3>Processing status</h3></div>
      <p className="actionHint">
        New information is prepared automatically: collected items are normalized, broken into searchable pieces, and indexed in the background.
      </p>
      <div className="guidedManualActions responsiveToolbar productAdvanced" data-ui-full-only>
        <button type="button" data-vector-ensure>Prepare search index</button>
        <span className="actionHint uiBreakLong">
          Search: {searchIndex.label}
        </span>
      </div>
      <div className="productCard uiOverflowSafe">
        <span>Search</span>
        <strong data-search-status-label>{searchIndex.label}</strong>
        <StatusPill state={searchIndex.state} />
        {searchIndex.state !== "ok" ? (
          <button type="button" className="productGhostButton" data-vector-ensure>
            {vectorCollection.data.detail?.tcp_reachable ? "Repair search" : "Check System first"}
          </button>
        ) : null}
      </div>
      <form data-retrieval-search className="productAdvanced">
        <label><span>Try a search</span><input name="pipeline_search_message" placeholder="What did I upload today?" /></label>
        <label><span>Result limit</span><input name="pipeline_search_limit" type="number" min="1" max="20" defaultValue="5" /></label>
        <button type="submit">Preview what IGY6 found</button>
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
                  <button type="button" data-retry-failed-work data-work-item-id={item.id}>Retry</button>
                ) : null}
              </article>
            );
          })}
        </div>
      ) : <p className="actionHint">Processing is running automatically. No manual steps are required right now.</p>}
      <p data-pipeline-ops-result className="productHint">Ready.</p>
      <DomJsonScript marker="data-pipeline-queued-json" json={queuedJson} />
      <ClientScript script={script} />
    </section>
  );
}