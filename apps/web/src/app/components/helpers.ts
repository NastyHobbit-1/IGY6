import type { EvidenceItemRecord, EnvSettingsResponse, WorkItemRecord } from "./types";

export function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString("en-US", {
    dateStyle: "medium",
    timeStyle: "short"
  });
}

export function formatBytes(value: number | null): string {
  if (value === null) {
    return "unknown";
  }
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export function excerpt(value: string, maxLength = 110): string {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, maxLength - 3)}...`;
}

export function stringArrayFromUnknown(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
}

export function numberFromUnknown(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

export function uniqueStringValues(values: string[], maxItems: number): string[] {
  const seen = new Set<string>();
  const result: string[] = [];
  values.forEach((value) => {
    const trimmed = value.trim();
    if (!trimmed || seen.has(trimmed)) return;
    seen.add(trimmed);
    result.push(trimmed);
  });
  return result.slice(0, maxItems);
}

export function shortRecordId(value: string | null | undefined): string {
  if (!value) return "unknown";
  return value.length > 12 ? `${value.slice(0, 8)}...` : value;
}

export function jsonString(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}

export function jsonStringList(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
}

export function evidenceReviewState(item: EvidenceItemRecord): string {
  const reviewState = item.metadata_json?.review_state;
  if (!reviewState || typeof reviewState !== "object") {
    return "unreviewed";
  }
  const state = (reviewState as Record<string, unknown>).state;
  return typeof state === "string" && state.trim() ? state : "unreviewed";
}

export function evidenceReviewNote(item: EvidenceItemRecord): string | null {
  const reviewState = item.metadata_json?.review_state;
  if (!reviewState || typeof reviewState !== "object") {
    return null;
  }
  const note = (reviewState as Record<string, unknown>).correction_note;
  return typeof note === "string" && note.trim() ? note : null;
}

export function metadataMentionsId(metadata: Record<string, unknown> | null | undefined, id: string): boolean {
  if (!metadata || !id) return false;
  return JSON.stringify(metadata).includes(id);
}

export function workItemRelatedIds(workItem: WorkItemRecord): Array<{ label: string; values: string[] }> {
  const payload = workItem.payload_json ?? {};
  const related = [
    { label: "collection", values: [jsonString(payload.collection_run_id)].filter(Boolean) as string[] },
    { label: "source", values: [jsonString(payload.source_id)].filter(Boolean) as string[] },
    { label: "permission", values: [jsonString(payload.source_permission_id)].filter(Boolean) as string[] },
    { label: "artifact", values: jsonStringList(payload.raw_artifact_ids) },
    { label: "document", values: jsonStringList(payload.document_ids) },
    { label: "chunk", values: jsonStringList(payload.chunk_ids) },
    { label: "parent work", values: [jsonString(payload.parent_work_item_id)].filter(Boolean) as string[] }
  ];
  return related.filter((item) => item.values.length > 0);
}

export function workItemGuidance(workItem: WorkItemRecord): { outcome: string; next: string } {
  switch (workItem.status) {
    case "queued":
    case "pending_intent_verification":
      return {
        outcome: "Waiting for background processing.",
        next: "Refresh Work after the worker has had time to claim it. Use Advanced dispatch only when you know this specific queued item should be dispatched."
      };
    case "running":
      return {
        outcome: "Processing is in progress.",
        next: "Refresh Work to see the updated state. Avoid resubmitting the same upload while this item is running."
      };
    case "completed":
      return {
        outcome: "Processing completed successfully.",
        next: "Open Chat to inspect documents, chunks, evidence, and Ask over evidence."
      };
    case "failed":
      return {
        outcome: workItem.error_message ?? "Processing failed and needs review.",
        next: "Read the error and verify the source, permission, and uploaded UTF-8 text. No automatic retry action is exposed here."
      };
    case "canceled":
      return {
        outcome: "Processing was canceled.",
        next: "Review the source and collection record before creating new work."
      };
    default:
      return {
        outcome: "Status is recorded by the local API.",
        next: "Refresh Work or inspect Advanced raw queue JSON if this state is unexpected."
      };
  }
}

export function workItemDispatchVisibility(workItem: WorkItemRecord): Array<{ label: string; value: string; state: string }> {
  const supportedTypes: Record<string, string> = {
    collection_normalization: "collection.normalize_collection_run",
    document_chunking: "evidence.generate_document_chunks",
    chunk_vector_upsert: "memory.vector.upsert_chunks"
  };
  const payload = workItem.payload_json ?? {};
  const taskName = supportedTypes[workItem.work_type];
  const intentVerified = Boolean(payload.intent_verification) || payload.intent_verification_recorded === true;
  const safeDispatchOnly = payload.safe_dispatch_only === true || payload.rust_gateway_execution === "not_executed";
  const statusState = ["queued", "pending_intent_verification"].includes(workItem.status)
    ? "waiting"
    : ["running"].includes(workItem.status)
      ? "running"
      : ["completed"].includes(workItem.status)
        ? "completed"
        : ["failed"].includes(workItem.status)
          ? "failed"
          : "recorded";
  return [
    {
      label: "support",
      value: taskName ? `supported: ${taskName}` : "unsupported by bounded dispatch",
      state: taskName ? "supported" : "unsupported"
    },
    {
      label: "state",
      value: workItem.status,
      state: statusState
    },
    {
      label: "intent",
      value: intentVerified ? "intent verification recorded" : "intent verification not visible",
      state: intentVerified ? "verified" : "not-verified"
    },
    {
      label: "dispatch",
      value: safeDispatchOnly ? "dispatch metadata only / no arbitrary execution" : "worker-managed or not dispatched here",
      state: safeDispatchOnly ? "safe-dispatch-only" : "worker-managed"
    }
  ];
}
