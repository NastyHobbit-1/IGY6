import type { SourceRecord, ApprovalRecord, ApiResult } from "./types";
import { LOCAL_PROJECT_DIAGNOSTICS_MODES } from "./constants";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function LocalProjectPcDiagnosticsHardeningPanel({
  sources,
  approvals
}: {
  sources: ApiResult<SourceRecord[]>;
  approvals: ApiResult<ApprovalRecord[]>;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const localProjectSources = sources.data
    .filter((source) => source.enabled && source.source_type === "local_project")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      permissions: source.permissions ?? []
    }));
  const modesJson = JSON.stringify(LOCAL_PROJECT_DIAGNOSTICS_MODES).replace(/</g, "\\u003c");
  const localSourcesJson = JSON.stringify(localProjectSources).replace(/</g, "\\u003c");
  const approvalsJson = JSON.stringify(approvals.data).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-local-project-pc-diagnostics]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const modes = JSON.parse(root.querySelector("[data-local-project-pc-modes-json]")?.textContent || "[]");
  const localSources = JSON.parse(root.querySelector("[data-local-project-pc-sources-json]")?.textContent || "[]");
  const approvalData = JSON.parse(root.querySelector("[data-local-project-pc-approvals-json]")?.textContent || "[]");
  const form = root.querySelector("[data-local-project-pc-preview-form]");
  const modeSelect = root.querySelector("[name='lp_mode']");
  const modeStatus = root.querySelector("[data-local-project-pc-mode-status]");
  const result = root.querySelector("[data-local-project-pc-result]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const selectedMode = () => modes.find((item) => item.key === modeSelect?.value) || modes[0];
  const redactPath = (input) => {
    if (!input) return "not provided";
    const normalized = input.replace(/\\\\/g, "/");
    const parts = normalized.split("/").filter(Boolean);
    const tail = parts.slice(-2).join("/");
    return tail ? "[redacted]/" + tail : "[redacted path provided]";
  };
  const countList = (input) => input.split(/\\r?\\n|,/).map((item) => item.trim()).filter(Boolean).length;
  const hasSecretSignal = (input) => /(\\.env|id_rsa|private key|password|passwd|secret|token|cookie|authorization|api[_ -]?key|credential|ssh)/i.test(input);
  const render = (payload) => {
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
  const updateStatus = () => {
    const mode = selectedMode();
    if (!modeStatus || !mode) return;
    modeStatus.textContent = mode.label + ": " + mode.scope + " Excludes: " + mode.excluded;
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
  const permissionFor = (source) => (source.permissions || []).find((permission) => (permission.allowed_operations || []).includes("collect"));
  const looksLikeDirectory = (input) => /^[A-Za-z]:[\\\\/]/.test(input) || input.startsWith("/") || input.startsWith("\\\\");
  modeSelect?.addEventListener("change", updateStatus);
  updateStatus();
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const mode = selectedMode();
    const scope = value("lp_scope");
    const includeRules = value("lp_include");
    const excludeRules = value("lp_exclude");
    const pasted = value("lp_text");
    const maxFiles = Number(value("lp_max_files") || 0);
    const maxBytes = Number(value("lp_max_bytes") || 0);
    if (!scope || !pasted) {
      render({
        title: "Dry-run incomplete",
        message: "Enter explicit scope and paste an authorized manifest or diagnostics export before previewing.",
        details: [
          { label: "filesystem reads", value: "none" },
          { label: "system commands", value: "none" },
          { label: "collection", value: "not started" }
        ],
        next: ["Use a user-selected path label or diagnostic export only.", "Do not paste .env, SSH keys, credentials, tokens, cookies, or private account data."]
      });
      return;
    }
    const secretSignal = hasSecretSignal(scope + "\\n" + includeRules + "\\n" + excludeRules + "\\n" + pasted);
    render({
      title: "Scoped import preview",
      message: mode.key === "local_project_manifest" && looksLikeDirectory(scope)
        ? "Directory collection is available. Click Collect scoped import to read bounded files from the path."
        : "Text collection is available. Click Collect scoped import to store pasted diagnostics or manifest text.",
      details: [
        { label: "mode", value: mode.label },
        { label: "scope label", value: redactPath(scope) },
        { label: "include entries", value: String(countList(includeRules)) },
        { label: "exclude entries", value: String(countList(excludeRules)) },
        { label: "file count limit", value: maxFiles > 0 ? String(Math.min(maxFiles, 500)) : "100 default on collect" },
        { label: "byte limit", value: maxBytes > 0 ? String(Math.min(maxBytes, 10 * 1024 * 1024)) : "1 MB default on collect" },
        { label: "pasted text", value: pasted.length + " characters; content not echoed here" },
        { label: "secret signal", value: secretSignal ? "potential secret/path signal detected; redact before import" : "no obvious secret keyword detected" },
        { label: "would collect", value: mode.collect },
        { label: "will not collect", value: mode.excluded }
      ],
      next: ["Click Collect scoped import when the preview looks correct.", "Processing appears in Work; evidence appears in Results."]
    });
  });
  root.querySelector("[data-lp-collect]")?.addEventListener("click", async () => {
    const button = root.querySelector("[data-lp-collect]");
    if (button) { button.disabled = true; button.textContent = "Collecting..."; }
    try {
      const mode = selectedMode();
      const scope = value("lp_scope");
      const includeRules = value("lp_include");
      const excludeRules = value("lp_exclude");
      const pasted = value("lp_text");
      const maxFiles = Number(value("lp_max_files") || 100);
      const maxBytes = Number(value("lp_max_bytes") || 1048576);
      if (!scope || !pasted) throw new Error("Enter explicit scope and authorized text before collecting.");
      if (hasSecretSignal(scope + "\\n" + includeRules + "\\n" + excludeRules + "\\n" + pasted)) {
        throw new Error("Potential secret signal detected. Redact before collecting.");
      }
      if (mode.key === "local_project_manifest" && looksLikeDirectory(scope)) {
        let source = localSources.find((item) => item.location === scope) || null;
        if (!source) {
          const created = await postJson("/sources", {
            name: scope.split(/[\\\\/]/).filter(Boolean).slice(-1)[0] || "local-project",
            source_type: "local_project",
            location: scope,
            sensitivity: "internal",
            metadata_json: { created_from: "local_project_panel" },
            permission: {
              scope_json: {
                include: includeRules ? includeRules.split(/\\r?\\n|,/).map((item) => item.trim()).filter(Boolean) : ["**/*"],
                exclude: excludeRules ? excludeRules.split(/\\r?\\n|,/).map((item) => item.trim()).filter(Boolean) : [".env", "**/node_modules/**"],
                max_files: Math.min(maxFiles, 500),
                max_file_bytes: Math.min(maxBytes, 10 * 1024 * 1024)
              },
              allowed_operations: ["dry_run", "read", "collect"],
              external_model_policy: "blocked",
              approval_required: false
            }
          });
          source = { id: created.id, name: created.name, location: created.location, permissions: created.permissions || [] };
        }
        const permission = permissionFor(source);
        if (!source?.id || !permission?.id) throw new Error("Local project source or collect permission is unavailable.");
        const collection = await postJson("/collection-runs/local-project", {
          source_id: source.id,
          source_permission_id: permission.id,
          requested_by_actor_id: "local-owner"
        });
        render({
          title: "Local project collected",
          message: "Bounded directory files were stored locally and normalization work was queued.",
          details: [
            { label: "source", value: source.id },
            { label: "path", value: redactPath(scope) },
            { label: "collection run", value: collection?.id || "not returned" },
            { label: "files", value: String(collection?.summary_json?.collected_files ?? "unknown") }
          ],
          next: ["Open Work to watch processing.", "Open Results when evidence is ready."]
        });
      } else {
        const created = await postJson("/sources", {
          name: scope.slice(0, 80) || "diagnostics-export",
          source_type: "local_pc_diagnostics",
          location: scope,
          sensitivity: "internal",
          metadata_json: { mode: mode.key, created_from: "local_project_panel_paste" },
          permission: {
            scope_json: { mode: mode.key, scope_label: scope },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: false
          }
        });
        const permission = permissionFor(created);
        if (!created?.id || !permission?.id) throw new Error("Diagnostics source or collect permission was not created.");
        const upload = await postJson("/collection-runs/manual-upload", {
          source_id: created.id,
          source_permission_id: permission.id,
          filename: "diagnostics-export.txt",
          mime_type: "text/plain",
          content_base64: textToBase64(pasted),
          metadata_json: { submitted_from: "local_project_panel_paste", mode: mode.key },
          requested_by_actor_id: "local-owner"
        });
        render({
          title: "Diagnostics text collected",
          message: "Pasted diagnostics or manifest text was stored locally.",
          details: [
            { label: "collection run", value: upload?.id || "not returned" },
            { label: "work item", value: upload?.summary_json?.normalization_work_item_id || "not returned" }
          ],
          next: ["Open Work to watch processing.", "Open Results when evidence is ready."]
        });
      }
    } catch (error) {
      render({
        title: "Collection failed",
        message: error instanceof Error ? error.message : "Unknown error",
        details: [{ label: "collection", value: "not started" }],
        next: ["Verify the directory exists for local project mode.", "Use Guided Upload for approval-gated text collection."]
      });
    } finally {
      if (button) { button.disabled = false; button.textContent = "Collect scoped import"; }
    }
  });
})();
`;

  return (
    <section className="guidedManualText" id="local-project-pc-diagnostics" data-local-project-pc-diagnostics data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Local project and PC diagnostics collection.</strong>
        <span>Preview scope, then collect bounded directory files from an explicit path or store reviewed diagnostics/manifest text locally.</span>
      </div>
      <form className="guidedManualForm" data-local-project-pc-preview-form>
        <label>
          <span>Mode</span>
          <select name="lp_mode" defaultValue="local_project_manifest">
            {LOCAL_PROJECT_DIAGNOSTICS_MODES.map((mode) => (
              <option key={mode.key} value={mode.key}>{mode.label}</option>
            ))}
          </select>
        </label>
        <p className="actionHint" data-local-project-pc-mode-status />
        <label>
          <span>Explicit scope or selected path label</span>
          <input name="lp_scope" placeholder="D:/Projects/example-app or diagnostics-export-2026-06-07.txt" />
        </label>
        <label>
          <span>Include rules or diagnostic sections</span>
          <input name="lp_include" placeholder="src/**/*.rs, package.json, hardware summary" />
        </label>
        <label>
          <span>Exclude rules</span>
          <input name="lp_exclude" placeholder=".env, secrets, keys, node_modules, target, browser profiles" />
        </label>
        <div className="guidedManualNewSource">
          <label>
            <span>Max files preview cap</span>
            <input name="lp_max_files" type="number" min="1" max="500" defaultValue="100" />
          </label>
          <label>
            <span>Max bytes preview cap</span>
            <input name="lp_max_bytes" type="number" min="1024" max="10485760" defaultValue="1048576" />
          </label>
        </div>
        <label>
          <span>Authorized manifest or diagnostics text</span>
          <textarea name="lp_text" rows={7} placeholder="Paste reviewed project manifest, file list, or diagnostic export text. Do not paste secrets, credentials, .env, SSH keys, cookies, tokens, or private account data." />
        </label>
        <div className="guidedManualActions">
          <button type="submit">Preview scoped import</button>
          <button type="button" data-lp-collect>Collect scoped import</button>
          <span>Directory mode requires a real folder path. Diagnostics mode stores pasted UTF-8 text.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-local-project-pc-result>
        <strong>Ready</strong>
        <span>Enter explicit scope, include/exclude posture, and authorized text to preview and collect.</span>
      </div>
      <DomJsonScript marker="data-local-project-pc-modes-json" json={modesJson} />
      <DomJsonScript marker="data-local-project-pc-sources-json" json={localSourcesJson} />
      <DomJsonScript marker="data-local-project-pc-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}
