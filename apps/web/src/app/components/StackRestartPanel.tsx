import { ClientScript } from "@/lib/use-dom-script";

const script = `
(() => {
  const root = document.querySelector("[data-stack-restart]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const result = root.querySelector("[data-stack-restart-result]");
  const button = root.querySelector("[data-stack-restart-run]");
  const show = (message) => { if (result) result.textContent = message; };
  const postJson = async (path, body) => {
    const response = await fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body || {}),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  button?.addEventListener("click", async () => {
    button.disabled = true;
    show("Requesting restart approval...");
    try {
      const approval = await postJson("/api/approvals", {
        request_type: "agent_action",
        requested_by_actor_id: "local-owner",
        request_payload_json: {
          action_name: "restart_stack",
          parameters: {},
        },
      });
      const approvalId = approval?.id;
      if (!approvalId) throw new Error("Approval id missing");
      show("Approving restart...");
      await postJson("/api/approvals/" + encodeURIComponent(approvalId) + "/decision", {
        status: "approved",
        decided_by_actor_id: "local-owner",
        decision_reason: "Local owner requested stack rebuild to pick up code changes.",
      });
      show("Rebuilding and restarting stack (this can take several minutes)...");
      const executed = await postJson("/api/agent/actions/restart_stack/execute", {
        actor_id: "local-owner",
        approval_id: approvalId,
        parameters: {},
      });
      show("Restart finished. Refresh the page when you are ready to load the updated stack.");
    } catch (error) {
      show(String(error));
    } finally {
      button.disabled = false;
    }
  });
})();
`;

export function StackRestartPanel() {
  return (
    <section className="panelInset stackRestartPanel" data-stack-restart>
      <div className="subHeader"><h3>Restart stack</h3></div>
      <p className="actionHint">
        Rebuild and restart api, worker, and web containers after code changes. Volumes and stored data are preserved.
      </p>
      <button type="button" data-stack-restart-run>Restart running stack</button>
      <p data-stack-restart-result>Ready.</p>
      <ClientScript script={script} />
    </section>
  );
}