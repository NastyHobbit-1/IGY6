import type { ReactNode } from "react";
import { StatusBadge } from "../ui/StatusBadge";

export function SectionHeader({
  eyebrow,
  title,
  lead,
  statusState,
  statusLabel,
  children,
}: {
  eyebrow: string;
  title: string;
  lead?: string;
  statusState?: string;
  statusLabel?: string;
  children?: ReactNode;
}) {
  return (
    <header className="productPanelHeader">
      <div>
        <p className="productEyebrow">{eyebrow}</p>
        <h2>{title}</h2>
        {lead ? <p className="productLead">{lead}</p> : null}
        {children}
      </div>
      {statusState ? <StatusBadge state={statusState} label={statusLabel} /> : null}
    </header>
  );
}