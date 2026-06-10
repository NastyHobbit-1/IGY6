import fs from "fs";
import path from "path";

const componentsDir = path.resolve("apps/web/src/app/components");

function read(rel) {
  return fs.readFileSync(path.join(componentsDir, rel), "utf8");
}

function write(rel, content) {
  fs.writeFileSync(path.join(componentsDir, rel), content.endsWith("\n") ? content : content + "\n");
}

// Export types
let types = read("types.ts");
types = types.replace(/^type /gm, "export type ");
write("types.ts", types);

// Export constants
let constants = read("constants.ts");
constants = constants.replace(/^const /gm, "export const ");
write("constants.ts", constants);

// Export helpers
let helpers = read("helpers.ts");
helpers = helpers.replace(/^function /gm, "export function ");
write("helpers.ts", helpers);

// Export api
let api = read("api.ts");
api = api.replace(/^async function /m, "export async function ");
write("api.ts", api);

// Export llm-helpers
let llm = read("llm-helpers.ts");
llm = llm.replace(/^function /gm, "export function ");
write("llm-helpers.ts", llm);

// Export UI components
for (const name of ["StatusPill", "EmptyState", "Skeleton", "SkeletonBlock", "TermHelp", "HelpHeading"]) {
  let content = read(`ui/${name}.tsx`);
  content = content.replace(/^function /m, `export function ${name}`);
  if (!content.startsWith("import") && name === "TermHelp") {
    content = 'import { TERM_HELP } from "../constants";\n\n' + content;
  }
  if (!content.startsWith("import") && name === "HelpHeading") {
    content = 'import { TermHelp } from "./TermHelp";\n\n' + content;
  }
  if (!content.startsWith("import") && name === "SkeletonBlock") {
    content = 'import { Skeleton } from "./Skeleton";\n\n' + content;
  }
  write(`ui/${name}.tsx`, content);
}

const panelFiles = fs.readdirSync(componentsDir).filter((f) => f.endsWith(".tsx"));

const typeNames = [
  "HealthResponse", "SourcePermission", "SourceRecord", "CollectionRunRecord", "RawArtifactRecord",
  "NormalizedDocumentRecord", "ChunkRecord", "EvidenceItemRecord", "EvidenceAnswerRecord", "ClaimRecord",
  "VectorCollectionStatus", "GraphSchemaStatus", "PatternRecord", "HypothesisRecord", "PredictionRecord",
  "RecommendationRecord", "CalibrationSummary", "WorkItemRecord", "AgentTaskPlanRecord", "ApprovalRecord",
  "FeedbackRecord", "OutcomeRecord", "ImprovementRecord", "ExperimentRecord", "ReportRecord", "AuditEventRecord",
  "EnvSettingRecord", "EnvUnmanagedRecord", "EnvSettingsResponse", "AgentActionCapability", "AgentCapabilitiesResponse",
  "ApiResult", "ConnectorContractStep", "SourceConnectorStatus", "BrowserWebRouterImportType", "MediaImportType",
  "LocalProjectDiagnosticsMode", "TermHelpContent", "LlmDisplay"
];

const constantNames = [
  "TERM_HELP", "CONNECTOR_CONTRACT_STEPS", "SOURCE_CONNECTOR_STATUS", "BROWSER_WEB_ROUTER_IMPORT_TYPES",
  "MEDIA_IMPORT_TYPES", "LOCAL_PROJECT_DIAGNOSTICS_MODES", "HOST_BRIDGE_AGENT_PORT", "WEB_FETCH_MAX_REACH_SCRIPT",
  "WEB_FETCH_AUTO_BYPASS_SCRIPT", "WEB_FETCH_BYPASS_SCRIPT", "WEB_FETCH_PUBLIC_SCRIPT", "MINIMAL_UI_TOGGLE_SCRIPT",
  "WORKSPACE_HASH_ROUTER_SCRIPT", "RUNTIME_POSTURE", "USER_READINESS"
];

const helperNames = [
  "formatDate", "formatBytes", "excerpt", "stringArrayFromUnknown", "numberFromUnknown", "uniqueStringValues",
  "shortRecordId", "jsonString", "jsonStringList", "evidenceReviewState", "evidenceReviewNote", "metadataMentionsId",
  "workItemRelatedIds", "workItemGuidance", "workItemDispatchVisibility"
];

const llmHelperNames = ["buildLlmDisplay", "settingValue", "redactLlmUrl"];

const uiNames = ["StatusPill", "EmptyState", "Skeleton", "SkeletonBlock", "TermHelp", "HelpHeading"];

const panelNames = panelFiles
  .map((f) => f.replace(".tsx", ""))
  .filter((n) => n !== "HomePage");

function buildImports(fileName, content) {
  const imports = [];
  const usedTypes = typeNames.filter((t) => new RegExp(`\\b${t}\\b`).test(content));
  if (usedTypes.length) {
    imports.push(`import type { ${usedTypes.join(", ")} } from "./types";`);
  }

  const usedConstants = constantNames.filter((c) => content.includes(c));
  if (usedConstants.length) {
    imports.push(`import { ${usedConstants.join(", ")} } from "./constants";`);
  }

  const usedHelpers = helperNames.filter((h) => new RegExp(`\\b${h}\\(`).test(content) || new RegExp(`\\b${h}\\b`).test(content));
  if (usedHelpers.length) {
    imports.push(`import { ${[...new Set(usedHelpers)].join(", ")} } from "./helpers";`);
  }

  const usedLlm = llmHelperNames.filter((h) => content.includes(h));
  if (usedLlm.length) {
    imports.push(`import { ${usedLlm.join(", ")} } from "./llm-helpers";`);
  }

  if (content.includes("getJson")) {
    imports.push('import { getJson } from "./api";');
  }

  if (content.includes("ClientScript") || content.includes("DomJsonScript")) {
    imports.push('import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";');
  }

  const usedUi = uiNames.filter((u) => new RegExp(`<${u}[\\s/>]`).test(content) || content.includes(`${u}(`));
  for (const ui of usedUi) {
    imports.push(`import { ${ui} } from "./ui/${ui}";`);
  }

  const usedPanels = panelNames.filter((p) => p !== fileName.replace(".tsx", "") && (
    new RegExp(`<${p}[\\s/>]`).test(content) || content.includes(`${p}(`)
  ));
  for (const panel of usedPanels) {
    imports.push(`import { ${panel} } from "./${panel}";`);
  }

  return [...new Set(imports)].join("\n");
}

for (const file of panelFiles) {
  if (file === "HomePage.tsx") continue;
  let content = read(file);
  const fnName = file.replace(".tsx", "");
  if (!content.startsWith("type ") && !content.startsWith("export type ")) {
    content = content.replace(/^function /m, `export function ${fnName}`);
  } else {
    // BaselinePatternExpansionPanel has leading type
    content = content.replace(/^type /m, "type ");
    content = content.replace(/^function /m, `export function ${fnName}`);
  }
  const importBlock = buildImports(file, content);
  write(file, `${importBlock}\n\n${content}`);
}

// HomePage special handling
let home = read("HomePage.tsx");
home = home.replace(
  "export async function HomePage() {\n    health,",
  "export async function HomePage() {\n  const [\n    health,"
);
const homeImports = buildImports("HomePage.tsx", home);
write("HomePage.tsx", `${homeImports}\n\n${home}`);

console.log("Imports added to", panelFiles.length, "files");