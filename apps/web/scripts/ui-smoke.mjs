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

includesAll("visible tab labels", uiCorpus, [
  ">Chat</label>",
  ">Data</label>",
  ">Work</label>",
  ">Settings</label>",
  ">More</label>"
]);

includesAll("tab inputs", uiCorpus, [
  'id="tab-results"',
  'id="tab-add-data"',
  'id="tab-work"',
  'id="tab-settings"',
  'id="tab-advanced"'
]);

includesAll("core panels", uiCorpus, [
  "UnifiedChatHub",
  "ChatRetrievalPreview",
  "ChatWebFetchDock",
  "GuidedManualTextUpload",
  "MediaImportMvp",
  "BrowserWebRouterCollectorMvp",
  "LocalProjectPcDiagnosticsHardeningPanel",
  "SettingsPanel",
  "UserSecurityPanel",
  "TroubleshootingLogsPanel",
  "MvpActionConsole",
  "PipelineOperationsPanel"
]);

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
  "Deep fetch · Public fetch · Session fetch",
  "/ensure-max-reach",
  "Deep fetch",
  "data-max-reach-url-fetch",
  "max_reach: true",
  "deep fetch https://example.com",
  "session fetch https://...",
  "public fetch https://...",
  'data-chat-chip="open web fetch"',
  'data-chat-chip="help"',
  "deep fetch https://example.com"
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
  "data-media-upload-binary",
  "Upload media file",
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

const requiredApiRoutes = [
  "chat/evidence-answer/route.ts",
  "chat/retrieval-preview/route.ts",
  "settings/env/route.ts",
  "settings/env/verify/route.ts",
  "settings/env/apply/route.ts",
  "ops/runtime-logs/route.ts",
  "ops/runtime-logs/append/route.ts",
  "user/status/route.ts",
  "user/change-password/route.ts",
  "user/generate-totp/route.ts",
  "user/confirm-totp/route.ts",
  "user/verify-unlock/route.ts",
  "media/import/route.ts",
  "host-bridge/status/route.ts",
  "host-bridge/ensure-max-reach/route.ts",
  "collection-runs/full-access/route.ts"
];

for (const route of requiredApiRoutes) {
  check(`api route file: ${route}`, existsSync(join(apiDir, route)));
}

check("responsive CSS helpers present", /responsiveToolbar|responsiveStatusRow|responsivePanelGrid/.test(styles));
check("skeleton CSS present", styles.includes(".skeleton"));

if (failures.length) {
  console.error("UI smoke checks failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`UI smoke checks passed (${componentFiles.length} component files scanned).`);
