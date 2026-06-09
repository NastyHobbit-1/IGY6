import { Skeleton } from "./Skeleton";
export function SkeletonBlock({ lines = 3 }: { lines?: number }) {
  return (
    <div className="skeletonBlock" aria-busy="true" aria-label="Loading">
      {Array.from({ length: lines }, (_, index) => (
        <Skeleton key={index} className={index === lines - 1 ? "skeletonShort" : ""} />
      ))}
    </div>
  );
}
