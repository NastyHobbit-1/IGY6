import type { ApprovalRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function SourceCollectionApprovalReview({ approvals }: { approvals: ApiResult<ApprovalRecord[]> }) {
  const browserApiBaseUrl = "/api";
  const collectionApprovals = approvals.data
    .filter((approval) => approval.request_type === "manual_upload_collection" || approval.request_type === "agent_action")
    .slice(0, 12);
  const pendingCollectionApprovals = collectionApprovals.filter((approval) => approval.status === "pending");
  const approvalsJson = JSON.stringify(collectionApprovals).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-source-collection-approval-review]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const result = root.querySelector("[data-source-collection-approval-result]");
  const buttons = root.querySelectorAll("[data-approval-decision-button]");
  const show = (title, payload) => {
    if (!result) return;
    result.textContent = title + "\\n" + JSON.stringify(payload, null, 2);
  };
  buttons.forEach((button) => {
    button.addEventListener("click", async () => {
      const approvalId = button.getAttribute("data-approval-id");
      const status = button.getAttribute("data-decision-status");
      if (!approvalId || !status) return;
      show("Saving approval decision", { approval_status: status });
      try {
        const response = await fetch(apiBaseUrl + "/approvals/" + encodeURIComponent(approvalId) + "/decision", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            status,
            decision_reason: status === "approved"
              ? "Approved from normal Settings source onboarding review"
              : "Denied from normal Settings source onboarding review"
          })
        });
        const payload = await response.json().catch(() => ({}));
        show(response.ok ? "Approval decision saved" : "Approval decision failed", payload);
        if (response.ok) {
          button.closest("[data-collection-approval-item]")?.setAttribute("data-state", status);
          buttons.forEach((candidate) => {
            if (candidate.getAttribute("data-approval-id") === approvalId) candidate.disabled = true;
          });
        }
      } catch (error) {
        show("Approval decision error", { detail: error instanceof Error ? error.message : "Unknown error" });
      }
    });
  });
})();
`;

  return (
    <section className="guidedManualText sourceCollectionApprovals" data-source-collection-approval-review data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Collection and agent action approvals.</strong>
        <span>Review pending manual/conversation/observation collection requests and agent action requests. After approval, return to the same workflow or Action engine to execute.</span>
      </div>
      {approvals.error ? <p className="errorText">Approval list could not be loaded: {approvals.error}</p> : null}
      <div className="stack">
        {collectionApprovals.map((approval) => {
          const payload = approval.request_payload_json ?? {};
          const sourceType = typeof payload.source_type === "string" ? payload.source_type : approval.request_type === "agent_action" ? "agent action" : "manual_upload";
          const filename = typeof payload.filename === "string" ? payload.filename : typeof payload.action_name === "string" ? payload.action_name : "no filename recorded";
          return (
            <article className="item evidenceItem" key={approval.id} data-collection-approval-item data-state={approval.status}>
              <div>
                <strong>{approval.request_type === "agent_action" ? "Agent action" : sourceType.replaceAll("_", " ") + " collection"}</strong>
                <span>{filename} · requested by {approval.requested_by_actor_id}</span>
              </div>
              <div>
                <StatusPill state={approval.status} />
                <span>{approval.status === "pending" ? "decision needed" : approval.decision_reason ?? "decided"}</span>
              </div>
              {approval.status === "pending" ? (
                <div className="guidedManualActions">
                  <button type="button" data-approval-decision-button data-approval-id={approval.id} data-decision-status="approved">Approve collection</button>
                  <button type="button" data-approval-decision-button data-approval-id={approval.id} data-decision-status="denied">Deny</button>
                </div>
              ) : null}
            </article>
          );
        })}
      </div>
      {collectionApprovals.length === 0 ? <EmptyState label="No source collection approvals recorded yet." /> : null}
      {pendingCollectionApprovals.length === 0 && collectionApprovals.length > 0 ? <p className="actionHint">No source collection approval is waiting for a decision.</p> : null}
      <details className="advancedPanel">
        <summary>Details: collection approval records for audit</summary>
        <pre>{JSON.stringify(collectionApprovals, null, 2)}</pre>
        <pre data-source-collection-approval-result>Decision results appear here.</pre>
      </details>
      <DomJsonScript marker="data-source-collection-approvals-json" json={approvalsJson} />
      <ClientScript script={script} />
    </section>
  );
}

