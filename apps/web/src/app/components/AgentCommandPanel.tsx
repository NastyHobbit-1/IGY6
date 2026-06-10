import type { AgentTaskPlanRecord, ApprovalRecord, AgentCapabilitiesResponse, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";

export function AgentCommandPanel({
  capabilities,
  approvals,
  taskPlans
}: {
  capabilities: ApiResult<AgentCapabilitiesResponse>;
  approvals: ApiResult<ApprovalRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
}) {
  const data = capabilities.data;
  const recentTaskPlans = taskPlans.data.slice(0, 5);
  const stackActions = data.actions.filter((action) => action.script_backed);
  const approvedAgentApprovals = approvals.data.filter((approval) => approval.request_type === "agent_action" && approval.status === "approved");
  const pendingAgentApprovals = approvals.data.filter((approval) => approval.request_type === "agent_action" && approval.status === "pending");
  const actionLabels: Record<string, string> = {
    show_project_health: "Show project health",
    show_git_status: "Show git status",
    show_latest_diff: "Show latest DIFF",
    show_work_items: "Show work items",
    run_retrieval_preview: "Run retrieval preview",
    start_stack: "Start stack",
    stop_stack: "Stop stack",
    run_last_healthy_stack: "Run last healthy stack"
  };
  const script = `
(() => {
  const root = document.querySelector("[data-agent-command]");
  if (!root) return;

  const commandInput = root.querySelector("[data-agent-command-input]");
  const paramsInput = root.querySelector("[data-agent-params]");
  const approvalInput = root.querySelector("[data-agent-approval-id]");
	  const previewButton = root.querySelector("[data-agent-preview]");
	  const executeButton = root.querySelector("[data-agent-execute]");
	  const approvalButton = root.querySelector("[data-agent-request-approval]");
	  const savePlanButton = root.querySelector("[data-agent-save-plan]");
	  const evidenceButton = root.querySelector("[data-agent-check-evidence]");
	  const planEvidenceButtons = root.querySelectorAll("[data-agent-plan-check-evidence]");
	  const proposeWorkSpecButtons = root.querySelectorAll("[data-agent-plan-propose-work-spec]");
	  const executeApprovedButton = root.querySelector("[data-agent-execute-approved]");
	  const actionSelect = root.querySelector("[data-agent-action-select]");
	  const approvalSelect = root.querySelector("[data-agent-approval-select]");
	  const intentPanel = root.querySelector("[data-agent-intent]");
	  const resultPanel = root.querySelector("[data-agent-result]");
	  const statusPanel = root.querySelector("[data-agent-status]");
	  const bridgePanel = root.querySelector("[data-agent-approval-bridge]");
	  const summaryPanel = root.querySelector("[data-agent-understanding-summary]");
	  const categoryPanel = root.querySelector("[data-agent-understanding-category]");
	  const posturePanel = root.querySelector("[data-agent-understanding-posture]");
	  const nextStepPanel = root.querySelector("[data-agent-understanding-next]");
	  const plannerPanel = root.querySelector("[data-agent-intake-planner]");
	  const evidencePanel = root.querySelector("[data-agent-planner-evidence]");
	  const capabilitiesPayload = JSON.parse(root.querySelector("[data-agent-capabilities-json]")?.textContent || "{}");
	  const approvalPayload = JSON.parse(root.querySelector("[data-agent-approvals-json]")?.textContent || "[]");
	  let latestIntent = null;
	  let latestTaskPlanId = null;

  const showJson = (node, label, payload) => {
    if (!node) return;
    node.textContent = label + "\\n" + JSON.stringify(payload, null, 2);
  };

  const parseParams = () => {
    const raw = paramsInput?.value?.trim() || "{}";
    if (!raw) return {};
    return JSON.parse(raw);
  };

	  const capabilityFor = (actionName) => {
	    return (capabilitiesPayload.actions || []).find((action) => action.name === actionName) || null;
	  };

	  const approvalMatches = (approval, actionName, parameters) => {
	    const payload = approval?.request_payload_json || {};
	    return approval?.status === "approved"
	      && approval?.request_type === "agent_action"
	      && payload.action_name === actionName
	      && JSON.stringify(payload.parameters || {}) === JSON.stringify(parameters || {});
	  };

	  const matchingApproval = (actionName, parameters) => {
	    return (approvalPayload || []).find((approval) => approvalMatches(approval, actionName, parameters)) || null;
	  };

	  const renderBridge = (intent) => {
	    if (!bridgePanel) return;
	    const parameters = (() => {
	      try { return parseParams(); } catch (_) { return {}; }
	    })();
	    const actionName = intent?.proposed_action || actionSelect?.value || "";
	    const capability = actionName ? capabilityFor(actionName) : null;
	    const approval = actionName ? matchingApproval(actionName, parameters) : null;
	    if (approval && approvalInput) approvalInput.value = approval.id;
	    const actionLabel = actionName || "No action selected";
	    const approvalState = !capability
	      ? "unsupported"
	      : capability.approval_required
	        ? approval
	          ? "approved"
	          : "approval needed"
	        : "not required";
	    bridgePanel.textContent = "Action: " + actionLabel
	      + " | class: " + (capability?.action_type || "unknown")
	      + " | approval: " + approvalState
	      + " | execution: " + (capability?.executable_in_api_runtime === false ? "runtime blocked" : "bounded route only");
	  };

	  const plannerCopy = (understanding, intent, capability) => {
	    if (understanding.unsupported_or_unsafe) {
	      return {
	        state: "unsupported",
	        title: "Unsupported or unsafe as written",
	        body: understanding.reason || intent.reason || "IGY6 will not turn this request into work or execution.",
	        next: "Rewrite the request as evidence review, data intake, report creation, feedback, outcome recording, or a listed bounded action."
	      };
	    }
	    if (understanding.clarification_needed) {
	      return {
	        state: "clarification-needed",
	        title: "Clarification needed",
	        body: (understanding.missing_information || []).join(", ") || "IGY6 needs more detail before it can choose a safe next step.",
	        next: understanding.next_step || "Add the missing target, evidence scope, or desired output."
	      };
	    }
	    if (understanding.approval_required || intent.approval_required) {
	      return {
	        state: "approval-required",
	        title: "Approval required before action",
	        body: capability?.interpreted_intent || "This request may affect the local runtime or another sensitive workflow.",
	        next: "Review the proposed bounded action and create an approval only if it matches what you want."
	      };
	    }
	    if (understanding.evidence_required) {
	      return {
	        state: "evidence-needed",
	        title: "Evidence needed",
	        body: "IGY6 should use stored local evidence before answering or creating a report.",
	        next: "Use Ask over evidence, or add/process more data if retrieval has no matches."
	      };
	    }
	    if (understanding.work_item_should_be_created) {
	      return {
	        state: "work-confirmation",
	        title: "May become bounded work after confirmation",
	        body: "The request looks like a workflow request, but this planner does not create work in this DIFF.",
	        next: understanding.next_step || "Review the request and use an existing supported workflow."
	      };
	    }
	    return {
	      state: intent.proposed_action ? "bounded-action" : "review-only",
	      title: intent.proposed_action ? "Bounded action matched" : "Review next step",
	      body: capability?.interpreted_intent || understanding.wants || "IGY6 can summarize the request posture.",
	      next: understanding.next_step || "Use the existing visible workflow that matches this category."
	    };
	  };

	  const planStatusFor = (understanding, intent) => {
	    if (understanding.unsupported_or_unsafe) return "unsupported";
	    if (understanding.clarification_needed) return "needs_clarification";
	    if (understanding.approval_required || intent.approval_required) return "approval_required";
	    if (understanding.evidence_required) return "evidence_needed";
	    return "proposed";
	  };

	  const supportedStateFor = (understanding, intent) => {
	    if (understanding.unsupported_or_unsafe) return "unsupported";
	    if (understanding.clarification_needed) return "clarification_needed";
	    if (understanding.approval_required || intent.approval_required) return "approval_required";
	    if (understanding.evidence_required) return "evidence_needed";
	    return "supported";
	  };

	  const boundedWorkSpecFor = (understanding, copy) => {
	    if (understanding?.category !== "create_report" || understanding.unsupported_or_unsafe) return null;
	    const summary = understanding.wants || commandInput?.value?.trim() || "Agent task plan report request";
	    return {
	      work_type: "report_generation",
	      expected_output: (copy.next || "Create a bounded report from this task plan.").slice(0, 1000),
	      payload_json: {
	        report_type: "agent_task_plan",
	        requested_summary: summary.slice(0, 1000),
	        intent_category: "create_report"
	      },
	      proposal_source: "agent_intake_planner",
	      safety_constraints: [
	        "Supported report_generation work item type only.",
	        "No shell command or user-provided argv.",
	        "Work creation remains approval-gated when approval is required."
	      ]
	    };
	  };

	  const taskPlanPayload = (intent) => {
	    const understanding = intent?.request_understanding || {};
	    const capability = intent?.proposed_action ? capabilityFor(intent.proposed_action) : null;
	    const copy = plannerCopy(understanding, intent || {}, capability);
	    const workSpec = boundedWorkSpecFor(understanding, copy);
	    const requestSummary = understanding.wants || commandInput?.value?.trim() || "Agent task plan preview";
	    const requiredEvidence = understanding.evidence_required
	      ? ["Check stored local evidence before creating work or answering."]
	      : [];
	    return {
	      user_request_summary: requestSummary.slice(0, 1000),
	      intent_category: understanding.category || "unclear",
	      status: workSpec ? (understanding.approval_required || intent?.approval_required ? "approval_required" : "ready") : planStatusFor(understanding, intent || {}),
	      proposed_steps: [copy.next || understanding.next_step || "Review the safe next step."],
	      required_evidence: requiredEvidence,
	      approval_required: Boolean(understanding.approval_required || intent?.approval_required),
	      supported_state: workSpec ? "supported" : supportedStateFor(understanding, intent || {}),
	      next_safe_action: (understanding.next_step || copy.next || "Review the safe next step.").slice(0, 1000),
	      requested_by_actor_id: "local-owner",
	      metadata_json: {
	        created_from: "agent_intake_planner",
	        proposed_action: intent?.proposed_action || null,
	        work_item_should_be_created: Boolean(understanding.work_item_should_be_created),
	        unsupported_or_unsafe: Boolean(understanding.unsupported_or_unsafe),
	        saved_preview_only: !workSpec,
	        ...(workSpec ? { plan_to_work: workSpec } : {})
	      }
	    };
	  };

	  const evidenceLabel = (hit, index) => {
	    const evidence = Array.isArray(hit.evidence_items) ? hit.evidence_items[0] : null;
	    const parts = [];
	    if (evidence?.id) parts.push("evidence " + evidence.id);
	    if (hit.chunk?.id || hit.qdrant_payload?.chunk_id) parts.push("chunk " + (hit.chunk?.id || hit.qdrant_payload?.chunk_id));
	    if (hit.document?.id || hit.qdrant_payload?.document_id) parts.push("document " + (hit.document?.id || hit.qdrant_payload?.document_id));
	    if (hit.source?.id) parts.push("source " + hit.source.id);
	    return parts.length ? parts.join(" | ") : "hit " + (index + 1);
	  };

	  const renderEvidenceSummary = (payload) => {
	    const hits = Array.isArray(payload?.retrieval_context?.hits) ? payload.retrieval_context.hits : [];
	    const labels = hits.slice(0, 5).map(evidenceLabel);
	    const summary = {
	      answer_status: payload?.answer_status || "unknown",
	      retrieved_count: hits.length,
	      labels,
	      missing_evidence: hits.length === 0
	    };
	    if (evidencePanel) {
	      evidencePanel.innerHTML = "";
	      addPlannerRow(
	        evidencePanel,
	        hits.length > 0 ? "Evidence check" : "Missing evidence",
	        hits.length > 0
	          ? "Retrieved " + hits.length + " relevant local evidence hit(s)."
	          : "No relevant local evidence was retrieved. Add/process data or narrow the request before proceeding.",
	        hits.length > 0 ? "retrieved" : "missing"
	      );
	      labels.forEach((label) => addPlannerRow(evidencePanel, "Evidence label", label, "safe-id"));
	    }
	    return summary;
	  };

	  const evidenceSummaryPayload = (summary) => ({
	    actor_id: "local-owner",
	    answer_status: summary.answer_status || "unknown",
	    retrieved_count: Number(summary.retrieved_count || 0),
	    safe_labels: Array.isArray(summary.labels) ? summary.labels.slice(0, 5) : [],
	    missing_evidence: Boolean(summary.missing_evidence),
	    missing_evidence_guidance: summary.missing_evidence
	      ? "No relevant local evidence was retrieved. Add/process data or narrow the request before proceeding."
	      : "Relevant local evidence was retrieved. Review safe labels before creating work or answering."
	  });

	  const persistEvidenceSummary = async (taskPlanId, summary) => {
	    const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/evidence-summary", {
	      method: "POST",
	      headers: { "Content-Type": "application/json" },
	      body: JSON.stringify(evidenceSummaryPayload(summary))
	    });
	    const payload = await response.json();
	    if (!response.ok) {
	      throw new Error(payload?.detail || response.statusText || "Evidence summary persistence failed");
	    }
	    return payload;
	  };

	  const addPlannerRow = (parent, label, value, state) => {
	    const item = document.createElement("article");
	    item.className = "agentPlannerCard";
	    item.setAttribute("data-agent-planner-card", label);
	    const title = document.createElement("strong");
	    title.textContent = label;
	    const body = document.createElement("span");
	    body.textContent = value || "not returned";
	    item.append(title, body);
	    if (state) {
	      const pill = document.createElement("em");
	      pill.textContent = state;
	      item.appendChild(pill);
	    }
	    parent.appendChild(item);
	  };

	  const renderPlanner = (intent) => {
	    if (!plannerPanel) return;
	    plannerPanel.innerHTML = "";
	    const understanding = intent?.request_understanding;
	    if (!understanding) {
	      addPlannerRow(plannerPanel, "Planner", "Preview a request to see the next safe step.", "waiting");
	      return;
	    }
	    const capability = intent.proposed_action ? capabilityFor(intent.proposed_action) : null;
	    const copy = plannerCopy(understanding, intent, capability);
	    addPlannerRow(plannerPanel, "Status", copy.title, copy.state);
	    addPlannerRow(plannerPanel, "Category", understanding.category || "unclear", understanding.category || "unclear");
	    addPlannerRow(plannerPanel, "Evidence", understanding.evidence_required ? "Stored evidence should be checked first." : "No evidence lookup required before this preview.", understanding.evidence_required ? "needed" : "not-needed");
	    addPlannerRow(plannerPanel, "Approval", understanding.approval_required || intent.approval_required ? "Explicit approval is required before execution." : "No approval is required for this preview.", understanding.approval_required || intent.approval_required ? "required" : "not-required");
	    addPlannerRow(plannerPanel, "Next safe step", copy.next, "guidance");
	  };

  const renderUnderstanding = (intent) => {
    const understanding = intent?.request_understanding;
    if (!understanding) return;
    if (summaryPanel) summaryPanel.textContent = understanding.wants || "IGY6 needs more detail before it can continue.";
    if (categoryPanel) categoryPanel.textContent = "Category: " + (understanding.category || "unclear");
    const posture = [];
    posture.push(understanding.evidence_required ? "Evidence needed" : "No evidence lookup required first");
    posture.push(understanding.clarification_needed ? "Needs clarification" : "Clear enough to preview");
    posture.push(understanding.approval_required ? "Approval required" : "No approval required for preview");
    posture.push(understanding.work_item_should_be_created ? "May become work after confirmation" : "No work item now");
    posture.push(understanding.unsupported_or_unsafe ? "Unsupported or unsafe as written" : "Supported posture");
	    if (posturePanel) posturePanel.textContent = posture.join(" | ");
	    if (nextStepPanel) nextStepPanel.textContent = understanding.next_step || "";
	    renderPlanner(intent);
	    renderBridge(intent);
	  };

  const setButtons = () => {
    if (!executeButton || !approvalButton || !executeApprovedButton) return;
    executeButton.disabled = true;
    approvalButton.disabled = true;
    if (savePlanButton) savePlanButton.disabled = !latestIntent?.request_understanding;
    if (evidenceButton) evidenceButton.disabled = !latestIntent?.request_understanding;
    executeApprovedButton.disabled = true;
    if (!latestIntent?.proposed_action || latestIntent.missing_parameters?.length) return;
    const capability = capabilityFor(latestIntent.proposed_action);
    const runtimeExecutable = capability?.executable_in_api_runtime !== false;
    if (latestIntent.approval_required) {
      approvalButton.disabled = false;
      executeApprovedButton.disabled = !(runtimeExecutable && approvalInput?.value?.trim());
      return;
    }
    executeButton.disabled = !(latestIntent.executable_now && runtimeExecutable);
  };

  const previewIntent = async () => {
    const parameters = parseParams();
    const response = await fetch("/api/agent/intent", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message: commandInput?.value || "", parameters, actor_id: "local-owner" })
    });
    const payload = await response.json();
    latestIntent = payload;
    renderUnderstanding(payload);
    showJson(intentPanel, response.ok ? "Agent intent preview" : "Intent preview failed", payload);
    const capability = payload.proposed_action ? capabilityFor(payload.proposed_action) : null;
    const runtimeNote = capability?.reason || capabilitiesPayload.runtime?.reason || "Runtime allows this action class.";
	    if (statusPanel) {
	      statusPanel.textContent = payload.proposed_action
	        ? "Runtime: " + (capability?.executable_in_api_runtime ? "executable" : "blocked") + " | " + runtimeNote
	        : "Rejected by typed registry. No shell command will run.";
	    }
	    if (payload.proposed_action && actionSelect) actionSelect.value = payload.proposed_action;
	    setButtons();
	  };

  previewButton?.addEventListener("click", async () => {
    try {
      await previewIntent();
    } catch (error) {
      showJson(intentPanel, "Intent preview error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  executeButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || latestIntent.approval_required) return;
      const response = await fetch("/api/agent/actions/" + encodeURIComponent(latestIntent.proposed_action) + "/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ parameters: parseParams(), actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Read-only action result" : "Action failed", payload);
    } catch (error) {
      showJson(resultPanel, "Action error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  approvalButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || !latestIntent.approval_required) return;
      const response = await fetch("/api/approvals", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          request_type: "agent_action",
          requested_by_actor_id: "local-owner",
          request_payload_json: {
            action_name: latestIntent.proposed_action,
            parameters: parseParams()
          }
        })
      });
      const payload = await response.json();
	      if (payload?.id && approvalInput) approvalInput.value = payload.id;
	      showJson(resultPanel, response.ok ? "Approval request created" : "Approval request failed", payload);
	      if (bridgePanel) bridgePanel.textContent = response.ok
	        ? "Approval requested. Review it in Settings before running the approved action."
	        : "Approval request failed. No action was executed.";
	      setButtons();
    } catch (error) {
      showJson(resultPanel, "Approval request error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  savePlanButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.request_understanding) return;
      const response = await fetch("/api/agent/task-plans", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(taskPlanPayload(latestIntent))
      });
      const payload = await response.json();
      if (response.ok && payload?.id) latestTaskPlanId = payload.id;
      showJson(resultPanel, response.ok ? "Task plan saved" : "Task plan save failed", payload);
      if (statusPanel) {
        statusPanel.textContent = response.ok
          ? "Saved task plan metadata. No work item was created and no action was executed."
          : "Task plan was not saved. No work item was created and no action was executed.";
      }
    } catch (error) {
      showJson(resultPanel, "Task plan save error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  evidenceButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.request_understanding) return;
      evidenceButton.disabled = true;
      if (evidencePanel) evidencePanel.textContent = "Checking local evidence...";
      const response = await fetch("/api/chat/retrieval-preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: commandInput?.value || latestIntent.request_understanding.wants || "",
          limit: 5
        })
      });
      const payload = await response.json();
      if (!response.ok) {
        showJson(resultPanel, "Evidence check failed", { detail: payload?.detail || response.statusText });
        if (evidencePanel) evidencePanel.textContent = "Evidence check failed. No plan action was taken.";
        return;
      }
      const summary = renderEvidenceSummary(payload);
      if (latestTaskPlanId) {
        try {
          await persistEvidenceSummary(latestTaskPlanId, summary);
          showJson(resultPanel, "Evidence check summary saved to task plan", summary);
          if (statusPanel) statusPanel.textContent = "Saved safe evidence summary on the latest task plan. Reload to review persisted evidence readiness.";
        } catch (persistError) {
          showJson(resultPanel, "Evidence check summary persistence failed", {
            summary,
            detail: persistError instanceof Error ? persistError.message : "Unknown persistence error"
          });
        }
      } else {
        showJson(resultPanel, "Evidence check summary", summary);
      }
    } catch (error) {
      showJson(resultPanel, "Evidence check error", { detail: error instanceof Error ? error.message : "Unknown error" });
      if (evidencePanel) evidencePanel.textContent = "Evidence check failed. No plan action was taken.";
    } finally {
      evidenceButton.disabled = false;
    }
  });

	  root.querySelectorAll("[data-agent-plan-create-work]").forEach((button) => {
    button.addEventListener("click", async () => {
      const taskPlanId = button.getAttribute("data-task-plan-id");
      if (!taskPlanId) return;
      button.disabled = true;
      try {
        const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/work-item", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ actor_id: "local-owner", approval_id: null })
        });
        const payload = await response.json();
        showJson(resultPanel, response.ok ? "Work item created from plan" : "Plan-to-work blocked", payload);
        if (statusPanel) {
          statusPanel.textContent = response.ok
            ? "Created a work item from the persisted task plan. It still requires the normal work queue safety flow."
            : "Plan-to-work was blocked. No action was executed.";
        }
      } catch (error) {
        showJson(resultPanel, "Plan-to-work error", { detail: error instanceof Error ? error.message : "Unknown error" });
      } finally {
        button.disabled = false;
      }
    });
  });

  executeApprovedButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || !latestIntent.approval_required || !approvalInput?.value?.trim()) return;
      const response = await fetch("/api/agent/actions/" + encodeURIComponent(latestIntent.proposed_action) + "/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          parameters: parseParams(),
          approval_id: approvalInput.value.trim(),
          actor_id: "local-owner"
        })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Approved action result" : "Approved action blocked or failed", payload);
    } catch (error) {
      showJson(resultPanel, "Approved action error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });
	  approvalInput?.addEventListener("input", setButtons);
	  approvalSelect?.addEventListener("change", () => {
	    if (approvalInput) approvalInput.value = approvalSelect.value || "";
	    setButtons();
	  });

	  planEvidenceButtons.forEach((button) => {
	    button.addEventListener("click", async () => {
	      const taskPlanId = button.getAttribute("data-task-plan-id");
	      const query = button.getAttribute("data-task-plan-summary") || "";
	      if (!taskPlanId || !query.trim()) return;
	      button.disabled = true;
	      try {
	        const response = await fetch("/api/chat/retrieval-preview", {
	          method: "POST",
	          headers: { "Content-Type": "application/json" },
	          body: JSON.stringify({ message: query, limit: 5 })
	        });
	        const payload = await response.json();
	        if (!response.ok) {
	          showJson(resultPanel, "Task plan evidence check failed", { detail: payload?.detail || response.statusText });
	          return;
	        }
	        const summary = renderEvidenceSummary(payload);
	        await persistEvidenceSummary(taskPlanId, summary);
	        showJson(resultPanel, "Task plan evidence summary saved", summary);
	        if (statusPanel) {
	          statusPanel.textContent = "Saved safe evidence summary on task plan " + taskPlanId + ". Reload to review it in task history.";
	        }
	      } catch (error) {
	        showJson(resultPanel, "Task plan evidence summary error", { detail: error instanceof Error ? error.message : "Unknown error" });
	      } finally {
	        button.disabled = false;
	      }
	    });
	  });

	  proposeWorkSpecButtons.forEach((button) => {
	    button.addEventListener("click", async () => {
	      const taskPlanId = button.getAttribute("data-task-plan-id");
	      if (!taskPlanId) return;
	      button.disabled = true;
	      try {
	        const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/work-spec", {
	          method: "POST",
	          headers: { "Content-Type": "application/json" },
	          body: JSON.stringify({
	            actor_id: "local-owner",
	            work_type: "report_generation",
	            expected_output: "Create a bounded report from this reviewed task plan."
	          })
	        });
	        const payload = await response.json();
	        showJson(resultPanel, response.ok ? "Work spec proposed" : "Work spec proposal blocked", payload);
	        if (statusPanel) {
	          statusPanel.textContent = response.ok
	            ? "Added a bounded report_generation work spec. Reload to show work-item eligibility; no work was created."
	            : "Work spec proposal was blocked. No work item was created and no action was executed.";
	        }
	      } catch (error) {
	        showJson(resultPanel, "Work spec proposal error", { detail: error instanceof Error ? error.message : "Unknown error" });
	      } finally {
	        button.disabled = false;
	      }
	    });
	  });
	  actionSelect?.addEventListener("change", () => {
	    const selected = actionSelect.selectedOptions?.[0];
	    if (commandInput && selected?.getAttribute("data-prompt")) {
	      commandInput.value = selected.getAttribute("data-prompt");
	    }
	    latestIntent = null;
	    renderBridge({ proposed_action: actionSelect.value, approval_required: capabilityFor(actionSelect.value)?.approval_required });
	    setButtons();
	  });
	})();
	`;

  return (
    <section className="panel agentCommandPanel chatEnginePanel" id="agent-command" data-agent-command>
      <details className="chatEngineDetails" open>
        <summary>Action engine (runs from chat)</summary>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Safe local actions</p>
          <h2>Action Preview And Execution</h2>
        </div>
        <div className="topStatus">
          <StatusPill state={data.runtime.docker_control_available ? "stack-control-ready" : "stack-control-blocked"} />
          <StatusPill state="no-shell" />
          <StatusPill state="approval-gated" />
        </div>
      </div>

      <div className="agentNotice">
        <strong>Preview first.</strong>
        <span>IGY6 first summarizes what it thinks you want. Ambiguous, unsupported, or risky requests stay in clarification or approval posture instead of silently becoming work.</span>
      </div>
      {capabilities.error ? <p className="errorText">{capabilities.error}</p> : null}

      <section className="agentRuntimeGrid">
        <article><span>Docker CLI</span><strong>{data.runtime.docker_cli_available ? "available" : "unavailable"}</strong></article>
        <article><span>Docker Compose</span><strong>{data.runtime.docker_compose_available ? "available" : "unavailable"}</strong></article>
        <article><span>Docker control</span><strong>{data.runtime.docker_control_available ? "available" : "blocked"}</strong></article>
        <article><span>Socket/control path</span><strong>{data.runtime.docker_socket_available ? data.runtime.docker_socket_path ?? "configured" : "unavailable"}</strong></article>
      </section>

      {data.runtime.reason ? <p className="agentRuntimeReason">{data.runtime.reason}</p> : null}

	      <section className="agentActionList">
	        {data.actions.map((action) => (
	          <article className="agentActionCard" key={action.name}>
            <div>
              <strong>{actionLabels[action.name] ?? action.name.replaceAll("_", " ")}</strong>
              <span>{action.interpreted_intent}</span>
            </div>
            <div className="messageMeta">
              <StatusPill state={action.action_type} />
              <StatusPill state={action.approval_required ? "approval-required" : "read-only"} />
              {action.script_backed ? <StatusPill state={action.executable_in_api_runtime ? "runtime-ready" : "runtime-blocked"} /> : null}
            </div>
            {action.reason ? <p>{action.reason}</p> : null}
          </article>
	        ))}
	      </section>

	      <section className="agentApprovalBridge">
	        <div className="guidedManualNotice">
	          <strong>Approval-to-action bridge</strong>
	          <span>Choose a fixed action, preview it, then request or select a matching approval when the action requires one. No arbitrary commands or user-provided argv are accepted.</span>
	        </div>
	        <section className="agentCommandGrid">
	          <label>
	            <span>Bounded action</span>
	            <select data-agent-action-select defaultValue="show_project_health">
	              {data.actions.map((action) => (
	                <option
	                  key={action.name}
	                  value={action.name}
	                  data-prompt={actionLabels[action.name] ?? action.interpreted_intent}
	                >
	                  {actionLabels[action.name] ?? action.name.replaceAll("_", " ")} · {action.approval_required ? "approval required" : "read-only"}
	                </option>
	              ))}
	            </select>
	          </label>
	          <label>
	            <span>Approved agent approval</span>
	            <select data-agent-approval-select defaultValue="" disabled={approvedAgentApprovals.length === 0}>
	              <option value="">No approved approval selected</option>
	              {approvedAgentApprovals.map((approval) => (
	                <option key={approval.id} value={approval.id}>{approval.id}</option>
	              ))}
	            </select>
	          </label>
	        </section>
	        <p className="agentStatus" data-agent-approval-bridge>
	          {pendingAgentApprovals.length > 0
	            ? `${pendingAgentApprovals.length} pending agent approval request(s) need review in Settings.`
	            : approvedAgentApprovals.length > 0
	              ? `${approvedAgentApprovals.length} approved agent approval(s) available for matching actions.`
	              : "No approved agent action approval is available yet."}
	        </p>
	      </section>

      <section className="agentCommandGrid">
        <label>
          <span>Action request</span>
          <small>Plain English request. Example: "What did I upload today?", "Create a report about failed builds", or "Show project health."</small>
          <textarea data-agent-command-input rows={3} placeholder="Show project health." defaultValue="Show project health." />
        </label>
      </section>

      <section className="agentCommandActions">
        <button type="button" data-agent-preview>Preview action</button>
        <button type="button" data-agent-check-evidence disabled>Check evidence</button>
        <button type="button" data-agent-save-plan disabled>Save task plan</button>
        <button type="button" data-agent-execute disabled>Run safe action</button>
        <button type="button" data-agent-request-approval disabled>Request approval</button>
        <button type="button" data-agent-execute-approved disabled>Run with approval</button>
      </section>

      <p className="agentStatus" data-agent-status>
        Stack-control actions: {stackActions.every((action) => action.executable_in_api_runtime) ? "executable from API runtime" : "blocked unless API runtime has Docker CLI, Compose, and Docker control access."}
      </p>

	      <section className="agentUnderstanding">
	        <div>
	          <span>IGY6 understood this as</span>
	          <strong data-agent-understanding-summary>Preview a request to see the request summary.</strong>
	        </div>
	        <p data-agent-understanding-category>Category: not previewed</p>
	        <p data-agent-understanding-posture>Evidence, clarification, approval, and work-item posture will appear here.</p>
	        <p data-agent-understanding-next>Next step will appear here.</p>
	      </section>

	      <section className="agentPlanner" data-agent-intake-planner aria-label="Agent task intake planner">
	        <article className="agentPlannerCard" data-agent-planner-card="Planner">
	          <strong>Planner</strong>
	          <span>Preview a request to see the next safe step.</span>
	          <em>waiting</em>
	        </article>
	      </section>

	      <section className="agentPlanner" data-agent-planner-evidence aria-label="Agent planner evidence check">
	        <article className="agentPlannerCard">
	          <strong>Evidence check</strong>
	          <span>Preview a request, then check whether local evidence appears relevant before work or action proceeds.</span>
	          <em>not-checked</em>
	        </article>
	      </section>

	      <section className="agentPlanner" data-agent-task-plan-records aria-label="Persisted agent task plans">
	        {taskPlans.error ? <p className="errorText">{taskPlans.error}</p> : null}
	        {recentTaskPlans.length === 0 ? (
	          <article className="agentPlannerCard">
	            <strong>Persisted task plans</strong>
	            <span>No task plans have been saved yet. Preview a request, then save the plan metadata if it should be remembered.</span>
	            <em>empty</em>
	          </article>
	        ) : recentTaskPlans.map((plan) => {
	          const planToWork = plan.metadata_json?.plan_to_work;
	          const evidenceSummary = plan.metadata_json?.evidence_summary;
	          const evidenceSummaryObject = evidenceSummary && typeof evidenceSummary === "object" ? evidenceSummary as Record<string, unknown> : null;
	          const evidenceCount = typeof evidenceSummaryObject?.retrieved_count === "number" ? evidenceSummaryObject.retrieved_count : null;
	          const evidenceStatus = typeof evidenceSummaryObject?.answer_status === "string" ? evidenceSummaryObject.answer_status : null;
	          const evidenceLabels = Array.isArray(evidenceSummaryObject?.safe_labels)
	            ? evidenceSummaryObject.safe_labels.filter((label): label is string => typeof label === "string").slice(0, 3)
	            : [];
	          const hasWorkSpec = Boolean(planToWork && typeof planToWork === "object" && "work_type" in planToWork);
	          const workType = hasWorkSpec && planToWork && typeof planToWork === "object" && "work_type" in planToWork
	            ? String((planToWork as { work_type?: unknown }).work_type ?? "unknown")
	            : null;
	          const eligibleForWork = hasWorkSpec
	            && plan.supported_state === "supported"
	            && !plan.approval_required
	            && (plan.status === "proposed" || plan.status === "ready");
	          const canProposeReportWorkSpec = !hasWorkSpec
	            && plan.intent_category === "create_report"
	            && plan.supported_state !== "unsupported"
	            && plan.status !== "converted_to_work"
	            && plan.status !== "canceled";
	          const guidance = plan.approval_required
	            ? "Approval is required before this plan can create work."
	            : plan.supported_state !== "supported"
	              ? "This plan is not supported for work creation yet."
	              : hasWorkSpec
	                ? "This plan includes a supported " + workType + " work spec."
	                : "This plan has no supported work-item specification yet.";
	          return (
	            <article className="agentPlannerCard" key={plan.id} data-agent-task-plan-record>
	              <strong>{plan.user_request_summary}</strong>
	              <span>{plan.next_safe_action}</span>
	              <em>{plan.status} · {plan.intent_category} · {plan.approval_required ? "approval required" : "no approval required"} · {hasWorkSpec ? "eligible spec" : "preview only"}</em>
	              <span>{guidance}</span>
	              <span>Evidence readiness: {evidenceStatus ? `${evidenceStatus} · ${evidenceCount ?? 0} hit(s)` : "not checked"}</span>
	              {evidenceLabels.length > 0 ? <span>Evidence labels: {evidenceLabels.join(" | ")}</span> : null}
	              {workType ? <span>Supported work type: {workType}</span> : null}
	              <button type="button" data-agent-plan-check-evidence data-task-plan-id={plan.id} data-task-plan-summary={plan.user_request_summary}>Check and save evidence</button>
	              {canProposeReportWorkSpec ? (
	                <button type="button" data-agent-plan-propose-work-spec data-task-plan-id={plan.id}>Propose report work spec</button>
	              ) : null}
	              {eligibleForWork ? (
	                <button type="button" data-agent-plan-create-work data-task-plan-id={plan.id}>Create work item</button>
	              ) : null}
	            </article>
	          );
	        })}
	      </section>

      <details className="advancedPanel">
        <summary>Advanced: raw parameters, approval ID, response JSON, and route details</summary>
        <section className="agentCommandGrid">
          <label>
            <span>Raw parameters JSON</span>
            <small>Advanced only. Example: {"{}"}</small>
            <textarea data-agent-params rows={3} defaultValue="{}" />
          </label>
          <label>
            <span>Approval ID for approved action</span>
            <small>Paste an approval ID only after approving the matching request in Safety & Audit.</small>
            <input data-agent-approval-id placeholder="approval id after explicit approval" />
          </label>
        </section>
        <section className="agentResultGrid">
          <pre data-agent-intent>Agent intent preview appears here.</pre>
          <pre data-agent-result>Agent action result appears here.</pre>
        </section>
        <p className="routeHint">Routes used: /agent/intent, /agent/task-plans, /agent/task-plans/:id/evidence-summary, /agent/task-plans/:id/work-spec, /agent/task-plans/:id/work-item, /agent/actions/:action/execute, /approvals.</p>
      </details>
      </details>
	      <DomJsonScript marker="data-agent-capabilities-json" json={JSON.stringify(data)} />
	      <DomJsonScript marker="data-agent-approvals-json" json={JSON.stringify(approvals.data)} />
	      <ClientScript script={script} />
	    </section>
  );
}

