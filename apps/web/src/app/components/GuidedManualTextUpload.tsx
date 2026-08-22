import type { SourceRecord, ApprovalRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function GuidedManualTextUpload({ sources, approvals }: { sources: ApiResult<SourceRecord[]>; approvals: ApiResult<ApprovalRecord[]> }) {
  const browserApiBaseUrl = "/api";
  const manualSources = sources.data
    .filter((source) => source.enabled && source.source_type === "manual_upload")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      sensitivity: source.sensitivity,
      permissions: source.permissions ?? [],
    }));
  const manualSourcesJson = JSON.stringify(manualSources).replace(/</g, "\\u003c");
  const approvalsJson = JSON.stringify(approvals.data).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-guided-manual-upload]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const sourceData = JSON.parse(root.querySelector("[data-guided-manual-sources-json]")?.textContent || "[]");
  const approvalData = JSON.parse(root.querySelector("[data-guided-manual-approvals-json]")?.textContent || "[]");
  const result = root.querySelector("[data-guided-manual-result]");
  const debug = root.querySelector("[data-guided-manual-debug]");
  const submit = root.querySelector("[data-guided-manual-submit]");
  const sourceSelect = root.querySelector("[name='guided_source_choice']");
  const newSourceFields = root.querySelector("[data-new-source-fields]");
  const approvalHint = root.querySelector("[data-guided-approval-hint]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const writeResult = (state, message, nextSteps, payload, details) => {
    if (result) {
      result.innerHTML = "";
      const title = document.createElement("strong");
      title.textContent = state;
      const body = document.createElement("span");
      body.textContent = message;
      result.append(title, body);
      if (details?.length) {
        const detailList = document.createElement("dl");
        detailList.setAttribute("data-guided-manual-work-status", "");
        details.forEach((detail) => {
          const term = document.createElement("dt");
          term.textContent = detail.label;
          const description = document.createElement("dd");
          description.textContent = detail.value;
          detailList.append(term, description);
        });
        result.appendChild(detailList);
      }
      if (nextSteps?.length) {
        const list = document.createElement("ul");
        nextSteps.forEach((step) => {
          const item = document.createElement("li");
          item.textContent = step;
          list.appendChild(item);
        });
        result.appendChild(list);
      }
    }
    if (debug) debug.textContent = payload ? JSON.stringify(payload, null, 2) : "";
  };
  const setBusy = (busy) => {
    if (submit) {
      submit.disabled = busy;
      submit.textContent = busy ? "Submitting..." : "Submit manual text";
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
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => {
      binary += String.fromCharCode(byte);
    });
    return btoa(binary);
  };
  const safeFilename = (filename) => {
    const cleaned = (filename || "manual-note.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim();
    return cleaned || "manual-note.txt";
  };
  const permissionFor = (source) => (source.permissions || []).find((permission) => {
    const operations = permission.allowed_operations || [];
    return operations.includes("collect") || operations.includes("read");
  });
  const approvalMatches = (approval, source, permission) => {
    const payload = approval?.request_payload_json || {};
    const payloadSourceType = payload.source_type || source?.source_type;
    return approval?.request_type === "manual_upload_collection"
      && payload.source_id === source?.id
      && payload.source_permission_id === permission?.id
      && payload.operation === "manual_upload_collection"
      && payloadSourceType === source?.source_type;
  };
  const matchingApproval = (status, source, permission) => approvalData.find((approval) => approval?.status === status && approvalMatches(approval, source, permission)) || null;
  const selectedSource = () => {
    if (sourceSelect?.value === "new") return null;
    const index = Number(sourceSelect?.value || -1);
    return Number.isInteger(index) ? sourceData[index] : null;
  };
  const refreshSourceHints = () => {
    const source = selectedSource();
    if (newSourceFields) newSourceFields.hidden = Boolean(source);
    if (!approvalHint) return;
    if (!source) {
      approvalHint.textContent = checked("guided_approval_required")
        ? "This new source will request approval first. Text will not be collected until an approval is approved."
        : "This new source can collect this manual text immediately under the created permission.";
      return;
    }
    const permission = permissionFor(source);
    if (!permission) {
      approvalHint.textContent = "This source has no collect/read permission visible to the guided flow. Use Advanced for diagnostics.";
      return;
    }
    approvalHint.textContent = permission.approval_required
      ? "This source requires approval before collection. Submitting will create an approval request and stop in pending state."
      : "This source permission allows immediate manual text collection.";
  };
  sourceSelect?.addEventListener("change", refreshSourceHints);
  root.querySelector("[name='guided_approval_required']")?.addEventListener("change", refreshSourceHints);
  refreshSourceHints();

  root.querySelector("[data-guided-manual-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = value("guided_text");
    if (!text) {
      writeResult("Text required", "Paste authorized UTF-8 text before submitting.", ["This path does not accept binary files, images, audio, or video."]);
      return;
    }
    setBusy(true);
    try {
      let source = selectedSource();
      let permission = source ? permissionFor(source) : null;
      const sourceName = value("guided_source_name") || value("guided_text_title") || "Manual Text Notes";
      if (!source) {
        source = await postJson("/sources", {
          name: sourceName,
          source_type: "manual_upload",
          location: value("guided_source_description") || "Manual text entered in Add Data",
          sensitivity: value("guided_sensitivity") || "internal",
          metadata_json: { created_from: "guided_add_data_manual_text" },
          permission: {
            scope_json: {
              path: "manual_text",
              entered_from: "Add Data guided manual text"
            },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: checked("guided_approval_required")
          }
        });
        permission = permissionFor(source);
      }
      if (!source?.id || !permission?.id) {
        throw new Error("No source permission was available for guided manual text collection.");
      }
      const filename = safeFilename(value("guided_text_title") || sourceName || "manual-note.txt");
      let approvedApproval = null;
      if (permission.approval_required) {
        approvedApproval = matchingApproval("approved", source, permission);
        const pendingApproval = matchingApproval("pending", source, permission);
        if (!approvedApproval && pendingApproval) {
          writeResult(
            "Approval pending",
            "A matching manual text collection approval is already pending. The text was not uploaded before approval.",
            ["Open Settings to approve or deny the pending collection request.", "After approving it, return to this guided form and submit again; IGY6 will use the matching approved approval automatically.", "Processing status appears in Work after collection, and evidence appears in Results."],
            { source: { name: source.name, type: source.source_type }, permission: { approval_required: permission.approval_required }, approval: pendingApproval },
            [
              { label: "source", value: source.name + " (" + source.source_type + ")" },
              { label: "permission", value: "approval required" },
              { label: "approval", value: "pending" },
              { label: "upload", value: "not started" },
              { label: "next safe action", value: "review pending approval in Settings" }
            ]
          );
          return;
        }
        if (!approvedApproval) {
          const approval = await postJson("/approvals", {
          request_type: "manual_upload_collection",
          request_payload_json: {
            source_id: source.id,
            source_permission_id: permission.id,
            operation: "manual_upload_collection",
            source_type: source.source_type,
            filename
          }
          });
          writeResult(
            "Approval pending",
            "IGY6 created the manual text source context and requested collection approval. The text was not uploaded because this permission requires an approved approval record.",
            ["Open Settings to approve or deny the pending collection request.", "After approving it, return to this guided form and submit again; IGY6 will use the matching approved approval automatically.", "Processing status appears in Work after collection, and evidence appears in Results."],
            { source: { name: source.name, type: source.source_type }, permission: { approval_required: permission.approval_required }, approval },
            [
              { label: "source", value: source.name + " (" + source.source_type + ")" },
              { label: "permission", value: "approval required" },
              { label: "approval", value: "pending" },
              { label: "upload", value: "not started" },
              { label: "next safe action", value: "review pending approval in Settings" }
            ]
          );
          return;
        }
      }
      const upload = await postJson("/collection-runs/manual-upload", {
        source_id: source.id,
        source_permission_id: permission.id,
        approval_id: approvedApproval?.id || null,
        filename,
        mime_type: "text/plain",
        content_base64: textToBase64(text),
        metadata_json: {
          submitted_from: "guided_add_data_manual_text",
          title: value("guided_text_title") || null
        }
      });
      const summary = upload?.summary_json || {};
      const workItemId = summary.normalization_work_item_id || "not returned";
      const artifactIds = Array.isArray(summary.raw_artifact_ids) ? summary.raw_artifact_ids.join(", ") : "not returned";
      writeResult(
        "Manual text submitted",
        "IGY6 accepted the UTF-8 text and queued normalization work for background processing.",
        ["Open Work and look for the work item below.", "When the work item completes, open Results to inspect documents, chunks, and evidence.", "Use Ask over evidence after results appear."],
        { source: { name: source.name, type: source.source_type, id: source.id }, upload },
        [
          { label: "source", value: source.id },
          { label: "permission", value: permission.approval_required ? "approved collection permission" : "immediate collection permission" },
          { label: "approval", value: approvedApproval ? "approved and matched automatically" : "not required" },
          { label: "collection run", value: upload?.id || "not returned" },
          { label: "work item", value: workItemId },
          { label: "work type", value: "collection_normalization" },
          { label: "raw artifact", value: artifactIds },
          { label: "current status", value: "queued, then running, then completed when normalization finishes" }
        ]
      );
    } catch (error) {
      writeResult(
        "Submission failed",
        String(error),
        ["Check that the local API is running and the selected source is enabled.", "Use Advanced only for low-level route diagnostics if this guided flow cannot continue."]
      );
    } finally {
      setBusy(false);
    }
  });
})();
`;

  return (
    <section className="guidedManualText" data-guided-manual-upload data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Manual UTF-8 text only.</strong>
        <span>This guided path accepts pasted text. It does not parse PDF, images, audio, video, screenshots, or web pages.</span>
      </div>
      {sources.error ? <p className="errorText">Source list could not be loaded: {sources.error}</p> : null}
      <form className="guidedManualForm" data-guided-manual-form>
        <label>
          <span>Use source</span>
          <select name="guided_source_choice" defaultValue="new">
            <option value="new">Create a new manual text source</option>
            {manualSources.map((source, index) => (
              <option value={index} key={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <div className="guidedManualNewSource" data-new-source-fields>
          <label>
            <span>Source name</span>
            <input name="guided_source_name" placeholder="Router Troubleshooting Notes" />
          </label>
          <label>
            <span>Description</span>
            <input name="guided_source_description" placeholder="Pasted notes copied from my local troubleshooting log" />
          </label>
          <label>
            <span>Sensitivity</span>
            <select name="guided_sensitivity" defaultValue="internal">
              <option value="public">public</option>
              <option value="internal">internal</option>
              <option value="sensitive">sensitive</option>
              <option value="secret">secret</option>
            </select>
          </label>
          <label className="checkLine">
            <input name="guided_approval_required" type="checkbox" />
            Require approval before this source can collect text
          </label>
        </div>
        <p className="actionHint" data-guided-approval-hint />
        <label>
          <span>Text title or filename</span>
          <input name="guided_text_title" defaultValue="manual-note.txt" />
        </label>
        <label>
          <span>Authorized text</span>
          <textarea name="guided_text" rows={8} placeholder="Paste authorized UTF-8 text here." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-guided-manual-submit>Submit manual text</button>
          <span>Next: Work for processing, Results for evidence.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-guided-manual-result>
        <strong>Ready</strong>
        <span>Create or select a manual source, paste text, and submit. Raw IDs stay in Advanced.</span>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: guided route response details</summary>
        <pre data-guided-manual-debug />
      </details>
      <DomJsonScript marker="data-guided-manual-sources-json" json={manualSourcesJson} />
      <DomJsonScript marker="data-guided-manual-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}

