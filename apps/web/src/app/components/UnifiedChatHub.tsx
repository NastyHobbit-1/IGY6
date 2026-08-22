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

  const interpretFetchIntent = (text, lower, url) => {
    if (!url) return null;
    const site = siteLabel(url);
    const isPaidPlatform = /patreon\\.com|patreonusercontent\\.com|onlyfans\\.com|fansly\\.com/i.test(url);
    if (isPaidPlatform) {
      return { clear: true, intent: "max_reach", url, site, paid_platform: true };
    }
    const wantsMedia = /\\b(images?|photos?|pics?|pictures?|videos?|media|thumbnails?|full[ -]?res|download|onlyfans|fansly)\\b/i.test(lower);
    const wantsPaid = /\\b(paid|premium|paywall|subscriber|members?[- ]only|locked|vip|patreon|subscription)\\b/i.test(lower);
    const wantsHard = /\\b(hardest|everything|all of it|go hard|aggressive|free|bypass|login wall|locked content|paid content)\\b/i.test(lower);
    const wantsPublic = /\\b(public|no login|without logging|open page|just the page)\\b/i.test(lower);
    const wantsSession = /\\b(logged in|my account|cookie|session|already pay|i subscribe)\\b/i.test(lower);
    const scores = {
      max_reach: (wantsPaid ? 4 : 0) + (wantsHard ? 3 : 0) + (wantsMedia ? 2 : 0) + (/max reach|go anywhere|omnifetch|go all out/.test(lower) ? 6 : 0),
      auto_bypass: (/auto bypass|full auto|try harder/.test(lower) ? 6 : 0) + (wantsHard ? 2 : 0),
      public_fetch: (wantsPublic ? 6 : 0) + (/fetch public|public only/.test(lower) ? 6 : 0),
      session_fetch: (wantsSession ? 6 : 0) + (/bypass fetch|my login/.test(lower) ? 5 : 0)
    };
    const sorted = Object.entries(scores).sort((a, b) => b[1] - a[1]);
    const top = sorted[0];
    const second = sorted[1] || ["none", 0];
    if (top[1] >= 6 && top[1] - second[1] >= 3) {
      return { clear: true, intent: top[0], url, site };
    }
    if (top[1] > 0 || wantsMedia || /\\b(get|grab|pull|scrape|fetch|download|save|stuff from)\\b/.test(lower)) {
      if (wantsPaid || wantsMedia || wantsHard) {
        return {
          clear: false,
          site,
          question: "Do you want me to try and get the paid or locked content from " + site + " for free? Fair warning — it can take several minutes and might not work on every site.",
          options: [
            { label: "Yeah, try the hard way", intent: "max_reach", url },
            { label: "Just what's public", intent: "public_fetch", url },
            { label: "I already have a login", intent: "session_fetch", url }
          ]
        };
      }
      return {
        clear: false,
        site,
        question: "Want me to grab stuff from " + site + "? I can stick to public pages, use authorized session options, or go all out with Deep Fetch if it's really locked down.",
        options: [
          { label: "Public pages only", intent: "public_fetch", url },
          { label: "Try harder", intent: "auto_bypass", url },
          { label: "Go all out", intent: "max_reach", url }
        ]
      };
    }
    return null;
  };

  const resolveClarificationReply = (text, lower, pending) => {
    if (!pending) return null;
    if (/^(no|nah|nope|don't|do not|never mind|cancel)\\b/i.test(lower)) return { cancel: true };
    if (/^(yes|yeah|yep|sure|do it|go ahead|try it|ok|okay|yup)\\b/i.test(lower)) {
      return pending.options[0];
    }
    for (const option of pending.options) {
      const label = option.label.toLowerCase();
      if (lower.includes(label) || lower.includes(option.intent.replace(/_/g, " "))) return option;
    }
    if (/public/.test(lower)) return pending.options.find((option) => option.intent === "public_fetch") || pending.options[1];
    if (/login|session|cookie|account/.test(lower)) return pending.options.find((option) => option.intent === "session_fetch");
    if (/hard|max|aggressive|paid|locked/.test(lower)) return pending.options.find((option) => option.intent === "max_reach");
    if (/bypass|trick/.test(lower)) return pending.options.find((option) => option.intent === "auto_bypass");
    return null;
  };

  const askClarification = (interpretation) => {
    pendingClarification = interpretation;
    appendMessage("assistant", "Quick question", interpretation.question);
    interpretation.options.forEach((option) => {
      addAction(option.label, async () => {
        pendingClarification = null;
        await executeResolvedFetchIntent(option);
      });
    });
    setStatus("Waiting for your call");
  };

  const executeResolvedFetchIntent = async (option) => {
    const url = option.url;
    const site = siteLabel(url);
    const depth = 1;
    if (option.intent === "session_fetch") {
      appendMessage(
        "assistant",
        "Login needed",
        "To use your account on " + site + ", paste a cookie or token here — like: my login for " + url + " cookie: session_id=..."
      );
      fillWebFetchField("bypass_page_url", url);
      return;
    }
    if (option.intent === "max_reach") {
      setStatus("Getting things ready...");
      const paidNote = option.paid_platform
        ? " This is a paid-content site — I'll use your browser login, Patreon API calls, and media extraction."
        : "";
      appendMessage("assistant", "On it", "Alright — I'll pull out all the stops for " + site + ". Might take a few minutes." + paidNote);
      try {
        await ensureMaxReachInfrastructure();
        const summary = await runCollectionFetch({
          requested_by_actor_id: "local-owner",
          max_reach: true,
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url]
        });
        appendMessage("assistant", "Done", friendlyCollectionDone(summary, site));
        addAction("Check on processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "That didn't work", error instanceof Error ? error.message : "Something went wrong.");
      }
      setStatus("Ready");
      return;
    }
    if (option.intent === "auto_bypass") {
      setStatus("Warming up...");
      appendMessage("assistant", "On it", "I'll use authorized session options for " + site + " — your provided session header where needed.");
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 120000);
        await fetch("http://127.0.0.1:" + agentPort + "/ensure", { method: "POST", signal: controller.signal });
        clearTimeout(timeoutId);
        await fetch("/api/host-bridge/ensure-max-reach", { method: "POST" });
        const summary = await runCollectionFetch({
          requested_by_actor_id: "local-owner",
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url]
        });
        appendMessage("assistant", "Done", friendlyCollectionDone(summary, site));
        addAction("Check on processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "That didn't work", error instanceof Error ? error.message : "Something went wrong.");
      }
      setStatus("Ready");
      return;
    }
    setStatus("Fetching...");
    appendMessage("assistant", "On it", "I'll grab the public parts of " + site + " — public only.");
    try {
      const summary = await runCollectionFetch({
        requested_by_actor_id: "local-owner",
        web_only: true,
        safe_mode: true,
        max_depth: depth,
        scope: [url]
      });
      appendMessage("assistant", "Done", friendlyCollectionDone(summary, site));
      addAction("Check on processing", () => navigateTo("tab-work", "work-processing"));
    } catch (error) {
      appendMessage("assistant", "That didn't work", error instanceof Error ? error.message : "Something went wrong.");
    }
    setStatus("Ready");
  };

  const navigationFromMessage = (text) => {
    const lower = text.toLowerCase();
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(web fetch|auto bypass|bypass fetch|fetch tools)\\b/.test(lower)) {
      return navigateTo("tab-chat", "chat-web-fetch");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(uploads|guided upload)\\b/.test(lower)) {
      return navigateTo("tab-add-data", "uploads-collection");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(sources)\\b/.test(lower)) {
      return navigateTo("tab-add-data", "sources-panel");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(user security|password|totp|2fa)\\b/.test(lower)) {
      return navigateTo("tab-settings", "user-security");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(configuration|environment|env settings)\\b/.test(lower)) {
      return navigateTo("tab-settings", "settings");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(safety|audit|approvals)\\b/.test(lower)) {
      return navigateTo("tab-settings", "safety-audit");
    }
    if (/\\b(open|show|go to)\\b/.test(lower) && /\\b(media library|media)\\b/.test(lower)) {
      navigateTo("tab-add-data", "browser-web-router-import");
      document.querySelector("[data-grok-open-media]")?.click();
      return true;
    }
    if (/\\b(add data|upload|new source|import data|bring in)\\b/.test(lower)) return navigateTo("tab-add-data", "uploads-collection");
    if (/\\b(auto bypass|full auto bypass|bypass fetch|fetch public|web fetch)\\b/.test(lower) && !extractUrl(text)) {
      return navigateTo("tab-chat", "chat-web-fetch");
    }
    if (/\\b(processing|work queue|work item|check processing|pipeline)\\b/.test(lower)) return navigateTo("tab-work", "work-processing");
    if (/\\b(settings|password|totp|llm provider|environment)\\b/.test(lower)) return navigateTo("tab-settings", "settings");
    if (/\\b(approval|safety|audit)\\b/.test(lower)) return navigateTo("tab-settings", "safety-audit");
    if (/\\b(diagnostics|advanced|service readiness)\\b/.test(lower)) return navigateTo("tab-advanced", "advanced-diagnostics");
    if (/\\b(evidence library|documents|reports|memory)\\b/.test(lower)) return navigateTo("tab-chat", "evidence-panel");
    if (/\\b(open chat|back to chat)\\b/.test(lower)) return navigateTo("tab-chat", "assistant");
    return false;
  };

  const executeChatCommand = async (text) => {
    const lower = text.toLowerCase();
    const url = extractUrl(text);
    const depth = extractDepth(lower);

    if (/\\b(what's still running|what is still running|still running|check on it|processing status)\\b/.test(lower)) {
      setStatus("Checking...");
      appendMessage("assistant", "Work queue", "Let me show you what's still processing — open the Work tab or say 'show work items' for details.");
      navigateTo("tab-work", "work-processing");
      setStatus("Ready");
      return true;
    }

    if (/\\b(what do you know|what have you got|what did you save|what do you know so far)\\b/.test(lower)) {
      appendMessage("assistant", "Your stuff", "I'll search what you've already saved locally. Ask a specific question next — like 'what did I upload today?'");
      setStatus("Ready");
      return false;
    }

    if (/\\b(paste|add notes|add text|upload text|i want to paste)\\b/.test(lower)) {
      navigateTo("tab-add-data", "uploads-collection");
      appendMessage("assistant", "Add stuff", "Opened the upload area — paste your text there, or just drop it in chat and tell me what it is.");
      setStatus("Ready");
      return true;
    }

    if (/\\b(help|what can you do|commands|capabilities|feature list)\\b/.test(lower)) {
      appendMessage(
        "assistant",
        "What I can do",
        "Talk normally — paste a link and say what you want from it, ask questions about stuff you've saved, paste notes to add, check what's still processing, or open settings. If I'm unsure, I'll ask a plain question instead of making you learn command names."
      );
      addAction("Open web fetch tools", () => navigateTo("tab-chat", "chat-web-fetch"));
      addAction("Show project health", () => handleSend("Show project health."));
      return true;
    }

    if (/\\b(max reach|go anywhere|omnifetch|max bypass)\\b/.test(lower) && url) {
      setStatus("Preparing deep fetch...");
      appendMessage("assistant", "Deep Fetch", "Starting host bridge and Playwright if needed, then fetching " + url + " with strongest tier (CDP/headed Playwright, multi-profile, scroll/expand, session header where provided).");
      try {
        await ensureMaxReachInfrastructure();
        setStatus("Running deep fetch...");
        const summary = await runCollectionFetch({
          requested_by_actor_id: "local-owner",
          max_reach: true,
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url]
        });
        appendMessage("assistant", "Deep fetch complete", summarizeCollection(summary) + " Mode: " + String(summary?.mode || "web_max_reach_fetch"));
        addAction("Check processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "Deep fetch failed", error instanceof Error ? error.message : "Unknown error");
        addAction("Open web fetch tools", () => navigateTo("tab-chat", "chat-web-fetch"));
      }
      setStatus("Ready");
      return true;
    }

    if (/\\b(auto bypass|full auto bypass)\\b/.test(lower) && url) {
      setStatus("Preparing deep fetch...");
      appendMessage("assistant", "Deep Fetch", "Starting host bridge if needed, then fetching " + url + " with authorized collection techniques, Playwright, and session header.");
      try {
        try {
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), 120000);
          await fetch("http://127.0.0.1:" + agentPort + "/ensure", { method: "POST", signal: controller.signal });
          clearTimeout(timeoutId);
        } catch (error) {
          console.warn("Host bridge ensure agent unavailable:", error);
        }
        const ensureResponse = await fetch("/api/host-bridge/ensure-max-reach", { method: "POST" });
        if (!ensureResponse.ok) {
          const ensurePayload = await ensureResponse.json().catch(() => ({}));
          throw new Error(ensurePayload?.detail || "Host bridge is not ready");
        }
        setStatus("Running deep fetch...");
        const summary = await runCollectionFetch({
          requested_by_actor_id: "local-owner",
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url]
        });
        appendMessage("assistant", "Deep fetch complete", summarizeCollection(summary) + " Ask a question over the new evidence.");
        addAction("Check processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "Deep fetch failed", error instanceof Error ? error.message : "Unknown error");
        addAction("Open web fetch tools", () => navigateTo("tab-chat", "chat-web-fetch"));
      }
      setStatus("Ready");
      return true;
    }

    if (/\\b(bypass fetch|authorized bypass|session bypass)\\b/.test(lower) && url) {
      const cookie = extractCookie(text);
      const authorization = extractBearer(text);
      if (!cookie && !authorization) {
        fillWebFetchField("bypass_page_url", url);
        navigateTo("tab-chat", "chat-web-fetch");
        appendMessage(
          "assistant",
          "Session needed",
          "Opened session fetch with your URL. Paste a Cookie header or bearer token in Web fetch tools, or say: session fetch " + url + " cookie: session_id=..."
        );
        return true;
      }
      setStatus("Running session fetch...");
      appendMessage("assistant", "Session Fetch", "Using your session to fetch " + url + ".");
      try {
        const body = {
          requested_by_actor_id: "local-owner",
          bypass_auth: true,
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url],
          referer: url
        };
        if (cookie) body.cookie = cookie;
        if (authorization) body.authorization = authorization;
        const summary = await runCollectionFetch(body);
        appendMessage("assistant", "Session fetch complete", summarizeCollection(summary));
        addAction("Check processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "Session fetch failed", error instanceof Error ? error.message : "Unknown error");
      }
      setStatus("Ready");
      return true;
    }

    if (/\\b(fetch public|fetch page|fetch url|scrape url|scrape page)\\b/.test(lower) && url) {
      setStatus("Fetching public page...");
      appendMessage("assistant", "Public fetch", "Fetching " + url + ".");
      try {
        const summary = await runCollectionFetch({
          requested_by_actor_id: "local-owner",
          web_only: true,
          safe_mode: true,
          max_depth: depth,
          scope: [url]
        });
        appendMessage("assistant", "Fetch complete", summarizeCollection(summary));
        addAction("Check processing", () => navigateTo("tab-work", "work-processing"));
      } catch (error) {
        appendMessage("assistant", "Fetch failed", error instanceof Error ? error.message : "Unknown error");
      }
      setStatus("Ready");
      return true;
    }

    if (/\\b(get stuff from|grab from|pull from|stuff from)\\b/.test(lower) && url) {
      const interpretation = interpretFetchIntent(text, lower, url);
      if (interpretation && !interpretation.clear) {
        askClarification(interpretation);
        return true;
      }
      if (interpretation?.clear) {
        await executeResolvedFetchIntent({ intent: interpretation.intent, url });
        return true;
      }
    }

    if (/\\b(fetch|import url|collect url)\\b/.test(lower) && url && !/\\b(bypass|public)\\b/.test(lower)) {
      const interpretation = interpretFetchIntent(text, lower, url);
      if (interpretation && !interpretation.clear) {
        askClarification(interpretation);
        return true;
      }
      await executeResolvedFetchIntent({ intent: interpretation?.intent || "auto_bypass", url });
      return true;
    }

    if (/\\b(user status|auth status|am i logged in)\\b/.test(lower)) {
      setStatus("Checking user status...");
      try {
        const response = await fetch("/api/user/status");
        const payload = await response.json();
        appendMessage("assistant", "User status", JSON.stringify(payload));
      } catch (error) {
        appendMessage("assistant", "Error", error instanceof Error ? error.message : "Status check failed");
      }
      setStatus("Ready");
      return true;
    }

    if (/\\b(open media|media library|show media)\\b/.test(lower)) {
      navigateTo("tab-add-data", "browser-web-router-import");
      document.querySelector("[data-grok-open-media]")?.click();
      appendMessage("assistant", "Media library", "Opened the media library viewer.");
      return true;
    }

    return false;
  };

  const appendMessage = (role, label, text, extraClass) => {
    if (!feed) return null;
    const article = document.createElement("article");
    article.className = "message " + (role === "user" ? "userMessage" : role === "system" ? "systemMessage" : "assistantMessage") + (extraClass ? " " + extraClass : "");
    const avatar = document.createElement("div");
    avatar.className = "avatar";
    avatar.textContent = role === "user" ? "YOU" : role === "system" ? "SYS" : "IG";
    const bubble = document.createElement("div");
    bubble.className = "messageBubble";
    const messageLabel = document.createElement("span");
    messageLabel.className = "messageLabel";
    messageLabel.textContent = label;
    const paragraph = document.createElement("p");
    paragraph.textContent = text;
    bubble.append(messageLabel, paragraph);
    article.append(avatar, bubble);
    feed.appendChild(article);
    feed.scrollTop = feed.scrollHeight;
    return article;
  };

  const setStatus = (text) => {
    if (status) status.textContent = text;
  };

  const clearActions = () => {
    if (!actions) return;
    actions.replaceChildren();
    actions.hidden = true;
  };

  const addAction = (label, handler) => {
    if (!actions) return;
    actions.hidden = false;
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = label;
    button.addEventListener("click", handler);
    actions.appendChild(button);
  };

  const syncInputs = (text) => {
    if (chatMessage) chatMessage.value = text;
    if (agentInput) agentInput.value = text;
  };

  const looksLikeEvidenceQuestion = (intent, text) => {
    const understanding = intent?.request_understanding || {};
    const lower = text.toLowerCase();
    if (/\\b(max reach|go anywhere|omnifetch|auto bypass|bypass fetch|fetch public|fetch url|scrape|open web fetch|help|what can you do|commands)\\b/.test(lower)) return false;
    if (extractUrl(text) && /\\b(fetch|bypass|scrape|import url)\\b/.test(lower)) return false;
    if (understanding.evidence_required) return true;
    if (understanding.category === "ask_question" || understanding.category === "review_evidence") return true;
    if (/\\b(what|why|how|when|where|who|cite|evidence|document|upload|bill|log|say|mean|failed|summary)\\b/.test(lower)) return true;
    if (!intent?.proposed_action && !understanding.unsupported_or_unsafe) return true;
    return false;
  };

  const actionLabels = {
    show_project_health: "Show project health",
    show_git_status: "Show git status",
    show_latest_diff: "Show latest DIFF",
    show_work_items: "Show work items",
    run_retrieval_preview: "Run retrieval preview",
    start_stack: "Start stack",
    stop_stack: "Stop stack",
    run_last_healthy_stack: "Run last healthy stack"
  };

  const summarizeIntent = (intent) => {
    const understanding = intent?.request_understanding || {};
    const parts = [];
    if (understanding.wants) parts.push(understanding.wants);
    if (understanding.category) parts.push("Category: " + understanding.category);
    if (understanding.next_step) parts.push(understanding.next_step);
    if (intent?.proposed_action) {
      parts.push("Matched action: " + (actionLabels[intent.proposed_action] || intent.proposed_action));
    }
    if (understanding.evidence_required) parts.push("I'll check local evidence first.");
    if (understanding.approval_required || intent?.approval_required) parts.push("This needs approval before it can run.");
    if (understanding.unsupported_or_unsafe) parts.push(understanding.reason || "This request is not supported as written.");
    return parts.filter(Boolean).join(" ");
  };

  const llmEnabled = hub.getAttribute("data-llm-enabled") === "true";

  const runEvidencePath = async () => {
    if (!chatForm) throw new Error("Evidence engine is unavailable.");
    chatForm.requestSubmit();
    await new Promise((resolve) => setTimeout(resolve, 400));
    const hitCount = previewResults?.getAttribute("data-hit-count") || "0";
    const answerStatus = previewResults?.getAttribute("data-answer-status") || "unknown";
    previewResults?.closest(".chatResultsDock")?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    return { hitCount: Number(hitCount), answerStatus };
  };

  const runLlmEvidenceAnswer = async (text) => {
    const response = await fetch("/api/chat/evidence-answer", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        message: text,
        limit: Number(chatLimit?.value || 5)
      })
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload?.detail || response.statusText || "Evidence answer failed");
    }
    return payload;
  };

  const runAgentPreview = async () => {
    if (!agentPreview) throw new Error("Action engine is unavailable.");
    agentPreview.click();
    await new Promise((resolve) => setTimeout(resolve, 500));
    const summary = document.querySelector("[data-agent-understanding-summary]");
    return summary?.textContent || "Action preview complete.";
  };

  const handleSend = async (textOverride) => {
    const text = (textOverride || input?.value || "").trim();
    if (!text) return;
    if (input) input.value = text;
    syncInputs(text);
    clearActions();
    appendMessage("user", "You", text);
    setStatus("Understanding request...");
    const lower = text.toLowerCase();
    const url = extractUrl(text);

    if (pendingClarification) {
      const resolved = resolveClarificationReply(text, lower, pendingClarification);
      if (resolved?.cancel) {
        pendingClarification = null;
        appendMessage("assistant", "No problem", "Okay, I won't fetch that. What else?");
        setStatus("Ready");
        return;
      }
      if (resolved?.intent) {
        pendingClarification = null;
        await executeResolvedFetchIntent(resolved);
        return;
      }
      appendMessage("assistant", "Quick check", "Pick one of the buttons below, or say yeah / no / public / login.");
      pendingClarification.options.forEach((option) => {
        addAction(option.label, async () => {
          pendingClarification = null;
          await executeResolvedFetchIntent(option);
        });
      });
      setStatus("Waiting for your call");
      return;
    }

    const hasExplicitCommand = /\\b(max reach|auto bypass|fetch public|bypass fetch|go all out|try harder)\\b/.test(lower);
    const fetchInterpretation = interpretFetchIntent(text, lower, url);
    if (fetchInterpretation && !hasExplicitCommand) {
      if (fetchInterpretation.clear) {
        await executeResolvedFetchIntent({ intent: fetchInterpretation.intent, url: fetchInterpretation.url });
        return;
      }
      askClarification(fetchInterpretation);
      return;
    }

    if (await executeChatCommand(text)) {
      return;
    }

    if (navigationFromMessage(text)) {
      appendMessage("assistant", "Navigation", "Opened the matching workspace view. You can keep chatting here or use the panel that opened.");
      setStatus("Ready");
      return;
    }

    try {
      const intentResponse = await fetch("/api/agent/intent", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ message: text, parameters: {}, actor_id: "local-owner" })
      });
      const intent = await intentResponse.json();
      const summary = summarizeIntent(intent) || "Request received.";
      appendMessage("assistant", "IGY6", summary);

      if (looksLikeEvidenceQuestion(intent, text)) {
        setStatus(llmEnabled ? "Searching evidence and calling local Ollama..." : "Searching local evidence...");
        let llmPayload = null;
        if (llmEnabled) {
          try {
            llmPayload = await runLlmEvidenceAnswer(text);
          } catch (error) {
            appendMessage(
              "assistant",
              "Local model",
              error instanceof Error ? error.message : "Local model call failed; showing deterministic evidence instead."
            );
          }
        }
        const evidence = await runEvidencePath();
        if (llmPayload?.llm_text) {
          appendMessage("assistant", "Ollama answer", llmPayload.llm_text);
        } else if (llmPayload?.generation_mode === "local_llm_evidence_grounded" && llmPayload?.redacted_output_preview) {
          appendMessage("assistant", "Ollama answer", llmPayload.redacted_output_preview);
        } else if (llmPayload && llmEnabled) {
          appendMessage(
            "assistant",
            "Local model",
            llmPayload.llm_error
              ? "Ollama was unavailable (" + llmPayload.llm_error + "). Deterministic evidence is shown below."
              : "No local model text returned (" + (llmPayload.generation_mode || "unknown") + "). Deterministic evidence is shown below."
          );
        }
        appendMessage(
          "assistant",
          "Evidence",
          evidence.hitCount > 0
            ? "Found " + evidence.hitCount + " local evidence hit(s). Review citations below, then save the answer if you want history."
            : "No matching local evidence yet. Say 'add data' to upload, or 'check processing' to see pipeline status."
        );
        if (evidence.hitCount > 0 && saveAnswer) {
          addAction("Save answer record", () => saveAnswer.click());
        }
        if (evidence.hitCount === 0) {
          addAction("Add data", () => switchTab("tab-add-data"));
          addAction("Check processing", () => switchTab("tab-work"));
        }
      } else if (intent?.proposed_action) {
        setStatus("Previewing bounded action...");
        const previewSummary = await runAgentPreview();
        appendMessage("assistant", "Action preview", previewSummary);
        if (agentExecute && !agentExecute.disabled) addAction("Run safe action", () => agentExecute.click());
        if (agentApproval && !agentApproval.disabled) addAction("Request approval", () => agentApproval.click());
        if (agentExecuteApproved && !agentExecuteApproved.disabled) addAction("Run with approval", () => agentExecuteApproved.click());
      } else {
        appendMessage(
          "assistant",
          "Not sure yet",
          url
            ? "I see a link in there — want me to try pulling content from " + siteLabel(url) + "? Say what you're after (public pages, locked stuff, images, etc.) and I'll ask if I'm still unsure."
            : "I'm not totally sure what you want. You can paste a link, ask about stuff you've saved, say you want to add notes, or ask what's still running."
        );
        if (url) {
          const interpretation = interpretFetchIntent(text, lower, url);
          if (interpretation && !interpretation.clear) {
            askClarification(interpretation);
            setStatus("Waiting for your call");
            return;
          }
        }
        addAction("Show project health", () => handleSend("Show project health."));
        addAction("Add data", () => switchTab("tab-add-data"));
      }
      setStatus("Ready");
    } catch (error) {
      const detail = error instanceof Error ? error.message : "Unknown error";
      appendMessage("assistant", "Error", detail);
      setStatus("Error");
    }
  };

  sendButton?.addEventListener("click", () => handleSend());
  input?.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  });

  chips.forEach((chip) => {
    chip.addEventListener("click", () => {
      const prompt = chip.getAttribute("data-chat-chip") || "";
      if (input) input.value = prompt;
      handleSend(prompt);
    });
  });
})();
`;

  return (
    <section
      className="unifiedChatHub"
      data-unified-chat
      data-llm-enabled={llmEnabled ? "true" : "false"}
      data-llm-model={llmModel || "not-selected"}
      aria-label="IGY6 chat"
    >
      <div className="chatHubHeader">
        <div>
          <p className="eyebrow">Chat</p>
          <h2>Ask, upload, check status, or run safe actions</h2>
        </div>
        <div className="chatHubHeaderActions">
          <button type="button" className="simpleModeToggle" data-minimal-ui-toggle aria-pressed="false">Simple mode</button>
          <span className="statusText" data-chat-status>Ready</span>
        </div>
      </div>
      <div className="retrievalStrip chatHubStats" aria-label="Workspace snapshot">
        <span>{llmEnabled ? `Ollama · ${llmModel || "model not set"}` : "Deterministic evidence mode"}</span>
        <span>{sourceCount} sources</span>
        <span>{evidenceCount} evidence items</span>
        <span>{chunkCount} chunks</span>
        <span>{workItemCount} work items</span>
        <span>{pendingApprovals} pending approvals</span>
        <span>{vectorReady ? "Vector memory ready" : "Vector memory missing"}</span>
      </div>
      <div className="conversationWindow chatHubFeed" data-chat-feed>
        <article className="message systemMessage">
          <div className="avatar">SYS</div>
          <div className="messageBubble">
            <span className="messageLabel">Welcome</span>
            <p data-chat-welcome-text>Type anything here: ask over evidence, run deep fetch or public fetch with a URL, open any panel (settings, uploads, web fetch), or run bounded actions like project health and stack control. Say help for the full command list.</p>
            <div className="messageMeta">
              <StatusPill state="local-first" />
              <StatusPill state="read-only-default" />
            </div>
          </div>
        </article>
      </div>
      <div className="chatQuickChips" aria-label="Quick starts">
        <button type="button" data-chat-chip="help">Commands</button>
        <button type="button" data-chat-chip="open web fetch">Web fetch</button>
        <button type="button" data-chat-chip="deep fetch https://example.com">Deep Fetch</button>
        <button type="button" data-chat-chip="Show project health.">Project health</button>
        <button type="button" data-chat-chip="Check processing status">Check processing</button>
        <button type="button" data-chat-chip="Add data">Add data</button>
        <button type="button" data-chat-chip="Open settings">Settings</button>
        <button type="button" data-chat-chip="What failed in this build log? Cite the evidence.">Cite build log</button>
      </div>
      <div className="chatComposer">
        <label>
          <span className="srOnly">Message</span>
          <textarea
            data-chat-input
            rows={2}
            placeholder="Talk to me — paste a link, ask a question, say what you want..."
            defaultValue=""
          />
        </label>
        <button type="button" data-chat-send>Send</button>
      </div>
      <div className="chatFollowUpActions" data-chat-actions hidden />
      <ClientScript script={script} />
    </section>
  );
}

