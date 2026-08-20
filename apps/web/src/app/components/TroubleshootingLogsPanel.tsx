import { ClientScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";

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

export function TroubleshootingLogsPanel() {
  return (
    <section className="panel settingsPanel tabContent" id="troubleshooting-logs" data-tab-panel="settings" data-troubleshooting-logs>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Settings</p>
          <h2>Troubleshooting logs</h2>
        </div>
        <StatusPill state="local-only" />
      </div>
      <div className="guidedManualNotice">
        <strong>Startup and error logs.</strong>
        <span>
          Recent CLI and gateway steps stored under your data folder in <code>ops/startup.log</code> and <code>ops/error.log</code>. Sensitive values are redacted.
        </span>
      </div>
      <div className="guidedManualActions">
        <button type="button" data-log-refresh>Refresh logs</button>
      </div>
      <div className="settingsResultGrid troubleshootingLogGrid">
        <article>
          <strong>Startup log</strong>
          <pre data-log-startup>No startup log entries yet.</pre>
        </article>
        <article>
          <strong>Error log</strong>
          <pre data-log-errors>No error log entries yet.</pre>
        </article>
      </div>
      <p className="actionHint" data-log-status>Logs load from local runtime storage.</p>
      <ClientScript script={script} />
    </section>
  );
}
