import { BROWSER_WEB_ROUTER_IMPORT_TYPES } from "./constants";
import { excerpt } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { WebFetchToolsPanels } from "./WebFetchToolsPanels";

export function BrowserWebRouterCollectorMvp() {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const importTypesJson = JSON.stringify(BROWSER_WEB_ROUTER_IMPORT_TYPES).replace(/</g, "\\u003c");

  const script = `
(() => {
  const root = document.querySelector("[data-browser-web-router-mvp]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const importTypes = JSON.parse(root.querySelector("[data-browser-web-router-types-json]")?.textContent || "[]");
  const form = root.querySelector("[data-browser-web-router-preview-form]");
  const result = root.querySelector("[data-browser-web-router-result]");
  const typeSelect = root.querySelector("[name='bwr_type']");
  const scopeInput = root.querySelector("[name='bwr_scope']");
  const textInput = root.querySelector("[name='bwr_text']");
  const statusText = root.querySelector("[data-browser-web-router-type-status]");
  const fieldValue = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const selectedType = () => importTypes.find((item) => item.key === typeSelect?.value) || importTypes[0];
  const writeStatus = () => {
    const type = selectedType();
    if (!statusText || !type) return;
    statusText.textContent = type.label + " — paste-only preview below, or use Auto bypass / Fetch public web page / Bypass fetch above for automatic URL collection. " + type.excluded;
  };
  const looksSensitive = (text) => /(password|passwd|secret|token|cookie|authorization|bearer|private key|ssid|wpa|api[_ -]?key)/i.test(text);
  const renderResult = (payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = payload.title;
    const body = document.createElement("span");
    body.textContent = payload.message;
    result.append(title, body);
    const details = document.createElement("dl");
    payload.details.forEach((detail) => {
      const term = document.createElement("dt");
      term.textContent = detail.label;
      const description = document.createElement("dd");
      description.textContent = detail.value;
      details.append(term, description);
    });
    result.append(details);
    const list = document.createElement("ul");
    payload.next.forEach((step) => {
      const item = document.createElement("li");
      item.textContent = step;
      list.appendChild(item);
    });
    result.append(list);
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
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => { binary += String.fromCharCode(byte); });
    return btoa(binary);
  };
  const safeFilename = (name) => (name || "paste-import.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim() || "paste-import.txt";
  const sourceTypeFor = (key) => {
    if (key === "router_status_text") return "router_network";
    if (key === "web_page_text") return "web_public";
    return "browser_export";
  };
  const collectPastedText = async () => {
    const type = selectedType();
    const scope = fieldValue("bwr_scope");
    const text = fieldValue("bwr_text");
    if (!scope || !text) throw new Error("Enter explicit scope and authorized pasted text before collecting.");
    if (looksSensitive(text)) throw new Error("Sensitive terms detected. Redact credentials, cookies, and tokens before collecting.");
    const sourceType = sourceTypeFor(type.key);
    const source = await postJson("/sources", {
      name: scope.slice(0, 80) || type.label,
      source_type: sourceType,
      location: scope,
      sensitivity: "internal",
      metadata_json: { import_type: type.key, created_from: "browser_web_router_paste" },
      permission: {
        scope_json: { import_type: type.key, scope_label: scope },
        allowed_operations: ["dry_run", "read", "collect"],
        external_model_policy: "blocked",
        approval_required: false
      }
    });
    const permission = (source.permissions || []).find((item) => (item.allowed_operations || []).includes("collect"));
    if (!source.id || !permission?.id) throw new Error("Source or permission was not created for paste collection.");
    const upload = await postJson("/collection-runs/manual-upload", {
      source_id: source.id,
      source_permission_id: permission.id,
      filename: safeFilename(scope),
      mime_type: "text/plain",
      content_base64: textToBase64(text),
      metadata_json: { submitted_from: "browser_web_router_paste", import_type: type.key, scope },
      requested_by_actor_id: "local-owner"
    });
    return { source, upload, type };
  };
  typeSelect?.addEventListener("change", writeStatus);
  writeStatus();
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const type = selectedType();
    const scope = fieldValue("bwr_scope");
    const text = fieldValue("bwr_text");
    if (!scope || !text) {
      renderResult({
        title: "Dry-run incomplete",
        message: "Enter an explicit scope and paste authorized text before previewing.",
        details: [
          { label: "collection", value: "not started" },
          { label: "external requests", value: "none" },
          { label: "router writes", value: "none" }
        ],
        next: ["Add a scope label and text excerpt.", "Remove credentials, cookies, tokens, and private account data before any import."]
      });
      return;
    }
    const lineCount = text.split(/\\r?\\n/).filter((line) => line.trim()).length;
    const sensitive = looksSensitive(text);
    renderResult({
      title: "Dry-run preview",
      message: "No collection ran yet. Click Collect pasted text to store this locally through the real ingestion pipeline.",
      details: [
        { label: "scope entered", value: scope },
        { label: "source posture", value: type.label + " · manual paste import" },
        { label: "would collect", value: type.collected },
        { label: "will not collect", value: type.excluded },
        { label: "sensitivity", value: sensitive ? "sensitive terms detected; redact before import" : type.sensitivity },
        { label: "text size", value: text.length + " characters across " + lineCount + " non-empty line(s)" }
      ],
      next: [
        "Click Collect pasted text when the preview looks correct.",
        "Processing appears in Work; evidence appears in Results after normalization completes."
      ]
    });
    if (scopeInput) scopeInput.setAttribute("data-last-previewed", "true");
    if (textInput) textInput.setAttribute("data-last-previewed", "true");
  });
  root.querySelector("[data-bwr-collect]")?.addEventListener("click", async () => {
    const button = root.querySelector("[data-bwr-collect]");
    if (button) { button.disabled = true; button.textContent = "Collecting..."; }
    try {
      const resultPayload = await collectPastedText();
      const summary = resultPayload.upload?.summary_json || {};
      renderResult({
        title: "Paste collected",
        message: "Authorized text was stored locally and normalization work was queued.",
        details: [
          { label: "source", value: resultPayload.source.id },
          { label: "import type", value: resultPayload.type.label },
          { label: "collection run", value: resultPayload.upload?.id || "not returned" },
          { label: "work item", value: summary.normalization_work_item_id || "not returned" }
        ],
        next: ["Open Work to watch processing.", "Open Results and Chat when evidence is ready."]
      });
    } catch (error) {
      renderResult({
        title: "Collection failed",
        message: error instanceof Error ? error.message : "Unknown error",
        details: [{ label: "collection", value: "not started" }],
        next: ["Fix validation issues and try again.", "Use Guided Upload if you need approval-gated collection."]
      });
    } finally {
      if (button) { button.disabled = false; button.textContent = "Collect pasted text"; }
    }
  });
})();
`;

  // Removed dev-only grok tools block with hardcoded password.

  return (
    <section className="guidedManualText" id="browser-web-router-import" data-browser-web-router-mvp data-api-base-url={browserApiBaseUrl}>
      <WebFetchToolsPanels />
      <div className="guidedManualNotice">
        <strong>Manual paste, preview, and collect</strong>
        <span>Paste authorized page, web, or router text here. Preview first, then collect through the real local ingestion pipeline.</span>
      </div>
      <details className="panelInset" style={{margin:'6px 0'}} open={false}>
        <summary><strong>Media tools</strong> <em>(local only)</em></summary>
        <div className="stack" data-product-tools>
          <button type="button" data-open-media-library>Open Media Library</button>
          <button type="button" data-run-deep-scan>Deep Thorough Scan</button>
          <p className="actionHint">Deep scan requires a program password (and TOTP if enabled). It prepares host-bridge if needed, then runs strongest-tier collection.</p>
        </div>
        <div id="media-viewer" data-media-viewer style={{display:'none',position:'fixed',top:'10%',left:'10%',width:'80%',height:'80%',background:'#000',color:'#0f0',zIndex:99999,padding:'1rem',overflow:'auto'}} />
        <ClientScript script={`
(function(){
  const tools = document.querySelector("[data-product-tools]");
  if (!tools || tools.getAttribute("data-wired") === "true") return;
  tools.setAttribute("data-wired","true");
  const openViewer = () => {
    const v = document.getElementById("media-viewer");
    if (!v) return;
    const gate = async () => {
      try {
        const status = await (await fetch("/api/user/status")).json();
        if (!status.password_set) { alert("Set a program password first in Settings → User & Security."); return false; }
        const current = prompt("Enter current program password:");
        if (!current) return false;
        let totp = "";
        if (status.totp_enabled) {
          totp = prompt("Enter current TOTP code:") || "";
          if (!totp) return false;
        }
        // Non-mutating verification (Tabby #7): proxy POST /api/user/verify-unlock
        const body = { current_password: current } as any;
        if (totp) body.totp_code = totp;
        const verify = await fetch("/api/user/verify-unlock", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body)
        });
        if (!verify.ok) {
          let msg = "Unlock failed";
          try { const p = await verify.json(); if (p?.detail) msg = p.detail; } catch {}
          alert(msg);
          return false;
        }
        return true;
      } catch { return false; }
    };
    v.style.display='block'; v.innerHTML = 'Loading...';
    gate().then(ok => {
      if (!ok) { v.style.display='none'; return; }
      fetch('/api/artifacts')
      .then(r => r.json())
      .then(as => {
        if (!Array.isArray(as)) as = [];
        const ms = as.filter(a => String(a.mime_type||'').toLowerCase().match(/^(image|video)/));
        v.innerHTML = (ms.map(a => "<div style='border:1px solid #0f0;margin:2px;padding:2px;cursor:pointer' data-media-id='"+a.id+"' data-media-mime='"+(a.mime_type||"")+"'>"+(a.mime_type||"")+" "+a.id+"</div>").join("")) || "No media yet.";
        v.querySelectorAll("[data-media-id]").forEach(node => {
          node.addEventListener("click", async () => {
            const id = node.getAttribute("data-media-id");
            const mime = node.getAttribute("data-media-mime")||"";
            v.innerHTML = 'Loading full res...';
            try {
              const c = await fetch('/api/artifacts/'+id+'/content'); const cj = await c.json();
              const pre = cj.data_url_prefix || ('data:'+ (cj.mime_type||mime) +';base64,');
              if ((mime||'').toLowerCase().indexOf('image')>=0) {
                v.innerHTML = '<img src="'+pre+cj.base64_content+'" style="max-width:100%" /><br><small>Full res from source. Click outside to close.</small>';
              } else {
                v.innerHTML = '<video src="'+pre+cj.base64_content+'" controls style="max-width:100%" /><br><small>Original res video. Click outside to close.</small>';
              }
            } catch(e) { v.innerHTML = 'Error loading: '+e; }
          });
        });
        v.addEventListener("click", () => { v.style.display='none'; }, { once: true });
      })
      .catch(e => { const v = document.getElementById("media-viewer"); if (v) v.innerHTML = 'Error: '+e; });
    });
  };
  tools.querySelector("[data-open-media-library]")?.addEventListener("click", openViewer);
  tools.querySelector("[data-run-deep-scan]")?.addEventListener("click", async () => {
    try {
      const status = await (await fetch("/api/user/status")).json();
      if (!status.password_set) { alert("Set a program password first in Settings → User & Security."); return; }
      const current = prompt("Enter current program password:");
      if (!current) return;
      let totp = "";
      if (status.totp_enabled) {
        totp = prompt("Enter current TOTP code:") || "";
      }
      // Prepare host-bridge infra
      const ensure = await fetch("/api/host-bridge/ensure-max-reach", { method: "POST" });
      const ensurePayload = await ensure.json().catch(()=>({}));
      if (!ensure.ok) { alert("Host bridge not ready: " + (ensurePayload?.detail || ensure.status)); return; }
      // Run full-access deep collection
      const body = { requested_by_actor_id: "ui", password: current, scope: ["everything"], media_focus: true };
      if (totp) (body as any).totp_code = totp;
      const resp = await fetch("/api/collection-runs/full-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
      });
      const payload = await resp.json().catch(()=>({}));
      if (!resp.ok) throw new Error(payload?.detail || resp.statusText || "Deep scan failed");
      alert("Deep scan started. Open Media Library or Work to monitor results.");
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    }
  });
})();`} />
      </details>
      <form className="guidedManualForm" data-browser-web-router-preview-form>
        <label>
          <span>Import type</span>
          <select name="bwr_type" defaultValue="browser_page_text">
            {BROWSER_WEB_ROUTER_IMPORT_TYPES.map((type) => (
              <option key={type.key} value={type.key}>{type.label}</option>
            ))}
          </select>
        </label>
        <p className="actionHint" data-browser-web-router-type-status />
        <label>
          <span>Explicit scope</span>
          <input name="bwr_scope" placeholder={BROWSER_WEB_ROUTER_IMPORT_TYPES[0].scopePrompt} />
        </label>
        <label>
          <span>Authorized pasted text</span>
          <textarea name="bwr_text" rows={7} placeholder="Paste redacted visible page text, web text, or read-only router status/export text. Do not paste cookies, tokens, credentials, or router secrets." />
        </label>
        <div className="guidedManualActions">
          <button type="submit">Preview paste plan</button>
          <button type="button" data-bwr-collect>Collect pasted text</button>
          <span>Does not fetch URLs. For automatic URL collection, use Max reach / Auto bypass / Fetch public above.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-browser-web-router-result>
        <strong>Ready</strong>
        <span>Choose a type, enter explicit scope, paste authorized text, preview, then collect locally.</span>
      </div>
      <DomJsonScript marker="data-browser-web-router-types-json" json={importTypesJson} />
      <ClientScript script={script} />
    </section>
  );
}

