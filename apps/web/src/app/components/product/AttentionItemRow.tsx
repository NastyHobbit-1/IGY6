import type { AttentionItem } from "./attention";
import { ActionButton } from "../ui/ActionButton";
import { DetailDrawer } from "../ui/DetailDrawer";

export function AttentionItemRow({ item }: { item: AttentionItem }) {
  return (
    <li className="productAttentionItem uiOverflowSafe">
      <div>
        <strong className="uiClampTitle">{item.title}</strong>
        <p className="uiClampSummary">{item.cause}</p>
      </div>
      <ActionButton
        variant="primary"
        data-attention-action
        data-attention-section={item.section}
        data-attention-scroll={item.scrollTarget ?? ""}
        data-retry-work-item-id={item.retryWorkItemId ?? ""}
      >
        {item.actionLabel}
      </ActionButton>
      {item.technical ? (
        <DetailDrawer summary="Technical details">
          <pre className="uiBreakLong">{item.technical}</pre>
        </DetailDrawer>
      ) : null}
    </li>
  );
}