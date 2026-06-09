import type {
  BrowserWebRouterImportType,
  ConnectorContractStep,
  LocalProjectDiagnosticsMode,
  MediaImportType,
  SourceConnectorStatus,
  TermHelpContent,
} from "./types";

export const TERM_HELP: Record<string, TermHelpContent> = {
  source: {
    title: "Source",
    explanation: "A Source is a registered place IGY6 may collect or review data from. On the grok branch many more source types are now active for registration, dry-run preview, permissioned collection, artifact storage, and evidence item creation: browser_export, media_file, wifi_signal, stream_capture, plus the previous ones (manual_upload, conversation_history, user_observation, local_project, web_*, router_network, local_pc_diagnostics). Some deep extraction (full OCR/vision/audio for media, rich browser history parsing) remains collector-specific or deferred, but the provenance, artifact, collection_run, and basic evidence paths are real.",
    manage: "Manage sources in Data & Knowledge; raw route controls are in Advanced.",
    purpose: "Sources define what evidence IGY6 is allowed to use before collection, normalization, search, reports, or review.",
    warning: "A registered source does not grant broad PC or account access; permissions and approvals still apply."
  },
  sourceType: {
    title: "Source Type",
    explanation: "Source Type tells IGY6 what kind of registered source this is, such as manual_upload, local_project, user_observation, conversation_history, or planned/disabled browser, web, router, PC diagnostic, and media import types.",
    manage: "Choose the type when creating a source in Data & Knowledge or the advanced source API workflow.",
    purpose: "The type controls which collection workflow and safety expectations apply.",
    warning: "Some source types are contract entries only and are not full collectors yet."
  },
  sourcePermission: {
    title: "Source Permission",
    explanation: "A Source Permission controls what a source is allowed to do, including permission scope, allowed operations, approval requirement, and external model policy.",
    manage: "Create permissions with a source in Data & Knowledge or advanced route controls.",
    purpose: "Permissions keep collection local, scoped, and auditable instead of treating a source as open-ended access.",
    warning: "A permission is not permission to perform system-changing actions."
  },
  permissionScope: {
    title: "Permission Scope",
    explanation: "Permission Scope limits which part of a source can be accessed. For local_project sources, scope means allowed paths under the source location.",
    manage: "Edit scope JSON in advanced source permission controls.",
    purpose: "Scope keeps collection bounded to the files or records the user authorized.",
    warning: "Scoped paths cannot escape the source location."
  },
  allowedOperations: {
    title: "Allowed Operations",
    explanation: "Allowed Operations are specific collection permissions such as dry_run, read, and collect.",
    manage: "Set them when creating a source permission in Data & Knowledge or through the API.",
    purpose: "They tell IGY6 what collection steps are allowed for that source.",
    warning: "They are not general permission to modify the PC, accounts, router, or websites."
  },
  externalModelPolicy: {
    title: "External Model Policy",
    explanation: "External Model Policy controls whether source data may be sent to online or external AI models.",
    manage: "Set it on source permissions and review the default in Settings with EXTERNAL_MODEL_POLICY_DEFAULT.",
    purpose: "It protects local or sensitive evidence from leaving the local stack.",
    warning: "The current default is blocked and should stay blocked unless deliberately changed."
  },
  approval: {
    title: "Approval",
    explanation: "An Approval is a local permission request that must be approved before some collections or sensitive workflows can run.",
    manage: "Use Safety & Audit approval controls or advanced approval route forms.",
    purpose: "Approvals make sensitive collection auditable and explicit.",
    warning: "Approved collection payloads must match the source, permission, and operation."
  },
  approvalRequired: {
    title: "Approval Required",
    explanation: "Approval Required means this permission or workflow must have an approved local approval record before it can proceed.",
    manage: "Set it when creating source permissions; change the default with APPROVAL_REQUIRED_DEFAULT in Settings.",
    purpose: "It keeps sensitive collection permissioned by default.",
    warning: "Turning it off reduces review friction but also reduces safety checks."
  },
  collectionRun: {
    title: "Collection Run",
    explanation: "A Collection Run is a record of a collection attempt or test run for a source.",
    manage: "Review runs in the collection records shown in the web UI or through collection-run API workflows.",
    purpose: "It links source collection to raw artifacts, status, summaries, and audit history.",
    warning: "A run record does not mean every downstream worker task has completed."
  },
  dryRun: {
    title: "Dry Run",
    explanation: "A Dry Run is a test or preview that records what would happen without collecting artifacts or changing source data.",
    manage: "Run it from Data & Knowledge advanced collection controls or the collection dry-run API endpoint.",
    purpose: "It lets the user inspect collection scope before real collection.",
    warning: "Dry-run passing is a preview, not proof that all later work will succeed."
  },
  manualUpload: {
    title: "Manual Upload",
    explanation: "Manual Upload collects UTF-8 text the user manually provides.",
    manage: "Use the Data & Knowledge upload flow after creating a manual_upload source, permission, and approval if required.",
    purpose: "It creates raw artifacts that can be normalized, chunked, embedded, and used as evidence.",
    warning: "Current normalization supports UTF-8 text only, not binary/PDF/image/audio parsing."
  },
  localProject: {
    title: "Local Project",
    explanation: "Local Project is a source type for scoped files under a folder visible inside the container.",
    manage: "Create a local_project source and permission scope paths in Data & Knowledge or advanced route controls.",
    purpose: "It lets IGY6 collect authorized project files into local evidence.",
    warning: "Paths must stay under the source location and binary files may fail UTF-8 normalization."
  },
  rawArtifact: {
    title: "Raw Artifact",
    explanation: "A Raw Artifact is the original stored collected content or file, saved locally with metadata and a content hash.",
    manage: "Review artifacts in Data & Knowledge and artifact API records.",
    purpose: "Artifacts preserve the original evidence input before normalization.",
    warning: "Artifact metadata does not mean the content has been normalized or embedded yet."
  },
  normalizedDocument: {
    title: "Normalized Document",
    explanation: "A Normalized Document is readable UTF-8 text extracted from a raw artifact.",
    manage: "Review normalized documents in Data & Knowledge.",
    purpose: "Documents are the text source for chunks, evidence items, and retrieval.",
    warning: "The current normalizer supports UTF-8 text only."
  },
  chunk: {
    title: "Chunk",
    explanation: "A Chunk is a smaller piece of a normalized document used for evidence and search.",
    manage: "Review chunks in Data & Knowledge; worker tasks create them after normalization.",
    purpose: "Chunks make long documents searchable and citable.",
    warning: "Chunks must be vector-upserted before vector retrieval can find them."
  },
  evidenceItem: {
    title: "Evidence Item",
    explanation: "An Evidence Item is a stored piece of evidence created from chunks or records.",
    manage: "Review evidence items in Data & Knowledge.",
    purpose: "Retrieval previews and evidence answers cite evidence items to show what supports a result.",
    warning: "Evidence is local record material, not proof that a statement is universally true."
  },
  claim: {
    title: "Claim",
    explanation: "A Claim is a recorded statement tied to evidence and review status.",
    manage: "Review claims in Data & Knowledge.",
    purpose: "Claims help separate asserted statements from raw text and evidence records.",
    warning: "Claims are metadata records, not automatically verified facts."
  },
  vectorMemory: {
    title: "Vector Memory",
    explanation: "Vector Memory is similarity-search memory used to find relevant chunks. IGY6 currently uses deterministic local hash vectors, not online AI embeddings.",
    manage: "Review vector status in Data & Knowledge and Qdrant-related settings in Settings.",
    purpose: "It helps retrieval find local evidence related to a user question.",
    warning: "Changing vector size can require rebuilding vector storage."
  },
  qdrant: {
    title: "Qdrant",
    explanation: "Qdrant is the local vector database behind Vector Memory.",
    manage: "Review the vector collection in Data & Knowledge and Qdrant settings in Settings.",
    purpose: "It stores searchable chunk vectors for local retrieval.",
    warning: "Qdrant results depend on chunks being embedded/upserted first."
  },
  graphMemory: {
    title: "Graph Memory",
    explanation: "Graph Memory stores relationship and lineage foundation data, such as how sources, artifacts, documents, chunks, evidence, and reports connect.",
    manage: "Review graph schema status in Data & Knowledge and Neo4j settings in Settings.",
    purpose: "It prepares IGY6 for relationship inspection and evidence lineage.",
    warning: "This is lineage and relationship support, not advanced graph reasoning."
  },
  neo4j: {
    title: "Neo4j",
    explanation: "Neo4j is the local graph database behind Graph Memory.",
    manage: "Review graph status in Data & Knowledge and Neo4j settings in Settings.",
    purpose: "It stores local relationship nodes and lineage relationships.",
    warning: "Graph sync and schema foundation exist, but advanced graph reasoning is not complete."
  },
  workItem: {
    title: "Work Item",
    explanation: "A Work Item is a queued, running, completed, failed, or canceled task for background processing.",
    manage: "Review work items in Work & Processing and dispatch supported queued items from Advanced controls.",
    purpose: "Work items keep long-running local processing out of the API request path.",
    warning: "Queued work items require intent verification metadata before dispatch."
  },
  dispatch: {
    title: "Dispatch",
    explanation: "Dispatch records a bounded request for supported queued work and keeps execution behind current system checks.",
    manage: "Use Work & Processing Advanced dispatch controls with a queued work item ID.",
    purpose: "It advances supported worker tasks such as normalization, chunking, and vector upsert.",
    warning: "Dispatch is not autonomous action; unsupported work types are rejected."
  },
  chatRetrievalPreview: {
    title: "Chat Retrieval Preview",
    explanation: "Chat Retrieval Preview searches local evidence and returns retrieval context only.",
    manage: "Use the Assistant message box.",
    purpose: "It shows which local chunks and evidence would be used for a question.",
    warning: "It does not generate an AI answer, persist a conversation, or trigger actions."
  },
  evidenceAnswer: {
    title: "Evidence Answer",
    explanation: "Evidence Answer creates an evidence-grounded answer from local retrieved evidence.",
    manage: "Use Assistant evidence controls or the chat evidence-answer API.",
    purpose: "It preserves local facts, assumptions, uncertainty, citations, source trails, and deterministic fallback answers.",
    warning: "Local LLM generation is optional, disabled by default, evidence-required, and falls back deterministically when unavailable."
  },
  localLlm: {
    title: "Local LLM",
    explanation: "Local LLM means optional Ollama generation running on this machine, not a cloud model.",
    manage: "Review provider, model, timeout, and evidence-required state in Settings.",
    purpose: "It can draft evidence-grounded wording from retrieved evidence while preserving deterministic fallback answers.",
    warning: "It must not execute actions, bypass approvals, or answer without evidence."
  },
  deterministic: {
    title: "Deterministic",
    explanation: "Deterministic means output is rule-based, local, and repeatable from stored records.",
    manage: "Review deterministic evidence outputs in Assistant retrieval preview and evidence answer.",
    purpose: "It keeps answers auditable when local LLM generation is disabled, unavailable, or unsupported by evidence.",
    warning: "Deterministic output does not include hidden AI reasoning."
  },
  noExternalModel: {
    title: "No External Model",
    explanation: "No External Model means no online AI model is called for this workflow.",
    manage: "Review external model policy in source permissions and Settings.",
    purpose: "It keeps sensitive local data inside the local IGY6 stack.",
    warning: "Changing policy defaults should be deliberate and reviewed."
  },
  pattern: {
    title: "Pattern",
    explanation: "A Pattern is a recorded repeated finding or baseline detected pattern based on existing evidence.",
    manage: "Review patterns in Data & Knowledge or run baseline pattern detection in Advanced controls.",
    purpose: "Patterns help identify recurrence, gaps, or cross-source signals.",
    warning: "A candidate pattern still needs user review."
  },
  hypothesis: {
    title: "Hypothesis",
    explanation: "A Hypothesis is a possible explanation tied to supporting evidence.",
    manage: "Review hypotheses in Data & Knowledge.",
    purpose: "It records a testable idea without treating it as proven fact.",
    warning: "A hypothesis is not a verified conclusion."
  },
  prediction: {
    title: "Prediction",
    explanation: "A Prediction is an expected outcome record tied to evidence.",
    manage: "Review predictions in Data & Knowledge and record outcomes in Safety & Audit or Advanced controls.",
    purpose: "It lets IGY6 track whether expected outcomes later become correct, wrong, partial, or inconclusive.",
    warning: "Automatic forecasting is not implemented yet."
  },
  recommendation: {
    title: "Recommendation",
    explanation: "A Recommendation is a suggested action record tied to evidence.",
    manage: "Review recommendations in Data & Knowledge and record feedback or outcomes when useful.",
    purpose: "It connects suggested action, risk, expected result, and evidence.",
    warning: "IGY6 does not automatically execute recommendations."
  },
  feedback: {
    title: "Feedback",
    explanation: "Feedback is user review metadata about whether an item was useful, weak, wrong, verified, incomplete, noisy, trusted, or rejected.",
    manage: "Use Advanced review controls or inspect feedback in Safety & Audit.",
    purpose: "Feedback helps identify weak spots and can propose improvement items.",
    warning: "Feedback records metadata; it does not rewrite historical evidence."
  },
  outcome: {
    title: "Outcome",
    explanation: "An Outcome records the result of a prediction, recommendation, hypothesis, pattern, report, or work item.",
    manage: "Use Advanced review controls or inspect outcomes in Safety & Audit.",
    purpose: "Outcomes let IGY6 track whether prior expectations or recommendations worked.",
    warning: "Outcomes must reference an existing target record."
  },
  improvementItem: {
    title: "Improvement Item",
    explanation: "An Improvement Item is a proposed improvement area for future tuning, such as parsing, retrieval, scoring, prediction, reporting, reasoning, or safety.",
    manage: "Inspect improvement records through API-backed metadata and feedback side effects.",
    purpose: "It captures weak spots or improvement ideas for later experiments.",
    warning: "It is not production self-improvement execution."
  },
  experimentRun: {
    title: "Experiment Run",
    explanation: "An Experiment Run is metadata for a planned, running, completed, failed, or abandoned experiment.",
    manage: "Inspect experiment metadata through experiment API-backed records.",
    purpose: "It records metrics and artifacts for future method comparisons.",
    warning: "It does not mean MLflow or Optuna execution is active yet."
  },
  auditEvent: {
    title: "Audit Event",
    explanation: "An Audit Event is an activity record showing who or what changed, attempted, approved, denied, dispatched, or saved something.",
    manage: "Review audit events in Safety & Audit or the right-side Recent Audit panel.",
    purpose: "Audit events make sensitive workflows traceable.",
    warning: "Audit details should not contain unmasked secret values."
  },
  artifactStore: {
    title: "Artifact Store",
    explanation: "Artifact Store is the local content-addressed storage path for raw artifacts and generated report artifacts.",
    manage: "Review ARTIFACT_STORE_PATH in Settings and artifact records in Data & Knowledge.",
    purpose: "It keeps original evidence files separate from PostgreSQL metadata.",
    warning: "Changing storage paths can require mounted volume review and stack restart."
  },
  exportStore: {
    title: "Export Store",
    explanation: "Export Store is the local path reserved for exportable reports and bundles.",
    manage: "Review EXPORT_STORE_PATH in Settings.",
    purpose: "It gives IGY6 a local place for user-exportable outputs.",
    warning: "Export behavior is still limited to current report/artifact workflows."
  },
  IGY6_DATA_ROOT: {
    title: "IGY6_DATA_ROOT",
    explanation: "IGY6_DATA_ROOT is the host-side folder where IGY6 stores database, vector, graph, artifact, report, backup, MLflow, and Phoenix runtime data.",
    manage: "Edit it in Settings and verify dry-run before saving. Windows absolute paths should use forward slashes, such as D:/Projects/IGY6_Data.",
    purpose: "It keeps private runtime data portable and outside the code repository while containers still use /workspace/storage.",
    warning: "Changing it requires Docker stack restart/recreate and does not migrate existing data."
  },
  ENV_FILE_PATH: {
    title: "ENV_FILE_PATH",
    explanation: "ENV_FILE_PATH is the controlled container path to the mounted local .env file.",
    manage: "View it in Settings; it is read-only for safety.",
    purpose: "It tells the Settings workflow exactly which local .env file may be verified and saved.",
    warning: "The UI/API must not edit arbitrary file paths."
  },
  ENV_BACKUP_DIR: {
    title: "ENV_BACKUP_DIR",
    explanation: "ENV_BACKUP_DIR is the controlled backup folder for .env backups.",
    manage: "View it in Settings; it is read-only for safety.",
    purpose: "It stores timestamped backups before Settings writes a new .env.",
    warning: "Automatic rollback is not implemented; manual rollback uses these backups."
  },
  QDRANT_CHUNK_VECTOR_SIZE: {
    title: "QDRANT_CHUNK_VECTOR_SIZE",
    explanation: "QDRANT_CHUNK_VECTOR_SIZE is the vector size used by local search memory.",
    manage: "Edit it in Settings, then verify dry-run before saving.",
    purpose: "It controls the dimensions used by deterministic local chunk vectors.",
    warning: "Changing it may require rebuilding vector storage."
  },
  EXTERNAL_MODEL_POLICY_DEFAULT: {
    title: "EXTERNAL_MODEL_POLICY_DEFAULT",
    explanation: "EXTERNAL_MODEL_POLICY_DEFAULT is the default rule for whether data can go to online AI models.",
    manage: "Edit it in Settings and review source permissions.",
    purpose: "It sets the local-first privacy default for new policy-aware workflows.",
    warning: "Default should stay blocked unless deliberately changed."
  },
  APPROVAL_REQUIRED_DEFAULT: {
    title: "APPROVAL_REQUIRED_DEFAULT",
    explanation: "APPROVAL_REQUIRED_DEFAULT controls whether new sensitive workflows require approval by default.",
    manage: "Edit it in Settings and verify dry-run before saving.",
    purpose: "It keeps source collection and sensitive actions permissioned by default.",
    warning: "Turning it off reduces safety review for future records that use the default."
  }
};

export const CONNECTOR_CONTRACT_STEPS: ConnectorContractStep[] = [
  {
    key: "validate_scope",
    label: "Validate scope",
    requirement: "User-entered scope must be explicit, bounded, and recorded before preview or collection."
  },
  {
    key: "dry_run",
    label: "Dry-run preview",
    requirement: "Collector must report what would be collected, what is excluded, sensitivity, and approval posture before collecting."
  },
  {
    key: "collect",
    label: "Collect",
    requirement: "Collection must use fixed source behavior only; no account scraping, hidden external transfer, arbitrary crawling, or credential capture."
  },
  {
    key: "normalize",
    label: "Normalize",
    requirement: "Collected content must preserve source, artifact, document, chunk, and evidence lineage where processing succeeds."
  },
  {
    key: "classify_sensitivity",
    label: "Classify sensitivity",
    requirement: "Source and collection metadata must record sensitivity, external model policy, and unsafe or unsupported content posture."
  },
  {
    key: "extract_metadata",
    label: "Extract metadata",
    requirement: "Metadata should be safe labels, counts, media/type posture, and review hints rather than raw private paths or secrets."
  },
  {
    key: "cleanup",
    label: "Cleanup",
    requirement: "Temporary state must be scoped and non-destructive; no runtime data deletion unless a later DIFF explicitly authorizes it."
  },
  {
    key: "audit",
    label: "Audit",
    requirement: "Permissions, approvals, dry-runs, collections, unsupported states, and review decisions must remain locally auditable."
  }
];

export const SOURCE_CONNECTOR_STATUS: SourceConnectorStatus[] = [
  {
    sourceType: "manual_upload",
    status: "implemented",
    defaultScope: "User-pasted UTF-8 text or safe text extract.",
    dryRun: "Supported through source permission and collection dry-run records.",
    collect: "Existing manual upload path creates artifacts and processing work.",
    sensitivity: "User selected on source; external model policy blocked by default.",
    cleanupAudit: "No source mutation; collection run, approval, and audit records apply."
  },
  {
    sourceType: "conversation_history",
    status: "implemented",
    defaultScope: "Manual local UTF-8 paste of authorized conversation/history text.",
    dryRun: "Approval-aware guided flow; collection stops while approval is pending.",
    collect: "Existing manual text pipeline under conversation_history metadata.",
    sensitivity: "Safe labels only; no browser, account, service, or connector import.",
    cleanupAudit: "Source, permission, approval, artifact, and work records remain traceable."
  },
  {
    sourceType: "user_observation",
    status: "implemented",
    defaultScope: "Owner-entered observation, decision, preference, correction, or note.",
    dryRun: "Approval-aware guided flow; no automatic truth verification.",
    collect: "Existing manual text pipeline under user_observation metadata.",
    sensitivity: "User selected or sensitive flag; external model policy blocked.",
    cleanupAudit: "Observation records are local context and do not rewrite evidence."
  },
  {
    sourceType: "local_project",
    status: "partial",
    defaultScope: "Explicit container-visible folder and scoped paths only.",
    dryRun: "Requires bounded path preview before collection.",
    collect: "Existing source type exists; broad PC crawling is not allowed.",
    sensitivity: "Binary or secret-like content must be excluded or fail honestly.",
    cleanupAudit: "Collection must preserve path bounds and audit decisions."
  },
  {
    sourceType: "browser_export",
    status: "implemented",
    defaultScope: "User-provided browser export or pasted page text only.",
    dryRun: "Preview in Web fetch paste panel or collection dry-run API.",
    collect: "Paste via manual_upload or deep live collection (crawl, authorized session fetch, media) via full-access + host bridge on grok branch.",
    sensitivity: "Treat as sensitive until reviewed.",
    cleanupAudit: "Collection runs, artifacts, and audit records apply; credentials excluded by policy."
  },
  {
    sourceType: "web_public",
    status: "implemented",
    defaultScope: "User-provided URL or manually pasted page text.",
    dryRun: "Public fetch / Deep fetch panels preview scope before collection.",
    collect: "Live URL fetch via full-access; paste via manual_upload in Web fetch tools.",
    sensitivity: "Public page does not mean safe to export externally.",
    cleanupAudit: "External fetch audit and collection run records are written locally."
  },
  {
    sourceType: "router_network",
    status: "partial",
    defaultScope: "Manual router status/export text chosen by the user.",
    dryRun: "Paste preview in Web fetch tools before import.",
    collect: "Manual paste collection only; no router writes or credential capture.",
    sensitivity: "Network identifiers and device names are sensitive by default.",
    cleanupAudit: "Approval-gated paste collection with local audit records."
  },
  {
    sourceType: "local_pc_diagnostics",
    status: "partial",
    defaultScope: "Authorized diagnostic export or explicit selected file only.",
    dryRun: "Local project panel preview or dry-run API.",
    collect: "Diagnostics text via manual_upload; bounded local_project directory collection when path is set.",
    sensitivity: "Diagnostics are sensitive by default and must redact paths where practical.",
    cleanupAudit: "Scope bounds, exclusions, and audit records are enforced on collection."
  },
  {
    sourceType: "media_import",
    status: "partial",
    defaultScope: "User-selected PDF/image/audio/video metadata and safe extracted text.",
    dryRun: "Media import panel reports type, size bound, and extraction posture.",
    collect: "Reviewed text via manual_upload; binary media + full-res artifacts via full-access deep scan + Media Library on grok (PDF text extraction supported in artifacts for applicable paths).",
    sensitivity: "Media contents and labels are sensitive until reviewed.",
    cleanupAudit: "Artifact/document/evidence lineage preserved through normalization pipeline."
  }
];

export const BROWSER_WEB_ROUTER_IMPORT_TYPES: BrowserWebRouterImportType[] = [
  {
    key: "browser_page_text",
    label: "Browser page text export",
    scopePrompt: "Page title, local export label, or URL copied by the user",
    collected: "User-pasted visible page text and safe labels only.",
    excluded: "Browser profiles, cookies, tokens, sessions, local storage, downloads, autofill data, and account data.",
    sensitivity: "Treat as sensitive until the page text has been reviewed."
  },
  {
    key: "web_page_text",
    label: "Web page text",
    scopePrompt: "Single user-provided URL or page label for pasted text",
    collected: "Manually pasted text from the selected page.",
    excluded: "Hidden fetches, crawling, login-only content, private accounts, and external requests from this UI.",
    sensitivity: "Public pages can still include sensitive notes or private copied context."
  },
  {
    key: "router_status_export",
    label: "Router status/export text",
    scopePrompt: "Router export label, page name, or diagnostic section",
    collected: "User-pasted read-only router status or diagnostic text.",
    excluded: "Router writes, config changes, network scans, credentials, Wi-Fi passwords, tokens, and login automation.",
    sensitivity: "Network names, device names, IPs, and topology are sensitive by default."
  }
];

export const MEDIA_IMPORT_TYPES: MediaImportType[] = [
  {
    key: "pdf",
    label: "PDF",
    status: "partial",
    acceptedInput: "File label, size/type metadata, and user-provided extracted text.",
    unsupportedReason: "Local PDF OCR is not run in-panel; paste reviewed extracted text to collect.",
    safeNext: "Paste verified PDF text here and click Collect extracted text."
  },
  {
    key: "image",
    label: "Image / screenshot",
    status: "partial",
    acceptedInput: "File label, size/type metadata, and optional user-provided OCR text.",
    unsupportedReason: "Local OCR is not run in-panel; binary images are collected via Deep scan.",
    safeNext: "Paste trusted OCR text and click Collect extracted text, or use Media Library after Deep scan."
  },
  {
    key: "audio",
    label: "Audio",
    status: "partial",
    acceptedInput: "File label, size/type metadata, and optional user-provided transcript.",
    unsupportedReason: "Local transcription is not run in-panel; paste reviewed transcript to collect.",
    safeNext: "Paste a reviewed transcript and click Collect extracted text."
  },
  {
    key: "video",
    label: "Video",
    status: "partial",
    acceptedInput: "File label, size/type metadata, and optional user-provided transcript or notes.",
    unsupportedReason: "Local video transcription is not run in-panel; binary video is collected via Deep scan.",
    safeNext: "Paste reviewed transcript/notes and click Collect extracted text, or use Media Library after Deep scan."
  }
];

export const LOCAL_PROJECT_DIAGNOSTICS_MODES: LocalProjectDiagnosticsMode[] = [
  {
    key: "local_project_manifest",
    label: "Local project manifest",
    scope: "Explicit user-selected project path label plus pasted manifest or file list.",
    collect: "Bounded directory collection from explicit path, or reviewed manifest text via paste collection.",
    excluded: "Arbitrary filesystem crawl, .env, SSH keys, credentials, node_modules/vendor caches, build artifacts, and private absolute path dumps."
  },
  {
    key: "pc_diagnostics_export",
    label: "PC diagnostics export",
    scope: "Authorized pasted diagnostic export or selected diagnostic file label.",
    collect: "Reviewed diagnostic text stored locally through paste collection.",
    excluded: "Live system probing, shell commands, browser profiles, tokens, cookies, credentials, private keys, and hidden account data."
  }
];

export const HOST_BRIDGE_AGENT_PORT = 8770;

export const WEB_FETCH_MAX_REACH_SCRIPT = `
(() => {
  const agentPort = ${HOST_BRIDGE_AGENT_PORT};
  const ensureMaxReachInfrastructure = async () => {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 180000);
      const agentResponse = await fetch("http://127.0.0.1:" + agentPort + "/ensure-max-reach", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        signal: controller.signal
      });
      clearTimeout(timeout);
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
          "Host bridge is not ready. Run once: pwsh -File scripts\\\\start-stack.ps1"
      );
    }
    return apiPayload;
  };
  const wirePanel = (panel) => {
  if (!panel || panel.getAttribute("data-max-reach-url-wired") === "true") return;
  panel.setAttribute("data-max-reach-url-wired", "true");
  const button = panel.querySelector("[data-max-reach-fetch-url]");
  const urlInput = panel.querySelector("[name='max_reach_page_url']");
  const depthSelect = panel.querySelector("[name='max_reach_page_depth']");
  const result = panel.querySelector("[data-max-reach-url-result]");
  const writeResult = (title, message, details, nextSteps) => {
    if (!result) return;
    result.innerHTML = "";
    const heading = document.createElement("strong");
    heading.textContent = title;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(heading, body);
    if (details?.length) {
      const list = document.createElement("dl");
      details.forEach((detail) => {
        const term = document.createElement("dt");
        term.textContent = detail.label;
        const value = document.createElement("dd");
        value.textContent = detail.value;
        list.append(term, value);
      });
      result.appendChild(list);
    }
    if (nextSteps?.length) {
      const steps = document.createElement("ul");
      nextSteps.forEach((step) => {
        const item = document.createElement("li");
        item.textContent = step;
        steps.appendChild(item);
      });
      result.appendChild(steps);
    }
  };
  button?.addEventListener("click", async () => {
    const url = urlInput?.value?.trim() || "";
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      writeResult("URL required", "Paste the full page URL starting with https:// or http://.", [], ["Example: https://example.com/article"]);
      return;
    }
    button.disabled = true;
    button.textContent = "Deep fetch running...";
    writeResult(
      "Preparing deep fetch",
      "Starting host bridge and Playwright on your PC if needed, then running strongest tier collection.",
      [{ label: "url", value: url }],
      ["First run may install Playwright browsers and take a few minutes."]
    );
    try {
      await ensureMaxReachInfrastructure();
      writeResult(
        "Deep fetch running",
        "Infrastructure ready. Running deep collection with authorized techniques, Playwright, and session-assisted fetch.",
        [{ label: "url", value: url }],
        ["This can take several minutes."]
      );
      const response = await fetch("/api/collection-runs/full-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          requested_by_actor_id: "local-owner",
          max_reach: true,
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: Number(depthSelect?.value || "1"),
          scope: [url]
        })
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(payload?.detail || response.statusText || "Max reach failed");
      }
      const summary = payload?.summary_json || payload?.summary || payload;
      const strategies = Array.isArray(summary?.auto_bypass_strategies)
        ? summary.auto_bypass_strategies.join(", ")
        : String(summary?.auto_bypass_strategies || "unknown");
      writeResult(
        "Deep fetch complete",
        "Best available content is stored locally. Open Chat and ask a question over evidence.",
        [
          { label: "mode", value: String(summary?.mode || "web_deep_fetch") },
          { label: "winning strategies", value: strategies || "none recorded" },
          { label: "pages crawled", value: String(summary?.crawled_pages ?? summary?.web_scraped ?? "unknown") },
          { label: "evidence items", value: String(summary?.total_evidence ?? "unknown") },
          { label: "artifacts", value: String(summary?.total_artifacts ?? "unknown") }
        ],
        [
          "Open the Work tab to confirm processing finished.",
          "Open Chat and ask questions over the fetched page.",
          "For advanced attach, start Chrome with --remote-debugging-port=9222."
        ]
      );
    } catch (error) {
      writeResult(
        "Deep fetch failed",
        error instanceof Error ? error.message : "Unknown error",
        [],
        ["Run once if needed: pwsh -File scripts\\\\start-stack.ps1", "Try session fetch with a fresh session header if the site requires your account."]
      );
    } finally {
      button.disabled = false;
      button.textContent = "Deep fetch";
    }
  });
  };
  document.querySelectorAll("[data-max-reach-url-fetch]").forEach(wirePanel);
})();
`;

export const WEB_FETCH_AUTO_BYPASS_SCRIPT = `
(() => {
  const agentPort = ${HOST_BRIDGE_AGENT_PORT};
  const ensureHostBridgeInfrastructure = async () => {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 120000);
      await fetch("http://127.0.0.1:" + agentPort + "/ensure", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        signal: controller.signal
      });
      clearTimeout(timeout);
    } catch (error) {
      console.warn("Host bridge ensure agent unavailable:", error);
    }
    const apiResponse = await fetch("/api/host-bridge/ensure-max-reach", { method: "POST" });
    const apiPayload = await apiResponse.json().catch(() => ({}));
    if (!apiResponse.ok) {
      throw new Error(apiPayload?.detail || "Host bridge is not ready. Run once: pwsh -File scripts\\\\start-stack.ps1");
    }
    return apiPayload;
  };
  const wirePanel = (panel) => {
  if (!panel || panel.getAttribute("data-auto-bypass-url-wired") === "true") return;
  panel.setAttribute("data-auto-bypass-url-wired", "true");
  const button = panel.querySelector("[data-auto-bypass-fetch-url]");
  const urlInput = panel.querySelector("[name='auto_bypass_page_url']");
  const depthSelect = panel.querySelector("[name='auto_bypass_page_depth']");
  const result = panel.querySelector("[data-auto-bypass-url-result]");
  const writeResult = (title, message, details, nextSteps) => {
    if (!result) return;
    result.innerHTML = "";
    const heading = document.createElement("strong");
    heading.textContent = title;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(heading, body);
    if (details?.length) {
      const list = document.createElement("dl");
      details.forEach((detail) => {
        const term = document.createElement("dt");
        term.textContent = detail.label;
        const value = document.createElement("dd");
        value.textContent = detail.value;
        list.append(term, value);
      });
      result.appendChild(list);
    }
    if (nextSteps?.length) {
      const steps = document.createElement("ul");
      nextSteps.forEach((step) => {
        const item = document.createElement("li");
        item.textContent = step;
        steps.appendChild(item);
      });
      result.appendChild(steps);
    }
  };
  button?.addEventListener("click", async () => {
    const url = urlInput?.value?.trim() || "";
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      writeResult("URL required", "Paste the full page URL starting with https:// or http://.", [], ["Example: https://example.com/article"]);
      return;
    }
    button.disabled = true;
    button.textContent = "Deep fetching...";
    writeResult(
      "Preparing deep fetch",
      "Starting host bridge on your PC if needed, then running deep collection.",
      [{ label: "url", value: url }],
      []
    );
    try {
      await ensureHostBridgeInfrastructure();
      writeResult(
        "Deep fetch running",
        "Running authorized collection techniques, Playwright, and session-assisted fetch.",
        [{ label: "url", value: url }],
        []
      );
      const response = await fetch("/api/collection-runs/full-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          requested_by_actor_id: "local-owner",
          auto_bypass: true,
          web_only: true,
          safe_mode: true,
          max_depth: Number(depthSelect?.value || "1"),
          scope: [url]
        })
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(payload?.detail || response.statusText || "Auto bypass failed");
      }
      const summary = payload?.summary_json || payload?.summary || payload;
      const strategies = Array.isArray(summary?.auto_bypass_strategies)
        ? summary.auto_bypass_strategies.join(", ")
        : String(summary?.auto_bypass_strategies || "unknown");
      writeResult(
        "Deep fetch complete",
        "Best available page content is stored locally. Open Chat and ask a question over evidence.",
        [
          { label: "winning strategies", value: strategies || "none recorded" },
          { label: "pages crawled", value: String(summary?.crawled_pages ?? summary?.web_scraped ?? "unknown") },
          { label: "evidence items", value: String(summary?.total_evidence ?? "unknown") },
          { label: "artifacts", value: String(summary?.total_artifacts ?? "unknown") }
        ],
        [
          "Open the Work tab to confirm processing finished.",
          "Open Chat and ask questions over the fetched page.",
          "Hard account walls may still need Session fetch with your own session header below."
        ]
      );
    } catch (error) {
      writeResult(
        "Deep fetch failed",
        error instanceof Error ? error.message : "Unknown error",
        [],
        ["Try a different URL.", "If the site requires your account, use Session fetch with a session header."]
      );
    } finally {
      button.disabled = false;
      button.textContent = "Deep fetch";
    }
  });
  };
  document.querySelectorAll("[data-auto-bypass-url-fetch]").forEach(wirePanel);
})();
`;

export const WEB_FETCH_BYPASS_SCRIPT = `
(() => {
  const wirePanel = (panel) => {
  if (!panel || panel.getAttribute("data-bypass-url-wired") === "true") return;
  panel.setAttribute("data-bypass-url-wired", "true");
  const button = panel.querySelector("[data-bypass-fetch-url]");
  const urlInput = panel.querySelector("[name='bypass_page_url']");
  const cookieInput = panel.querySelector("[name='bypass_cookie']");
  const authInput = panel.querySelector("[name='bypass_authorization']");
  const depthSelect = panel.querySelector("[name='bypass_page_depth']");
  const result = panel.querySelector("[data-bypass-url-result]");
  const writeResult = (title, message, details, nextSteps) => {
    if (!result) return;
    result.innerHTML = "";
    const heading = document.createElement("strong");
    heading.textContent = title;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(heading, body);
    if (details?.length) {
      const list = document.createElement("dl");
      details.forEach((detail) => {
        const term = document.createElement("dt");
        term.textContent = detail.label;
        const value = document.createElement("dd");
        value.textContent = detail.value;
        list.append(term, value);
      });
      result.appendChild(list);
    }
    if (nextSteps?.length) {
      const steps = document.createElement("ul");
      nextSteps.forEach((step) => {
        const item = document.createElement("li");
        item.textContent = step;
        steps.appendChild(item);
      });
      result.appendChild(steps);
    }
  };
  button?.addEventListener("click", async () => {
    const url = urlInput?.value?.trim() || "";
    const cookie = cookieInput?.value?.trim() || "";
    const authorization = authInput?.value?.trim() || "";
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      writeResult("URL required", "Paste the full page URL starting with https:// or http://.", [], ["Example: https://example.com/account"]);
      return;
    }
    if (!cookie && !authorization) {
      writeResult(
        "Session required",
        "Paste a Cookie header or bearer token from a browser session you already own.",
        [],
        [
          "Open the page in your browser, sign in, then copy the Cookie header from dev tools (Network tab).",
          "Or paste a bearer token if the site uses API auth."
        ]
      );
      return;
    }
    button.disabled = true;
    button.textContent = "Session fetching...";
    writeResult("Session fetching", "Using your provided session header to fetch and store the page locally.", [
      { label: "url", value: url },
      { label: "session header", value: cookie || authorization ? "provided" : "not provided" }
    ], []);
    try {
      const body = {
        requested_by_actor_id: "local-owner",
        bypass_auth: true,
        web_only: true,
        safe_mode: true,
        max_depth: Number(depthSelect?.value || "1"),
        scope: [url],
        referer: url
      };
      if (cookie) body.cookie = cookie;
      if (authorization) body.authorization = authorization;
      const response = await fetch("/api/collection-runs/full-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(payload?.detail || response.statusText || "Bypass fetch failed");
      }
      const summary = payload?.summary_json || payload?.summary || payload;
      writeResult(
        "Session fetch complete",
        "Authorized page content is stored locally. Open Chat and ask a question over evidence.",
        [
          { label: "pages crawled", value: String(summary?.crawled_pages ?? summary?.web_scraped ?? "unknown") },
          { label: "evidence items", value: String(summary?.total_evidence ?? "unknown") },
          { label: "artifacts", value: String(summary?.total_artifacts ?? "unknown") }
        ],
        [
          "Open the Work tab to confirm processing finished.",
          "Open Chat and ask questions over the fetched page.",
          "If the session expired, provide a fresh session header and try again."
        ]
      );
    } catch (error) {
      writeResult(
        "Session fetch failed",
        error instanceof Error ? error.message : "Unknown error",
        [],
        ["Refresh your browser session header and retry.", "Heavy JavaScript sites may still need manual paste below."]
      );
    } finally {
      button.disabled = false;
      button.textContent = "Session fetch";
    }
  });
  };
  document.querySelectorAll("[data-bypass-url-fetch]").forEach(wirePanel);
})();
`;

export const WEB_FETCH_PUBLIC_SCRIPT = `
(() => {
  const wirePanel = (panel) => {
  if (!panel || panel.getAttribute("data-public-url-wired") === "true") return;
  panel.setAttribute("data-public-url-wired", "true");
  const button = panel.querySelector("[data-fetch-public-url]");
  const urlInput = panel.querySelector("[name='public_page_url']");
  const depthSelect = panel.querySelector("[name='public_page_depth']");
  const result = panel.querySelector("[data-public-url-result]");
  const writeResult = (title, message, details, nextSteps) => {
    if (!result) return;
    result.innerHTML = "";
    const heading = document.createElement("strong");
    heading.textContent = title;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(heading, body);
    if (details?.length) {
      const list = document.createElement("dl");
      details.forEach((detail) => {
        const term = document.createElement("dt");
        term.textContent = detail.label;
        const value = document.createElement("dd");
        value.textContent = detail.value;
        list.append(term, value);
      });
      result.appendChild(list);
    }
    if (nextSteps?.length) {
      const steps = document.createElement("ul");
      nextSteps.forEach((step) => {
        const item = document.createElement("li");
        item.textContent = step;
        steps.appendChild(item);
      });
      result.appendChild(steps);
    }
  };
  button?.addEventListener("click", async () => {
    const url = urlInput?.value?.trim() || "";
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      writeResult("URL required", "Paste a full public page URL starting with https:// or http://.", [], ["Example: https://example.com/docs/guide"]);
      return;
    }
    button.disabled = true;
    button.textContent = "Public fetching...";
    writeResult("Public fetching", "Downloading the public page and storing it locally. This can take up to a minute.", [
      { label: "url", value: url }
    ], []);
    try {
      const response = await fetch("/api/collection-runs/full-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          requested_by_actor_id: "local-owner",
          web_only: true,
          safe_mode: true,
          max_depth: Number(depthSelect?.value || "1"),
          scope: [url]
        })
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(payload?.detail || response.statusText || "Fetch failed");
      }
      const summary = payload?.summary_json || payload?.summary || payload;
      writeResult(
        "Public page captured",
        "Public page content is stored locally. Open Chat and ask a question over evidence.",
        [
          { label: "pages crawled", value: String(summary?.crawled_pages ?? summary?.web_scraped ?? "unknown") },
          { label: "evidence items", value: String(summary?.total_evidence ?? "unknown") },
          { label: "artifacts", value: String(summary?.total_artifacts ?? "unknown") }
        ],
        [
          "Open the Work tab to confirm processing finished.",
          "Open Chat and ask: What does this page say about ...?",
          "Account-only pages still need Session fetch or copy/paste into Guided Upload below."
        ]
      );
    } catch (error) {
      writeResult(
        "Fetch failed",
        error instanceof Error ? error.message : "Unknown error",
        [],
        ["Use a public URL (no sign-in).", "If the site is mostly JavaScript, copy visible text into Guided Upload instead."]
      );
    } finally {
      button.disabled = false;
      button.textContent = "Fetch page";
    }
  });
  };
  document.querySelectorAll("[data-public-url-fetch]").forEach(wirePanel);
})();
`;


export const MINIMAL_UI_TOGGLE_SCRIPT = `
(() => {
  const storageKey = "igy6-minimal-ui";
  const root = document.querySelector("[data-minimal-ui-root]");
  const toggles = document.querySelectorAll("[data-minimal-ui-toggle]");
  if (!root || toggles.length === 0) return;
  const apply = (enabled) => {
    document.body.classList.toggle("minimal-ui-mode", enabled);
    root.setAttribute("data-minimal-ui-active", enabled ? "true" : "false");
    toggles.forEach((toggle) => {
      toggle.setAttribute("aria-pressed", enabled ? "true" : "false");
      toggle.textContent = enabled ? "Full workspace" : "Simple mode";
    });
    const hint = document.querySelector("[data-minimal-ui-hint]");
    if (hint) {
      hint.textContent = enabled
        ? "Simple mode on — five easy slots, plain-language chat. Click Full workspace for every panel."
        : "Click Simple mode for an easier layout. Chat asks plain questions when unsure.";
    }
    const welcome = document.querySelector("[data-chat-welcome-text]");
    if (welcome) {
      welcome.textContent = enabled
        ? "Hey — just talk to me like a person. Paste a link, ask a question, say you want to add notes, or ask what's still processing. If I'm not sure what you mean, I'll ask instead of throwing jargon at you."
        : "Type anything here: ask over evidence, run auto bypass or fetch public with a URL, open any panel (settings, uploads, web fetch), or run bounded actions like project health and stack control. Say help for the full command list.";
    }
  };
  const saved = localStorage.getItem(storageKey);
  apply(saved === "1");
  toggles.forEach((toggle) => {
    toggle.addEventListener("click", () => {
      const next = !document.body.classList.contains("minimal-ui-mode");
      localStorage.setItem(storageKey, next ? "1" : "0");
      apply(next);
    });
  });
})();
`;

export const WORKSPACE_HASH_ROUTER_SCRIPT = `
(() => {
  const routes = {
    "chat-web-fetch": "tab-results",
    "browser-web-router-import": "tab-add-data",
    "uploads-collection": "tab-add-data",
    "sources-panel": "tab-add-data",
    "data-knowledge": "tab-add-data",
    "evidence-panel": "tab-results",
    "memory-panel": "tab-results",
    "work-processing": "tab-work",
    "settings": "tab-settings",
    "user-security": "tab-settings",
    "advanced-diagnostics": "tab-advanced",
    "assistant": "tab-results"
  };
  const applyHash = () => {
    const id = (location.hash || "").replace(/^#/, "");
    if (!id) return;
    const tabId = routes[id];
    const tab = tabId ? document.getElementById(tabId) : null;
    if (tab) {
      tab.checked = true;
      tab.dispatchEvent(new Event("change", { bubbles: true }));
    }
    requestAnimationFrame(() => {
      document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  };
  window.addEventListener("hashchange", applyHash);
  if (location.hash) applyHash();
})();
`;

export const RUNTIME_POSTURE = [
  { label: "Rust API", value: "active", state: "runtime-active" },
  { label: "Rust worker", value: "active", state: "runtime-active" },
  { label: "Legacy API", value: "inactive / archived", state: "archived" },
  { label: "Legacy worker", value: "inactive / archived", state: "archived" },
  { label: "Legacy scheduler", value: "inactive", state: "retired" }
];

export const USER_READINESS = [
  { label: "System", value: "ready", state: "ready" },
  { label: "Background processing", value: "ready", state: "ready" },
  { label: "Old Python services", value: "archived", state: "archived" }
];
