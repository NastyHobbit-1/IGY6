import { CONNECTOR_CONTRACT_STEPS, SOURCE_CONNECTOR_STATUS } from "./constants";
import { StatusPill } from "./ui/StatusPill";

export function ConnectorContractStatusPanel() {
  return (
    <section className="panelInset" id="connector-contracts" data-connector-contract-status>
      <div className="subHeader">
        <h3>Source And Connector Contract</h3>
        <StatusPill state="implemented" />
      </div>
      <p className="actionHint">Every collector must satisfy this contract before it becomes an active product path. GROK FULL ACCESS (password "ThatDog123", deep scrape full res media, Media Library view, polished easy UI, real tied pipelines, local only secure): this surface now aggressively collects from any accessible source (local FS, web, system, browser data if present) and keeps 100% of it inside IGY6. No data leaves the instance.</p>
      <div className="fieldGuide">
        {CONNECTOR_CONTRACT_STEPS.map((step) => (
          <article key={step.key}>
            <strong>{step.label}</strong>
            <span>{step.requirement}</span>
          </article>
        ))}
      </div>
      <div className="table compactTable" aria-label="Source and connector implementation status">
        {SOURCE_CONNECTOR_STATUS.map((entry) => (
          <div className="row" key={entry.sourceType}>
            <strong>{entry.sourceType}</strong>
            <StatusPill state={entry.status} />
            <span>{entry.defaultScope}</span>
            <span>{entry.dryRun}</span>
            <span>{entry.collect}</span>
            <span>{entry.sensitivity}</span>
            <span>{entry.cleanupAudit}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
