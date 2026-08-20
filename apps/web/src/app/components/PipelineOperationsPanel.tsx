import type { WorkItemRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function PipelineOperationsPanel({ workItems }: { workItems: ApiResult<WorkItemRecord[]> }) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const queuedWorkItems = workItems.data.filter((item) => item.status === "queued" || item.status === "pending_intent_verification").slice(0, 8);
  const queuedJson = JSON.stringify(queuedWorkItems.map((item) => ({ id: item.id, work_type: item.work_type, status: item.status }))).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-pipeline-operations]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const queued = JSON.parse(root.querySelector("[data-pipeline-queued-json]")?.textContent || "[]");
  const result = root.querySelector("[data-pipeline-ops-result]");
  const show = (message) => { if (result) result.textContent = message; };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body || {}) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  root.querySelector("[data-vector-ensure]")?.addEventListener("click", async () => {
    show("Ensuring vector chunks...");
    try { show("Vector ensure: " + JSON.stringify(await postJson("/memory/vector/chunks/ensure", {}))); } catch (e) { show(String(e)); }
  });
  root.querySelector("[data-retrieval-search]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const message = root.querySelector("[name='pipeline_search_message']")?.value?.trim() || "";
    const limit = Number(root.querySelector("[name='pipeline_search_limit']")?.value || 5);
    if (!message) return;
    show("Searching...");
    try { show("Retrieval: " + JSON.stringify(await postJson("/chat/retrieval-preview", { message, limit }))); } catch (e) { show(String(e)); }
  });
  root.querySelectorAll("[data-dispatch-work-item]").forEach((button) => {
    button.addEventListener("click", async () => {
      const workItemId = button.getAttribute("data-work-item-id");
      if (!workItemId) return;
      button.disabled = true;
      show("Dispatching " + workItemId + "...");
      try {
        show("Dispatch: " + JSON.stringify(await postJson("/work-items/" + encodeURIComponent(workItemId) + "/dispatch", {})));
      } catch (e) {
        show(String(e));
      } finally {
        button.disabled = false;
      }
    });
  });
})();
`;
  return (
    <section className="panelInset pipelineOperations" id="data-search" data-pipeline-operations data-api-base-url={browserApiBaseUrl}>
      <div className="subHeader"><h3>Pipeline operations</h3></div>
      <p className="actionHint">Run memory, search, and work dispatch actions without opening Advanced.</p>
      <div className="guidedManualActions">
        <button type="button" data-vector-ensure>Ensure vector chunks</button>
      </div>
      <form data-retrieval-search>
        <label><span>Search your data</span><input name="pipeline_search_message" placeholder="What did I upload today?" /></label>
        <label><span>Result limit</span><input name="pipeline_search_limit" type="number" min="1" max="20" defaultValue="5" /></label>
        <button type="submit">Run retrieval preview</button>
      </form>
      {queuedWorkItems.length > 0 ? (
        <div className="stack">
          {queuedWorkItems.map((item) => (
            <article className="item evidenceItem" key={item.id}>
              <div><strong>{item.work_type}</strong><span>{item.id}</span></div>
              <button type="button" data-dispatch-work-item data-work-item-id={item.id}>Dispatch</button>
            </article>
          ))}
        </div>
      ) : <p className="actionHint">No queued work items need dispatch right now.</p>}
      <p data-pipeline-ops-result>Ready.</p>
      <DomJsonScript marker="data-pipeline-queued-json" json={queuedJson} />
      <ClientScript script={script} />
    </section>
  );
}

