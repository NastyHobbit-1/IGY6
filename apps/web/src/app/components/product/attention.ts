import type { ApprovalRecord, VectorCollectionStatus, WorkItemRecord } from "../types";
import type { ProductSection } from "./helpers";

export type AttentionItem = {
  id: string;
  title: string;
  cause: string;
  actionLabel: string;
  section: ProductSection;
  scrollTarget?: string;
  technical?: string;
  retryWorkItemId?: string;
};

export function buildAttentionItems(input: {
  pendingApprovals: ApprovalRecord[];
  failedWork: WorkItemRecord[];
  searchIndexState: "ok" | "warning" | "error" | "pending";
  searchHint?: string;
  searchEnsureError?: string | null;
}): AttentionItem[] {
  const items: AttentionItem[] = [];

  for (const approval of input.pendingApprovals) {
    items.push({
      id: `approval-${approval.id}`,
      title: "Approval needed",
      cause: `${approval.request_type.replace(/_/g, " ")} is waiting for your decision.`,
      actionLabel: "Review approval",
      section: "activity",
      scrollTarget: "approvals-panel",
      technical: `approval_id=${approval.id}`,
    });
  }

  const seenWork = new Set<string>();
  for (const work of input.failedWork) {
    const key = `${work.work_type}:${work.error_message ?? "unknown"}`;
    if (seenWork.has(key)) continue;
    seenWork.add(key);
    const label =
      work.work_type === "collection_normalization"
        ? "Preparing collected information failed"
        : work.work_type === "chunk_vector_upsert"
          ? "Search indexing failed"
          : work.work_type === "document_chunking"
            ? "Breaking documents into pieces failed"
            : "Background work failed";
    const cause = work.error_message
      ? work.error_message.slice(0, 220)
      : "The background worker reported a failure without a detailed message.";
    items.push({
      id: `work-${work.id}`,
      title: label,
      cause,
      actionLabel: "Retry in Activity",
      section: "activity",
      scrollTarget: "activity-work",
      retryWorkItemId: work.id,
      technical: JSON.stringify({
        work_item_id: work.id,
        work_type: work.work_type,
        status: work.status,
        error_message: work.error_message,
      }),
    });
  }

  if (input.searchIndexState !== "ok" && input.searchIndexState !== "pending") {
    items.push({
      id: "search-index",
      title: "Search needs attention",
      cause: input.searchEnsureError ?? input.searchHint ?? "Search memory is not ready yet.",
      actionLabel: "Repair search",
      section: "system",
      scrollTarget: "search-status",
      technical: input.searchEnsureError ?? input.searchHint,
    });
  }

  return items;
}