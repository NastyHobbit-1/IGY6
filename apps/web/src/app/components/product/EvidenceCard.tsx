import type { EvidenceCardModel } from "./evidence-display";
import { StatusPill } from "../ui/StatusPill";

export function EvidenceCard({ model }: { model: EvidenceCardModel }) {
  return (
    <article className="productEvidenceCard uiOverflowSafe" data-evidence-card>
      <span className="productTypeBadge" data-content-type={model.contentType}>
        {model.contentType}
      </span>
      <div className="productEvidenceHead">
        <h3 className="uiClampTitle" title={model.title}>
          {model.title}
        </h3>
        <StatusPill state={model.statusState} />
      </div>
      <p className="uiClampSummary">{model.summary}</p>
      <dl className="productMeta uiMetaSafe">
        <div>
          <dt>Source</dt>
          <dd className="uiBreakLong">{model.sourceLabel}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{model.statusLabel}</dd>
        </div>
        {model.dateLabel ? (
          <div>
            <dt>Date</dt>
            <dd>{model.dateLabel}</dd>
          </div>
        ) : null}
      </dl>
      <div className="productCardActions">
        {model.viewHref ? (
          <a className="productGhostButton" href={model.viewHref} target="_blank" rel="noreferrer">
            Open
          </a>
        ) : (
          <button type="button" className="productGhostButton" data-product-go="evidence">
            View
          </button>
        )}
      </div>
      <details className="productAdvanced">
        <summary>Technical details</summary>
        <pre className="uiBreakLong">{JSON.stringify(model.technical, null, 2)}</pre>
      </details>
    </article>
  );
}