import { SkeletonBlock } from "./SkeletonBlock";

export function LoadingState({ label = "Loading..." }: { label?: string }) {
  return (
    <div className="productStateCard productStateLoading uiOverflowSafe" aria-busy="true">
      <span className="productHint">{label}</span>
      <SkeletonBlock lines={3} />
    </div>
  );
}