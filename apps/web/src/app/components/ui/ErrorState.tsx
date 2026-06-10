import { ActionButton } from "./ActionButton";

export function ErrorState({
  title,
  message,
  retryLabel,
  onRetryDataAttr,
}: {
  title: string;
  message: string;
  retryLabel?: string;
  onRetryDataAttr?: string;
}) {
  return (
    <div className="productStateCard productStateError uiOverflowSafe" role="alert">
      <strong className="uiClampTitle">{title}</strong>
      <p className="uiClampSummary">{message}</p>
      {retryLabel && onRetryDataAttr ? (
        <ActionButton variant="ghost" data-action={onRetryDataAttr}>
          {retryLabel}
        </ActionButton>
      ) : null}
    </div>
  );
}