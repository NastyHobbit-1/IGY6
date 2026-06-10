import type { ReactNode } from "react";
import { StatusBadge } from "../ui/StatusBadge";

export function SystemServiceCard({
  name,
  label,
  state,
  detail,
  action,
  searchStatusLabel,
}: {
  name: string;
  label: string;
  state: string;
  detail?: string | null;
  action?: ReactNode;
  searchStatusLabel?: boolean;
}) {
  return (
    <article className="productServiceCard uiOverflowSafe">
      <span>{name}</span>
      <strong {...(searchStatusLabel ? { "data-search-status-label": "" } : {})}>{label}</strong>
      <StatusBadge state={state} />
      {detail ? <small className="uiBreakLong productAdvanced">{detail}</small> : null}
      {action}
    </article>
  );
}