import type { SourceRecord, ApprovalRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function ConversationHistoryImport({ sources, approvals }: { sources: ApiResult<SourceRecord[]>; approvals: ApiResult<ApprovalRecord[]> }) {
  const browserApiBaseUrl = "/api";
  const conversationSources = sources.data
    .filter((source) => source.enabled && source.source_type === "conversation_history")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      sensitivity: source.sensitivity,
      permissions: source.permissions ?? [],
    }));
  const conversationSourcesJson = JSON.stringify(conversationSources).replace(/</g, "\\u003c");
  const approvalsJson = JSON.stringify(approvals.data).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-conversation-history-import]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const sourceData = JSON.parse(root.querySelector("[data-conversation-history-sources-json]")?.textContent || "[]");
  const approvalData = JSON.parse(root.querySelector("[data-conversation-history-approvals-json]")?.textContent || "[]");
  const result = root.querySelector("[data-conversation-history-result]");
  const debug = root.querySelector("[data-conversation-history-debug]");
  const submit = root.querySelector("[data-conversation-history-submit]");
  const sourceSelect = root.querySelector("[name='conversation_source_choice']");
  const newSourceFields = root.querySelector("[data-conversation-new-source-fields]");
  const approvalHint = root.querySelector("[data-conversation-approval-hint]");
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
        detailList.setAttribute("data-conversation-history-work-status", "");
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
      submit.textContent = busy ? "Importing..." : "Import conversation text";
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
    const cleaned = (filename || "conversation-history.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim();
    return cleaned || "conversation-history.txt";
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
      approvalHint.textContent = checked("conversation_approval_required")
        ? "This new conversation source will request approval first. Text will not be collected until an approval is approved."
        : "This new conversation source can collect pasted UTF-8 text immediately under the created permission.";
      return;
    }
    const permission = permissionFor(source);
    if (!permission) {
      approvalHint.textContent = "This conversation source has no collect/read permission visible to the guided flow. Use Advanced for diagnostics.";
      return;
    }
    approvalHint.textContent = permission.approval_required
      ? "This conversation source requires approval before collection. Submitting will create an approval request and stop in pending state."
      : "This conversation source permission allows immediate local text import.";
  };
  sourceSelect?.addEventListener("change", refreshSourceHints);
  root.querySelector("[name='conversation_approval_required']")?.addEventListener("change", refreshSourceHints);
  refreshSourceHints();

  root.querySelector("[data-conversation-history-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = value("conversation_text");
    if (!text) {
      writeResult("Text required", "Paste authorized UTF-8 conversation/history text before importing.", ["This MVP does not import browser, account, connector, binary, image, audio, video, or external service data."]);
      return;
    }
    setBusy(true);
    try {
      let source = selectedSource();
      let permission = source ? permissionFor(source) : null;
      const conversationTitle = value("conversation_title") || "Conversation History";
      if (!source) {
        source = await postJson("/sources", {
          name: value("conversation_source_name") || conversationTitle,
          source_type: "conversation_history",
          location: value("conversation_context_note") || "Manual local conversation history import",
          sensitivity: value("conversation_sensitivity") || "internal",
          metadata_json: {
            created_from: "conversation_history_import_mvp",
            import_path: "manual_local_utf8_paste",
            manual_local_import_only: true
          },
          permission: {
            scope_json: {
              path: "manual_conversation_history",
              entered_from: "Add Data conversation history import",
              import_type: "manual_utf8_paste"
            },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: checked("conversation_approval_required")
          }
        });
        permission = permissionFor(source);
      }
      if (!source?.id || !permission?.id) {
        throw new Error("No source permission was available for conversation history import.");
      }
      const filename = safeFilename(conversationTitle + ".txt");
      const metadata = {
        submitted_from: "conversation_history_import_mvp",
        title: conversationTitle,
        conversation_title: conversationTitle,
        conversation_date_range: value("conversation_date_range") || null,
        participants: value("conversation_participants") || null,
        context_note: value("conversation_context_note") || null,
        contains_corrections: checked("conversation_contains_corrections"),
        contains_decisions: checked("conversation_contains_decisions"),
        contains_instructions_preferences: checked("conversation_contains_instructions_preferences"),
        manual_local_import_only: true,
        browser_account_connector_import: false,
        binary_media_import: false
      };
      let approvedApproval = null;
      if (permission.approval_required) {
        approvedApproval = matchingApproval("approved", source, permission);
        const pendingApproval = matchingApproval("pending", source, permission);
        if (!approvedApproval && pendingApproval) {
          writeResult(
            "Approval pending",
            "A matching conversation history collection approval is already pending. The pasted text was not uploaded before approval.",
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
            source_type: "conversation_history",
            filename,
            metadata_json: metadata
          }
          });
          writeResult(
            "Approval pending",
            "IGY6 created the conversation history source context and requested collection approval. The pasted text was not uploaded because this permission requires an approved approval record.",
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
        metadata_json: metadata
      });
      const summary = upload?.summary_json || {};
      const workItemId = summary.normalization_work_item_id || "not returned";
      const artifactIds = Array.isArray(summary.raw_artifact_ids) ? summary.raw_artifact_ids.join(", ") : "not returned";
      writeResult(
        "Conversation history submitted",
        "IGY6 accepted the pasted UTF-8 conversation text and queued normalization work for local evidence processing.",
        ["Open Work and look for the work item below.", "When the work item completes, open Results to inspect documents, chunks, and evidence.", "Use Ask over evidence after results appear."],
        { source: { name: source.name, type: source.source_type, id: source.id }, upload },
        [
          { label: "source", value: source.id },
          { label: "source type", value: source.source_type },
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
        "Import failed",
        String(error),
        ["Check that the local API is running and the selected conversation source is enabled.", "Use Advanced only for low-level route diagnostics if this guided flow cannot continue."]
      );
    } finally {
      setBusy(false);
    }
  });
})();
`;

  return (
    <section className="guidedManualText" data-conversation-history-import data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Conversation history import MVP.</strong>
        <span>Manual local UTF-8 paste plus Web fetch tools for URL collection, paste import, local project directory collection, and media text import.</span>
      </div>
      {sources.error ? <p className="errorText">Source list could not be loaded: {sources.error}</p> : null}
      <form className="guidedManualForm" data-conversation-history-form>
        <label>
          <span>Conversation source</span>
          <select name="conversation_source_choice" defaultValue="new">
            <option value="new">Create a new conversation history source</option>
            {conversationSources.map((source, index) => (
              <option value={index} key={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <div className="guidedManualNewSource" data-conversation-new-source-fields>
          <label>
            <span>Source name</span>
            <input name="conversation_source_name" placeholder="Chat History Import" />
          </label>
          <label>
            <span>Sensitivity</span>
            <select name="conversation_sensitivity" defaultValue="internal">
              <option value="public">public</option>
              <option value="internal">internal</option>
              <option value="sensitive">sensitive</option>
              <option value="secret">secret</option>
            </select>
          </label>
          <label className="checkLine">
            <input name="conversation_approval_required" type="checkbox" />
            Require approval before this conversation source can collect text
          </label>
        </div>
        <p className="actionHint" data-conversation-approval-hint />
        <label>
          <span>Conversation title</span>
          <input name="conversation_title" placeholder="Support chat about router setup" />
        </label>
        <label>
          <span>Date/time range if known</span>
          <input name="conversation_date_range" placeholder="2026-05-01 to 2026-05-03" />
        </label>
        <label>
          <span>Participants or roles</span>
          <input name="conversation_participants" placeholder="me, support agent, project lead" />
        </label>
        <label>
          <span>Purpose or context note</span>
          <textarea name="conversation_context_note" rows={2} placeholder="Why this conversation matters or what it was about." />
        </label>
        <div className="checkGrid">
          <label className="checkLine">
            <input name="conversation_contains_corrections" type="checkbox" />
            Contains corrections
          </label>
          <label className="checkLine">
            <input name="conversation_contains_decisions" type="checkbox" />
            Contains decisions
          </label>
          <label className="checkLine">
            <input name="conversation_contains_instructions_preferences" type="checkbox" />
            Contains instructions or preferences
          </label>
        </div>
        <label>
          <span>Conversation/history text</span>
          <textarea name="conversation_text" rows={10} placeholder="Paste authorized UTF-8 conversation or history text here." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-conversation-history-submit>Import conversation text</button>
          <span>Next: Work for processing, Results for evidence. No account or browser access is used.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-conversation-history-result>
        <strong>Ready</strong>
        <span>Create or select a conversation source, paste authorized text, and import it locally.</span>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: conversation import route response details</summary>
        <pre data-conversation-history-debug />
      </details>
      <DomJsonScript marker="data-conversation-history-sources-json" json={conversationSourcesJson} />
      <DomJsonScript marker="data-conversation-history-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}
