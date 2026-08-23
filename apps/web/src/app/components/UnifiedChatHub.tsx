import { HOST_BRIDGE_AGENT_PORT } from "./constants";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";

export function UnifiedChatHub({
  sourceCount,
  evidenceCount,
  chunkCount,
  workItemCount,
  pendingApprovals,
  vectorReady,
  llmEnabled,
  llmModel
}: {
  sourceCount: number;
  evidenceCount: number;
  chunkCount: number;
  workItemCount: number;
  pendingApprovals: number;
  vectorReady: boolean;
  llmEnabled: boolean;
  llmModel: string;
}) {
  const script = `
(() => {
  const hub = document.querySelector("[data-unified-chat]");
  if (!hub) return;
  if (hub.getAttribute("data-chat-wired") === "true") return;
  hub.setAttribute("data-chat-wired", "true");

  const feed = hub.querySelector("[data-chat-feed]");
  const input = hub.querySelector("[data-chat-input]");
  const sendButton = hub.querySelector("[data-chat-send]");
  const status = hub.querySelector("[data-chat-status]");
  const actions = hub.querySelector("[data-chat-actions]");
  const composer = hub.querySelector(".chatComposer");
  const chips = hub.querySelectorAll("[data-chat-chip]");
  const chatForm = document.querySelector("[data-chat-preview-form]");
  const chatMessage = document.querySelector("[data-chat-preview-message]");
  const chatLimit = document.querySelector("[data-chat-preview-limit]");
  const agentInput = document.querySelector("[data-agent-command-input]");
  const agentPreview = document.querySelector("[data-agent-preview]");
  const agentExecute = document.querySelector("[data-agent-execute]");
  const agentApproval = document.querySelector("[data-agent-request-approval]");
  const agentExecuteApproved = document.querySelector("[data-agent-execute-approved]");
  const saveAnswer = document.querySelector("[data-chat-save-answer]");
  const previewResults = document.querySelector("[data-chat-preview-results]");
  const agentPort = ${HOST_BRIDGE_AGENT_PORT};

  const tabFor = (id) => document.getElementById(id);

  const switchTab = (tabId) => {
    const tab = tabFor(tabId);
    if (!tab) return false;
    tab.checked = true;
    tab.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  };

  const scrollToSection = (sectionId) => {
    if (!sectionId) return;
    requestAnimationFrame(() => {
      document.getElementById(sectionId)?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  };

  const navigateTo = (tabId, sectionId) => {
    const opened = switchTab(tabId);
    scrollToSection(sectionId);
    return opened;
  };

  const extractUrl = (text) => {
    const match = text.match(/https?:\\/\\/[^\\s<>"']+/i);
    if (!match) return null;
    return match[0].replace(/[.,;:!?)]+$/, "");
  };

  const extractDepth = (lower) => {
    if (/\\b(two levels|depth 2|2 levels)\\b/.test(lower)) return 2;
    if (/\\b(this page only|depth 0|no links|single page)\\b/.test(lower)) return 0;
    return 1;
  };

  const extractCookie = (text) => {
    const cookieMatch = text.match(/\\bcookie\\s*[:=]\\s*([^\\n]+)/i);
    return cookieMatch ? cookieMatch[1].trim() : "";
  };

  const extractBearer = (text) => {
    const bearerMatch = text.match(/\\b(bearer|token|authorization)\\s*[:=]\\s*([^\\n]+)/i);
    return bearerMatch ? bearerMatch[2].trim() : "";
  };

  const ensureMaxReachInfrastructure = async () => {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 180000);
      const agentResponse = await fetch("http://127.0.0.1:" + agentPort + "/ensure-max-reach", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        signal: controller.signal
      });
      clearTimeout(timeoutId);
      if (!agentResponse.ok) {
        const payload = await agentResponse.json().catch(() => ({}));
        console.warn("Host bridge ensure agent:", payload?.stderr || payload?.detail || agentResponse.status);
      }
    } catch (error) {
      console.warn("Host bridge ensure agent unavailable:", error);
    }
    const apiResponse = await fetch("/api/host-bridge/ensure-max-reach", { method: "POST" });
    const apiPayload = await apiResponse.json().catch(() => ({}));
    if (!apiResponse.ok) {
      throw new Error(
        apiPayload?.detail ||
          "Host bridge is not ready. Run once: pwsh -File scripts\\start-stack.ps1"
      );
    }
    return apiPayload;
  };

  const runCollectionFetch = async (body) => {
    const response = await fetch("/api/collection-runs/full-access", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload?.detail || response.statusText || "Collection fetch failed");
    }
    return payload?.summary_json || payload?.summary || payload;
  };

  const summarizeCollection = (summary) => {
    const strategies = Array.isArray(summary?.auto_bypass_strategies)
      ? summary.auto_bypass_strategies.join(", ")
      : String(summary?.auto_bypass_strategies || "");
    return [
      strategies ? "strategies: " + strategies : null,
      "pages: " + String(summary?.crawled_pages ?? summary?.web_scraped ?? "unknown"),
      "evidence: " + String(summary?.total_evidence ?? "unknown"),
      "artifacts: " + String(summary?.total_artifacts ?? "unknown")
    ].filter(Boolean).join(" · ");
  };

  const fillWebFetchField = (name, value) => {
    const field = document.querySelector("[name='" + name + "']");
    if (field && value) field.value = value;
  };

  let pendingClarification = null;

  const siteLabel = (url) => {
    try {
      return new URL(url).hostname.replace(/^www\\./, "");
    } catch {
      return "that site";
    }
  };

  const friendlyCollectionDone = (summary, site) => {
    const pages = String(summary?.crawled_pages ?? summary?.web_scraped ?? "some");
    const evidence = String(summary?.total_evidence ?? "new");
    return "Finished with " + site + ". I grabbed " + pages + " page(s) and stored " + evidence + " evidence piece(s) locally. Want me to answer questions about it?";
  };

  // ... (truncated for length - using full content from local resolved file)
