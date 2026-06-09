import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { SkeletonBlock } from "./ui/SkeletonBlock";

export function BypassIntelPanel() {
  const script = `
(() => {
  const root = document.querySelector("[data-bypass-intel-panel]");
  if (!root || root.getAttribute("data-wired") === "true") return;
  root.setAttribute("data-wired", "true");
  const statusNode = root.querySelector("[data-bypass-intel-status]");
  const domainsNode = root.querySelector("[data-bypass-intel-domains]");
  const harvestButton = root.querySelector("[data-bypass-intel-harvest]");
  const writeStatus = (payload) => {
    if (!statusNode) return;
    const last = payload?.last_run || {};
    const domains = payload?.sample_domains || [];
    const harvested = last?.domains_harvested ?? payload?.targets_count ?? 0;
    const techniques = last?.techniques_total ?? payload?.techniques_total ?? 0;
    const when = last?.finished_at || payload?.playbook_updated_at || "not yet";
    statusNode.textContent =
      "Tracking " + (payload?.targets_count ?? 0) +
      " site(s). Playbook has " + techniques +
      " technique(s). Last harvest: " + when +
      " (" + harvested + " domains researched).";
    if (domainsNode) {
      domainsNode.textContent = domains.length
        ? "Includes: " + domains.join(", ")
        : "No target domains recorded yet — they are added automatically when you fetch URLs.";
    }
  };
  const refresh = async () => {
    try {
      const response = await fetch("/api/bypass-intel/status");
      const payload = await response.json();
      if (response.ok) writeStatus(payload);
    } catch (error) {
      if (statusNode) {
        statusNode.textContent = error instanceof Error ? error.message : "Could not load bypass research status.";
      }
    }
  };
  harvestButton?.addEventListener("click", async () => {
    if (harvestButton) {
      harvestButton.disabled = true;
      harvestButton.textContent = "Researching...";
    }
    if (statusNode) statusNode.textContent = "Searching the web for bypass techniques on tracked sites...";
    try {
      const response = await fetch("/api/bypass-intel/harvest", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ force: true, requested_by_actor_id: "local-owner" })
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload?.detail || "Harvest failed");
      writeStatus(payload?.status || payload);
    } catch (error) {
      if (statusNode) {
        statusNode.textContent = error instanceof Error ? error.message : "Bypass research failed.";
      }
    } finally {
      if (harvestButton) {
        harvestButton.disabled = false;
        harvestButton.textContent = "Refresh bypass research";
      }
      void refresh();
    }
  });
  void refresh();
})();
`;
  return (
    <section className="panel panelInset bypassIntelPanel" id="bypass-intel" data-bypass-intel-panel data-tab-panel="settings" aria-label="Bypass research">
      <div className="subHeader">
        <h3>Bypass research (automatic)</h3>
        <StatusPill state="local-first" />
      </div>
      <p className="actionHint">
        IGY6 regularly searches the open web for bypass ideas on sites you have fetched (including Patreon and other paywalled targets) and folds working tricks into auto bypass.
      </p>
      <div data-bypass-intel-status>
        <SkeletonBlock lines={2} />
      </div>
      <p className="statusText" data-bypass-intel-domains />
      <button type="button" data-bypass-intel-harvest>Refresh bypass research</button>
      <ClientScript script={script} />
    </section>
  );
}

