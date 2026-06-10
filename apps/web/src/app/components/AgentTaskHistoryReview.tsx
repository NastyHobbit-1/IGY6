import type { WorkItemRecord, AgentTaskPlanRecord, ApprovalRecord, FeedbackRecord, OutcomeRecord, ImprovementRecord, ApiResult } from "./types";
import { formatDate } from "./helpers";
import { StatusPill } from "./ui/StatusPill";

export function AgentTaskHistoryReview({
  taskPlans,
  workItems,
  approvals,
  feedback,
  outcomes,
  improvements
}: {
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  approvals: ApiResult<ApprovalRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
}) {
  const recentPlans = taskPlans.data.slice(0, 8);
  const metadataString = (metadata: Record<string, unknown> | null | undefined, key: string): string | null => {
    const value = metadata?.[key];
    return typeof value === "string" && value.trim() ? value : null;
  };
  const planApproval = (planId: string) => approvals.data.find((approval) => (
    approval.request_type === "agent_task_plan"
    && approval.request_payload_json?.task_plan_id === planId
  ));
  const linkedWorkItem = (plan: AgentTaskPlanRecord) => {
    const workItemId = metadataString(plan.metadata_json, "work_item_id");
    return workItemId ? workItems.data.find((item) => item.id === workItemId) ?? null : null;
  };
  const linkedFeedback = (workItemId: string | null) => workItemId
    ? feedback.data.find((item) => item.target_type === "work_item" && item.target_id === workItemId) ?? null
    : null;
  const linkedOutcome = (workItemId: string | null) => workItemId
    ? outcomes.data.find((item) => item.target_type === "work_item" && item.target_id === workItemId) ?? null
    : null;
  const linkedImprovement = (planId: string, workItemId: string | null) => improvements.data.find((item) => (
    item.metadata_json?.agent_task_plan_id === planId
    || (workItemId ? item.metadata_json?.work_item_id === workItemId : false)
  )) ?? null;
  const evidenceSummaryFor = (plan: AgentTaskPlanRecord): Record<string, unknown> | null => {
    const summary = plan.metadata_json?.evidence_summary;
    return summary && typeof summary === "object" ? summary as Record<string, unknown> : null;
  };

  return (
    <section className="panel workflowSection" data-agent-task-history-review>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Task history</p>
          <h2>Agent Task History And Outcomes</h2>
        </div>
        <StatusPill state={recentPlans.length > 0 ? "history-available" : "history-empty"} />
      </div>
      <p className="workflowLead">Review persisted plans, linked work, approvals, outcomes, and improvement records. Missing links are shown honestly; this surface does not create or execute work.</p>
      <section className="agentPlanner" aria-label="Recent agent task history">
        {recentPlans.length === 0 ? (
          <article className="agentPlannerCard">
            <strong>No task history yet</strong>
            <span>Saved agent task plans will appear here after the planner records them.</span>
            <em>empty</em>
          </article>
        ) : recentPlans.map((plan) => {
          const workItem = linkedWorkItem(plan);
          const workItemId = workItem?.id ?? metadataString(plan.metadata_json, "work_item_id");
          const approval = planApproval(plan.id);
          const outcome = linkedOutcome(workItemId);
          const feedbackRecord = linkedFeedback(workItemId);
          const improvement = linkedImprovement(plan.id, workItemId);
          const evidenceSummary = evidenceSummaryFor(plan);
          const evidenceStatus = typeof evidenceSummary?.answer_status === "string" ? evidenceSummary.answer_status : null;
          const evidenceCount = typeof evidenceSummary?.retrieved_count === "number" ? evidenceSummary.retrieved_count : null;
          const safeNextAction = plan.status === "converted_to_work"
            ? "Review the linked work item status before dispatch or outcome review."
            : plan.approval_required && approval?.status !== "approved"
              ? "Review or create a matching approval before creating work."
              : plan.next_safe_action;
          return (
            <article className="agentPlannerCard" key={plan.id} data-agent-task-history-item>
              <strong>{plan.user_request_summary}</strong>
              <span>{safeNextAction}</span>
              <em>{plan.status} · {plan.intent_category} · {formatDate(plan.created_at)}</em>
              <dl className="workStatusIds" aria-label={`Task history for ${plan.id}`}>
                <dt>plan</dt><dd>{plan.id}</dd>
                <dt>work item</dt><dd>{workItem ? `${workItem.id} · ${workItem.status}` : workItemId ?? "not linked"}</dd>
                <dt>approval</dt><dd>{approval ? `${approval.id} · ${approval.status}` : plan.approval_required ? "approval required, not linked" : "not required"}</dd>
                <dt>feedback</dt><dd>{feedbackRecord ? `${feedbackRecord.id} · ${feedbackRecord.label}` : "not linked"}</dd>
                <dt>outcome</dt><dd>{outcome ? `${outcome.id} · ${outcome.outcome_status}` : "not linked"}</dd>
                <dt>improvement</dt><dd>{improvement ? `${improvement.id} · ${improvement.status}` : "not linked"}</dd>
                <dt>evidence</dt><dd>{evidenceStatus ? `${evidenceStatus} · ${evidenceCount ?? 0} hit(s)` : "not checked"}</dd>
              </dl>
            </article>
          );
        })}
      </section>
    </section>
  );
}

