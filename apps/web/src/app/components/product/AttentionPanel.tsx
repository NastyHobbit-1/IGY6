import type { AttentionItem } from "./attention";

export function AttentionPanel({ items }: { items: AttentionItem[] }) {
  if (items.length === 0) return null;

  return (
    <div className="productAttentionWrap" data-attention-panel>
      <button type="button" className="productAttentionButton" data-attention-toggle>
        {items.length} item{items.length === 1 ? "" : "s"} need attention
      </button>
      <div className="productAttentionDrawer" data-attention-drawer hidden>
        <header className="productAttentionDrawerHead">
          <strong>What needs attention</strong>
          <button type="button" className="productGhostButton" data-attention-close>
            Close
          </button>
        </header>
        <ul className="productAttentionList">
          {items.map((item) => (
            <li key={item.id} className="productAttentionItem uiOverflowSafe">
              <div>
                <strong className="uiClampTitle">{item.title}</strong>
                <p className="uiClampSummary">{item.cause}</p>
              </div>
              <button
                type="button"
                className="productPrimaryButton"
                data-attention-action
                data-attention-section={item.section}
                data-attention-scroll={item.scrollTarget ?? ""}
                data-retry-work-item-id={item.retryWorkItemId ?? ""}
              >
                {item.actionLabel}
              </button>
              {item.technical ? (
                <details className="productAdvanced">
                  <summary>Technical details</summary>
                  <pre className="uiBreakLong">{item.technical}</pre>
                </details>
              ) : null}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}