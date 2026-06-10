import type { AttentionItem } from "./attention";
import { AttentionItemRow } from "./AttentionItemRow";

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
            <AttentionItemRow key={item.id} item={item} />
          ))}
        </ul>
      </div>
    </div>
  );
}