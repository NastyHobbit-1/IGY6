import fs from "fs";
import path from "path";

const root = path.resolve("apps/web/src/app");
const pagePath = process.argv[2]
  ? path.resolve(process.argv[2])
  : path.join(root, "page.tsx");
const componentsDir = path.join(root, "components");
const uiDir = path.join(componentsDir, "ui");

const lines = fs.readFileSync(pagePath, "utf8").split(/\r?\n/);

function slice(start, end) {
  return lines.slice(start - 1, end).join("\n");
}

function writeFile(relPath, content) {
  const full = path.join(root, relPath);
  fs.mkdirSync(path.dirname(full), { recursive: true });
  fs.writeFileSync(full, content.endsWith("\n") ? content : content + "\n");
}

// types.ts (lines 3-436)
writeFile(
  "components/types.ts",
  slice(3, 436)
);

// constants blocks
const constantBlocks = [
  slice(438, 754),
  slice(876, 1080),
  slice(1115, 1608),
  slice(1740, 1776),
  slice(1879, 1911),
  slice(2830, 2842),
].join("\n\n");

writeFile(
  "components/constants.ts",
  `import type {
  BrowserWebRouterImportType,
  ConnectorContractStep,
  LocalProjectDiagnosticsMode,
  MediaImportType,
  SourceConnectorStatus,
  TermHelpContent,
} from "./types";

${constantBlocks}
`
);

// api.ts
writeFile(
  "components/api.ts",
  `import type { ApiResult } from "./types";

${slice(778, 795)}
`
);

// helpers.ts
writeFile(
  "components/helpers.ts",
  `import type { EvidenceItemRecord, EnvSettingsResponse, WorkItemRecord } from "./types";

${slice(797, 852)}

${slice(2703, 2828)}
`
);

const uiComponents = [
  { name: "StatusPill", start: 854, end: 857 },
  { name: "EmptyState", start: 858, end: 861 },
  { name: "Skeleton", start: 862, end: 865 },
  { name: "SkeletonBlock", start: 866, end: 874 },
  { name: "TermHelp", start: 756, end: 772 },
  { name: "HelpHeading", start: 774, end: 776 },
];

for (const ui of uiComponents) {
  const extraImports =
    ui.name === "SkeletonBlock"
      ? 'import { Skeleton } from "./Skeleton";\n'
      : ui.name === "HelpHeading"
        ? 'import { TermHelp } from "./TermHelp";\n'
        : ui.name === "TermHelp"
          ? 'import { TERM_HELP } from "../constants";\n'
          : "";
  writeFile(
    `components/ui/${ui.name}.tsx`,
    `${extraImports}${slice(ui.start, ui.end)}\n`
  );
}

const panelComponents = [
  { name: "ConnectorContractStatusPanel", start: 1082, end: 1113 },
  { name: "WebFetchToolsPanels", start: 1609, end: 1738 },
  { name: "MinimalWorkspacePanel", start: 1778, end: 1857 },
  { name: "ChatWebFetchDock", start: 1858, end: 1877 },
  { name: "BrowserWebRouterCollectorMvp", start: 1913, end: 2212 },
  { name: "MediaImportMvp", start: 2213, end: 2402 },
  { name: "LocalProjectPcDiagnosticsHardeningPanel", start: 2403, end: 2701 },
  { name: "BypassIntelPanel", start: 2844, end: 2930 },
  { name: "SettingsPanel", start: 2931, end: 3191 },
  { name: "SettingsHubNav", start: 3192, end: 3201 },
  { name: "UserSecurityPanel", start: 3202, end: 3292 },
  { name: "LocalLlmStatusPanel", start: 3312, end: 3367 },
  { name: "OnboardingJourney", start: 3453, end: 3504 },
  { name: "UnifiedChatHub", start: 3505, end: 4414 },
  { name: "ChatRetrievalPreview", start: 4415, end: 4811 },
  { name: "MissingEvidencePromptPanel", start: 4812, end: 4926 },
  { name: "AgentCommandPanel", start: 4927, end: 5752 },
  { name: "GuidedManualTextUpload", start: 5753, end: 6077 },
  { name: "ConversationHistoryImport", start: 6078, end: 6442 },
  { name: "UserObservationIngestion", start: 6443, end: 6823 },
  { name: "SourceCollectionApprovalReview", start: 6824, end: 6920 },
  { name: "SourceTrustSensitivityManagement", start: 6921, end: 7116 },
  { name: "SourceDetailPanel", start: 7117, end: 7285 },
  { name: "EvidenceCorrectionSupersessionWorkflow", start: 7286, end: 7443 },
  { name: "GraphLineageExplanationPanel", start: 7444, end: 7594 },
  { name: "PipelineOperationsPanel", start: 7595, end: 7672 },
  { name: "EntityClaimEventFoundationPanel", start: 7673, end: 7907 },
  { name: "EvidenceDetailPanel", start: 7908, end: 8086 },
  { name: "BasicReportWorkflow", start: 8087, end: 8321 },
  { name: "LifecycleAuditStatusPanel", start: 8322, end: 8507 },
  { name: "EvidenceFeedbackWorkflow", start: 8508, end: 8890 },
  { name: "OutcomeLearningSummary", start: 8891, end: 9080 },
  { name: "PredictionRecommendationCreator", start: 9081, end: 9319 },
  { name: "PredictionRecommendationOutcomeReview", start: 9320, end: 9607 },
  { name: "BaselinePatternExpansionPanel", start: 9620, end: 9965 },
  { name: "EvidenceAnswerHistory", start: 9966, end: 10013 },
  { name: "AgentTaskHistoryReview", start: 10014, end: 10110 },
  { name: "ImprovementExperimentReview", start: 10111, end: 10309 },
  { name: "SourceEvidenceHistory", start: 10310, end: 10398 },
  { name: "MvpActionConsole", start: 10399, end: 10714 },
];

// LLM helpers used by multiple panels
writeFile(
  "components/llm-helpers.ts",
  `import type { EnvSettingsResponse } from "./types";

type LlmDisplay = {
  provider: string;
  model: string;
  baseUrl: string;
  status: string;
  detail: string;
  reachable: boolean;
  configured: boolean;
  blockedExternal: boolean;
  restartRequired: boolean;
  warnings: string[];
};

${slice(3368, 3452)}

export type { LlmDisplay };
`
);

for (const panel of panelComponents) {
  let extra = "";
  if (panel.name === "BaselinePatternExpansionPanel") {
    extra = slice(9608, 9618) + "\n\n";
  }
  writeFile(
    `components/${panel.name}.tsx`,
    `${extra}${slice(panel.start, panel.end)}\n`
  );
}

// Home page body (export as HomePage)
const homeBody = slice(10717, 11843);
writeFile(
  "components/HomePage.tsx",
  `export async function HomePage() {
${homeBody.split("\n").slice(1).join("\n")}
`
);

console.log("Split complete:", panelComponents.length, "panels,", uiComponents.length, "ui components");