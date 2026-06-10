import { PRODUCT_SECTIONS } from "../product/helpers";

export function ProductNav({ activeSection = "command" }: { activeSection?: string }) {
  return (
    <nav className="productNav" aria-label="Main sections">
      {PRODUCT_SECTIONS.map((section) => (
        <button
          key={section.id}
          type="button"
          className="productNavButton"
          data-product-nav={section.id}
          aria-current={section.id === activeSection ? "page" : "false"}
        >
          <span>{section.label}</span>
          <small>{section.hint}</small>
        </button>
      ))}
    </nav>
  );
}