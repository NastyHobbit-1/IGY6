import { StatusPill } from "./StatusPill";

export function StatusBadge({ state, label }: { state: string; label?: string }) {
  if (label) {
    return (
      <span className="statusBadge">
        <StatusPill state={state} />
        <span className="statusBadgeLabel">{label}</span>
      </span>
    );
  }
  return <StatusPill state={state} />;
}