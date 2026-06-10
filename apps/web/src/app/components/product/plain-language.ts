const TERM_MAP: Record<string, string> = {
  "retrieval preview": "What IGY6 found",
  "connector registry": "Ways to add information",
  "task plan": "Planned work",
  "work item": "Work",
  "work items": "Work",
  "vector chunk": "Evidence detail",
  "vector chunks": "Evidence details",
  qdrant: "Search memory",
  neo4j: "Relationship memory",
  schema: "database setup",
  postgresql: "Database",
  postgres: "Database",
  mlflow: "Experiment tracking",
  phoenix: "Tracing",
};

export function plainTerm(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return value;
  const lower = trimmed.toLowerCase();
  if (TERM_MAP[lower]) return TERM_MAP[lower];
  for (const [needle, replacement] of Object.entries(TERM_MAP)) {
    if (lower.includes(needle)) {
      return trimmed.replace(new RegExp(needle, "gi"), replacement);
    }
  }
  return trimmed.replace(/_/g, " ");
}

export function plainAuditLabel(eventType: string): string {
  return plainTerm(eventType.replace(/_/g, " "));
}

export function plainApprovalType(requestType: string): string {
  return plainTerm(requestType.replace(/_/g, " "));
}