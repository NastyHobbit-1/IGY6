import type { SourceRecord, ApprovalRecord, ApiResult } from "../types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function AddInformationFlow({
  sources,
  approvals,
}: {
  sources: ApiResult<SourceRecord[]>;
  approvals: ApiResult<ApprovalRecord[]>;
}) {
  const apiBase = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const sourcesJson = JSON.stringify(sources.data).replace(/</g, "\\u003c");
  const approvalsJson = JSON.stringify(approvals.data).replace(/</g, "\\u003c");

  const script = `
(() => {
  const root = document.querySelector("[data-add-information]");
  if (!root || root.getAttribute("data-add-wired") === "true") return;
  root.setAttribute("data-add-wired", "true");
  const apiBaseUrl = root.getAttribute("data-api-base-url") || "";
  const sources = JSON.parse(root.querySelector("[data-add-sources-json]")?.textContent || "[]");
  const result = root.querySelector("[data-add-result]");
  const stepKind = root.querySelector("[name='add_kind']");
  const panels = {
    text: root.querySelector("[data-add-panel='text']"),
    website: root.querySelector("[data-add-panel='website']"),
    folder: root.querySelector("[data-add-panel='folder']"),
  };
  const show = (title, message, nextSteps) => {
    if (!result) return;
    result.innerHTML = "";
    const strong = document.createElement("strong");
    strong.textContent = title;
    const span = document.createElement("span");
    span.textContent = message;
    result.append(strong, span);
    if (nextSteps?.length) {
      const list = document.createElement("ul");
      nextSteps.forEach((step) => {
        const item = document.createElement("li");
        item.textContent = step;
        list.appendChild(item);
      });
      result.appendChild(list);
    }
  };
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const postJson = async (path, body) => {
    const response = await fetch(path.startsWith("/api") ? path : apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body || {}),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => { binary += String.fromCharCode(byte); });
    return btoa(binary);
  };
  const syncPanels = () => {
    const kind = stepKind?.value || "text";
    Object.entries(panels).forEach(([key, panel]) => {
      if (panel) panel.hidden = key !== kind;
    });
  };
  stepKind?.addEventListener("change", syncPanels);
  syncPanels();

  root.querySelector("[data-add-submit]")?.addEventListener("click", async () => {
    const button = root.querySelector("[data-add-submit]");
    if (button) button.disabled = true;
    const kind = stepKind?.value || "text";
    show("Working", "IGY6 is adding your information...", []);
    try {
      if (kind === "text") {
        const text = value("add_text");
        if (!text) throw new Error("Paste some text first.");
        let sourceId = value("add_text_source");
        if (!sourceId) {
          const created = await postJson(apiBaseUrl + "/sources", {
            name: value("add_text_source_name") || "My notes",
            source_type: "manual_upload",
            location: "manual",
            sensitivity: "internal",
            permission: {
              scope_json: {},
              allowed_operations: ["dry_run", "read", "collect"],
              external_model_policy: "blocked",
              approval_required: false,
            },
          });
          sourceId = created.id;
        }
        const source = sources.find((item) => item.id === sourceId) || { permissions: [] };
        const permission = (source.permissions || []).find((p) => (p.allowed_operations || []).includes("collect"))
          || (source.permissions || [])[0];
        if (!permission?.id) throw new Error("No permission on source. Create a source with collect permission first.");
        const payload = await postJson(apiBaseUrl + "/collection-runs/manual-upload", {
          source_id: sourceId,
          source_permission_id: permission.id,
          approval_id: null,
          requested_by_actor_id: "local-owner",
          filename: (value("add_text_filename") || "note.txt").replace(/[^A-Za-z0-9._ -]/g, "-") || "note.txt",
          mime_type: "text/plain",
          content_base64: textToBase64(text),
        });
        const summary = payload?.summary_json || {};
        show("Saved", "Your text was saved and queued for automatic processing.", [
          summary.normalization_work_item_id ? "Processing is running in the background." : "Check Activity for progress.",
          "Ask IGY6 about this content once processing finishes.",
        ]);
        if (typeof window.igy6SwitchSection === "function") window.igy6SwitchSection("activity", "work-processing");
        return;
      }
      if (kind === "website") {
        const url = value("add_website_url");
        if (!url || !/^https?:\\/\\//i.test(url)) throw new Error("Enter a valid http(s) URL.");
        const payload = await postJson("/api/collection-runs/full-access", {
          requested_by_actor_id: "local-owner",
          scope: [url],
          mode: "full_access_igy6",
          max_depth: Number(value("add_website_depth") || 1),
          safe_mode: true,
        });
        const summary = payload?.summary_json || payload || {};
        show("Collection started", "IGY6 is collecting from the website and will process results automatically.", [
          "Watch Activity for progress.",
          typeof summary.ai_review_summary === "string" ? summary.ai_review_summary : "A summary will appear when indexing completes.",
        ]);
        if (typeof window.igy6SwitchSection === "function") window.igy6SwitchSection("activity", "work-processing");
        return;
      }
      if (kind === "folder") {
        const sourceId = value("add_folder_source");
        const permissionId = value("add_folder_permission");
        const path = value("add_folder_path");
        if (!sourceId || !permissionId) throw new Error("Select a local folder source and permission.");
        if (!path) throw new Error("Enter a folder path under the source location.");
        const payload = await postJson(apiBaseUrl + "/collection-runs/local-project", {
          source_id: sourceId,
          source_permission_id: permissionId,
          approval_id: null,
          requested_by_actor_id: "local-owner",
          paths: [path],
        });
        show("Import started", "IGY6 is importing from your local folder.", [
          payload?.summary_json?.normalization_work_item_id ? "Processing was queued automatically." : "Check Activity for the result.",
        ]);
        if (typeof window.igy6SwitchSection === "function") window.igy6SwitchSection("activity", "work-processing");
        return;
      }
    } catch (error) {
      show("Could not add information", error instanceof Error ? error.message : String(error), [
        "Fix the input and try again.",
        "Open System if services are not running.",
      ]);
    } finally {
      if (button) button.disabled = false;
    }
  });
})();
`;

  const manualSources = sources.data.filter((s) => s.enabled && s.source_type === "manual_upload");
  const projectSources = sources.data.filter((s) => s.enabled && s.source_type === "local_project");

  return (
    <section className="productPanel" id="uploads-collection" data-add-information data-api-base-url={apiBase}>
      <header className="productPanelHeader">
        <div>
          <p className="productEyebrow">Add Information</p>
          <h2>Bring information into IGY6</h2>
        </div>
      </header>
      <p className="productLead">
        Choose what you want to add. IGY6 picks the best supported method, processes it automatically, and indexes it for search.
      </p>
      <div className="productFormGrid">
        <label className="productField">
          <span>What do you want to add?</span>
          <select name="add_kind" defaultValue="text">
            <option value="text">Add text</option>
            <option value="website">Add a website</option>
            <option value="folder">Add a local folder</option>
          </select>
        </label>
      </div>

      <div className="productFormStack" data-add-panel="text">
        <label className="productField">
          <span>Use an existing notes source (optional)</span>
          <select name="add_text_source" defaultValue="">
            <option value="">Create a new notes source</option>
            {manualSources.map((source) => (
              <option key={source.id} value={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <label className="productField">
          <span>New source name (if creating)</span>
          <input name="add_text_source_name" placeholder="My notes" />
        </label>
        <label className="productField">
          <span>Filename</span>
          <input name="add_text_filename" placeholder="note.txt" defaultValue="note.txt" />
        </label>
        <label className="productField productFieldGrow">
          <span>Paste your text</span>
          <textarea name="add_text" rows={8} placeholder="Paste notes, logs, exports, or any UTF-8 text..." />
        </label>
      </div>

      <div className="productFormStack" data-add-panel="website" hidden>
        <label className="productField">
          <span>Website URL</span>
          <input name="add_website_url" type="url" placeholder="https://example.com/page" />
        </label>
        <label className="productField">
          <span>How deep to follow links</span>
          <select name="add_website_depth" defaultValue="1">
            <option value="0">This page only</option>
            <option value="1">One level of links</option>
            <option value="2">Two levels of links</option>
          </select>
        </label>
        <p className="productHint">IGY6 collects authorized pages locally and processes them automatically. Paid or login-only pages may need extra setup.</p>
      </div>

      <div className="productFormStack" data-add-panel="folder" hidden>
        <label className="productField">
          <span>Local folder source</span>
          <select name="add_folder_source" defaultValue="">
            <option value="">Select a source</option>
            {projectSources.map((source) => (
              <option key={source.id} value={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <label className="productField">
          <span>Permission ID</span>
          <input name="add_folder_permission" placeholder="Permission ID from your source" />
        </label>
        <label className="productField">
          <span>Folder path (under source location)</span>
          <input name="add_folder_path" placeholder="reports/2026" />
        </label>
        {projectSources.length === 0 ? (
          <p className="productHint">No local folder sources yet. Create one in Settings → Advanced if you need recurring imports.</p>
        ) : null}
      </div>

      <div className="productActions">
        <button type="button" className="productPrimaryButton" data-add-submit>Add to IGY6</button>
        <button type="button" className="productGhostButton" data-product-go="evidence" data-product-scroll="evidence-panel">Review what was found</button>
      </div>
      <div className="productResult" data-add-result>Ready to add information.</div>
      <details className="productAdvanced">
        <summary>Technical details</summary>
        <p className="productHint">Routes: manual-upload, full-access collection, local-project collection. Processing chains run automatically without manual evidence review.</p>
      </details>
      <DomJsonScript marker="data-add-sources-json" json={sourcesJson} />
      <DomJsonScript marker="data-add-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}