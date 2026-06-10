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
    if (leftRelative === "ProductApp.tsx") return -1;
    if (rightRelative === "ProductApp.tsx") return 1;
    return leftRelative.localeCompare(rightRelative);
  });

  const componentFiles = componentPaths.map((filePath) => ({
    path: relative(appRoot, filePath).replace(/\\/g, "/"),
    content: readFileSync(filePath, "utf8"),
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

const { page: pageContent, uiCorpus: corpus } = { page, uiCorpus };

check("page entry imports ProductApp", pageContent.includes('import { ProductApp } from "./components/ProductApp";'));
check("page entry exports ProductApp default", pageContent.includes("export default ProductApp;"));
check("ProductApp component exists", existsSync(join(componentsDir, "ProductApp.tsx")));
check("components directory is populated", componentFiles.length >= 40);

includesAll("product navigation", corpus, [
  "data-product-app",
  "data-product-nav",
  "data-product-section",
  "Command Center",
  "Add Information",
  "Evidence",
  "Activity",
  "System",
  "Settings",
  "igy6SwitchSection",
]);

includesAll("chat command center", corpus, [
  "data-unified-chat",
  "data-chat-input",
  "data-chat-send",
  "data-chat-chip",
  "data-chat-wired",
  "UnifiedChatHub",
  "ClientScript",
  "Ask a question or request an action...",
  "data-chat-preview-form",
]);

includesAll("add information flow", corpus, [
  "data-add-information",
  "Add text",
  "Add a website",
  "Add a local folder",
  "data-add-submit",
  "/api/collection-runs/full-access",
]);

includesAll("browser api proxies", corpus, [
  "/api/user/status",
  "/api/user/change-password",
  "/api/artifacts",
  "/api/collection-runs/full-access",
  "/api/chat/evidence-answer",
  "/api/memory/vector/chunks/ensure",
]);

const requiredApiRoutes = [
  "user/status/route.ts",
  "user/change-password/route.ts",
  "artifacts/route.ts",
  "artifacts/[artifact_id]/content/route.ts",
  "collection-runs/full-access/route.ts",
  "host-bridge/ensure-max-reach/route.ts",
  "chat/evidence-answer/route.ts",
  "chat/retrieval-preview/route.ts",
  "memory/vector/chunks/ensure/route.ts",
  "settings/env/route.ts",
  "settings/env/verify/route.ts",
  "settings/env/apply/route.ts",
  "work-items/[work_item_id]/status/route.ts",
];

for (const route of requiredApiRoutes) {
  check(`api route file: ${route}`, existsSync(join(apiDir, route)));
}

includesAll("evidence and activity sections", corpus, [
  'id="evidence-panel"',
  "productEvidenceCard",
  'id="work-processing"',
  "productActivityCard",
  "PipelineOperationsPanel",
  "data-pipeline-operations",
]);

includesAll("system and settings", corpus, [
  'id="advanced-diagnostics"',
  "productServiceGrid",
  "StackRestartPanel",
  "data-stack-restart",
  "SettingsPanel",
  "data-settings-env",
  "UserSecurityPanel",
]);

includesAll("plain language policy", corpus, [
  "Technical details",
  "What IGY6 found",
  "Ask IGY6",
  "Bring information into IGY6",
]);

includesAll("legacy tab sync for chat navigation", corpus, [
  'id="tab-results"',
  'id="tab-add-data"',
  'id="tab-work"',
  'id="tab-settings"',
  'id="tab-advanced"',
  "legacySectionMap",
]);

includesAll("split component structure", corpus, [
  "export async function ProductApp",
  "export function UnifiedChatHub",
  "export function AddInformationFlow",
  "export function SettingsPanel",
  "export function PipelineOperationsPanel",
  "export function StackRestartPanel",
]);

includesAll("supporting styles", styles, [
  ".igy6-app",
  ".productSidebar",
  ".productNavButton",
  ".productSection",
  ".productEvidenceCard",
  ".productServiceGrid",
  ".unifiedChatHub",
  ".chatComposer",
  ".chatQuickChips",
]);

includesAll("safety posture", corpus, [
  "local-first",
  "approval",
  "Technical details",
]);

if (failures.length > 0) {
  console.error("UI smoke checks failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`UI smoke checks passed (${componentFiles.length} component files scanned).`);