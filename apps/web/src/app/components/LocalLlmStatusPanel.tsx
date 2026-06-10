import type { EnvSettingsResponse, ApiResult } from "./types";
import { buildLlmDisplay } from "./llm-helpers";
import { StatusPill } from "./ui/StatusPill";
import { HelpHeading } from "./ui/HelpHeading";

export function LocalLlmStatusPanel({
  envSettings,
  context
}: {
  envSettings: ApiResult<EnvSettingsResponse>;
  context: "assistant" | "settings";
}) {
  const llm = buildLlmDisplay(envSettings.data);
  return (
    <section className={context === "settings" ? "settingsGroup llmStatusPanel" : "llmStatusPanel"} data-llm-status>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">{context === "settings" ? "Local model provider" : "Answer mode"}</p>
          <h3><HelpHeading term="localLlm">Local LLM Status</HelpHeading></h3>
        </div>
        <StatusPill state={llm.healthStatus} />
      </div>
      <div className="metrics compact">
        <article><span>Enabled</span><strong>{llm.enabledState}</strong></article>
        <article><span>Provider</span><strong>{llm.provider}</strong></article>
        <article><span>Model</span><strong>{llm.model}</strong></article>
        <article><span>Health status</span><strong>{llm.healthStatus}</strong></article>
        <article><span>Answer mode</span><strong>{llm.answerMode}</strong></article>
        <article><span>Routing</span><strong>{llm.routingState}</strong></article>
        <article><span>Fallback</span><strong>{llm.fallbackState}</strong></article>
        <article><span>Evidence required</span><strong>{llm.evidenceRequired}</strong></article>
        <article><span>Hosted/external AI</span><strong>{llm.externalUse}</strong></article>
      </div>
      <p className="agentRuntimeReason">{llm.healthDetail}</p>
      <div className="guidedManualNotice">
        <strong>{llm.guidance}</strong>
        <span>No hosted AI call is made by default. No hidden data transfer is performed by this status panel, and Settings does not contact Ollama.</span>
      </div>
      <ul className="workflowSteps">
        {llm.limitations.map((item) => (
          <li key={item}><strong>{item.split(":")[0]}</strong><span>{item.includes(":") ? item.slice(item.indexOf(":") + 1).trim() : item}</span></li>
        ))}
      </ul>
      <div className="exampleGrid">
        <article>
          <span>Normal user example</span>
          <strong>Use local model to summarize uploaded warranty note using only evidence.</strong>
        </article>
        <article>
          <span>Coder example</span>
          <strong>Use local model to explain build log failure with citations.</strong>
        </article>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: raw provider diagnostics</summary>
        <pre>{JSON.stringify(llm.rawDiagnostics, null, 2)}</pre>
      </details>
    </section>
  );
}

