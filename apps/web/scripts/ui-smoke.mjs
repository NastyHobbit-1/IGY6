import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const appDir = join(scriptDir, "..");
const appRoot = join(appDir, "src/app");
const apiDir = join(appRoot, "api");
const componentsDir = join(appRoot, "components");

const failures = [];

function walkSourceFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    if (statSync(fullPath).isDirectory()) {
      files.push(...walkSourceFiles(fullPath));
      continue;
    }
    if (/\.(tsx?|jsx?)$/.test(entry)) {
      files.push(fullPath);
    }
  }
  return files;
}

function loadUiSources() {
  const pagePath = join(appRoot, "page.tsx");
  const page = readFileSync(pagePath, "utf8");
  const componentPaths = walkSourceFiles(componentsDir).sort((left, right) => {
    const leftRelative = relative(componentsDir, left);
    const rightRelative = relative(componentsDir, right);
    if (leftRelative === "HomePage.tsx") return -1;
    if (rightRelative === "HomePage.tsx") return 1;
    return leftRelative.localeCompare(rightRelative);
  });

  const componentFiles = componentPaths.map((filePath) => ({
    path: relative(appRoot, filePath).replace(/\\/g, "/"),
    content: readFileSync(filePath, "utf8")
  }));

  const uiCorpus = [page, ...componentFiles.map((file) => file.content)].join("\n");
  return { page, componentFiles, uiCorpus };
}

const { page, componentFiles, uiCorpus } = loadUiSources();
const styles = readFileSync(join(appRoot, "globals.css"), "utf8");

function check(name, condition) {
  if (!condition) {
    failures.push(name);
  }
}

function includesAll(name, text, values) {
  for (const value of values) {
    check(`${name}: ${value}`, text.includes(value));
  }
}

function appearsAfterInSomeFile(name, earlier, later) {
  const matched = componentFiles.some((file) => {
    const earlierIndex = file.content.indexOf(earlier);
    const laterIndex = file.content.indexOf(later);
    return earlierIndex >= 0 && laterIndex > earlierIndex;
  });
  check(`${name}: ${later} appears after ${earlier} in a component file`, matched);
}

check("page entry re-exports HomePage", page.includes('import { HomePage } from "./components/HomePage";'));
check("page entry exports HomePage default", page.includes("export default HomePage;"));
check("HomePage component exists", existsSync(join(componentsDir, "HomePage.tsx")));
check("components directory is populated", componentFiles.length >= 40);

includesAll("top-level tabs", uiCorpus, [
  'htmlFor="tab-results">Chat',
  'htmlFor="tab-add-data">Data',
  'htmlFor="tab-work">Work',
  'htmlFor="tab-settings">Settings',
  'htmlFor="tab-advanced">More'
]);

includesAll("chat-first shell", uiCorpus, [
  "data-unified-chat",
  "data-chat-input",
  "data-chat-send",
  "data-chat-chip",
  "data-chat-wired",
  "UnifiedChatHub",
  "OnboardingJourney",
  "journeyStrip",
  "chatFirstShell",
  "ClientScript",
  "Ask a question or request an action..."
]);

includesAll("browser api proxies", uiCorpus, [
  "/api/user/status",
  "/api/user/change-password",
  "/api/artifacts",
  "/api/collection-runs/full-access",
  "/api/chat/evidence-answer"
]);

const requiredApiRoutes = [
  "user/status/route.ts",
  "user/change-password/route.ts",
  "user/generate-totp/route.ts",
  "user/confirm-totp/route.ts",
  "artifacts/route.ts",
  "artifacts/[artifact_id]/content/route.ts",
  "collection-runs/full-access/route.ts",
  "host-bridge/ensure-max-reach/route.ts",
  "bypass-intel/status/route.ts",
  "bypass-intel/harvest/route.ts",
  "chat/evidence-answer/route.ts",
  "chat/retrieval-preview/route.ts",
  "settings/env/route.ts",
  "settings/env/verify/route.ts",
  "settings/env/apply/route.ts",
  "ops/runtime-logs/route.ts",
  "ops/runtime-logs/append/route.ts"
];

for (const route of requiredApiRoutes) {
  check(`api route file: ${route}`, existsSync(join(apiDir, route)));
}

includesAll("chat web fetch dock", uiCorpus, [
  "ChatWebFetchDock",
  'id="chat-web-fetch"',
  "data-chat-web-fetch",
  "Web fetch tools",
  "executeChatCommand",
  "ensureMaxReachInfrastructure",
  "ensureHostBridgeInfrastructure",
  "/api/host-bridge/ensure-max-reach",
  "/api/bypass-intel/status",
  "/api/bypass-intel/harvest",
  "BypassIntelPanel",
  "data-bypass-intel-panel",
  "Deep fetch · Public fetch · Session fetch",  /* visible em text */
  "/ensure-max-reach",  /* internal route kept */
  "Deep fetch",
  "data-max-reach-url-fetch",  /* data attr kept for behavior */
  "max_reach: true",  /* payload field kept */
  "deep fetch https://example.com",
  "session fetch https://...",
  "public fetch https://...",
  'data-chat-chip="open web fetch"',
  'data-chat-chip="help"',
  "deep fetch https://example.com"  /* visible command example now present in ChatWebFetchDock help text */
]);

includesAll("minimal ui mode", uiCorpus, [
  "data-minimal-ui-root",
  "data-minimal-ui-toggle",
  "MINIMAL_UI_TOGGLE_SCRIPT",
  "MinimalWorkspacePanel",
  "data-minimal-workspace",
  "interpretFetchIntent",
  "pendingClarification",
  "Do you want me to try and get the paid or locked content from",
  "Simple mode"
]);

includesAll("implemented collection panels", uiCorpus, [
  "data-bwr-collect",
  "Collect pasted text",
  "data-lp-collect",
  "Collect scoped import",
  "data-media-import-mvp",
  "Upload media file",
  "Preview media status",
  "PipelineOperationsPanel",
  "data-pipeline-operations",
  "data-hypothesis-create-form",
  "data-experiment-status-button",
  "data-graph-lineage-ops",
  "agent_action"
]);

includesAll("workflow section anchors", uiCorpus, [
  'id="home"',
  'id="assistant"',
  'id="chat-web-fetch"',
  'id="data-knowledge"',
  'id="sources-panel"',
  'id="uploads-collection"',
  '#browser-web-router-import">Web fetch',
  'id="evidence-panel"',
  'id="memory-panel"',
  'id="analysis-panel"',
  'id="data-search"',
  'id="work-processing"',
  'id="reports"',
  'id="safety-audit"',
  'id="settings"',
  'id="user-security"'
]);

includesAll("settings hub navigation", uiCorpus, [
  "SettingsHubNav",
  "settingsHubNav",
  "UserSecurityPanel",
  "TroubleshootingLogsPanel",
  "data-settings-wired",
  "data-troubleshooting-logs",
  "data-log-refresh",
  "/api/ops/runtime-logs",
  "Configuration",
  "User &amp; Security",
  "Troubleshooting",
  "Safety & Audit"
]);

includesAll("tab panel mapping", uiCorpus, [
  'data-tab-panel="home"',
  'data-tab-panel="add-data"',
  'data-tab-panel="work"',
  'data-tab-panel="results"',
  'data-tab-panel="settings"',
  'data-tab-panel="advanced"'
]);

check(
  "old navigation array is not restored",
  !uiCorpus.includes('["Chat", "Agent Command", "Sources", "Evidence", "Memory", "Work Queue", "Approvals", "Reports", "Audit", "Settings"]')
);

includesAll("assistant action labels", uiCorpus, [
  "Show project health",
  "Show git status",
  "Show latest DIFF",
  "Show work items",
  "Run retrieval preview",
  "Start stack",
  "Stop stack",
  "Run last healthy stack"
]);

includesAll("assistant action gating controls", uiCorpus, [
  "data-agent-preview",
  "data-agent-execute disabled",
  "data-agent-request-approval disabled",
  "data-agent-execute-approved disabled",
  "Preview action",
  "Run safe action",
  "Request approval",
  "Run with approval"
]);

includesAll("assistant evidence controls", uiCorpus, [
  "Ask a question or request an action...",
  "Ask over evidence",
  "What does this document say about my bill?",
  "What failed in this build log? Cite the evidence.",
  "deterministic evidence",
  "local LLM evidence-grounded",
  "unavailable until model is selected"
]);

includesAll("local llm status copy", uiCorpus, [
  "Local LLM Status",
  "Provider",
  "Health status",
  "Answer mode",
  "Evidence required",
  "Use local model to summarize uploaded warranty note using only evidence.",
  "Use local model to explain build log failure with citations.",
  "Advanced: raw provider diagnostics",
  "No model calls are made while LLM_PROVIDER is none"
]);

includesAll("advanced panels", uiCorpus, [
  "<details",
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Advanced: source IDs, permission IDs, and raw source data",
  "Advanced: raw artifact IDs, collection run IDs, and upload JSON",
  "Advanced: dispatch controls, work item IDs, and raw queue JSON",
  "Advanced: approval IDs, audit JSON, route filters, and raw safety records",
  "Advanced: report render route, report IDs, output JSON, and export details",
  "Advanced Route Console"
]);

appearsAfterInSomeFile(
  "raw parameters hidden behind advanced",
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Raw parameters JSON"
);

appearsAfterInSomeFile(
  "approval id hidden behind advanced",
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Approval ID for approved action"
);

appearsAfterInSomeFile(
  "legacy route console hidden behind advanced",
  "Advanced Route Console",
  "<MvpActionConsole />"
);

includesAll("manual upload guided workflow", uiCorpus, [
  "Guided Upload",
  "Step 1: Select or create source.",
  "Step 2: Check approval status.",
  "Step 3: Request approval if required.",
  "Step 4: Upload text or a safe file extract.",
  "Step 5: Review created records.",
  "Step 6: Next action.",
  "Router Troubleshooting Notes",
  "IGY6 Build Logs",
  "Allow IGY6 to process this uploaded troubleshooting note.",
  "Approve processing this local build log for evidence extraction."
]);

includesAll("empty and next-step guidance states", uiCorpus, [
  "No sources registered yet.",
  "No collection runs recorded yet.",
  "No evidence items recorded yet.",
  "No work items recorded yet.",
  "No approvals recorded yet.",
  "No reports recorded yet.",
  "Add a data source first.",
  "Add approved text and check processing.",
  "Ask a question over local evidence."
]);

includesAll("safety posture", uiCorpus, [
  "local-first",
  "System ready",
  "Background worker ready",
  "Rust API",
  "Rust worker",
  "Legacy API",
  "inactive / archived",
  "Legacy scheduler",
  "approval-gated",
  "Background processing is ready",
  "Old Python services"
]);

includesAll("split component structure", uiCorpus, [
  "export async function HomePage",
  "export function UnifiedChatHub",
  "export function GuidedManualTextUpload",
  "export function LifecycleAuditStatusPanel",
  "export function SettingsPanel",
  "export function PipelineOperationsPanel"
]);

includesAll("supporting styles", styles, [
  ".productTabs",
  ".tabList",
  ".tabContent",
  ".readinessStrip",
  ".runtimePosture",
  ".primaryWorkflowGrid",
  ".advancedPanel",
  ".workflowSteps",
  ".workflowTabs",
  ".lifecycleFlow",
  ".fieldGuide",
  ".quickStartGrid",
  ".unifiedChatHub",
  ".chatComposer",
  ".chatQuickChips",
  ".chatEnginePanel",
  ".chatFirstShell",
  ".journeyStrip",
  ".journeyCard",
  ".chatWebFetchDock"
]);

if (failures.length > 0) {
  console.error("UI smoke checks failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`UI smoke checks passed (${componentFiles.length} component files scanned).`);