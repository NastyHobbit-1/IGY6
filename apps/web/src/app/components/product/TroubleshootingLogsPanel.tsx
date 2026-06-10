import type { RuntimeLogsResponse } from "../types";
import { ClientScript } from "@/lib/use-dom-script";

const script = `
(() => {
  const root = document.querySelector("[data-troubleshooting-logs]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const startupEl = root.querySelector("[data-log-startup]");
  const errorEl = root.querySelector("[data-log-errors]");
  const statusEl = root.querySelector("[data-log-status]");
  const show = (message) => { if (statusEl) statusEl.textContent = message; };
  const renderLines = (element, lines, emptyLabel) => {
    if (!element) return;
    if (!Array.isArray(lines) || lines.length === 0) {
      element.textContent = emptyLabel;
      return;
    }
    element.textContent = lines.join("\\n");
  };
  const refresh = async () => {
    show("Refreshing logs...");
    try {
      const response = await fetch("/api/ops/runtime-logs?limit=120");
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(JSON.stringify(payload));
      renderLines(startupEl, payload?.startup_log?.lines, "No startup log entries yet.");
      renderLines(errorEl, payload?.error_log?.lines, "No error log entries yet.");
      show("Updated " + new Date().toLocaleTimeString());
    } catch (error) {
      show("Could not refresh logs: " + String(error));
    }
  };
  root.querySelector("[data-log-refresh]")?.addEventListener("click", refresh);
  void refresh();
})();
`;

export function TroubleshootingLogsPanel({
  logs,
}: {
  logs: RuntimeLogsResponse;
}) {
  const startupLines = logs.startup_log?.lines ?? [];
  const errorLines = logs.error_log?.lines ?? [];

  return (
    <section className="productSubsection uiOverflowSafe" id="troubleshooting-logs" data-troubleshooting-logs>
      <header className="productPanelHeader">
        <div>
          <p className="productEyebrow">Troubleshooting</p>
          <h3>Startup and error logs</h3>
        </div>
        <button type="button" className="productGhostButton" data-log-refresh>
          Refresh logs
        </button>
      </header>
      <p className="productLead">
        Recent startup steps and errors from IGY6 CLI and services. Logs are stored under your data folder in
        {" "}
        <code className="uiBreakLong">ops/startup.log</code>
        {" "}
        and
        {" "}
        <code className="uiBreakLong">ops/error.log</code>
        . Sensitive values are redacted.
      </p>
      <div className="productLogGrid">
        <article className="productLogCard uiOverflowSafe">
          <strong>Startup log</strong>
          <pre className="productLogBody uiBreakLong" data-log-startup>
            {startupLines.length > 0 ? startupLines.join("\n") : "No startup log entries yet."}
          </pre>
        </article>
        <article className="productLogCard uiOverflowSafe">
          <strong>Error log</strong>
          <pre className="productLogBody uiBreakLong" data-log-errors>
            {errorLines.length > 0 ? errorLines.join("\n") : "No error log entries yet."}
          </pre>
        </article>
      </div>
      <p className="productHint" data-log-status>
        {logs.error ? `Log read issue: ${logs.error}` : "Logs loaded from local runtime storage."}
      </p>
      <ClientScript script={script} />
    </section>
  );
}