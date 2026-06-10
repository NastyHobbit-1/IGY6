import type {
  HealthResponse,
  WorkItemRecord,
  EvidenceItemRecord,
  SourceRecord,
  VectorCollectionStatus,
} from "../types";

type HealthCheck = { status: string; detail?: string };

export type ProductSection = "command" | "add" | "evidence" | "gallery" | "activity" | "system" | "settings";

export const PRODUCT_SECTIONS: Array<{ id: ProductSection; label: string; hint: string }> = [
  { id: "command", label: "Command Center", hint: "Ask IGY6 and see what needs attention" },
  { id: "add", label: "Add Information", hint: "Bring in text, websites, or local folders" },
  { id: "evidence", label: "Evidence", hint: "Review what IGY6 saved" },
  { id: "gallery", label: "Images", hint: "Browse saved images in one place" },
  { id: "activity", label: "Activity", hint: "Processing, collections, and approvals" },
  { id: "system", label: "System", hint: "Service health and setup status" },
  { id: "settings", label: "Settings", hint: "Ports, storage, model, and privacy" },
];

const HEALTH_CHECK_LABELS: Record<string, string> = {
  rust_gateway: "API",
  api_ready: "API",
  fastapi_fallback: "Legacy API",
  web_ui: "Web UI",
  qdrant: "Search index",
  postgres: "PostgreSQL",
  postgresql: "PostgreSQL",
  neo4j: "Graph database",
  mlflow: "MLflow",
  phoenix: "Phoenix",
  ollama: "Local model",
  worker: "Background worker",
};

const SERVICE_LABELS: Record<string, string> = {
  postgres: "PostgreSQL",
  postgresql: "PostgreSQL",
  qdrant: "Search index",
  neo4j: "Graph database",
  mlflow: "MLflow",
  phoenix: "Phoenix",
  ollama: "Local model",
  llm: "Local model",
  gateway: "API",
  worker: "Background worker",
  web: "Web UI",
};

const LEGACY_HEALTH_CHECKS = new Set(["fastapi_fallback"]);

export function isVisibleHealthCheck(key: string, check: HealthCheck): boolean {
  const normalized = key.toLowerCase();
  if (LEGACY_HEALTH_CHECKS.has(normalized)) return false;
  const status = (check.status ?? "").toLowerCase();
  if (status === "removed" || status === "retired" || status === "archived") return false;
  return true;
}

export function visibleHealthChecks(checks: Record<string, HealthCheck>): Array<[string, HealthCheck]> {
  return Object.entries(checks).filter(([key, check]) => isVisibleHealthCheck(key, check));
}

export function plainServiceName(key: string): string {
  const normalized = key.toLowerCase();
  if (HEALTH_CHECK_LABELS[normalized]) return HEALTH_CHECK_LABELS[normalized];

  const segments = normalized.split(/[^a-z0-9]+/).filter(Boolean);
  for (const [needle, label] of Object.entries(SERVICE_LABELS)) {
    if (segments.includes(needle)) return label;
  }
  return key.replace(/_/g, " ");
}

export function dedupeHealthChecksByLabel(
  entries: Array<[string, HealthCheck]>,
): Array<[string, HealthCheck]> {
  const bestByLabel = new Map<string, [string, HealthCheck]>();
  for (const entry of entries) {
    const label = plainServiceName(entry[0]);
    const existing = bestByLabel.get(label);
    if (!existing) {
      bestByLabel.set(label, entry);
      continue;
    }
    const existingOk = plainHealthStatus(existing[1].status).state === "ok";
    const nextOk = plainHealthStatus(entry[1].status).state === "ok";
    if (nextOk && !existingOk) bestByLabel.set(label, entry);
  }
  return Array.from(bestByLabel.values());
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
  const entries = dedupeHealthChecksByLabel(visibleHealthChecks(health.checks ?? {}));
  if (entries.length === 0) {
    return health.status === "ok" ? "IGY6 is ready." : "IGY6 readiness is unknown — check System.";
  }
  const ok = entries.filter(([, check]) => check.status === "ok").length;
  if (ok === entries.length) return "All checked services are running.";
  if (ok === 0) return "Services need attention — open System for details.";
  return `${ok} of ${entries.length} services are running.`;
}

export function searchIndexStatus(vector: VectorCollectionStatus): {
  label: string;
  state: "ok" | "pending" | "warning" | "error";
} {
  if (vector.exists) return { label: "Search is ready", state: "ok" };
  if (vector.detail?.tcp_reachable) {
    return { label: "Search is being prepared", state: "pending" };
  }
  return { label: "Search needs attention", state: "error" };
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
    case "failed": {
      const workLabel = workActivityLabel(item);
      const cause = item.error_message?.trim();
      const affected =
        item.work_type === "collection_normalization"
          ? "Some recently collected information may not be searchable yet."
          : "A background step did not finish.";
      return {
        label: "Needs attention",
        state: "failed",
        next: cause
          ? `${workLabel} did not finish. ${affected} Likely cause: ${cause.slice(0, 160)}. Use Retry below or add the information again.`
          : `${workLabel} did not finish. ${affected} Use Retry below or open Add Information to try again.`,
      };
    }
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