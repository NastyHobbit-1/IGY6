import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const appDir = join(scriptDir, "..");
const page = readFileSync(join(appDir, "src/app/page.tsx"), "utf8");
const styles = readFileSync(join(appDir, "src/app/globals.css"), "utf8");

const failures = [];

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

function appearsAfter(name, text, earlier, later) {
  const earlierIndex = text.indexOf(earlier);
  const laterIndex = text.indexOf(later);
  check(`${name}: ${later} appears after ${earlier}`, earlierIndex >= 0 && laterIndex > earlierIndex);
}

includesAll("top-level navigation", page, [
  'href="#home">Home',
  'href="#assistant">Assistant',
  'href="#data-knowledge">Data &amp; Knowledge',
  'href="#work-processing">Work &amp; Processing',
  'href="#reports">Reports',
  'href="#safety-audit">Safety &amp; Audit',
  'href="#settings">Settings'
]);

includesAll("workflow section anchors", page, [
  'id="home"',
  'id="assistant"',
  'id="data-knowledge"',
  'id="sources-panel"',
  'id="uploads-collection"',
  'id="evidence-panel"',
  'id="memory-panel"',
  'id="analysis-panel"',
  'id="data-search"',
  'id="work-processing"',
  'id="reports"',
  'id="safety-audit"',
  'id="settings"'
]);

check(
  "old navigation array is not restored",
  !page.includes('["Chat", "Agent Command", "Sources", "Evidence", "Memory", "Work Queue", "Approvals", "Reports", "Audit", "Settings"]')
);

includesAll("assistant action labels", page, [
  "Show project health",
  "Show git status",
  "Show latest DIFF",
  "Show work items",
  "Run retrieval preview",
  "Start stack",
  "Stop stack",
  "Run last healthy stack"
]);

includesAll("assistant action gating controls", page, [
  "data-agent-preview",
  "data-agent-execute disabled",
  "data-agent-request-approval disabled",
  "data-agent-execute-approved disabled",
  "Preview action",
  "Run safe action",
  "Request approval",
  "Run with approval"
]);

includesAll("assistant evidence controls", page, [
  "Ask a question or request an action...",
  "Ask over evidence",
  "What does this document say about my bill?",
  "What failed in this build log? Cite the evidence."
]);

includesAll("advanced panels", page, [
  "<details",
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Advanced: source IDs, permission IDs, and raw source data",
  "Advanced: raw artifact IDs, collection run IDs, and upload JSON",
  "Advanced: dispatch controls, work item IDs, and raw queue JSON",
  "Advanced: approval IDs, audit JSON, route filters, and raw safety records",
  "Advanced: report render route, report IDs, output JSON, and export details",
  "Advanced Route Console"
]);

appearsAfter(
  "raw parameters hidden behind advanced",
  page,
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Raw parameters JSON"
);

appearsAfter(
  "approval id hidden behind advanced",
  page,
  "Advanced: raw parameters, approval ID, response JSON, and route details",
  "Approval ID for approved action"
);

appearsAfter(
  "legacy route console hidden behind advanced",
  page,
  "Advanced Route Console",
  "<MvpActionConsole />"
);

includesAll("manual upload guided workflow", page, [
  "Guided Manual Upload",
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

includesAll("empty and next-step guidance states", page, [
  "No sources registered yet.",
  "No collection runs recorded yet.",
  "No evidence items recorded yet.",
  "No work items recorded yet.",
  "No approvals recorded yet.",
  "No reports recorded yet.",
  "Create a manual_upload source in Data & Knowledge.",
  "Upload approved text and check processing.",
  "Ask Assistant a question over local evidence."
]);

includesAll("safety posture", page, [
  "local-first",
  "evidence-only",
  "no-external-model",
  "approval-gated",
  "Dispatch is safe-limited",
  "does not invoke Celery or arbitrary runtime execution"
]);

includesAll("supporting styles", styles, [
  ".advancedPanel",
  ".workflowSteps",
  ".workflowTabs",
  ".lifecycleFlow",
  ".fieldGuide",
  ".quickStartGrid"
]);

if (failures.length > 0) {
  console.error("UI smoke checks failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("UI smoke checks passed.");
