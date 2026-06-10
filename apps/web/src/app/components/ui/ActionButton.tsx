import type { ButtonHTMLAttributes, ReactNode } from "react";

type ActionVariant = "primary" | "ghost" | "warn" | "chip";

const VARIANT_CLASS: Record<ActionVariant, string> = {
  primary: "productPrimaryButton",
  ghost: "productGhostButton",
  warn: "productChip productChipWarn",
  chip: "productChip",
};

export function ActionButton({
  variant = "primary",
  children,
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ActionVariant;
  children: ReactNode;
}) {
  const base = VARIANT_CLASS[variant];
  const merged = className ? `${base} ${className}` : base;
  return (
    <button type="button" className={merged} {...props}>
      {children}
    </button>
  );
}