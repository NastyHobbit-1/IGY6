import type { ReactNode } from "react";

export function DetailDrawer({
  summary,
  children,
  className,
  id,
}: {
  summary: string;
  children: ReactNode;
  className?: string;
  id?: string;
}) {
  const merged = className ? `productAdvanced ${className}` : "productAdvanced";
  return (
    <details className={merged} id={id}>
      <summary>{summary}</summary>
      {children}
    </details>
  );
}