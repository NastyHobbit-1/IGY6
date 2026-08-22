import type { SourceRecord, ApprovalRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function UserObservationIngestion({ sources, approvals }: { sources: ApiResult<SourceRecord[]>; approvals: ApiResult<ApprovalRecord[]> }) {
  const browserApiBaseUrl = "/api";
  const observationSources = sources.data
    .filter((source) => source.enabled && source.source_type === "user_observation")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      sensitivity: source.sensitivity,
      permissions: source.permissions ?? [],
    }));
  const observationSourcesJson = JSON.stringify(observationSources).replace(/</g, "\\u003c");
  const approvalsJson = JSON.stringify(approvals.data).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-user-observation-ingestion]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const sourceData = JSON.parse(root.querySelector("[data-user-observation-sources-json]")?.textContent || "[]");
  const approvalData = JSON.parse(root.querySelector("[data-user-observation-approvals-json]")?.textContent || "[]");
  const result = root.querySelector("[data-user-observation-result]");
  const debug = root.querySelector("[data-user-observation-debug]");
  const submit = root.querySelector("[data-user-observation-submit]");
  const sourceSelect = root.querySelector("[name='observation_source_choice']");
  const newSourceFields = root.querySelector("[data-observation-new-source-fields]");
  const approvalHint = root.querySelector("[data-observation-approval-hint]");
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
        detailList.setAttribute("data-user-observation-work-status", "");
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
      submit.textContent = busy ? "Recording..." : "Record observation";
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
    const cleaned = (filename || "user-observation.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim();
    return cleaned || "user-observation.txt";
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
      approvalHint.textContent = checked("observation_approval_required")
        ? "This new observation source will request approval first. The observation text will not be collected until an approval is approved."
        : "This new observation source can collect owner-entered UTF-8 observations immediately under the created permission.";
      return;
    }
    const permission = permissionFor(source);
    if (!permission) {
      approvalHint.textContent = "This observation source has no collect/read permission visible to the guided flow. Use Advanced for diagnostics.";
      return;
    }
    approvalHint.textContent = permission.approval_required
      ? "This observation source requires approval before collection. Submitting will create an approval request and stop in pending state."
      : "This observation source permission allows immediate local text ingestion.";
  };
  sourceSelect?.addEventListener("change", refreshSourceHints);
  root.querySelector("[name='observation_approval_required']")?.addEventListener("change", refreshSourceHints);
  refreshSourceHints();

  root.querySelector("[data-user-observation-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = value("observation_text");
    if (!text) {
      writeResult("Observation required", "Enter an owner-provided UTF-8 observation, decision, preference, correction, or note before recording.", ["This MVP does not extract hidden memory, scrape accounts, read browsers, call hosted AI, or verify the observation automatically."]);
      return;
    }
    setBusy(true);
    try {
      let source = selectedSource();
      let permission = source ? permissionFor(source) : null;
      const observationTitle = value("observation_title") || "User Observation";
      const observationType = value("observation_type") || "observation";
      const observationSensitivity = checked("observation_sensitive") ? "sensitive" : (value("observation_sensitivity") || "internal");
      if (!source) {
        source = await postJson("/sources", {
          name: value("observation_source_name") || "User Observations",
          source_type: "user_observation",
          location: "Manual owner-provided observation entry",
          sensitivity: observationSensitivity,
          trust_level: "trusted",
          metadata_json: {
            created_from: "user_observation_ingestion_mvp",
            import_path: "manual_local_utf8_entry",
            owner_provided_first_party_context: true,
            automatic_verification: false
          },
          permission: {
            scope_json: {
              path: "manual_user_observation",
              entered_from: "Add Data user observation ingestion",
              import_type: "manual_utf8_entry"
            },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: checked("observation_approval_required")
          }
        });
        permission = permissionFor(source);
      }
      if (!source?.id || !permission?.id) {
        throw new Error("No source permission was available for user observation ingestion.");
      }
      const filename = safeFilename(observationTitle + ".txt");
      const metadata = {
        submitted_from: "user_observation_ingestion_mvp",
        title: observationTitle,
        observation_title: observationTitle,
        observation_type: observationType,
        observed_at_or_decided_at: value("observation_observed_at") || null,
        confidence: value("observation_confidence") || "likely",
        tags: value("observation_tags") || null,
        related_record_labels_or_ids: value("observation_related_ids") || null,
        related_links_validated: false,
        sensitivity_flag: checked("observation_sensitive"),
        owner_provided_first_party_context: true,
        automatic_truth_verification: false,
        hidden_memory_extraction: false,
        account_or_browser_scraping: false,
        hosted_ai_processing: false,
        external_service_collection: false
      };
      let approvedApproval = null;
      if (permission.approval_required) {
        approvedApproval = matchingApproval("approved", source, permission);
        const pendingApproval = matchingApproval("pending", source, permission);
        if (!approvedApproval && pendingApproval) {
          writeResult(
            "Approval pending",
            "A matching user observation collection approval is already pending. The observation text was not uploaded before approval.",
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
            source_type: "user_observation",
            filename,
            metadata_json: metadata
          }
          });
          writeResult(
            "Approval pending",
            "IGY6 created the user observation source context and requested collection approval. The observation text was not uploaded because this permission requires an approved approval record.",
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
        "Observation submitted",
        "IGY6 accepted the owner-provided UTF-8 observation and queued normalization work for local evidence processing. This records context; it does not automatically verify truth.",
        ["Open Work and look for the work item below.", "When the work item completes, open Results to inspect documents, chunks, and evidence.", "Use source and evidence review states when observations need correction or verification later."],
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
        "Observation failed",
        String(error),
        ["Check that the local API is running and the selected observation source is enabled.", "Use Advanced only for low-level route diagnostics if this guided flow cannot continue."]
      );
    } finally {
      setBusy(false);
    }
  });
})();
`;

  return (
    <section className="guidedManualText" data-user-observation-ingestion data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>User observation ingestion MVP.</strong>
        <span>Owner-provided first-party local context only. This does not verify truth automatically, extract hidden memory, scrape accounts or browsers, use connectors, call hosted AI, or read external services.</span>
      </div>
      {sources.error ? <p className="errorText">Source list could not be loaded: {sources.error}</p> : null}
      <form className="guidedManualForm" data-user-observation-form>
        <label>
          <span>Observation source</span>
          <select name="observation_source_choice" defaultValue="new">
            <option value="new">Create a new user observation source</option>
            {observationSources.map((source, index) => (
              <option value={index} key={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <div className="guidedManualNewSource" data-observation-new-source-fields>
          <label>
            <span>Source name</span>
            <input name="observation_source_name" placeholder="User Observations" />
          </label>
          <label>
            <span>Default sensitivity</span>
            <select name="observation_sensitivity" defaultValue="internal">
              <option value="public">public</option>
              <option value="internal">internal</option>
              <option value="sensitive">sensitive</option>
              <option value="secret">secret</option>
            </select>
          </label>
          <label className="checkLine">
            <input name="observation_approval_required" type="checkbox" />
            Require approval before this observation source can collect text
          </label>
        </div>
        <p className="actionHint" data-observation-approval-hint />
        <label>
          <span>Observation title</span>
          <input name="observation_title" placeholder="Decision about warranty follow-up" />
        </label>
        <label>
          <span>Observation type</span>
          <select name="observation_type" defaultValue="observation">
            <option value="observation">observation</option>
            <option value="decision">decision</option>
            <option value="preference">preference</option>
            <option value="correction">correction</option>
            <option value="note">note</option>
          </select>
        </label>
        <label>
          <span>Observed or decided at if known</span>
          <input name="observation_observed_at" placeholder="2026-06-05 14:30 or early June 2026" />
        </label>
        <label>
          <span>Confidence</span>
          <select name="observation_confidence" defaultValue="likely">
            <option value="certain">certain</option>
            <option value="likely">likely</option>
            <option value="unsure">unsure</option>
          </select>
        </label>
        <label>
          <span>Tags</span>
          <input name="observation_tags" placeholder="router, warranty, preference" />
        </label>
        <label>
          <span>Related source/evidence/task IDs or labels</span>
          <input name="observation_related_ids" placeholder="Optional plain text; links are not validated in this MVP" />
        </label>
        <label className="checkLine">
          <input name="observation_sensitive" type="checkbox" />
          Mark this observation as sensitive
        </label>
        <label>
          <span>Observation text</span>
          <textarea name="observation_text" rows={8} placeholder="Enter what you directly observed, decided, prefer, corrected, or want IGY6 to remember as local context." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-user-observation-submit>Record observation</button>
          <span>Next: Work for processing, Results for evidence. User-provided context is not automatic verification.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-user-observation-result>
        <strong>Ready</strong>
        <span>Create or select an observation source, enter owner-provided context, and record it locally.</span>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: observation ingestion route response details</summary>
        <pre data-user-observation-debug />
      </details>
      <DomJsonScript marker="data-user-observation-sources-json" json={observationSourcesJson} />
      <DomJsonScript marker="data-user-observation-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}

