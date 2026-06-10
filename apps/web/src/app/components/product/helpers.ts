import type { HealthResponse, WorkItemRecord, EvidenceItemRecord, SourceRecord } from "../types";

export type ProductSection = "command" | "add" | "evidence" | "activity" | "system" | "settings";

export const PRODUCT_SECTIONS: Array<{ id: ProductSection; label: string; hint: string }> = [
  { id: "command", label: "Command Center", hint: "Ask IGY6 and see what needs attention" },
  { id: "add", label: "Add Information", hint: "Bring in text, websites, or local folders" },
  { id: "evidence", label: "Evidence", hint: "Review what IGY6 saved and indexed" },
  { id: "activity", label: "Activity", hint: "Processing, collections, and approvals" },
  { id: "system", label: "System", hint: "Service health and setup status" },
  { id: "settings", label: "Settings", hint: "Ports, storage, model, and privacy" },
];

const SERVICE_LABELS: Record<string, string> = {
  postgres: "PostgreSQL",
  postgresql: "PostgreSQL",
  qdrant: "Search index",
  neo4j: "Graph database",
  mlflow: "MLflow",
  phoenix: "Phoenix",
  ollama: "Local model",
  llm: "Local model",
  api: "API",
  gateway: "API",
  worker: "Background worker",
  web: "Web UI",
};

export function plainServiceName(key: string): string {
  const normalized = key.toLowerCase().replace(/[^a-z0-9]+/g, "_");
  for (const [needle, label] of Object.entries(SERVICE_LABELS)) {
    if (normalized.includes(needle)) return label;
  }
  return key.replace(/_/g, " ");
}

export function plainHealthStatus(status: string | undefined): { label: string; state: string } {
  const value = (status ?? "unknown").toLowerCase();
  if (value === "ok" || value === "ready" || value === "healthy") {
    return { label: "Running", state: "ok" };
  }
  if (value === "degraded" || value === "warn" || value === "warning") {
    return { label: "Needs attention", state: "warning" };
  }
  if (value === "error" || value === "failed" || value === "down") {
    return { label: "Stopped", state: "error" };
  }
  if (value === "starting" || value === "pending") {
    return { label: "Starting", state: "pending" };
  }
  return { label: "Unknown", state: "unknown" };
}

export function summarizeRuntime(health: HealthResponse): string {
  const checks = health.checks ?? {};
  const entries = Object.entries(checks);
  if (entries.length === 0) {
    return health.status === "ok" ? "IGY6 is ready." : "IGY6 readiness is unknown — check System.";
  }
  const ok = entries.filter(([, check]) => check.status === "ok").length;
  if (ok === entries.length) return "All checked services are running.";
  if (ok === 0) return "Services need attention — open System for details.";
  return `${ok} of ${entries.length} services are running.`;
}

export function workActivityLabel(item: WorkItemRecord): string {
  const map: Record<string, string> = {
    collection_normalization: "Preparing collected information",
    document_chunking: "Breaking documents into searchable pieces",
    chunk_vector_upsert: "Indexing for search",
    report_generation: "Creating a report",
  };
  return map[item.work_type] ?? "Background work";
}

export function workStatusPlain(item: WorkItemRecord): { label: string; state: string; next: string } {
  switch (item.status) {
    case "running":
      return {
        label: "Running",
        state: "running",
        next: "IGY6 is processing this now. Check back in Activity in a moment.",
      };
    case "queued":
    case "pending_intent_verification":
      return {
        label: item.status === "pending_intent_verification" ? "Needs approval" : "Waiting",
        state: "waiting",
        next: item.status === "pending_intent_verification"
          ? "Approve this in Activity before processing can continue."
          : "The background worker will pick this up automatically.",
      };
    case "completed":
      return {
        label: "Finished",
        state: "completed",
        next: "Open Evidence to review results or ask IGY6 in Command Center.",
      };
    case "failed":
      return {
        label: "Failed",
        state: "failed",
        next: item.error_message
          ? `Likely cause: ${item.error_message.slice(0, 180)}. Try Add Information again or check System.`
          : "Check Activity details and try again from Add Information.",
      };
    case "canceled":
      return { label: "Canceled", state: "canceled", next: "Start a new add flow if you still need this data." };
    default:
      return { label: "Recorded", state: "unknown", next: "Refresh Activity to see the latest state." };
  }
}

export function evidenceCardTitle(item: EvidenceItemRecord, source?: SourceRecord): string {
  const statement = item.statement?.trim();
  if (statement && statement.length > 8) {
    return statement.length > 72 ? `${statement.slice(0, 69)}...` : statement;
  }
  if (source?.name) return `Evidence from ${source.name}`;
  return "Saved evidence";
}

export function evidenceWhyItMatters(item: EvidenceItemRecord): string {
  if (item.evidence_type === "document_chunk") {
    return "This excerpt can support answers and summaries in chat.";
  }
  return "This record may be cited when you ask IGY6 questions.";
}