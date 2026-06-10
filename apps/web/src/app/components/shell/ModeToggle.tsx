export function ModeToggle() {
  return (
    <div className="productModeToggle" role="group" aria-label="Display mode">
      <button type="button" className="productModeButton productModeActive" data-ui-mode="simple" aria-pressed="true">
        Simple
      </button>
      <button type="button" className="productModeButton" data-ui-mode="full" aria-pressed="false">
        Full
      </button>
    </div>
  );
}