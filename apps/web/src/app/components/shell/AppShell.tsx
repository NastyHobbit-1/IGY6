import type { ReactNode } from "react";
import { ClientScript } from "@/lib/use-dom-script";
import { PRODUCT_NAV_SCRIPT } from "../product/nav-script";
import type { AttentionItem } from "../product/attention";
import { AttentionPanel } from "../product/AttentionPanel";
import { StatusBadge } from "../ui/StatusBadge";
import { ModeToggle } from "./ModeToggle";
import { ProductNav } from "./ProductNav";

export function AppShell({
  healthStatus,
  runtimeSummary,
  attentionItems,
  searchState,
  searchReachable,
  children,
}: {
  healthStatus: string;
  runtimeSummary: string;
  attentionItems: AttentionItem[];
  searchState: string;
  searchReachable: boolean;
  children: ReactNode;
}) {
  return (
    <div
      className="igy6-app"
      data-product-app
      data-ui-mode="simple"
      data-search-state={searchState}
      data-search-reachable={searchReachable ? "true" : "false"}
    >
      <input className="tabInput" id="tab-results" name="main-dashboard-tab" type="radio" defaultChecked hidden />
      <input className="tabInput" id="tab-add-data" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-work" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-settings" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-advanced" name="main-dashboard-tab" type="radio" hidden />
      <input className="tabInput" id="tab-home" name="main-dashboard-tab" type="radio" hidden />

      <aside className="productSidebar" aria-label="IGY6 navigation">
        <div className="productBrand">
          <div className="productBrandMark">IG</div>
          <div>
            <strong>IGY6</strong>
            <span>Local evidence workspace</span>
          </div>
        </div>

        <ModeToggle />
        <ProductNav />

        <div className="productSidebarStatus">
          <StatusBadge state={healthStatus} />
          <span className="uiBreakLong">{runtimeSummary}</span>
          <span className="productHint uiBreakLong" data-product-status-banner aria-live="polite" />
          <AttentionPanel items={attentionItems} />
        </div>
      </aside>

      <main className="productMain" data-product-main>
        {children}
      </main>

      <ClientScript script={PRODUCT_NAV_SCRIPT} />
    </div>
  );
}