import type { WorkItemRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function PipelineOperationsPanel({ workItems }: { workItems: ApiResult<WorkItemRecord[]> }) {
  const browserApiBaseUrl = "/api";
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
  const addSkeleton = () => {
    const skeleton = document.createElement("div");
    skeleton.className = "skeletonBlock";
    skeleton.setAttribute("aria-busy", "true");
    result?.parentNode?.insertBefore(skeleton, result);
    return skeleton;
  };
  const clearSkeleton = (skeleton) => {
    if (skeleton && skeleton.parentNode) skeleton.parentNode.removeChild(skeleton);
  };
  const showRetry = (label, onClick) => {
    const box = document.createElement("div");
    box.className = "guidedManualActions";
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = label || "Retry";
    btn.addEventListener("click", onClick);
    box.appendChild(btn);
    result?.parentNode?.insertBefore(box, result.nextSibling);
    return box;
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body || {}) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  root.querySelector("[data-vector-ensure]")?.addEventListener("click", async () => {
    const sk = addSkeleton();
    show("Ensuring vector chunks...");
    try {
      const payload = await postJson("/memory/vector/chunks/ensure", {});
      show("Vector ensure: " + JSON.stringify(payload));
    } catch (e) {
      show("Vector ensure failed. Try again when memory is ready.");
      showRetry("Retry ensure vector", () => root.querySelector("[data-vector-ensure]")?.dispatchEvent(new Event("click", { bubbles: true })));
    } finally {
      clearSkeleton(sk);
    }
  });
  root.querySelector("[data-retrieval-search]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const message = root.querySelector("[name='pipeline_search_message']")?.value?.trim() || "";
    const limit = Number(root.querySelector("[name='pipeline_search_limit']")?.value || 5);
    if (!message) return;
    const sk = addSkeleton();
    show("Searching...");
    try {
      const payload = await postJson("/chat/retrieval-preview", { message, limit });
      show("Retrieval: " + JSON.stringify(payload));
    } catch (e) {
      show("Retrieval failed. Check that memory is ready, then retry.");
      showRetry("Retry search", () => root.querySelector("[data-retrieval-search]")?.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })));
    } finally {
      clearSkeleton(sk);
    }
  });
  root.querySelectorAll("[data-dispatch-work-item]").forEach((button) => {
    button.addEventListener("click", async () => {
      const workItemId = button.getAttribute("data-work-item-id");
      if (!workItemId) return;
      button.disabled = true;
      const sk = addSkeleton();
      show("Dispatching " + workItemId + "...");
      try {
        const payload = await postJson("/work-items/" + encodeURIComponent(workItemId) + "/dispatch", {});
        show("Dispatch: " + JSON.stringify(payload));
      } catch (e) {
        show("Dispatch failed. Open Work for details, then retry.");
        showRetry("Retry dispatch", () => button.click());
      } finally {
        button.disabled = false;
        clearSkeleton(sk);
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

