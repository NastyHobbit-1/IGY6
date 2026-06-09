import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { TermHelp } from "./ui/TermHelp";
import { HelpHeading } from "./ui/HelpHeading";

export function MvpActionConsole() {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const script = `
(() => {
  const root = document.querySelector("[data-mvp-actions]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const output = root.querySelector("[data-action-output]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const show = (label, payload) => {
    if (output) output.textContent = label + "\\n" + (typeof payload === "string" ? payload : JSON.stringify(payload, null, 2));
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
  const bind = (selector, handler) => {
    root.querySelector(selector)?.addEventListener("submit", async (event) => {
      event.preventDefault();
      try {
        const result = await handler();
        show("Done", result);
      } catch (error) {
        show("Error", String(error));
      }
    });
  };

  bind("[data-create-source]", () => postJson("/sources", {
    name: value("source_name"),
    source_type: value("source_type"),
    location: value("source_location") || null,
    sensitivity: value("source_sensitivity") || "internal",
    permission: {
      scope_json: value("source_scope_json") ? JSON.parse(value("source_scope_json")) : {},
      allowed_operations: value("source_allowed_operations").split(",").map((item) => item.trim()).filter(Boolean),
      external_model_policy: "blocked",
      approval_required: checked("source_approval_required")
    }
  }));

  bind("[data-approval]", () => postJson("/approvals", {
    request_type: value("approval_request_type"),
    request_payload_json: {
      source_id: value("approval_source_id") || undefined,
      source_permission_id: value("approval_permission_id") || undefined,
      operation: value("approval_operation") || undefined
    }
  }));

  bind("[data-approval-decision]", () => postJson("/approvals/" + value("decision_approval_id") + "/decision", {
    status: value("decision_status"),
    decision_reason: value("decision_reason") || null
  }));

  bind("[data-dry-run]", () => postJson("/collection-runs/dry-run", {
    source_id: value("dry_source_id"),
    source_permission_id: value("dry_permission_id")
  }));

  bind("[data-manual-upload]", () => postJson("/collection-runs/manual-upload", {
    source_id: value("upload_source_id"),
    source_permission_id: value("upload_permission_id"),
    approval_id: value("upload_approval_id") || null,
    filename: value("upload_filename") || "manual-note.txt",
    mime_type: "text/plain",
    content_base64: btoa(unescape(encodeURIComponent(value("upload_text"))))
  }));

  bind("[data-dispatch]", () => postJson("/work-items/" + value("dispatch_work_item_id") + "/dispatch", {}));

  bind("[data-local-project]", () => postJson("/collection-runs/local-project", {
    source_id: value("local_source_id"),
    source_permission_id: value("local_permission_id"),
    approval_id: value("local_approval_id") || null,
    requested_by_actor_id: "local-owner"
  }));

  bind("[data-hypothesis-create]", () => postJson("/analysis/hypotheses", {
    hypothesis_text: value("hypothesis_text"),
    supporting_evidence_ids: value("hypothesis_evidence_ids").split(",").map((item) => item.trim()).filter(Boolean)
  }));

  bind("[data-experiment-status]", () => postJson("/experiments/" + value("experiment_status_id") + "/status", {
    status: value("experiment_status_value"),
    actor_id: "local-owner",
    status_note: value("experiment_status_note") || null
  }));

  bind("[data-graph-ensure]", () => postJson("/memory/graph/schema/ensure", {}));
  bind("[data-graph-sync]", () => postJson("/memory/graph/lineage/sync", {}));
  bind("[data-vector-ensure]", () => postJson("/memory/vector/chunks/ensure", {}));

  bind("[data-answer]", () => postJson("/chat/evidence-answer", {
    message: value("answer_message"),
    limit: Number(value("answer_limit") || 5)
  }));

  bind("[data-review]", async () => {
    const action = value("review_action");
    if (action === "feedback") {
      return postJson("/feedback", {
        target_type: value("review_target_type"),
        target_id: value("review_target_id"),
        label: value("review_label"),
        note: value("review_note") || null
      });
    }
    if (action === "outcome") {
      return postJson("/outcomes", {
        target_type: value("review_target_type"),
        target_id: value("review_target_id"),
        outcome_status: value("review_label"),
        summary: value("review_note") || null
      });
    }
    return postJson("/analysis/patterns/" + value("review_target_id") + "/review", {
      status: value("review_label"),
      review_note: value("review_note") || null
    });
  });

  bind("[data-pattern-detect]", () => postJson("/analysis/patterns/detect-baseline", {
    recurrence_threshold: Number(value("pattern_threshold") || 3)
  }));

  bind("[data-report]", async () => {
    if (value("report_action") === "render") {
      return postJson("/reports/" + value("report_id") + "/render", { notes: value("report_notes") || null });
    }
    return postJson("/reports", {
      title: value("report_title") || "MVP report",
      report_type: value("report_type") || "summary"
    });
  });
})();
`;

  return (
    <section className="panel actionConsole" data-mvp-actions data-api-base-url={browserApiBaseUrl}>
      <div className="panelHeader">
        <h2>Advanced Route Console</h2>
        <span className="statusText">Advanced route controls</span>
      </div>
      <section className="actionGrid">
        <form className="actionBox" data-create-source>
          <h3><HelpHeading term="source">Source</HelpHeading></h3>
          <p className="actionHint"><TermHelp term="sourcePermission" label="Source Permission" /> controls scope, operations, approval, and external model policy.</p>
          <input name="source_name" placeholder="Source name" />
          <label className="fieldWithHelp">
            <TermHelp term="sourceType" label="Source Type" />
            <select name="source_type" defaultValue="manual_upload">
              <option value="manual_upload">manual_upload</option>
              <option value="local_project">local_project</option>
              <option value="conversation_history">conversation_history</option>
              <option value="user_observation">user_observation</option>
            </select>
          </label>
          <input name="source_location" placeholder="Location" />
          <select name="source_sensitivity" defaultValue="internal">
            <option value="public">public</option>
            <option value="internal">internal</option>
            <option value="sensitive">sensitive</option>
            <option value="secret">secret</option>
          </select>
          <label className="fieldWithHelp"><TermHelp term="allowedOperations" label="Allowed Operations" /><input name="source_allowed_operations" defaultValue="dry_run,read,collect" /></label>
          <label className="fieldWithHelp"><TermHelp term="permissionScope" label="Permission Scope" /><textarea name="source_scope_json" rows={2} defaultValue="{}" /></label>
          <p className="actionHint"><TermHelp term="manualUpload" label="Manual Upload" /> and <TermHelp term="localProject" label="Local Project" /> are the currently useful collection paths.</p>
          <label className="checkLine"><input name="source_approval_required" type="checkbox" defaultChecked /> <TermHelp term="approvalRequired" label="Approval required" /></label>
          <button type="submit">Create Source</button>
        </form>

        <form className="actionBox" data-approval>
          <h3><HelpHeading term="approval">Approval</HelpHeading></h3>
          <input name="approval_request_type" defaultValue="manual_upload_collection" />
          <input name="approval_source_id" placeholder="Source ID" />
          <input name="approval_permission_id" placeholder="Permission ID" />
          <input name="approval_operation" defaultValue="manual_upload_collection" />
          <button type="submit">Request</button>
        </form>

        <form className="actionBox" data-approval-decision>
          <h3>Decision</h3>
          <input name="decision_approval_id" placeholder="Approval ID" />
          <select name="decision_status" defaultValue="approved">
            <option value="approved">approved</option>
            <option value="denied">denied</option>
          </select>
          <input name="decision_reason" placeholder="Reason" />
          <button type="submit">Decide</button>
        </form>

        <form className="actionBox" data-dry-run>
          <h3><HelpHeading term="dryRun">Dry-Run</HelpHeading></h3>
          <input name="dry_source_id" placeholder="Source ID" />
          <input name="dry_permission_id" placeholder="Permission ID" />
          <button type="submit">Run</button>
        </form>

        <form className="actionBox wide" data-manual-upload>
          <h3><HelpHeading term="manualUpload">Manual Upload</HelpHeading></h3>
          <input name="upload_source_id" placeholder="Source ID" />
          <input name="upload_permission_id" placeholder="Permission ID" />
          <input name="upload_approval_id" placeholder="Approval ID" />
          <input name="upload_filename" defaultValue="manual-note.txt" />
          <textarea name="upload_text" rows={4} placeholder="Authorized text content" />
          <button type="submit">Collect</button>
        </form>

        <form className="actionBox" data-dispatch>
          <h3><HelpHeading term="dispatch">Dispatch</HelpHeading></h3>
          <input name="dispatch_work_item_id" placeholder="Queued work item ID" />
          <button type="submit">Dispatch</button>
        </form>

        <form className="actionBox" data-local-project>
          <h3><HelpHeading term="localProject">Local Project Collect</HelpHeading></h3>
          <input name="local_source_id" placeholder="local_project source ID" />
          <input name="local_permission_id" placeholder="Permission ID" />
          <input name="local_approval_id" placeholder="Approval ID (optional)" />
          <button type="submit">Collect Directory</button>
        </form>

        <form className="actionBox" data-hypothesis-create>
          <h3><HelpHeading term="hypothesis">Hypothesis</HelpHeading></h3>
          <textarea name="hypothesis_text" rows={2} placeholder="Hypothesis text" />
          <input name="hypothesis_evidence_ids" placeholder="evidence-id-1,evidence-id-2" />
          <button type="submit">Create</button>
        </form>

        <form className="actionBox" data-experiment-status>
          <h3><HelpHeading term="experimentRun">Experiment Status</HelpHeading></h3>
          <input name="experiment_status_id" placeholder="Experiment ID" />
          <select name="experiment_status_value" defaultValue="completed">
            <option value="planned">planned</option>
            <option value="running">running</option>
            <option value="completed">completed</option>
            <option value="failed">failed</option>
            <option value="abandoned">abandoned</option>
          </select>
          <input name="experiment_status_note" placeholder="Status note" />
          <button type="submit">Update</button>
        </form>

        <form className="actionBox" data-graph-ensure>
          <h3>Graph Schema</h3>
          <button type="submit">Ensure Neo4j Schema</button>
        </form>

        <form className="actionBox" data-graph-sync>
          <h3>Graph Sync</h3>
          <button type="submit">Sync Lineage</button>
        </form>

        <form className="actionBox" data-vector-ensure>
          <h3>Vector Memory</h3>
          <button type="submit">Ensure Vector Chunks</button>
        </form>

        <form className="actionBox wide" data-answer>
          <h3><HelpHeading term="evidenceAnswer">Evidence Answer</HelpHeading></h3>
          <textarea name="answer_message" rows={3} defaultValue="What does the system know?" />
          <input name="answer_limit" type="number" min="1" max="50" defaultValue="5" />
          <button type="submit">Build Evidence-Grounded Answer</button>
        </form>

        <form className="actionBox" data-review>
          <h3>Review</h3>
          <select name="review_action" defaultValue="feedback">
            <option value="feedback">feedback</option>
            <option value="outcome">outcome</option>
            <option value="pattern_review">pattern_review</option>
          </select>
          <input name="review_target_type" defaultValue="prediction" />
          <input name="review_target_id" placeholder="Target ID" />
          <input name="review_label" defaultValue="useful" />
          <input name="review_note" placeholder="Note" />
          <button type="submit">Record</button>
        </form>

        <form className="actionBox" data-pattern-detect>
          <h3><HelpHeading term="pattern">Patterns</HelpHeading></h3>
          <input name="pattern_threshold" type="number" min="2" max="20" defaultValue="3" />
          <button type="submit">Detect</button>
        </form>

        <form className="actionBox wide" data-report>
          <h3>Reports</h3>
          <select name="report_action" defaultValue="create">
            <option value="create">create</option>
            <option value="render">render</option>
          </select>
          <input name="report_id" placeholder="Report ID for render" />
          <input name="report_title" defaultValue="MVP report" />
          <select name="report_type" defaultValue="summary">
            <option value="summary">summary</option>
            <option value="evidence_review">evidence_review</option>
            <option value="handoff">handoff</option>
            <option value="experiment_summary">experiment_summary</option>
          </select>
          <textarea name="report_notes" rows={2} placeholder="Render notes" />
          <button type="submit">Run</button>
        </form>
      </section>
      <pre className="actionOutput" data-action-output>Action results appear here.</pre>
      <ClientScript script={script} />
    </section>
  );
}
