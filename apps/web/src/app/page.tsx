type HealthResponse = {
  status: string;
  checks?: Record<string, { status: string; detail?: string }>;
};

type SourcePermission = {
  id: string;
  allowed_operations: string[];
  approval_required: boolean;
  external_model_policy: string;
};

type SourceRecord = {
  id: string;
  name: string;
  source_type: string;
  location?: string | null;
  sensitivity: string;
  trust_level: string;
  enabled: boolean;
  updated_at?: string | null;
  permissions?: SourcePermission[];
};

type CollectionRunRecord = {
  id: string;
  source_id: string | null;
  status: string;
  dry_run: boolean;
  requested_by_actor_id: string;
  created_at: string;
  summary_json: Record<string, unknown>;
};

type RawArtifactRecord = {
  id: string;
  source_id: string | null;
  collection_run_id: string | null;
  content_hash: string;
  mime_type: string | null;
  size_bytes: number | null;
  created_at: string;
};

type NormalizedDocumentRecord = {
  id: string;
  raw_artifact_id: string | null;
  source_id: string | null;
  title: string | null;
  document_type: string;
  language: string | null;
  sensitivity: string;
  created_at: string;
};

type ChunkRecord = {
  id: string;
  document_id: string;
  chunk_index: number;
  embedding_status: string;
  created_at: string;
};

type EvidenceItemRecord = {
  id: string;
  source_id: string | null;
  document_id: string | null;
  chunk_id: string | null;
  evidence_type: string;
  statement: string;
  confidence: number | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

type EvidenceAnswerRecord = {
  id: string;
  user_question: string;
  answer_status: string;
  answer_text: string | null;
  facts?: string[];
  assumptions?: string[];
  inferences?: string[];
  uncertainty?: string[];
  missing_information?: string[];
  evidence_item_ids?: string[];
  document_ids?: string[];
  chunk_ids?: string[];
  source_ids?: string[];
  safe_labels?: string[];
  retrieval_mode: string;
  retrieval_count: number;
  local_model_status?: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

type ClaimRecord = {
  id: string;
  claim_text: string;
  claim_type: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

type VectorCollectionStatus = {
  collection_name: string;
  exists: boolean;
};

type GraphSchemaStatus = {
  constraints: Array<Record<string, unknown>>;
};

type PatternRecord = {
  id: string;
  pattern_type: string;
  status: string;
  summary: string;
  confidence: number | null;
  created_at: string;
};

type HypothesisRecord = {
  id: string;
  hypothesis_text: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

type PredictionRecord = {
  id: string;
  prediction_text: string;
  expected_result: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

type RecommendationRecord = {
  id: string;
  recommendation_text: string;
  risk_level: string;
  approval_required: boolean;
  status: string;
  confidence: number | null;
  created_at: string;
};

type WorkItemRecord = {
  id: string;
  work_type: string;
  status: string;
  requested_by_actor_id: string;
  payload_json?: Record<string, unknown> | null;
  error_message: string | null;
  created_at: string;
  updated_at?: string | null;
};

type AgentTaskPlanRecord = {
  id: string;
  user_request_summary: string;
  intent_category: string;
  status: string;
  proposed_steps?: string[];
  required_evidence?: string[];
  approval_required: boolean;
  supported_state: string;
  next_safe_action: string;
  requested_by_actor_id: string;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

type ApprovalRecord = {
  id: string;
  request_type: string;
  status: string;
  requested_by_actor_id: string;
  decided_by_actor_id: string | null;
  decision_reason: string | null;
  request_payload_json?: Record<string, unknown> | null;
  created_at: string;
};

type FeedbackRecord = {
  id: string;
  target_type: string;
  target_id: string;
  label: string;
  actor_id: string;
  note: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

type OutcomeRecord = {
  id: string;
  target_type: string;
  target_id: string;
  outcome_status: string;
  summary: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

type ImprovementRecord = {
  id: string;
  target_area: string;
  status: string;
  objective: string;
  proposed_by_actor_id: string;
  priority: string;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

type ExperimentRecord = {
  id: string;
  improvement_item_id: string | null;
  status: string;
  mlflow_run_id: string | null;
  optuna_study_name: string | null;
  metrics_json?: Record<string, unknown> | null;
  artifacts_json?: Record<string, unknown> | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

type ReportRecord = {
  id: string;
  title: string;
  report_type: string;
  status: string;
  requested_by_actor_id: string;
  artifact_path?: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

type AuditEventRecord = {
  id: number;
  actor_id: string;
  event_type: string;
  decision: string | null;
  resource_type: string | null;
  resource_id: string | null;
  created_at: string;
};

type EnvSettingRecord = {
  key: string;
  group: string;
  group_label: string;
  description: string;
  value: string | null;
  masked_value: string | null;
  has_value: boolean;
  secret: boolean;
  read_only: boolean;
  restart_required: boolean;
  source: string;
};

type EnvUnmanagedRecord = {
  key: string;
  masked_value: string;
  has_value: boolean;
  secret: boolean;
  read_only: boolean;
};

type EnvSettingsResponse = {
  file_status: {
    path: string;
    backup_dir: string;
    exists: boolean;
    writable: boolean;
    unknown_key_count: number;
    output_format: string;
  };
  groups: Array<{ key: string; label: string }>;
  settings: EnvSettingRecord[];
  unmanaged: EnvUnmanagedRecord[];
  warnings: string[];
};

type AgentActionCapability = {
  name: string;
  interpreted_intent: string;
  action_type: string;
  approval_required: boolean;
  risk_level: string;
  required_parameters: string[];
  script_backed: boolean;
  required_scripts: string[];
  scripts_exist: boolean;
  executable_in_api_runtime: boolean;
  reason: string | null;
};

type AgentCapabilitiesResponse = {
  actions: AgentActionCapability[];
  runtime: {
    repo_root: string;
    docker_cli_available: boolean;
    docker_compose_available: boolean;
    docker_socket_available: boolean;
    docker_host_configured: boolean;
    docker_control_available: boolean;
    docker_socket_path: string | null;
    reason: string | null;
  };
};

type ApiResult<T> = {
  data: T;
  error: string | null;
};

type TermHelpContent = {
  title: string;
  explanation: string;
  manage: string;
  purpose: string;
  examples?: string;
  warning?: string;
};

const TERM_HELP: Record<string, TermHelpContent> = {
  source: {
    title: "Source",
    explanation: "A Source is a registered place IGY6 may collect or review data from. Current source types include manual_upload for manually added UTF-8 text, local_project for scoped files under a container-visible folder, user_observation for notes, conversation_history for imported conversation records, and planned types such as local_pc_diagnostics, router_network, web_public, and web_authorized_account.",
    manage: "Manage sources in Data & Knowledge; raw route controls are in Advanced.",
    purpose: "Sources define what evidence IGY6 is allowed to use before collection, normalization, search, reports, or review.",
    warning: "A registered source does not grant broad PC or account access; permissions and approvals still apply."
  },
  sourceType: {
    title: "Source Type",
    explanation: "Source Type tells IGY6 what kind of registered source this is, such as manual_upload, local_project, user_observation, conversation_history, or planned router/web/PC diagnostic types.",
    manage: "Choose the type when creating a source in Data & Knowledge or the advanced source API workflow.",
    purpose: "The type controls which collection workflow and safety expectations apply.",
    warning: "Some source types are planned and not full collectors yet."
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
    warning: "This is not full autonomous graph reasoning yet."
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
    purpose: "It preserves local facts, assumptions, uncertainty, citations, source trails, and deterministic backup answers.",
    warning: "Local LLM generation is optional, disabled by default, evidence-required, and falls back deterministically when unavailable."
  },
  localLlm: {
    title: "Local LLM",
    explanation: "Local LLM means optional Ollama generation running on this machine, not a cloud model.",
    manage: "Review provider, model, timeout, and evidence-required state in Settings.",
    purpose: "It can draft evidence-grounded wording from retrieved evidence while preserving deterministic backup answers.",
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

function TermHelp({ term, label }: { term: keyof typeof TERM_HELP; label?: string }) {
  const help = TERM_HELP[term];
  return (
    <span className="termHelp">
      {label ? <span className="termLabel">{label}</span> : null}
      <button className="termHelpTrigger" type="button" aria-label={`Help: ${help.title}`}>?</button>
      <span className="termHelpBubble" role="tooltip">
        <strong>{help.title}</strong>
        <span>{help.explanation}</span>
        <span><b>Where:</b> {help.manage}</span>
        <span><b>Why it matters:</b> {help.purpose}</span>
        {help.examples ? <span><b>Examples:</b> {help.examples}</span> : null}
        {help.warning ? <span><b>Limit:</b> {help.warning}</span> : null}
      </span>
    </span>
  );
}

function HelpHeading({ children, term }: { children: string; term: keyof typeof TERM_HELP }) {
  return <span className="helpHeading"><span>{children}</span><TermHelp term={term} /></span>;
}

async function getJson<T>(path: string, fallback: T): Promise<ApiResult<T>> {
  const baseUrl = process.env.API_BASE_URL ?? "http://api:8000";

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      cache: "no-store"
    });
    if (!response.ok) {
      return { data: fallback, error: `${response.status} ${response.statusText}` };
    }
    return { data: (await response.json()) as T, error: null };
  } catch (error) {
    return {
      data: fallback,
      error: error instanceof Error ? error.message : "Unknown error"
    };
  }
}

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString("en-US", {
    dateStyle: "medium",
    timeStyle: "short"
  });
}

function formatBytes(value: number | null): string {
  if (value === null) {
    return "unknown";
  }
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function excerpt(value: string, maxLength = 110): string {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, maxLength - 3)}...`;
}

function StatusPill({ state }: { state: string }) {
  return <span className="pill" data-state={state}>{state}</span>;
}

function EmptyState({ label }: { label: string }) {
  return <p className="empty">{label}</p>;
}

function jsonString(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}

function jsonStringList(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
}

function evidenceReviewState(item: EvidenceItemRecord): string {
  const reviewState = item.metadata_json?.review_state;
  if (!reviewState || typeof reviewState !== "object") {
    return "unreviewed";
  }
  const state = (reviewState as Record<string, unknown>).state;
  return typeof state === "string" && state.trim() ? state : "unreviewed";
}

function evidenceReviewNote(item: EvidenceItemRecord): string | null {
  const reviewState = item.metadata_json?.review_state;
  if (!reviewState || typeof reviewState !== "object") {
    return null;
  }
  const note = (reviewState as Record<string, unknown>).correction_note;
  return typeof note === "string" && note.trim() ? note : null;
}

function workItemRelatedIds(workItem: WorkItemRecord): Array<{ label: string; values: string[] }> {
  const payload = workItem.payload_json ?? {};
  const related = [
    { label: "collection", values: [jsonString(payload.collection_run_id)].filter(Boolean) as string[] },
    { label: "source", values: [jsonString(payload.source_id)].filter(Boolean) as string[] },
    { label: "permission", values: [jsonString(payload.source_permission_id)].filter(Boolean) as string[] },
    { label: "artifact", values: jsonStringList(payload.raw_artifact_ids) },
    { label: "document", values: jsonStringList(payload.document_ids) },
    { label: "chunk", values: jsonStringList(payload.chunk_ids) },
    { label: "parent work", values: [jsonString(payload.parent_work_item_id)].filter(Boolean) as string[] }
  ];
  return related.filter((item) => item.values.length > 0);
}

function workItemGuidance(workItem: WorkItemRecord): { outcome: string; next: string } {
  switch (workItem.status) {
    case "queued":
    case "pending_intent_verification":
      return {
        outcome: "Waiting for background processing.",
        next: "Refresh Work after the worker has had time to claim it. Use Advanced dispatch only when you know this specific queued item should be dispatched."
      };
    case "running":
      return {
        outcome: "Processing is in progress.",
        next: "Refresh Work to see the updated state. Avoid resubmitting the same upload while this item is running."
      };
    case "completed":
      return {
        outcome: "Processing completed successfully.",
        next: "Open Results to inspect documents, chunks, evidence, and Ask over evidence."
      };
    case "failed":
      return {
        outcome: workItem.error_message ?? "Processing failed and needs review.",
        next: "Read the error and verify the source, permission, and uploaded UTF-8 text. No automatic retry action is exposed here."
      };
    case "canceled":
      return {
        outcome: "Processing was canceled.",
        next: "Review the source and collection record before creating new work."
      };
    default:
      return {
        outcome: "Status is recorded by the local API.",
        next: "Refresh Work or inspect Advanced raw queue JSON if this state is unexpected."
      };
  }
}

function workItemDispatchVisibility(workItem: WorkItemRecord): Array<{ label: string; value: string; state: string }> {
  const supportedTypes: Record<string, string> = {
    collection_normalization: "collection.normalize_collection_run",
    document_chunking: "evidence.generate_document_chunks",
    chunk_vector_upsert: "memory.vector.upsert_chunks"
  };
  const payload = workItem.payload_json ?? {};
  const taskName = supportedTypes[workItem.work_type];
  const intentVerified = Boolean(payload.intent_verification) || payload.intent_verification_recorded === true;
  const safeDispatchOnly = payload.safe_dispatch_only === true || payload.rust_gateway_execution === "not_executed";
  const statusState = ["queued", "pending_intent_verification"].includes(workItem.status)
    ? "waiting"
    : ["running"].includes(workItem.status)
      ? "running"
      : ["completed"].includes(workItem.status)
        ? "completed"
        : ["failed"].includes(workItem.status)
          ? "failed"
          : "recorded";
  return [
    {
      label: "support",
      value: taskName ? `supported: ${taskName}` : "unsupported by bounded dispatch",
      state: taskName ? "supported" : "unsupported"
    },
    {
      label: "state",
      value: workItem.status,
      state: statusState
    },
    {
      label: "intent",
      value: intentVerified ? "intent verification recorded" : "intent verification not visible",
      state: intentVerified ? "verified" : "not-verified"
    },
    {
      label: "dispatch",
      value: safeDispatchOnly ? "dispatch metadata only / no arbitrary execution" : "worker-managed or not dispatched here",
      state: safeDispatchOnly ? "safe-dispatch-only" : "worker-managed"
    }
  ];
}

const RUNTIME_POSTURE = [
  { label: "Rust API", value: "active", state: "runtime-active" },
  { label: "Rust worker", value: "active", state: "runtime-active" },
  { label: "Legacy API", value: "inactive / archived", state: "archived" },
  { label: "Legacy worker", value: "inactive / archived", state: "archived" },
  { label: "Legacy scheduler", value: "inactive", state: "retired" }
];

const USER_READINESS = [
  { label: "System", value: "ready", state: "ready" },
  { label: "Background processing", value: "ready", state: "ready" },
  { label: "Old Python services", value: "archived", state: "archived" }
];

function SettingsPanel({ envSettings }: { envSettings: ApiResult<EnvSettingsResponse> }) {
  const data = envSettings.data;
  const groupedSettings = data.groups.map((group) => ({
    ...group,
    settings: data.settings.filter((setting) => setting.group === group.key)
  }));
  const groupHelpTerms: Record<string, keyof typeof TERM_HELP> = {
    qdrant: "qdrant",
    neo4j: "neo4j",
    storage: "artifactStore",
    llm: "localLlm",
    policy: "externalModelPolicy"
  };
  const settingHelpTerms: Record<string, keyof typeof TERM_HELP> = {
    ENV_FILE_PATH: "ENV_FILE_PATH",
    ENV_BACKUP_DIR: "ENV_BACKUP_DIR",
    IGY6_DATA_ROOT: "IGY6_DATA_ROOT",
    QDRANT_CHUNK_VECTOR_SIZE: "QDRANT_CHUNK_VECTOR_SIZE",
    EXTERNAL_MODEL_POLICY_DEFAULT: "EXTERNAL_MODEL_POLICY_DEFAULT",
    APPROVAL_REQUIRED_DEFAULT: "APPROVAL_REQUIRED_DEFAULT",
    LLM_PROVIDER: "localLlm",
    OLLAMA_BASE_URL: "localLlm",
    OLLAMA_MODEL: "localLlm",
    LLM_TIMEOUT_SECONDS: "localLlm",
    LLM_EVIDENCE_REQUIRED: "localLlm",
    ARTIFACT_STORE_PATH: "artifactStore",
    EXPORT_STORE_PATH: "exportStore"
  };
  const script = `
(() => {
  const root = document.querySelector("[data-settings-env]");
  if (!root) return;

  const verifyButton = root.querySelector("[data-settings-verify]");
  const saveButton = root.querySelector("[data-settings-save]");
  const resultPanel = root.querySelector("[data-settings-result]");
  const changedPanel = root.querySelector("[data-settings-changed]");
  const warningPanel = root.querySelector("[data-settings-warnings]");
  const backupPanel = root.querySelector("[data-settings-backup]");
  const tokenInput = root.querySelector("[data-settings-token]");
  let verifiedToken = "";
  let verifiedPayload = "";

  const showJson = (node, label, payload) => {
    if (!node) return;
    node.textContent = label + "\\n" + JSON.stringify(payload, null, 2);
  };

  const collectChanges = () => {
    const values = {};
    root.querySelectorAll("[data-env-key]").forEach((field) => {
      const key = field.getAttribute("data-env-key");
      const secret = field.getAttribute("data-secret") === "true";
      const readOnly = field.getAttribute("data-read-only") === "true";
      if (!key || readOnly) return;

      if (secret) {
        const replace = root.querySelector("[data-secret-replace='" + key + "']");
        if (replace?.checked && field.value !== "") {
          values[key] = field.value;
        }
        return;
      }

      const current = field.getAttribute("data-current") ?? "";
      if (field.value !== current) {
        values[key] = field.value;
      }
    });
    return values;
  };

  const clearVerified = () => {
    verifiedToken = "";
    verifiedPayload = "";
    if (tokenInput) tokenInput.value = "";
    if (saveButton) saveButton.disabled = true;
  };

  root.querySelectorAll("[data-env-key], [data-secret-replace]").forEach((field) => {
    field.addEventListener("input", clearVerified);
    field.addEventListener("change", clearVerified);
  });

  verifyButton?.addEventListener("click", async () => {
    clearVerified();
    const values = collectChanges();
    showJson(resultPanel, "Verifying dry run", { changed_keys: Object.keys(values) });
    try {
      const response = await fetch("/api/settings/env/verify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ values, actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Dry-run result" : "Dry-run request failed", payload);
      showJson(changedPanel, "Changed keys", payload.changed_keys ?? Object.keys(values));
      showJson(warningPanel, "Warnings", payload.warnings ?? []);
      if (response.ok && payload.passed && payload.verification_token) {
        verifiedToken = payload.verification_token;
        verifiedPayload = JSON.stringify(values);
        if (tokenInput) tokenInput.value = verifiedToken;
        if (saveButton) saveButton.disabled = false;
      }
    } catch (error) {
      showJson(resultPanel, "Dry-run error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  saveButton?.addEventListener("click", async () => {
    const values = collectChanges();
    if (!verifiedToken || JSON.stringify(values) !== verifiedPayload) {
      clearVerified();
      showJson(resultPanel, "Save blocked", { detail: "Current edits do not match the latest passing dry run." });
      return;
    }
    showJson(resultPanel, "Saving verified candidate", { changed_keys: Object.keys(values) });
    try {
      const response = await fetch("/api/settings/env/apply", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ values, verification_token: verifiedToken, actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Save result" : "Save failed", payload);
      showJson(backupPanel, "Backup", { backup_path: payload.backup_path, restart_required: payload.restart_required, restart_notes: payload.restart_notes });
      if (response.ok && payload.saved) {
        if (saveButton) saveButton.disabled = true;
      }
    } catch (error) {
      showJson(resultPanel, "Save error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });
})();
`;

  return (
    <section className="panel settingsPanel tabContent" id="settings" data-settings-env data-tab-panel="settings">
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Local-only configuration</p>
          <h2>Settings</h2>
        </div>
        <div className="topStatus">
          <StatusPill state={data.file_status.exists ? "env-mounted" : "env-missing"} />
          <StatusPill state={data.file_status.writable ? "writable" : "read-only"} />
        </div>
      </div>

      <div className="settingsNotice">
        <strong>Verify before save.</strong>
        <span>Edits are dry-run validated before `.env` is written. Saved settings may require Docker stack restart/recreate before taking effect.</span>
      </div>
      {envSettings.error ? <p className="errorText">{envSettings.error}</p> : null}

      <LocalLlmStatusPanel envSettings={envSettings} context="settings" />

      <section className="settingsMeta">
        <article><span>Env file</span><strong>{data.file_status.path}</strong></article>
        <article><span>Backup dir</span><strong>{data.file_status.backup_dir}</strong></article>
        <article><span>Format</span><strong>{data.file_status.output_format}</strong></article>
      </section>

      {data.warnings.length > 0 ? (
        <div className="settingsWarnings">
          {data.warnings.map((warning) => <span key={warning}>{warning}</span>)}
        </div>
      ) : null}

      <div className="settingsGroups">
        {groupedSettings.map((group) => (
          <section className="settingsGroup" key={group.key}>
            <h3>{groupHelpTerms[group.key] ? <HelpHeading term={groupHelpTerms[group.key]}>{group.label}</HelpHeading> : group.label}</h3>
            <div className="settingsRows">
              {group.settings.map((setting) => (
                <article className="settingRow" key={setting.key}>
                  <div className="settingInfo">
                    <strong>
                      {settingHelpTerms[setting.key] ? (
                        <TermHelp term={settingHelpTerms[setting.key]} label={setting.key} />
                      ) : setting.key}
                    </strong>
                    <span>{setting.description}</span>
                    <div className="messageMeta">
                      {setting.secret ? <StatusPill state="secret-masked" /> : null}
                      {setting.read_only ? <StatusPill state="read-only" /> : null}
                      {setting.restart_required ? <StatusPill state="restart-likely" /> : null}
                    </div>
                  </div>
                  <div className="settingControl">
                    {setting.secret ? (
                      <>
                        <label className="checkLine">
                          <input type="checkbox" data-secret-replace={setting.key} disabled={setting.read_only} />
                          Replace value
                        </label>
                        <input
                          type="password"
                          placeholder={setting.has_value ? setting.masked_value ?? "masked" : "empty"}
                          data-env-key={setting.key}
                          data-secret="true"
                          data-read-only={setting.read_only ? "true" : "false"}
                          disabled={setting.read_only}
                        />
                      </>
                    ) : (
                      <input
                        defaultValue={setting.value ?? ""}
                        data-current={setting.value ?? ""}
                        data-env-key={setting.key}
                        data-secret="false"
                        data-read-only={setting.read_only ? "true" : "false"}
                        readOnly={setting.read_only}
                      />
                    )}
                  </div>
                </article>
              ))}
            </div>
          </section>
        ))}
      </div>

      {data.unmanaged.length > 0 ? (
        <section className="settingsGroup unmanagedSettings">
          <h3><HelpHeading term="permissionScope">Unmanaged read-only keys</HelpHeading></h3>
          <div className="settingsRows">
            {data.unmanaged.map((item) => (
              <article className="settingRow" key={item.key}>
                <div className="settingInfo">
                  <strong>{item.key}</strong>
                  <span>Unknown key preserved by backend, not editable from this UI.</span>
                </div>
                <div className="settingControl">
                  <input readOnly value={item.masked_value} />
                </div>
              </article>
            ))}
          </div>
        </section>
      ) : null}

      <section className="settingsActions">
        <button type="button" data-settings-verify>Verify Dry Run</button>
        <button type="button" data-settings-save disabled>Save Settings</button>
        <input data-settings-token readOnly placeholder="verification token appears after passing dry run" />
      </section>

      <section className="settingsResultGrid">
        <pre data-settings-result>Dry-run result appears here.</pre>
        <pre data-settings-changed>Changed keys appear here.</pre>
        <pre data-settings-warnings>Warnings appear here.</pre>
        <pre data-settings-backup>Backup path appears after save.</pre>
      </section>
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

type LlmDisplay = {
  provider: string;
  baseUrl: string;
  model: string;
  timeout: string;
  evidenceRequired: string;
  answerMode: string;
  healthStatus: string;
  healthDetail: string;
  rawDiagnostics: Record<string, string>;
};

function LocalLlmStatusPanel({
  envSettings,
  context
}: {
  envSettings: ApiResult<EnvSettingsResponse>;
  context: "assistant" | "settings";
}) {
  const llm = buildLlmDisplay(envSettings.data);
  return (
    <section className={context === "settings" ? "settingsGroup llmStatusPanel" : "llmStatusPanel"} data-llm-status>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">{context === "settings" ? "Local model provider" : "Answer mode"}</p>
          <h3><HelpHeading term="localLlm">Local LLM Status</HelpHeading></h3>
        </div>
        <StatusPill state={llm.healthStatus} />
      </div>
      <div className="metrics compact">
        <article><span>Provider</span><strong>{llm.provider}</strong></article>
        <article><span>Health status</span><strong>{llm.healthStatus}</strong></article>
        <article><span>Answer mode</span><strong>{llm.answerMode}</strong></article>
        <article><span>Evidence required</span><strong>{llm.evidenceRequired}</strong></article>
      </div>
      <p className="agentRuntimeReason">{llm.healthDetail}</p>
      <div className="exampleGrid">
        <article>
          <span>Normal user example</span>
          <strong>Use local model to summarize uploaded warranty note using only evidence.</strong>
        </article>
        <article>
          <span>Coder example</span>
          <strong>Use local model to explain build log failure with citations.</strong>
        </article>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: raw provider diagnostics</summary>
        <pre>{JSON.stringify(llm.rawDiagnostics, null, 2)}</pre>
      </details>
    </section>
  );
}

function buildLlmDisplay(data: EnvSettingsResponse): LlmDisplay {
  const provider = settingValue(data, "LLM_PROVIDER", "none") || "none";
  const baseUrl = redactLlmUrl(settingValue(data, "OLLAMA_BASE_URL", "http://host.docker.internal:11434"));
  const model = settingValue(data, "OLLAMA_MODEL", "") || "not selected";
  const timeout = settingValue(data, "LLM_TIMEOUT_SECONDS", "60") || "60";
  const evidenceRequired = settingValue(data, "LLM_EVIDENCE_REQUIRED", "true") || "true";
  const enabled = provider === "ollama";
  const configured = enabled && model !== "not selected";
  const healthStatus = !enabled ? "disabled" : configured ? "configured-local" : "needs-model";
  const answerMode = !enabled
    ? "deterministic evidence"
    : configured
      ? "local LLM evidence-grounded with deterministic backup"
      : "unavailable until model is selected";
  const healthDetail = !enabled
    ? "No model calls are made while LLM_PROVIDER is none. Assistant uses deterministic evidence answers and insufficient-evidence responses."
    : configured
      ? "Settings does not contact Ollama. Evidence answers perform a timeout-bound local call only when retrieved evidence exists, then use a deterministic backup answer if unavailable."
      : "Select a local Ollama model before enabling evidence-grounded local generation. No token or cloud endpoint is required.";
  return {
    provider,
    baseUrl,
    model,
    timeout,
    evidenceRequired,
    answerMode,
    healthStatus,
    healthDetail,
    rawDiagnostics: {
      provider,
      ollama_base_url: baseUrl,
      model,
      timeout_seconds: timeout,
      evidence_required: evidenceRequired,
      external_model_default: "blocked",
      secrets_required: "false"
    }
  };
}

function settingValue(data: EnvSettingsResponse, key: string, fallback: string): string {
  const setting = data.settings.find((item) => item.key === key);
  if (!setting) return fallback;
  if (setting.secret) return setting.masked_value ?? fallback;
  return setting.value ?? fallback;
}

function redactLlmUrl(value: string): string {
  if (value.includes("@")) return "http://[redacted]";
  return value;
}

function ChatRetrievalPreview() {
  const browserApiBaseUrl = "/api";

  const script = `
(() => {
  const form = document.querySelector("[data-chat-preview-form]");
  const message = document.querySelector("[data-chat-preview-message]");
  const limit = document.querySelector("[data-chat-preview-limit]");
  const status = document.querySelector("[data-chat-preview-status]");
  const results = document.querySelector("[data-chat-preview-results]");
  const saveButton = document.querySelector("[data-chat-save-answer]");
  const saveStatus = document.querySelector("[data-chat-save-answer-status]");
  const apiBaseUrl = form?.getAttribute("data-api-base-url");
  let lastPayload = null;
  let lastQuestion = "";

  if (!form || !message || !limit || !status || !results || !apiBaseUrl) {
    return;
  }

  const shortId = (value) => {
    if (!value || typeof value !== "string") return "unknown";
    return value.length > 12 ? value.slice(0, 8) + "..." : value;
  };

  const formatScore = (value) => {
    const score = Number(value);
    return Number.isFinite(score) ? score.toFixed(3) : "unscored";
  };

  const textPreview = (value, fallback) => {
    const text = typeof value === "string" ? value.replace(/\\s+/g, " ").trim() : "";
    if (!text) return fallback;
    return text.length > 280 ? text.slice(0, 277) + "..." : text;
  };

  const retrievalMode = (hit) => {
    return hit.qdrant_payload?.retrieval_mode
      || hit.qdrant_payload?.embedding_method
      || hit.qdrant_payload?.payload?.retrieval_mode
      || "text or vector search";
  };

  const addMeta = (parent, label, value) => {
    const meta = document.createElement("span");
    meta.textContent = label + ": " + value;
    parent.appendChild(meta);
  };

  const uniqueStrings = (values, maxItems) => {
    const seen = new Set();
    const result = [];
    for (const value of values) {
      if (typeof value !== "string") continue;
      const trimmed = value.trim();
      if (!trimmed || seen.has(trimmed)) continue;
      seen.add(trimmed);
      result.push(trimmed);
      if (result.length >= maxItems) break;
    }
    return result;
  };

  const answerStatusFor = (payload, hits) => {
    const status = typeof payload.answer_status === "string" ? payload.answer_status : "";
    if (status) return status;
    return hits.length > 0 ? "retrieved" : "insufficient_evidence";
  };

  const buildAnswerRecordPayload = () => {
    const payload = lastPayload || {};
    const hits = Array.isArray(payload.retrieval_context?.hits) ? payload.retrieval_context.hits : [];
    const evidenceItemIds = uniqueStrings(hits.flatMap((hit) => Array.isArray(hit.evidence_items) ? hit.evidence_items.map((item) => item.id) : []), 50);
    const documentIds = uniqueStrings(hits.map((hit) => hit.document?.id || hit.qdrant_payload?.document_id), 50);
    const chunkIds = uniqueStrings(hits.map((hit) => hit.chunk?.id || hit.qdrant_payload?.chunk_id), 50);
    const sourceIds = uniqueStrings(hits.map((hit) => hit.source?.id), 50);
    const safeLabels = uniqueStrings([
      ...evidenceItemIds.slice(0, 10).map((id) => "evidence " + shortId(id)),
      ...documentIds.slice(0, 5).map((id) => "document " + shortId(id)),
      ...chunkIds.slice(0, 5).map((id) => "chunk " + shortId(id)),
      ...sourceIds.slice(0, 5).map((id) => "source " + shortId(id))
    ], 30);
    const hitCount = hits.length;
    return {
      user_question: lastQuestion,
      answer_status: answerStatusFor(payload, hits),
      answer_text: hitCount > 0
        ? "Saved retrieval-preview answer record with " + hitCount + " local evidence hit(s). Review cited evidence IDs and source trail labels before relying on it."
        : "Saved retrieval-preview answer record with insufficient local evidence.",
      facts: [],
      assumptions: ["Saved from local retrieval-preview metadata.", "Original evidence, documents, chunks, sources, and raw artifacts are preserved."],
      inferences: hitCount > 0 ? ["The saved answer record is limited to the retrieved local evidence identifiers."] : [],
      uncertainty: ["Retrieval scores and saved answer records are review aids, not proof of correctness."],
      missing_information: hitCount > 0
        ? ["Relevant sources not yet ingested, chunked, and embedded are absent from this saved answer record."]
        : ["No matching local chunks or evidence items were retrieved for the saved question."],
      evidence_item_ids: evidenceItemIds,
      document_ids: documentIds,
      chunk_ids: chunkIds,
      source_ids: sourceIds,
      safe_labels: safeLabels,
      retrieval_mode: "retrieval_preview",
      retrieval_count: hitCount,
      local_model_status: "not_used_retrieval_preview",
      metadata_json: {
        created_from: "results_ask_over_evidence",
        raw_evidence_text_stored: false,
        full_chat_memory: false,
        hosted_ai_called: false
      }
    };
  };

  const renderReviewSummary = (payload, hits) => {
    const summary = document.createElement("article");
    summary.className = "item evidenceItem";
    summary.setAttribute("data-retrieval-review-summary", "");

    const left = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = hits.length > 0 ? "Evidence retrieved" : "No evidence found";
    const detail = document.createElement("span");
    detail.textContent = hits.length > 0
      ? "Evidence-backed review: use the chunks and evidence items below as support."
      : "Insufficient evidence: try a narrower question or add/process more local evidence.";
    left.append(title, detail);

    const right = document.createElement("div");
    addMeta(right, "answer_status", payload.answer_status || "unknown");
    addMeta(right, "hits", String(hits.length));
    addMeta(right, "collection", payload.retrieval_context?.collection_exists === false ? "missing" : "available");
    summary.append(left, right);
    return summary;
  };

  const renderHit = (hit, index) => {
    const item = document.createElement("article");
    item.className = "item evidenceItem";
    item.setAttribute("data-retrieval-review-hit", String(index + 1));

    const left = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = hit.document?.title || "Evidence hit " + (index + 1);
    const snippet = document.createElement("span");
    snippet.textContent = textPreview(
      hit.chunk?.text_content || hit.evidence_items?.[0]?.statement,
      "No text preview returned for this hit."
    );
    left.append(title, snippet);

    const right = document.createElement("div");
    addMeta(right, "score", formatScore(hit.score));
    addMeta(right, "mode", retrievalMode(hit));
    addMeta(right, "chunk", shortId(hit.chunk?.id || hit.qdrant_payload?.chunk_id));
    addMeta(right, "document", shortId(hit.document?.id || hit.qdrant_payload?.document_id));
    addMeta(right, "source", hit.source?.name || shortId(hit.source?.id));
    addMeta(right, "evidence", String(hit.evidence_items?.length ?? 0));

    item.append(left, right);

    const evidenceItems = Array.isArray(hit.evidence_items) ? hit.evidence_items.slice(0, 2) : [];
    for (const evidenceItem of evidenceItems) {
      const evidenceLine = document.createElement("p");
      evidenceLine.className = "messageMeta";
      evidenceLine.textContent = "Evidence item: " + textPreview(evidenceItem.statement, evidenceItem.evidence_type || "recorded evidence");
      item.appendChild(evidenceLine);
    }

    return item;
  };

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    status.textContent = "Retrieving context";
    results.replaceChildren();
    results.removeAttribute("data-answer-status");
    results.removeAttribute("data-hit-count");
    lastPayload = null;
    lastQuestion = message.value.trim();
    if (saveStatus) saveStatus.textContent = "Run retrieval before saving an answer record.";

    try {
      const response = await fetch(apiBaseUrl + "/chat/retrieval-preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: message.value,
          limit: Number(limit.value || 5)
        })
      });

      if (!response.ok) {
        status.textContent = "Error: " + response.status + " " + response.statusText;
        const error = document.createElement("article");
        error.className = "item evidenceItem";
        error.setAttribute("data-retrieval-review-error", "");
        const body = document.createElement("div");
        const title = document.createElement("strong");
        title.textContent = "Retrieval failed";
        const detail = document.createElement("span");
        detail.textContent = "No evidence-backed review is available until the retrieval request succeeds.";
        body.append(title, detail);
        error.appendChild(body);
        results.appendChild(error);
        return;
      }

      const payload = await response.json();
      lastPayload = payload;
      const hits = payload.retrieval_context?.hits ?? [];
      const answerStatus = payload.answer_status || "unknown";
      status.textContent = "answer_status: " + answerStatus + " | hits: " + hits.length;
      results.setAttribute("data-answer-status", answerStatus);
      results.setAttribute("data-hit-count", String(hits.length));
      results.appendChild(renderReviewSummary(payload, hits));

      if (hits.length > 0) {
        hits.forEach((hit, index) => results.appendChild(renderHit(hit, index)));
      }
    } catch (error) {
      status.textContent = "Error: " + (error instanceof Error ? error.message : "Unknown error");
      const item = document.createElement("article");
      item.className = "item evidenceItem";
      item.setAttribute("data-retrieval-review-error", "");
      const body = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = "Retrieval failed";
      const detail = document.createElement("span");
      detail.textContent = "No evidence-backed review is available until the local API responds.";
      body.append(title, detail);
      item.appendChild(body);
      results.appendChild(item);
    }
  });

  saveButton?.addEventListener("click", async () => {
    if (!lastPayload || !lastQuestion) {
      if (saveStatus) saveStatus.textContent = "Ask over evidence before saving an answer record.";
      return;
    }
    if (saveStatus) saveStatus.textContent = "Saving answer record";
    try {
      const response = await fetch(apiBaseUrl + "/evidence-answers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(buildAnswerRecordPayload())
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      }
      if (saveStatus) saveStatus.textContent = "Saved answer record " + (payload.id || "recorded") + ". Refresh Results to see it in history.";
    } catch (error) {
      if (saveStatus) saveStatus.textContent = "Answer record save failed: " + (error instanceof Error ? error.message : "Unknown error");
    }
  });
})();
`;

  return (
    <section className="panel chatPreviewPanel">
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Ask over local evidence</p>
          <h2><HelpHeading term="chatRetrievalPreview">Ask Over Evidence</HelpHeading></h2>
        </div>
        <span className="statusText" data-chat-preview-status>answer_status: not_generated</span>
      </div>
      <form className="previewForm" data-chat-preview-form data-api-base-url={browserApiBaseUrl}>
        <label>
          <span>Question or request</span>
          <small>Ask about local evidence. Example for home use: "What does this document say about my bill?" Example for coders: "What failed in this build log? Cite the evidence."</small>
          <textarea data-chat-preview-message name="message" rows={3} placeholder="Ask a question or request an action..." defaultValue="What did I upload today?" />
        </label>
        <label>
          <span>Evidence limit</span>
          <small>How many matching local chunks to show. Example: 5.</small>
          <input data-chat-preview-limit name="limit" type="number" min="1" max="50" defaultValue="5" />
        </label>
        <button type="submit">Ask over evidence</button>
      </form>
      <div className="guidedManualActions">
        <button type="button" data-chat-save-answer>Save answer record</button>
        <span data-chat-save-answer-status>Run retrieval before saving. Saved records preserve history and do not change evidence.</span>
      </div>
      <div className="previewNote">
        Retrieval context only until saved. <TermHelp term="noExternalModel" label="No external model" /> answer, hidden reasoning, external model call, full chat memory, or action execution.
        <span data-retrieval-review-guidance> Evidence-backed only when hits are present; empty results mean insufficient evidence, not proof the information does not exist.</span>
      </div>
      <div className="stack previewResults" data-chat-preview-results />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function AgentCommandPanel({
  capabilities,
  approvals,
  taskPlans
}: {
  capabilities: ApiResult<AgentCapabilitiesResponse>;
  approvals: ApiResult<ApprovalRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
}) {
  const data = capabilities.data;
  const recentTaskPlans = taskPlans.data.slice(0, 5);
  const stackActions = data.actions.filter((action) => action.script_backed);
  const approvedAgentApprovals = approvals.data.filter((approval) => approval.request_type === "agent_action" && approval.status === "approved");
  const pendingAgentApprovals = approvals.data.filter((approval) => approval.request_type === "agent_action" && approval.status === "pending");
  const actionLabels: Record<string, string> = {
    show_project_health: "Show project health",
    show_git_status: "Show git status",
    show_latest_diff: "Show latest DIFF",
    show_work_items: "Show work items",
    run_retrieval_preview: "Run retrieval preview",
    start_stack: "Start stack",
    stop_stack: "Stop stack",
    run_last_healthy_stack: "Run last healthy stack"
  };
  const script = `
(() => {
  const root = document.querySelector("[data-agent-command]");
  if (!root) return;

  const commandInput = root.querySelector("[data-agent-command-input]");
  const paramsInput = root.querySelector("[data-agent-params]");
  const approvalInput = root.querySelector("[data-agent-approval-id]");
	  const previewButton = root.querySelector("[data-agent-preview]");
	  const executeButton = root.querySelector("[data-agent-execute]");
	  const approvalButton = root.querySelector("[data-agent-request-approval]");
	  const savePlanButton = root.querySelector("[data-agent-save-plan]");
	  const evidenceButton = root.querySelector("[data-agent-check-evidence]");
	  const planEvidenceButtons = root.querySelectorAll("[data-agent-plan-check-evidence]");
	  const proposeWorkSpecButtons = root.querySelectorAll("[data-agent-plan-propose-work-spec]");
	  const executeApprovedButton = root.querySelector("[data-agent-execute-approved]");
	  const actionSelect = root.querySelector("[data-agent-action-select]");
	  const approvalSelect = root.querySelector("[data-agent-approval-select]");
	  const intentPanel = root.querySelector("[data-agent-intent]");
	  const resultPanel = root.querySelector("[data-agent-result]");
	  const statusPanel = root.querySelector("[data-agent-status]");
	  const bridgePanel = root.querySelector("[data-agent-approval-bridge]");
	  const summaryPanel = root.querySelector("[data-agent-understanding-summary]");
	  const categoryPanel = root.querySelector("[data-agent-understanding-category]");
	  const posturePanel = root.querySelector("[data-agent-understanding-posture]");
	  const nextStepPanel = root.querySelector("[data-agent-understanding-next]");
	  const plannerPanel = root.querySelector("[data-agent-intake-planner]");
	  const evidencePanel = root.querySelector("[data-agent-planner-evidence]");
	  const capabilitiesPayload = JSON.parse(root.querySelector("[data-agent-capabilities-json]")?.textContent || "{}");
	  const approvalPayload = JSON.parse(root.querySelector("[data-agent-approvals-json]")?.textContent || "[]");
	  let latestIntent = null;
	  let latestTaskPlanId = null;

  const showJson = (node, label, payload) => {
    if (!node) return;
    node.textContent = label + "\\n" + JSON.stringify(payload, null, 2);
  };

  const parseParams = () => {
    const raw = paramsInput?.value?.trim() || "{}";
    if (!raw) return {};
    return JSON.parse(raw);
  };

	  const capabilityFor = (actionName) => {
	    return (capabilitiesPayload.actions || []).find((action) => action.name === actionName) || null;
	  };

	  const approvalMatches = (approval, actionName, parameters) => {
	    const payload = approval?.request_payload_json || {};
	    return approval?.status === "approved"
	      && approval?.request_type === "agent_action"
	      && payload.action_name === actionName
	      && JSON.stringify(payload.parameters || {}) === JSON.stringify(parameters || {});
	  };

	  const matchingApproval = (actionName, parameters) => {
	    return (approvalPayload || []).find((approval) => approvalMatches(approval, actionName, parameters)) || null;
	  };

	  const renderBridge = (intent) => {
	    if (!bridgePanel) return;
	    const parameters = (() => {
	      try { return parseParams(); } catch (_) { return {}; }
	    })();
	    const actionName = intent?.proposed_action || actionSelect?.value || "";
	    const capability = actionName ? capabilityFor(actionName) : null;
	    const approval = actionName ? matchingApproval(actionName, parameters) : null;
	    if (approval && approvalInput) approvalInput.value = approval.id;
	    const actionLabel = actionName || "No action selected";
	    const approvalState = !capability
	      ? "unsupported"
	      : capability.approval_required
	        ? approval
	          ? "approved"
	          : "approval needed"
	        : "not required";
	    bridgePanel.textContent = "Action: " + actionLabel
	      + " | class: " + (capability?.action_type || "unknown")
	      + " | approval: " + approvalState
	      + " | execution: " + (capability?.executable_in_api_runtime === false ? "runtime blocked" : "bounded route only");
	  };

	  const plannerCopy = (understanding, intent, capability) => {
	    if (understanding.unsupported_or_unsafe) {
	      return {
	        state: "unsupported",
	        title: "Unsupported or unsafe as written",
	        body: understanding.reason || intent.reason || "IGY6 will not turn this request into work or execution.",
	        next: "Rewrite the request as evidence review, data intake, report creation, feedback, outcome recording, or a listed bounded action."
	      };
	    }
	    if (understanding.clarification_needed) {
	      return {
	        state: "clarification-needed",
	        title: "Clarification needed",
	        body: (understanding.missing_information || []).join(", ") || "IGY6 needs more detail before it can choose a safe next step.",
	        next: understanding.next_step || "Add the missing target, evidence scope, or desired output."
	      };
	    }
	    if (understanding.approval_required || intent.approval_required) {
	      return {
	        state: "approval-required",
	        title: "Approval required before action",
	        body: capability?.interpreted_intent || "This request may affect the local runtime or another sensitive workflow.",
	        next: "Review the proposed bounded action and create an approval only if it matches what you want."
	      };
	    }
	    if (understanding.evidence_required) {
	      return {
	        state: "evidence-needed",
	        title: "Evidence needed",
	        body: "IGY6 should use stored local evidence before answering or creating a report.",
	        next: "Use Ask over evidence, or add/process more data if retrieval has no matches."
	      };
	    }
	    if (understanding.work_item_should_be_created) {
	      return {
	        state: "work-confirmation",
	        title: "May become bounded work after confirmation",
	        body: "The request looks like a workflow request, but this planner does not create work in this DIFF.",
	        next: understanding.next_step || "Review the request and use an existing supported workflow."
	      };
	    }
	    return {
	      state: intent.proposed_action ? "bounded-action" : "review-only",
	      title: intent.proposed_action ? "Bounded action matched" : "Review next step",
	      body: capability?.interpreted_intent || understanding.wants || "IGY6 can summarize the request posture.",
	      next: understanding.next_step || "Use the existing visible workflow that matches this category."
	    };
	  };

	  const planStatusFor = (understanding, intent) => {
	    if (understanding.unsupported_or_unsafe) return "unsupported";
	    if (understanding.clarification_needed) return "needs_clarification";
	    if (understanding.approval_required || intent.approval_required) return "approval_required";
	    if (understanding.evidence_required) return "evidence_needed";
	    return "proposed";
	  };

	  const supportedStateFor = (understanding, intent) => {
	    if (understanding.unsupported_or_unsafe) return "unsupported";
	    if (understanding.clarification_needed) return "clarification_needed";
	    if (understanding.approval_required || intent.approval_required) return "approval_required";
	    if (understanding.evidence_required) return "evidence_needed";
	    return "supported";
	  };

	  const boundedWorkSpecFor = (understanding, copy) => {
	    if (understanding?.category !== "create_report" || understanding.unsupported_or_unsafe) return null;
	    const summary = understanding.wants || commandInput?.value?.trim() || "Agent task plan report request";
	    return {
	      work_type: "report_generation",
	      expected_output: (copy.next || "Create a bounded report from this task plan.").slice(0, 1000),
	      payload_json: {
	        report_type: "agent_task_plan",
	        requested_summary: summary.slice(0, 1000),
	        intent_category: "create_report"
	      },
	      proposal_source: "agent_intake_planner",
	      safety_constraints: [
	        "Supported report_generation work item type only.",
	        "No shell command or user-provided argv.",
	        "Work creation remains approval-gated when approval is required."
	      ]
	    };
	  };

	  const taskPlanPayload = (intent) => {
	    const understanding = intent?.request_understanding || {};
	    const capability = intent?.proposed_action ? capabilityFor(intent.proposed_action) : null;
	    const copy = plannerCopy(understanding, intent || {}, capability);
	    const workSpec = boundedWorkSpecFor(understanding, copy);
	    const requestSummary = understanding.wants || commandInput?.value?.trim() || "Agent task plan preview";
	    const requiredEvidence = understanding.evidence_required
	      ? ["Check stored local evidence before creating work or answering."]
	      : [];
	    return {
	      user_request_summary: requestSummary.slice(0, 1000),
	      intent_category: understanding.category || "unclear",
	      status: workSpec ? (understanding.approval_required || intent?.approval_required ? "approval_required" : "ready") : planStatusFor(understanding, intent || {}),
	      proposed_steps: [copy.next || understanding.next_step || "Review the safe next step."],
	      required_evidence: requiredEvidence,
	      approval_required: Boolean(understanding.approval_required || intent?.approval_required),
	      supported_state: workSpec ? "supported" : supportedStateFor(understanding, intent || {}),
	      next_safe_action: (understanding.next_step || copy.next || "Review the safe next step.").slice(0, 1000),
	      requested_by_actor_id: "local-owner",
	      metadata_json: {
	        created_from: "agent_intake_planner",
	        proposed_action: intent?.proposed_action || null,
	        work_item_should_be_created: Boolean(understanding.work_item_should_be_created),
	        unsupported_or_unsafe: Boolean(understanding.unsupported_or_unsafe),
	        saved_preview_only: !workSpec,
	        ...(workSpec ? { plan_to_work: workSpec } : {})
	      }
	    };
	  };

	  const evidenceLabel = (hit, index) => {
	    const evidence = Array.isArray(hit.evidence_items) ? hit.evidence_items[0] : null;
	    const parts = [];
	    if (evidence?.id) parts.push("evidence " + evidence.id);
	    if (hit.chunk?.id || hit.qdrant_payload?.chunk_id) parts.push("chunk " + (hit.chunk?.id || hit.qdrant_payload?.chunk_id));
	    if (hit.document?.id || hit.qdrant_payload?.document_id) parts.push("document " + (hit.document?.id || hit.qdrant_payload?.document_id));
	    if (hit.source?.id) parts.push("source " + hit.source.id);
	    return parts.length ? parts.join(" | ") : "hit " + (index + 1);
	  };

	  const renderEvidenceSummary = (payload) => {
	    const hits = Array.isArray(payload?.retrieval_context?.hits) ? payload.retrieval_context.hits : [];
	    const labels = hits.slice(0, 5).map(evidenceLabel);
	    const summary = {
	      answer_status: payload?.answer_status || "unknown",
	      retrieved_count: hits.length,
	      labels,
	      missing_evidence: hits.length === 0
	    };
	    if (evidencePanel) {
	      evidencePanel.innerHTML = "";
	      addPlannerRow(
	        evidencePanel,
	        hits.length > 0 ? "Evidence check" : "Missing evidence",
	        hits.length > 0
	          ? "Retrieved " + hits.length + " relevant local evidence hit(s)."
	          : "No relevant local evidence was retrieved. Add/process data or narrow the request before proceeding.",
	        hits.length > 0 ? "retrieved" : "missing"
	      );
	      labels.forEach((label) => addPlannerRow(evidencePanel, "Evidence label", label, "safe-id"));
	    }
	    return summary;
	  };

	  const evidenceSummaryPayload = (summary) => ({
	    actor_id: "local-owner",
	    answer_status: summary.answer_status || "unknown",
	    retrieved_count: Number(summary.retrieved_count || 0),
	    safe_labels: Array.isArray(summary.labels) ? summary.labels.slice(0, 5) : [],
	    missing_evidence: Boolean(summary.missing_evidence),
	    missing_evidence_guidance: summary.missing_evidence
	      ? "No relevant local evidence was retrieved. Add/process data or narrow the request before proceeding."
	      : "Relevant local evidence was retrieved. Review safe labels before creating work or answering."
	  });

	  const persistEvidenceSummary = async (taskPlanId, summary) => {
	    const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/evidence-summary", {
	      method: "POST",
	      headers: { "Content-Type": "application/json" },
	      body: JSON.stringify(evidenceSummaryPayload(summary))
	    });
	    const payload = await response.json();
	    if (!response.ok) {
	      throw new Error(payload?.detail || response.statusText || "Evidence summary persistence failed");
	    }
	    return payload;
	  };

	  const addPlannerRow = (parent, label, value, state) => {
	    const item = document.createElement("article");
	    item.className = "agentPlannerCard";
	    item.setAttribute("data-agent-planner-card", label);
	    const title = document.createElement("strong");
	    title.textContent = label;
	    const body = document.createElement("span");
	    body.textContent = value || "not returned";
	    item.append(title, body);
	    if (state) {
	      const pill = document.createElement("em");
	      pill.textContent = state;
	      item.appendChild(pill);
	    }
	    parent.appendChild(item);
	  };

	  const renderPlanner = (intent) => {
	    if (!plannerPanel) return;
	    plannerPanel.innerHTML = "";
	    const understanding = intent?.request_understanding;
	    if (!understanding) {
	      addPlannerRow(plannerPanel, "Planner", "Preview a request to see the next safe step.", "waiting");
	      return;
	    }
	    const capability = intent.proposed_action ? capabilityFor(intent.proposed_action) : null;
	    const copy = plannerCopy(understanding, intent, capability);
	    addPlannerRow(plannerPanel, "Status", copy.title, copy.state);
	    addPlannerRow(plannerPanel, "Category", understanding.category || "unclear", understanding.category || "unclear");
	    addPlannerRow(plannerPanel, "Evidence", understanding.evidence_required ? "Stored evidence should be checked first." : "No evidence lookup required before this preview.", understanding.evidence_required ? "needed" : "not-needed");
	    addPlannerRow(plannerPanel, "Approval", understanding.approval_required || intent.approval_required ? "Explicit approval is required before execution." : "No approval is required for this preview.", understanding.approval_required || intent.approval_required ? "required" : "not-required");
	    addPlannerRow(plannerPanel, "Next safe step", copy.next, "guidance");
	  };

  const renderUnderstanding = (intent) => {
    const understanding = intent?.request_understanding;
    if (!understanding) return;
    if (summaryPanel) summaryPanel.textContent = understanding.wants || "IGY6 needs more detail before it can continue.";
    if (categoryPanel) categoryPanel.textContent = "Category: " + (understanding.category || "unclear");
    const posture = [];
    posture.push(understanding.evidence_required ? "Evidence needed" : "No evidence lookup required first");
    posture.push(understanding.clarification_needed ? "Needs clarification" : "Clear enough to preview");
    posture.push(understanding.approval_required ? "Approval required" : "No approval required for preview");
    posture.push(understanding.work_item_should_be_created ? "May become work after confirmation" : "No work item now");
    posture.push(understanding.unsupported_or_unsafe ? "Unsupported or unsafe as written" : "Supported posture");
	    if (posturePanel) posturePanel.textContent = posture.join(" | ");
	    if (nextStepPanel) nextStepPanel.textContent = understanding.next_step || "";
	    renderPlanner(intent);
	    renderBridge(intent);
	  };

  const setButtons = () => {
    if (!executeButton || !approvalButton || !executeApprovedButton) return;
    executeButton.disabled = true;
    approvalButton.disabled = true;
    if (savePlanButton) savePlanButton.disabled = !latestIntent?.request_understanding;
    if (evidenceButton) evidenceButton.disabled = !latestIntent?.request_understanding;
    executeApprovedButton.disabled = true;
    if (!latestIntent?.proposed_action || latestIntent.missing_parameters?.length) return;
    const capability = capabilityFor(latestIntent.proposed_action);
    const runtimeExecutable = capability?.executable_in_api_runtime !== false;
    if (latestIntent.approval_required) {
      approvalButton.disabled = false;
      executeApprovedButton.disabled = !(runtimeExecutable && approvalInput?.value?.trim());
      return;
    }
    executeButton.disabled = !(latestIntent.executable_now && runtimeExecutable);
  };

  const previewIntent = async () => {
    const parameters = parseParams();
    const response = await fetch("/api/agent/intent", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message: commandInput?.value || "", parameters, actor_id: "local-owner" })
    });
    const payload = await response.json();
    latestIntent = payload;
    renderUnderstanding(payload);
    showJson(intentPanel, response.ok ? "Agent intent preview" : "Intent preview failed", payload);
    const capability = payload.proposed_action ? capabilityFor(payload.proposed_action) : null;
    const runtimeNote = capability?.reason || capabilitiesPayload.runtime?.reason || "Runtime allows this action class.";
	    if (statusPanel) {
	      statusPanel.textContent = payload.proposed_action
	        ? "Runtime: " + (capability?.executable_in_api_runtime ? "executable" : "blocked") + " | " + runtimeNote
	        : "Rejected by typed registry. No shell command will run.";
	    }
	    if (payload.proposed_action && actionSelect) actionSelect.value = payload.proposed_action;
	    setButtons();
	  };

  previewButton?.addEventListener("click", async () => {
    try {
      await previewIntent();
    } catch (error) {
      showJson(intentPanel, "Intent preview error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  executeButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || latestIntent.approval_required) return;
      const response = await fetch("/api/agent/actions/" + encodeURIComponent(latestIntent.proposed_action) + "/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ parameters: parseParams(), actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Read-only action result" : "Action failed", payload);
    } catch (error) {
      showJson(resultPanel, "Action error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  approvalButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || !latestIntent.approval_required) return;
      const response = await fetch("/api/approvals", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          request_type: "agent_action",
          requested_by_actor_id: "local-owner",
          request_payload_json: {
            action_name: latestIntent.proposed_action,
            parameters: parseParams()
          }
        })
      });
      const payload = await response.json();
	      if (payload?.id && approvalInput) approvalInput.value = payload.id;
	      showJson(resultPanel, response.ok ? "Approval request created" : "Approval request failed", payload);
	      if (bridgePanel) bridgePanel.textContent = response.ok
	        ? "Approval requested. Review it in Settings before running the approved action."
	        : "Approval request failed. No action was executed.";
	      setButtons();
    } catch (error) {
      showJson(resultPanel, "Approval request error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  savePlanButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.request_understanding) return;
      const response = await fetch("/api/agent/task-plans", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(taskPlanPayload(latestIntent))
      });
      const payload = await response.json();
      if (response.ok && payload?.id) latestTaskPlanId = payload.id;
      showJson(resultPanel, response.ok ? "Task plan saved" : "Task plan save failed", payload);
      if (statusPanel) {
        statusPanel.textContent = response.ok
          ? "Saved task plan metadata. No work item was created and no action was executed."
          : "Task plan was not saved. No work item was created and no action was executed.";
      }
    } catch (error) {
      showJson(resultPanel, "Task plan save error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  evidenceButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.request_understanding) return;
      evidenceButton.disabled = true;
      if (evidencePanel) evidencePanel.textContent = "Checking local evidence...";
      const response = await fetch("/api/chat/retrieval-preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: commandInput?.value || latestIntent.request_understanding.wants || "",
          limit: 5
        })
      });
      const payload = await response.json();
      if (!response.ok) {
        showJson(resultPanel, "Evidence check failed", { detail: payload?.detail || response.statusText });
        if (evidencePanel) evidencePanel.textContent = "Evidence check failed. No plan action was taken.";
        return;
      }
      const summary = renderEvidenceSummary(payload);
      if (latestTaskPlanId) {
        try {
          await persistEvidenceSummary(latestTaskPlanId, summary);
          showJson(resultPanel, "Evidence check summary saved to task plan", summary);
          if (statusPanel) statusPanel.textContent = "Saved safe evidence summary on the latest task plan. Reload to review persisted evidence readiness.";
        } catch (persistError) {
          showJson(resultPanel, "Evidence check summary persistence failed", {
            summary,
            detail: persistError instanceof Error ? persistError.message : "Unknown persistence error"
          });
        }
      } else {
        showJson(resultPanel, "Evidence check summary", summary);
      }
    } catch (error) {
      showJson(resultPanel, "Evidence check error", { detail: error instanceof Error ? error.message : "Unknown error" });
      if (evidencePanel) evidencePanel.textContent = "Evidence check failed. No plan action was taken.";
    } finally {
      evidenceButton.disabled = false;
    }
  });

	  root.querySelectorAll("[data-agent-plan-create-work]").forEach((button) => {
    button.addEventListener("click", async () => {
      const taskPlanId = button.getAttribute("data-task-plan-id");
      if (!taskPlanId) return;
      button.disabled = true;
      try {
        const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/work-item", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ actor_id: "local-owner", approval_id: null })
        });
        const payload = await response.json();
        showJson(resultPanel, response.ok ? "Work item created from plan" : "Plan-to-work blocked", payload);
        if (statusPanel) {
          statusPanel.textContent = response.ok
            ? "Created a work item from the persisted task plan. It still requires the normal work queue safety flow."
            : "Plan-to-work was blocked. No action was executed.";
        }
      } catch (error) {
        showJson(resultPanel, "Plan-to-work error", { detail: error instanceof Error ? error.message : "Unknown error" });
      } finally {
        button.disabled = false;
      }
    });
  });

  executeApprovedButton?.addEventListener("click", async () => {
    try {
      if (!latestIntent?.proposed_action || !latestIntent.approval_required || !approvalInput?.value?.trim()) return;
      const response = await fetch("/api/agent/actions/" + encodeURIComponent(latestIntent.proposed_action) + "/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          parameters: parseParams(),
          approval_id: approvalInput.value.trim(),
          actor_id: "local-owner"
        })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Approved action result" : "Approved action blocked or failed", payload);
    } catch (error) {
      showJson(resultPanel, "Approved action error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });
	  approvalInput?.addEventListener("input", setButtons);
	  approvalSelect?.addEventListener("change", () => {
	    if (approvalInput) approvalInput.value = approvalSelect.value || "";
	    setButtons();
	  });

	  planEvidenceButtons.forEach((button) => {
	    button.addEventListener("click", async () => {
	      const taskPlanId = button.getAttribute("data-task-plan-id");
	      const query = button.getAttribute("data-task-plan-summary") || "";
	      if (!taskPlanId || !query.trim()) return;
	      button.disabled = true;
	      try {
	        const response = await fetch("/api/chat/retrieval-preview", {
	          method: "POST",
	          headers: { "Content-Type": "application/json" },
	          body: JSON.stringify({ message: query, limit: 5 })
	        });
	        const payload = await response.json();
	        if (!response.ok) {
	          showJson(resultPanel, "Task plan evidence check failed", { detail: payload?.detail || response.statusText });
	          return;
	        }
	        const summary = renderEvidenceSummary(payload);
	        await persistEvidenceSummary(taskPlanId, summary);
	        showJson(resultPanel, "Task plan evidence summary saved", summary);
	        if (statusPanel) {
	          statusPanel.textContent = "Saved safe evidence summary on task plan " + taskPlanId + ". Reload to review it in task history.";
	        }
	      } catch (error) {
	        showJson(resultPanel, "Task plan evidence summary error", { detail: error instanceof Error ? error.message : "Unknown error" });
	      } finally {
	        button.disabled = false;
	      }
	    });
	  });

	  proposeWorkSpecButtons.forEach((button) => {
	    button.addEventListener("click", async () => {
	      const taskPlanId = button.getAttribute("data-task-plan-id");
	      if (!taskPlanId) return;
	      button.disabled = true;
	      try {
	        const response = await fetch("/api/agent/task-plans/" + encodeURIComponent(taskPlanId) + "/work-spec", {
	          method: "POST",
	          headers: { "Content-Type": "application/json" },
	          body: JSON.stringify({
	            actor_id: "local-owner",
	            work_type: "report_generation",
	            expected_output: "Create a bounded report from this reviewed task plan."
	          })
	        });
	        const payload = await response.json();
	        showJson(resultPanel, response.ok ? "Work spec proposed" : "Work spec proposal blocked", payload);
	        if (statusPanel) {
	          statusPanel.textContent = response.ok
	            ? "Added a bounded report_generation work spec. Reload to show work-item eligibility; no work was created."
	            : "Work spec proposal was blocked. No work item was created and no action was executed.";
	        }
	      } catch (error) {
	        showJson(resultPanel, "Work spec proposal error", { detail: error instanceof Error ? error.message : "Unknown error" });
	      } finally {
	        button.disabled = false;
	      }
	    });
	  });
	  actionSelect?.addEventListener("change", () => {
	    const selected = actionSelect.selectedOptions?.[0];
	    if (commandInput && selected?.getAttribute("data-prompt")) {
	      commandInput.value = selected.getAttribute("data-prompt");
	    }
	    latestIntent = null;
	    renderBridge({ proposed_action: actionSelect.value, approval_required: capabilityFor(actionSelect.value)?.approval_required });
	    setButtons();
	  });
	})();
	`;

  return (
    <section className="panel agentCommandPanel" id="agent-command" data-agent-command>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Safe local actions</p>
          <h2>Action Preview And Execution</h2>
        </div>
        <div className="topStatus">
          <StatusPill state={data.runtime.docker_control_available ? "stack-control-ready" : "stack-control-blocked"} />
          <StatusPill state="no-shell" />
          <StatusPill state="approval-gated" />
        </div>
      </div>

      <div className="agentNotice">
        <strong>Preview first.</strong>
        <span>IGY6 first summarizes what it thinks you want. Ambiguous, unsupported, or risky requests stay in clarification or approval posture instead of silently becoming work.</span>
      </div>
      {capabilities.error ? <p className="errorText">{capabilities.error}</p> : null}

      <section className="agentRuntimeGrid">
        <article><span>Docker CLI</span><strong>{data.runtime.docker_cli_available ? "available" : "unavailable"}</strong></article>
        <article><span>Docker Compose</span><strong>{data.runtime.docker_compose_available ? "available" : "unavailable"}</strong></article>
        <article><span>Docker control</span><strong>{data.runtime.docker_control_available ? "available" : "blocked"}</strong></article>
        <article><span>Socket/control path</span><strong>{data.runtime.docker_socket_available ? data.runtime.docker_socket_path ?? "configured" : "unavailable"}</strong></article>
      </section>

      {data.runtime.reason ? <p className="agentRuntimeReason">{data.runtime.reason}</p> : null}

	      <section className="agentActionList">
	        {data.actions.map((action) => (
	          <article className="agentActionCard" key={action.name}>
            <div>
              <strong>{actionLabels[action.name] ?? action.name.replaceAll("_", " ")}</strong>
              <span>{action.interpreted_intent}</span>
            </div>
            <div className="messageMeta">
              <StatusPill state={action.action_type} />
              <StatusPill state={action.approval_required ? "approval-required" : "read-only"} />
              {action.script_backed ? <StatusPill state={action.executable_in_api_runtime ? "runtime-ready" : "runtime-blocked"} /> : null}
            </div>
            {action.reason ? <p>{action.reason}</p> : null}
          </article>
	        ))}
	      </section>

	      <section className="agentApprovalBridge">
	        <div className="guidedManualNotice">
	          <strong>Approval-to-action bridge</strong>
	          <span>Choose a fixed action, preview it, then request or select a matching approval when the action requires one. No arbitrary commands or user-provided argv are accepted.</span>
	        </div>
	        <section className="agentCommandGrid">
	          <label>
	            <span>Bounded action</span>
	            <select data-agent-action-select defaultValue="show_project_health">
	              {data.actions.map((action) => (
	                <option
	                  key={action.name}
	                  value={action.name}
	                  data-prompt={actionLabels[action.name] ?? action.interpreted_intent}
	                >
	                  {actionLabels[action.name] ?? action.name.replaceAll("_", " ")} · {action.approval_required ? "approval required" : "read-only"}
	                </option>
	              ))}
	            </select>
	          </label>
	          <label>
	            <span>Approved agent approval</span>
	            <select data-agent-approval-select defaultValue="" disabled={approvedAgentApprovals.length === 0}>
	              <option value="">No approved approval selected</option>
	              {approvedAgentApprovals.map((approval) => (
	                <option key={approval.id} value={approval.id}>{approval.id}</option>
	              ))}
	            </select>
	          </label>
	        </section>
	        <p className="agentStatus" data-agent-approval-bridge>
	          {pendingAgentApprovals.length > 0
	            ? `${pendingAgentApprovals.length} pending agent approval request(s) need review in Settings.`
	            : approvedAgentApprovals.length > 0
	              ? `${approvedAgentApprovals.length} approved agent approval(s) available for matching actions.`
	              : "No approved agent action approval is available yet."}
	        </p>
	      </section>

      <section className="agentCommandGrid">
        <label>
          <span>Action request</span>
          <small>Plain English request. Example: "What did I upload today?", "Create a report about failed builds", or "Show project health."</small>
          <textarea data-agent-command-input rows={3} placeholder="Show project health." defaultValue="Show project health." />
        </label>
      </section>

      <section className="agentCommandActions">
        <button type="button" data-agent-preview>Preview action</button>
        <button type="button" data-agent-check-evidence disabled>Check evidence</button>
        <button type="button" data-agent-save-plan disabled>Save task plan</button>
        <button type="button" data-agent-execute disabled>Run safe action</button>
        <button type="button" data-agent-request-approval disabled>Request approval</button>
        <button type="button" data-agent-execute-approved disabled>Run with approval</button>
      </section>

      <p className="agentStatus" data-agent-status>
        Stack-control actions: {stackActions.every((action) => action.executable_in_api_runtime) ? "executable from API runtime" : "blocked unless API runtime has Docker CLI, Compose, and Docker control access."}
      </p>

	      <section className="agentUnderstanding">
	        <div>
	          <span>IGY6 understood this as</span>
	          <strong data-agent-understanding-summary>Preview a request to see the request summary.</strong>
	        </div>
	        <p data-agent-understanding-category>Category: not previewed</p>
	        <p data-agent-understanding-posture>Evidence, clarification, approval, and work-item posture will appear here.</p>
	        <p data-agent-understanding-next>Next step will appear here.</p>
	      </section>

	      <section className="agentPlanner" data-agent-intake-planner aria-label="Agent task intake planner">
	        <article className="agentPlannerCard" data-agent-planner-card="Planner">
	          <strong>Planner</strong>
	          <span>Preview a request to see the next safe step.</span>
	          <em>waiting</em>
	        </article>
	      </section>

	      <section className="agentPlanner" data-agent-planner-evidence aria-label="Agent planner evidence check">
	        <article className="agentPlannerCard">
	          <strong>Evidence check</strong>
	          <span>Preview a request, then check whether local evidence appears relevant before work or action proceeds.</span>
	          <em>not-checked</em>
	        </article>
	      </section>

	      <section className="agentPlanner" data-agent-task-plan-records aria-label="Persisted agent task plans">
	        {taskPlans.error ? <p className="errorText">{taskPlans.error}</p> : null}
	        {recentTaskPlans.length === 0 ? (
	          <article className="agentPlannerCard">
	            <strong>Persisted task plans</strong>
	            <span>No task plans have been saved yet. Preview a request, then save the plan metadata if it should be remembered.</span>
	            <em>empty</em>
	          </article>
	        ) : recentTaskPlans.map((plan) => {
	          const planToWork = plan.metadata_json?.plan_to_work;
	          const evidenceSummary = plan.metadata_json?.evidence_summary;
	          const evidenceSummaryObject = evidenceSummary && typeof evidenceSummary === "object" ? evidenceSummary as Record<string, unknown> : null;
	          const evidenceCount = typeof evidenceSummaryObject?.retrieved_count === "number" ? evidenceSummaryObject.retrieved_count : null;
	          const evidenceStatus = typeof evidenceSummaryObject?.answer_status === "string" ? evidenceSummaryObject.answer_status : null;
	          const evidenceLabels = Array.isArray(evidenceSummaryObject?.safe_labels)
	            ? evidenceSummaryObject.safe_labels.filter((label): label is string => typeof label === "string").slice(0, 3)
	            : [];
	          const hasWorkSpec = Boolean(planToWork && typeof planToWork === "object" && "work_type" in planToWork);
	          const workType = hasWorkSpec && planToWork && typeof planToWork === "object" && "work_type" in planToWork
	            ? String((planToWork as { work_type?: unknown }).work_type ?? "unknown")
	            : null;
	          const eligibleForWork = hasWorkSpec
	            && plan.supported_state === "supported"
	            && !plan.approval_required
	            && (plan.status === "proposed" || plan.status === "ready");
	          const canProposeReportWorkSpec = !hasWorkSpec
	            && plan.intent_category === "create_report"
	            && plan.supported_state !== "unsupported"
	            && plan.status !== "converted_to_work"
	            && plan.status !== "canceled";
	          const guidance = plan.approval_required
	            ? "Approval is required before this plan can create work."
	            : plan.supported_state !== "supported"
	              ? "This plan is not supported for work creation yet."
	              : hasWorkSpec
	                ? "This plan includes a supported " + workType + " work spec."
	                : "This plan has no supported work-item specification yet.";
	          return (
	            <article className="agentPlannerCard" key={plan.id} data-agent-task-plan-record>
	              <strong>{plan.user_request_summary}</strong>
	              <span>{plan.next_safe_action}</span>
	              <em>{plan.status} · {plan.intent_category} · {plan.approval_required ? "approval required" : "no approval required"} · {hasWorkSpec ? "eligible spec" : "preview only"}</em>
	              <span>{guidance}</span>
	              <span>Evidence readiness: {evidenceStatus ? `${evidenceStatus} · ${evidenceCount ?? 0} hit(s)` : "not checked"}</span>
	              {evidenceLabels.length > 0 ? <span>Evidence labels: {evidenceLabels.join(" | ")}</span> : null}
	              {workType ? <span>Supported work type: {workType}</span> : null}
	              <button type="button" data-agent-plan-check-evidence data-task-plan-id={plan.id} data-task-plan-summary={plan.user_request_summary}>Check and save evidence</button>
	              {canProposeReportWorkSpec ? (
	                <button type="button" data-agent-plan-propose-work-spec data-task-plan-id={plan.id}>Propose report work spec</button>
	              ) : null}
	              {eligibleForWork ? (
	                <button type="button" data-agent-plan-create-work data-task-plan-id={plan.id}>Create work item</button>
	              ) : null}
	            </article>
	          );
	        })}
	      </section>

      <details className="advancedPanel">
        <summary>Advanced: raw parameters, approval ID, response JSON, and route details</summary>
        <section className="agentCommandGrid">
          <label>
            <span>Raw parameters JSON</span>
            <small>Advanced only. Example: {"{}"}</small>
            <textarea data-agent-params rows={3} defaultValue="{}" />
          </label>
          <label>
            <span>Approval ID for approved action</span>
            <small>Paste an approval ID only after approving the matching request in Safety & Audit.</small>
            <input data-agent-approval-id placeholder="approval id after explicit approval" />
          </label>
        </section>
        <section className="agentResultGrid">
          <pre data-agent-intent>Agent intent preview appears here.</pre>
          <pre data-agent-result>Agent action result appears here.</pre>
        </section>
        <p className="routeHint">Routes used: /agent/intent, /agent/task-plans, /agent/task-plans/:id/evidence-summary, /agent/task-plans/:id/work-spec, /agent/task-plans/:id/work-item, /agent/actions/:action/execute, /approvals.</p>
      </details>
	      <script type="application/json" data-agent-capabilities-json dangerouslySetInnerHTML={{ __html: JSON.stringify(data) }} />
	      <script type="application/json" data-agent-approvals-json dangerouslySetInnerHTML={{ __html: JSON.stringify(approvals.data) }} />
	      <script dangerouslySetInnerHTML={{ __html: script }} />
	    </section>
  );
}

function GuidedManualTextUpload({ sources }: { sources: ApiResult<SourceRecord[]> }) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const manualSources = sources.data
    .filter((source) => source.enabled && source.source_type === "manual_upload")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      sensitivity: source.sensitivity,
      permissions: source.permissions ?? [],
    }));
  const manualSourcesJson = JSON.stringify(manualSources).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-guided-manual-upload]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const sourceData = JSON.parse(root.querySelector("[data-guided-manual-sources-json]")?.textContent || "[]");
  const result = root.querySelector("[data-guided-manual-result]");
  const debug = root.querySelector("[data-guided-manual-debug]");
  const submit = root.querySelector("[data-guided-manual-submit]");
  const sourceSelect = root.querySelector("[name='guided_source_choice']");
  const newSourceFields = root.querySelector("[data-new-source-fields]");
  const approvalHint = root.querySelector("[data-guided-approval-hint]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const writeResult = (state, message, nextSteps, payload, details) => {
    if (result) {
      result.innerHTML = "";
      const title = document.createElement("strong");
      title.textContent = state;
      const body = document.createElement("span");
      body.textContent = message;
      result.append(title, body);
      if (details?.length) {
        const detailList = document.createElement("dl");
        detailList.setAttribute("data-guided-manual-work-status", "");
        details.forEach((detail) => {
          const term = document.createElement("dt");
          term.textContent = detail.label;
          const description = document.createElement("dd");
          description.textContent = detail.value;
          detailList.append(term, description);
        });
        result.appendChild(detailList);
      }
      if (nextSteps?.length) {
        const list = document.createElement("ul");
        nextSteps.forEach((step) => {
          const item = document.createElement("li");
          item.textContent = step;
          list.appendChild(item);
        });
        result.appendChild(list);
      }
    }
    if (debug) debug.textContent = payload ? JSON.stringify(payload, null, 2) : "";
  };
  const setBusy = (busy) => {
    if (submit) {
      submit.disabled = busy;
      submit.textContent = busy ? "Submitting..." : "Submit manual text";
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => {
      binary += String.fromCharCode(byte);
    });
    return btoa(binary);
  };
  const safeFilename = (filename) => {
    const cleaned = (filename || "manual-note.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim();
    return cleaned || "manual-note.txt";
  };
  const permissionFor = (source) => (source.permissions || []).find((permission) => {
    const operations = permission.allowed_operations || [];
    return operations.includes("collect") || operations.includes("read");
  });
  const selectedSource = () => {
    if (sourceSelect?.value === "new") return null;
    const index = Number(sourceSelect?.value || -1);
    return Number.isInteger(index) ? sourceData[index] : null;
  };
  const refreshSourceHints = () => {
    const source = selectedSource();
    if (newSourceFields) newSourceFields.hidden = Boolean(source);
    if (!approvalHint) return;
    if (!source) {
      approvalHint.textContent = checked("guided_approval_required")
        ? "This new source will request approval first. Text will not be collected until an approval is approved."
        : "This new source can collect this manual text immediately under the created permission.";
      return;
    }
    const permission = permissionFor(source);
    if (!permission) {
      approvalHint.textContent = "This source has no collect/read permission visible to the guided flow. Use Advanced for diagnostics.";
      return;
    }
    approvalHint.textContent = permission.approval_required
      ? "This source requires approval before collection. Submitting will create an approval request and stop in pending state."
      : "This source permission allows immediate manual text collection.";
  };
  sourceSelect?.addEventListener("change", refreshSourceHints);
  root.querySelector("[name='guided_approval_required']")?.addEventListener("change", refreshSourceHints);
  refreshSourceHints();

  root.querySelector("[data-guided-manual-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = value("guided_text");
    if (!text) {
      writeResult("Text required", "Paste authorized UTF-8 text before submitting.", ["This path does not accept binary files, images, audio, or video."]);
      return;
    }
    setBusy(true);
    try {
      let source = selectedSource();
      let permission = source ? permissionFor(source) : null;
      const sourceName = value("guided_source_name") || value("guided_text_title") || "Manual Text Notes";
      if (!source) {
        source = await postJson("/sources", {
          name: sourceName,
          source_type: "manual_upload",
          location: value("guided_source_description") || "Manual text entered in Add Data",
          sensitivity: value("guided_sensitivity") || "internal",
          metadata_json: { created_from: "guided_add_data_manual_text" },
          permission: {
            scope_json: {
              path: "manual_text",
              entered_from: "Add Data guided manual text"
            },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: checked("guided_approval_required")
          }
        });
        permission = permissionFor(source);
      }
      if (!source?.id || !permission?.id) {
        throw new Error("No source permission was available for guided manual text collection.");
      }
      const filename = safeFilename(value("guided_text_title") || sourceName || "manual-note.txt");
      if (permission.approval_required) {
        const approval = await postJson("/approvals", {
          request_type: "manual_upload_collection",
          request_payload_json: {
            source_id: source.id,
            source_permission_id: permission.id,
            operation: "manual_upload_collection",
            filename
          }
        });
        writeResult(
          "Approval pending",
          "IGY6 created the manual text source context and requested collection approval. The text was not uploaded because this permission requires an approved approval record.",
          ["Open Settings to review pending approvals.", "After approval, use the Advanced route console with the approved approval record for this low-level collection path.", "Processing status appears in Work after collection, and evidence appears in Results."],
          { source: { name: source.name, type: source.source_type }, permission: { approval_required: permission.approval_required }, approval }
        );
        return;
      }
      const upload = await postJson("/collection-runs/manual-upload", {
        source_id: source.id,
        source_permission_id: permission.id,
        approval_id: null,
        filename,
        mime_type: "text/plain",
        content_base64: textToBase64(text),
        metadata_json: {
          submitted_from: "guided_add_data_manual_text",
          title: value("guided_text_title") || null
        }
      });
      const summary = upload?.summary_json || {};
      const workItemId = summary.normalization_work_item_id || "not returned";
      const artifactIds = Array.isArray(summary.raw_artifact_ids) ? summary.raw_artifact_ids.join(", ") : "not returned";
      writeResult(
        "Manual text submitted",
        "IGY6 accepted the UTF-8 text and queued normalization work for background processing.",
        ["Open Work and look for the work item below.", "When the work item completes, open Results to inspect documents, chunks, and evidence.", "Use Ask over evidence after results appear."],
        { source: { name: source.name, type: source.source_type, id: source.id }, upload },
        [
          { label: "source", value: source.id },
          { label: "collection run", value: upload?.id || "not returned" },
          { label: "work item", value: workItemId },
          { label: "work type", value: "collection_normalization" },
          { label: "raw artifact", value: artifactIds },
          { label: "current status", value: "queued, then running, then completed when normalization finishes" }
        ]
      );
    } catch (error) {
      writeResult(
        "Submission failed",
        String(error),
        ["Check that the local API is running and the selected source is enabled.", "Use Advanced only for low-level route diagnostics if this guided flow cannot continue."]
      );
    } finally {
      setBusy(false);
    }
  });
})();
`;

  return (
    <section className="guidedManualText" data-guided-manual-upload data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Manual UTF-8 text only.</strong>
        <span>This guided path accepts pasted text. It does not parse PDF, images, audio, video, screenshots, or web pages.</span>
      </div>
      {sources.error ? <p className="errorText">Source list could not be loaded: {sources.error}</p> : null}
      <form className="guidedManualForm" data-guided-manual-form>
        <label>
          <span>Use source</span>
          <select name="guided_source_choice" defaultValue="new">
            <option value="new">Create a new manual text source</option>
            {manualSources.map((source, index) => (
              <option value={index} key={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <div className="guidedManualNewSource" data-new-source-fields>
          <label>
            <span>Source name</span>
            <input name="guided_source_name" placeholder="Router Troubleshooting Notes" />
          </label>
          <label>
            <span>Description</span>
            <input name="guided_source_description" placeholder="Pasted notes copied from my local troubleshooting log" />
          </label>
          <label>
            <span>Sensitivity</span>
            <select name="guided_sensitivity" defaultValue="internal">
              <option value="public">public</option>
              <option value="internal">internal</option>
              <option value="sensitive">sensitive</option>
              <option value="secret">secret</option>
            </select>
          </label>
          <label className="checkLine">
            <input name="guided_approval_required" type="checkbox" />
            Require approval before this source can collect text
          </label>
        </div>
        <p className="actionHint" data-guided-approval-hint />
        <label>
          <span>Text title or filename</span>
          <input name="guided_text_title" defaultValue="manual-note.txt" />
        </label>
        <label>
          <span>Authorized text</span>
          <textarea name="guided_text" rows={8} placeholder="Paste authorized UTF-8 text here." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-guided-manual-submit>Submit manual text</button>
          <span>Next: Work for processing, Results for evidence.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-guided-manual-result>
        <strong>Ready</strong>
        <span>Create or select a manual source, paste text, and submit. Raw IDs stay in Advanced.</span>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: guided route response details</summary>
        <pre data-guided-manual-debug />
      </details>
      <script type="application/json" data-guided-manual-sources-json dangerouslySetInnerHTML={{ __html: manualSourcesJson }} />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function ConversationHistoryImport({ sources }: { sources: ApiResult<SourceRecord[]> }) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const conversationSources = sources.data
    .filter((source) => source.enabled && source.source_type === "conversation_history")
    .map((source) => ({
      id: source.id,
      name: source.name,
      location: source.location,
      sensitivity: source.sensitivity,
      permissions: source.permissions ?? [],
    }));
  const conversationSourcesJson = JSON.stringify(conversationSources).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-conversation-history-import]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const sourceData = JSON.parse(root.querySelector("[data-conversation-history-sources-json]")?.textContent || "[]");
  const result = root.querySelector("[data-conversation-history-result]");
  const debug = root.querySelector("[data-conversation-history-debug]");
  const submit = root.querySelector("[data-conversation-history-submit]");
  const sourceSelect = root.querySelector("[name='conversation_source_choice']");
  const newSourceFields = root.querySelector("[data-conversation-new-source-fields]");
  const approvalHint = root.querySelector("[data-conversation-approval-hint]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const writeResult = (state, message, nextSteps, payload, details) => {
    if (result) {
      result.innerHTML = "";
      const title = document.createElement("strong");
      title.textContent = state;
      const body = document.createElement("span");
      body.textContent = message;
      result.append(title, body);
      if (details?.length) {
        const detailList = document.createElement("dl");
        detailList.setAttribute("data-conversation-history-work-status", "");
        details.forEach((detail) => {
          const term = document.createElement("dt");
          term.textContent = detail.label;
          const description = document.createElement("dd");
          description.textContent = detail.value;
          detailList.append(term, description);
        });
        result.appendChild(detailList);
      }
      if (nextSteps?.length) {
        const list = document.createElement("ul");
        nextSteps.forEach((step) => {
          const item = document.createElement("li");
          item.textContent = step;
          list.appendChild(item);
        });
        result.appendChild(list);
      }
    }
    if (debug) debug.textContent = payload ? JSON.stringify(payload, null, 2) : "";
  };
  const setBusy = (busy) => {
    if (submit) {
      submit.disabled = busy;
      submit.textContent = busy ? "Importing..." : "Import conversation text";
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => {
      binary += String.fromCharCode(byte);
    });
    return btoa(binary);
  };
  const safeFilename = (filename) => {
    const cleaned = (filename || "conversation-history.txt").replace(/[^A-Za-z0-9._ -]/g, "-").trim();
    return cleaned || "conversation-history.txt";
  };
  const permissionFor = (source) => (source.permissions || []).find((permission) => {
    const operations = permission.allowed_operations || [];
    return operations.includes("collect") || operations.includes("read");
  });
  const selectedSource = () => {
    if (sourceSelect?.value === "new") return null;
    const index = Number(sourceSelect?.value || -1);
    return Number.isInteger(index) ? sourceData[index] : null;
  };
  const refreshSourceHints = () => {
    const source = selectedSource();
    if (newSourceFields) newSourceFields.hidden = Boolean(source);
    if (!approvalHint) return;
    if (!source) {
      approvalHint.textContent = checked("conversation_approval_required")
        ? "This new conversation source will request approval first. Text will not be collected until an approval is approved."
        : "This new conversation source can collect pasted UTF-8 text immediately under the created permission.";
      return;
    }
    const permission = permissionFor(source);
    if (!permission) {
      approvalHint.textContent = "This conversation source has no collect/read permission visible to the guided flow. Use Advanced for diagnostics.";
      return;
    }
    approvalHint.textContent = permission.approval_required
      ? "This conversation source requires approval before collection. Submitting will create an approval request and stop in pending state."
      : "This conversation source permission allows immediate local text import.";
  };
  sourceSelect?.addEventListener("change", refreshSourceHints);
  root.querySelector("[name='conversation_approval_required']")?.addEventListener("change", refreshSourceHints);
  refreshSourceHints();

  root.querySelector("[data-conversation-history-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = value("conversation_text");
    if (!text) {
      writeResult("Text required", "Paste authorized UTF-8 conversation/history text before importing.", ["This MVP does not import browser, account, connector, binary, image, audio, video, or external service data."]);
      return;
    }
    setBusy(true);
    try {
      let source = selectedSource();
      let permission = source ? permissionFor(source) : null;
      const conversationTitle = value("conversation_title") || "Conversation History";
      if (!source) {
        source = await postJson("/sources", {
          name: value("conversation_source_name") || conversationTitle,
          source_type: "conversation_history",
          location: value("conversation_context_note") || "Manual local conversation history import",
          sensitivity: value("conversation_sensitivity") || "internal",
          metadata_json: {
            created_from: "conversation_history_import_mvp",
            import_path: "manual_local_utf8_paste",
            manual_local_import_only: true
          },
          permission: {
            scope_json: {
              path: "manual_conversation_history",
              entered_from: "Add Data conversation history import",
              import_type: "manual_utf8_paste"
            },
            allowed_operations: ["dry_run", "read", "collect"],
            external_model_policy: "blocked",
            approval_required: checked("conversation_approval_required")
          }
        });
        permission = permissionFor(source);
      }
      if (!source?.id || !permission?.id) {
        throw new Error("No source permission was available for conversation history import.");
      }
      const filename = safeFilename(conversationTitle + ".txt");
      const metadata = {
        submitted_from: "conversation_history_import_mvp",
        title: conversationTitle,
        conversation_title: conversationTitle,
        conversation_date_range: value("conversation_date_range") || null,
        participants: value("conversation_participants") || null,
        context_note: value("conversation_context_note") || null,
        contains_corrections: checked("conversation_contains_corrections"),
        contains_decisions: checked("conversation_contains_decisions"),
        contains_instructions_preferences: checked("conversation_contains_instructions_preferences"),
        manual_local_import_only: true,
        browser_account_connector_import: false,
        binary_media_import: false
      };
      if (permission.approval_required) {
        const approval = await postJson("/approvals", {
          request_type: "manual_upload_collection",
          request_payload_json: {
            source_id: source.id,
            source_permission_id: permission.id,
            operation: "manual_upload_collection",
            source_type: "conversation_history",
            filename,
            metadata_json: metadata
          }
        });
        writeResult(
          "Approval pending",
          "IGY6 created the conversation history source context and requested collection approval. The pasted text was not uploaded because this permission requires an approved approval record.",
          ["Open Settings to review pending approvals.", "After approval, use the Advanced route console with the approved approval record for this low-level collection path.", "Processing status appears in Work after collection, and evidence appears in Results."],
          { source: { name: source.name, type: source.source_type }, permission: { approval_required: permission.approval_required }, approval }
        );
        return;
      }
      const upload = await postJson("/collection-runs/manual-upload", {
        source_id: source.id,
        source_permission_id: permission.id,
        approval_id: null,
        filename,
        mime_type: "text/plain",
        content_base64: textToBase64(text),
        metadata_json: metadata
      });
      const summary = upload?.summary_json || {};
      const workItemId = summary.normalization_work_item_id || "not returned";
      const artifactIds = Array.isArray(summary.raw_artifact_ids) ? summary.raw_artifact_ids.join(", ") : "not returned";
      writeResult(
        "Conversation history submitted",
        "IGY6 accepted the pasted UTF-8 conversation text and queued normalization work for local evidence processing.",
        ["Open Work and look for the work item below.", "When the work item completes, open Results to inspect documents, chunks, and evidence.", "Use Ask over evidence after results appear."],
        { source: { name: source.name, type: source.source_type, id: source.id }, upload },
        [
          { label: "source", value: source.id },
          { label: "source type", value: source.source_type },
          { label: "collection run", value: upload?.id || "not returned" },
          { label: "work item", value: workItemId },
          { label: "work type", value: "collection_normalization" },
          { label: "raw artifact", value: artifactIds },
          { label: "current status", value: "queued, then running, then completed when normalization finishes" }
        ]
      );
    } catch (error) {
      writeResult(
        "Import failed",
        String(error),
        ["Check that the local API is running and the selected conversation source is enabled.", "Use Advanced only for low-level route diagnostics if this guided flow cannot continue."]
      );
    } finally {
      setBusy(false);
    }
  });
})();
`;

  return (
    <section className="guidedManualText" data-conversation-history-import data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>Conversation history import MVP.</strong>
        <span>Manual local UTF-8 paste only. Browser extraction, account import, connectors, external service collection, and binary/media import are planned future work, not part of this DIFF.</span>
      </div>
      {sources.error ? <p className="errorText">Source list could not be loaded: {sources.error}</p> : null}
      <form className="guidedManualForm" data-conversation-history-form>
        <label>
          <span>Conversation source</span>
          <select name="conversation_source_choice" defaultValue="new">
            <option value="new">Create a new conversation history source</option>
            {conversationSources.map((source, index) => (
              <option value={index} key={source.id}>{source.name}</option>
            ))}
          </select>
        </label>
        <div className="guidedManualNewSource" data-conversation-new-source-fields>
          <label>
            <span>Source name</span>
            <input name="conversation_source_name" placeholder="Chat History Import" />
          </label>
          <label>
            <span>Sensitivity</span>
            <select name="conversation_sensitivity" defaultValue="internal">
              <option value="public">public</option>
              <option value="internal">internal</option>
              <option value="sensitive">sensitive</option>
              <option value="secret">secret</option>
            </select>
          </label>
          <label className="checkLine">
            <input name="conversation_approval_required" type="checkbox" />
            Require approval before this conversation source can collect text
          </label>
        </div>
        <p className="actionHint" data-conversation-approval-hint />
        <label>
          <span>Conversation title</span>
          <input name="conversation_title" placeholder="Support chat about router setup" />
        </label>
        <label>
          <span>Date/time range if known</span>
          <input name="conversation_date_range" placeholder="2026-05-01 to 2026-05-03" />
        </label>
        <label>
          <span>Participants or roles</span>
          <input name="conversation_participants" placeholder="me, support agent, project lead" />
        </label>
        <label>
          <span>Purpose or context note</span>
          <textarea name="conversation_context_note" rows={2} placeholder="Why this conversation matters or what it was about." />
        </label>
        <div className="checkGrid">
          <label className="checkLine">
            <input name="conversation_contains_corrections" type="checkbox" />
            Contains corrections
          </label>
          <label className="checkLine">
            <input name="conversation_contains_decisions" type="checkbox" />
            Contains decisions
          </label>
          <label className="checkLine">
            <input name="conversation_contains_instructions_preferences" type="checkbox" />
            Contains instructions or preferences
          </label>
        </div>
        <label>
          <span>Conversation/history text</span>
          <textarea name="conversation_text" rows={10} placeholder="Paste authorized UTF-8 conversation or history text here." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-conversation-history-submit>Import conversation text</button>
          <span>Next: Work for processing, Results for evidence. No account or browser access is used.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-conversation-history-result>
        <strong>Ready</strong>
        <span>Create or select a conversation source, paste authorized text, and import it locally.</span>
      </div>
      <details className="advancedPanel">
        <summary>Advanced: conversation import route response details</summary>
        <pre data-conversation-history-debug />
      </details>
      <script type="application/json" data-conversation-history-sources-json dangerouslySetInnerHTML={{ __html: conversationSourcesJson }} />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function SourceTrustSensitivityManagement({
  sources,
  collectionRuns,
  documents,
  evidenceItems
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const reviewableSources = sources.data.slice(0, 12);
  const sourceReviewRows = reviewableSources.map((source) => {
    const runCount = collectionRuns.data.filter((run) => run.source_id === source.id).length;
    const documentCount = documents.data.filter((document) => document.source_id === source.id).length;
    const evidenceCount = evidenceItems.data.filter((item) => item.source_id === source.id).length;
    return {
      id: source.id,
      name: source.name,
      source_type: source.source_type,
      sensitivity: source.sensitivity,
      trust_level: source.trust_level,
      enabled: source.enabled,
      updated_at: source.updated_at ?? null,
      run_count: runCount,
      document_count: documentCount,
      evidence_count: evidenceCount
    };
  });
  const sourceReviewJson = JSON.stringify(sourceReviewRows).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-source-review-management]");
  if (!root) return;
  const sources = JSON.parse(root.querySelector("[data-source-review-json]")?.textContent || "[]");
  const form = root.querySelector("[data-source-review-form]");
  const result = root.querySelector("[data-source-review-result]");
  const sourceSelect = root.querySelector("[name='source_review_source']");
  const stateSelect = root.querySelector("[name='source_review_state']");
  const sensitivitySelect = root.querySelector("[name='source_review_sensitivity']");
  const enabledInput = root.querySelector("[name='source_review_enabled']");
  const sourceSummary = root.querySelector("[data-source-review-selected]");
  const stateToTrust = (state) => state === "review_needed" ? "review_needed" : state;
  const selectedSource = () => sources.find((source) => source.id === sourceSelect?.value) || null;
  const stateForSource = (source) => {
    const current = source?.trust_level || "review_needed";
    return ["trusted", "noisy", "sensitive", "disabled", "review_needed"].includes(current) ? current : "review_needed";
  };
  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-source-review-status", "");
      [["source", payload.id], ["trust", payload.trust_level], ["sensitivity", payload.sensitivity], ["enabled", String(payload.enabled)]].forEach(([label, value]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = value || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const refreshSelected = () => {
    const source = selectedSource();
    if (!source) {
      if (sourceSummary) sourceSummary.textContent = "No source is available for review.";
      return;
    }
    if (stateSelect) stateSelect.value = stateForSource(source);
    if (sensitivitySelect) sensitivitySelect.value = source.sensitivity || "internal";
    if (enabledInput) enabledInput.checked = Boolean(source.enabled);
    if (sourceSummary) {
      sourceSummary.textContent = source.name + " has " + source.evidence_count + " evidence item(s), " + source.document_count + " document(s), and " + source.run_count + " collection run(s). Existing evidence stays visible after review updates.";
    }
  };
  stateSelect?.addEventListener("change", () => {
    if (enabledInput && stateSelect.value === "disabled") enabledInput.checked = false;
    if (sensitivitySelect && stateSelect.value === "sensitive") sensitivitySelect.value = "sensitive";
  });
  sourceSelect?.addEventListener("change", refreshSelected);
  refreshSelected();
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const source = selectedSource();
    if (!source) {
      show("No source selected", "Register a source before reviewing trust and sensitivity.");
      return;
    }
    const selectedState = stateSelect?.value || "review_needed";
    const enabled = selectedState === "disabled" ? false : Boolean(enabledInput?.checked);
    try {
      const response = await fetch("/api/sources/" + encodeURIComponent(source.id) + "/review-state", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          trust_level: stateToTrust(selectedState),
          sensitivity: sensitivitySelect?.value || "internal",
          enabled,
          review_note: root.querySelector("[name='source_review_note']")?.value?.trim() || null,
          actor_id: "local-owner"
        })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      show("Source review saved", "IGY6 updated the source record and audit trail. Reload to see refreshed source lists. Existing evidence was not hidden or deleted.", payload);
    } catch (error) {
      show("Source review failed", String(error));
    }
  });
})();
`;

  return (
    <section className="guidedManualText sourceReviewManagement" data-source-review-management>
      <div className="guidedManualNotice">
        <strong>Source trust and sensitivity</strong>
        <span>Review source state for future use. This does not delete sources, rewrite historical evidence, or silently hide evidence from Results.</span>
      </div>
      {sources.error ? <p className="errorText">Source state could not be loaded: {sources.error}</p> : null}
      <section className="stack" aria-label="Source review summary">
        {sourceReviewRows.slice(0, 6).map((source) => (
          <article className="item evidenceItem" key={source.id} data-source-review-item>
            <div>
              <strong>{source.name}</strong>
              <span>{source.source_type} · {source.sensitivity}</span>
              <span>{source.evidence_count} evidence item(s), {source.document_count} document(s), {source.run_count} collection run(s)</span>
            </div>
            <div>
              <StatusPill state={source.enabled ? "enabled" : "disabled"} />
              <StatusPill state={source.trust_level || "review_needed"} />
            </div>
          </article>
        ))}
      </section>
      {sourceReviewRows.length === 0 ? <EmptyState label="No sources are available for trust or sensitivity review yet." /> : null}
      <form className="guidedManualForm" data-source-review-form>
        <label>
          <span>Source</span>
          <select name="source_review_source" disabled={sourceReviewRows.length === 0}>
            {sourceReviewRows.map((source) => (
              <option key={source.id} value={source.id}>{source.name} · {source.source_type}</option>
            ))}
          </select>
        </label>
        <p className="actionHint" data-source-review-selected>
          {sourceReviewRows.length > 0 ? "Choose a source to review linked evidence counts." : "Register a source before reviewing trust state."}
        </p>
        <label>
          <span>Trust state</span>
          <select name="source_review_state" defaultValue="review_needed" disabled={sourceReviewRows.length === 0}>
            <option value="trusted">trusted</option>
            <option value="noisy">noisy</option>
            <option value="sensitive">sensitive</option>
            <option value="disabled">disabled</option>
            <option value="review_needed">review-needed</option>
          </select>
        </label>
        <label>
          <span>Sensitivity label</span>
          <select name="source_review_sensitivity" defaultValue="internal" disabled={sourceReviewRows.length === 0}>
            <option value="public">public</option>
            <option value="internal">internal</option>
            <option value="sensitive">sensitive</option>
            <option value="secret">secret</option>
          </select>
        </label>
        <label className="checkLine">
          <input name="source_review_enabled" type="checkbox" defaultChecked disabled={sourceReviewRows.length === 0} />
          Enabled for future collection workflows
        </label>
        <label>
          <span>Review note</span>
          <textarea name="source_review_note" rows={2} placeholder="Optional note explaining the review decision." disabled={sourceReviewRows.length === 0} />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={sourceReviewRows.length === 0}>Save source review</button>
          <span>Updates source metadata and audit records only; retrieval ranking and policy enforcement are not changed here.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-source-review-result>
        <strong>{sourceReviewRows.length > 0 ? "Ready for review" : "No source to review"}</strong>
        <span>{sourceReviewRows.length > 0 ? "Choose a source and save a real state update." : "Create a source first in Add Data."}</span>
      </div>
      <script type="application/json" data-source-review-json dangerouslySetInnerHTML={{ __html: sourceReviewJson }} />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function EvidenceCorrectionSupersessionWorkflow({
  evidenceItems
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const reviewableEvidence = evidenceItems.data.slice(0, 12);
  const evidenceRows = reviewableEvidence.map((item) => ({
    id: item.id,
    evidence_type: item.evidence_type,
    statement_preview: excerpt(item.statement, 96),
    review_state: evidenceReviewState(item),
    correction_note: evidenceReviewNote(item),
    source_id: item.source_id,
    document_id: item.document_id,
    chunk_id: item.chunk_id
  }));
  const evidenceRowsJson = JSON.stringify(evidenceRows).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-evidence-correction-workflow]");
  if (!root) return;
  const evidence = JSON.parse(root.querySelector("[data-evidence-correction-json]")?.textContent || "[]");
  const form = root.querySelector("[data-evidence-correction-form]");
  const evidenceSelect = root.querySelector("[name='evidence_correction_target']");
  const stateSelect = root.querySelector("[name='evidence_correction_state']");
  const supersedingSelect = root.querySelector("[name='evidence_superseding_id']");
  const result = root.querySelector("[data-evidence-correction-result]");
  const selectedEvidence = () => evidence.find((item) => item.id === evidenceSelect?.value) || null;
  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-evidence-correction-status", "");
      const reviewState = payload.metadata_json?.review_state || {};
      [["evidence", payload.id], ["state", reviewState.state], ["supersedes", reviewState.superseding_evidence_item_id || "not linked"], ["history", "original evidence preserved"]].forEach(([label, value]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = value || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const refreshSupersedingOptions = () => {
    const selected = selectedEvidence();
    if (!supersedingSelect || !selected) return;
    Array.from(supersedingSelect.options).forEach((option) => {
      option.disabled = option.value === selected.id;
    });
    if (supersedingSelect.value === selected.id) supersedingSelect.value = "";
  };
  evidenceSelect?.addEventListener("change", refreshSupersedingOptions);
  refreshSupersedingOptions();
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const selected = selectedEvidence();
    if (!selected) {
      show("No evidence selected", "Process text into evidence before recording correction state.");
      return;
    }
    try {
      const response = await fetch("/api/evidence/items/" + encodeURIComponent(selected.id) + "/review-state", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          review_state: stateSelect?.value || "needs_correction",
          correction_note: root.querySelector("[name='evidence_correction_note']")?.value?.trim() || null,
          superseding_evidence_item_id: supersedingSelect?.value || null,
          actor_id: "local-owner"
        })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      show("Evidence review saved", "IGY6 recorded review metadata and audit history. Original evidence, source, document, chunk, and artifact records were not deleted or rewritten.", payload);
    } catch (error) {
      show("Evidence review failed", String(error));
    }
  });
})();
`;

  return (
    <section className="guidedManualText evidenceCorrectionWorkflow" data-evidence-correction-workflow>
      <div className="guidedManualNotice">
        <strong>Evidence correction and supersession</strong>
        <span>Mark evidence review state without deleting, rewriting, or hiding the original evidence. Retrieval ranking and filtering are not changed here.</span>
      </div>
      {evidenceItems.error ? <p className="errorText">Evidence items could not be loaded: {evidenceItems.error}</p> : null}
      <section className="stack" aria-label="Evidence correction summary">
        {evidenceRows.slice(0, 6).map((item) => (
          <article className="item evidenceItem" key={item.id} data-evidence-correction-item>
            <div>
              <strong>{item.evidence_type}</strong>
              <span>{item.statement_preview}</span>
              <span>{item.correction_note ?? "No correction note recorded."}</span>
            </div>
            <div>
              <StatusPill state={item.review_state} />
              <span>{item.chunk_id ? "chunk-linked" : item.document_id ? "document-linked" : item.source_id ? "source-linked" : "lineage missing"}</span>
            </div>
          </article>
        ))}
      </section>
      {evidenceRows.length === 0 ? <EmptyState label="No evidence items are available for correction review yet." /> : null}
      <form className="guidedManualForm" data-evidence-correction-form>
        <label>
          <span>Evidence item</span>
          <select name="evidence_correction_target" disabled={evidenceRows.length === 0}>
            {evidenceRows.map((item) => (
              <option key={item.id} value={item.id}>{item.evidence_type} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Review state</span>
          <select name="evidence_correction_state" defaultValue="needs_correction" disabled={evidenceRows.length === 0}>
            <option value="needs_correction">needs correction</option>
            <option value="corrected">corrected</option>
            <option value="superseded">superseded</option>
            <option value="disputed">disputed</option>
            <option value="verified">verified</option>
          </select>
        </label>
        <label>
          <span>Superseding evidence</span>
          <select name="evidence_superseding_id" defaultValue="" disabled={evidenceRows.length < 2}>
            <option value="">No superseding evidence link</option>
            {evidenceRows.map((item) => (
              <option key={item.id} value={item.id}>{item.evidence_type} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Correction note</span>
          <textarea name="evidence_correction_note" rows={2} placeholder="Short note explaining the correction, dispute, or supersession." disabled={evidenceRows.length === 0} />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={evidenceRows.length === 0}>Save evidence review</button>
          <span>Records additive review metadata only. Existing source, artifact, document, chunk, and evidence history stays visible.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-evidence-correction-result>
        <strong>{evidenceRows.length > 0 ? "Ready for evidence review" : "No evidence to review"}</strong>
        <span>{evidenceRows.length > 0 ? "Choose an evidence item and save a real correction state." : "Add and process supported text before reviewing evidence correction state."}</span>
      </div>
      <script type="application/json" data-evidence-correction-json dangerouslySetInnerHTML={{ __html: evidenceRowsJson }} />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function BasicReportWorkflow({
  reports,
  evidenceCount,
  documentCount,
  chunkCount
}: {
  reports: ApiResult<ReportRecord[]>;
  evidenceCount: number;
  documentCount: number;
  chunkCount: number;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const reportReady = evidenceCount > 0 || documentCount > 0 || chunkCount > 0;
  const script = `
(() => {
  const root = document.querySelector("[data-basic-report-workflow]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const form = root.querySelector("[data-basic-report-form]");
  const result = root.querySelector("[data-basic-report-result]");
  const submit = root.querySelector("[data-basic-report-submit]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const show = (state, message, payload) => {
    if (result) {
      result.innerHTML = "";
      const title = document.createElement("strong");
      title.textContent = state;
      const body = document.createElement("span");
      body.textContent = message;
      result.append(title, body);
      if (payload) {
        const details = document.createElement("dl");
        details.setAttribute("data-basic-report-status", "");
        [
          ["report", payload.id],
          ["status", payload.status],
          ["type", payload.report_type],
          ["artifact", payload.artifact_path || "not rendered"]
        ].forEach(([label, detail]) => {
          const term = document.createElement("dt");
          term.textContent = label;
          const description = document.createElement("dd");
          description.textContent = detail || "not returned";
          details.append(term, description);
        });
        result.appendChild(details);
      }
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (submit) {
      submit.disabled = true;
      submit.textContent = "Creating...";
    }
    try {
      const report = await postJson("/reports", {
        title: value("basic_report_title") || "Evidence inventory report",
        report_type: value("basic_report_type") || "summary",
        status: "requested",
        metadata_json: {
          created_from: "results_basic_report_workflow",
          evidence_items_visible: Number(root.getAttribute("data-evidence-count") || 0),
          documents_visible: Number(root.getAttribute("data-document-count") || 0),
          chunks_visible: Number(root.getAttribute("data-chunk-count") || 0)
        }
      });
      let finalReport = report;
      if (checked("basic_report_render")) {
        finalReport = await postJson("/reports/" + report.id + "/render", {
          notes: value("basic_report_notes") || null
        });
      }
      show(
        finalReport.status === "ready" ? "Report ready" : "Report created",
        finalReport.status === "ready"
          ? "IGY6 rendered a local markdown metadata report artifact."
          : "IGY6 created the report metadata record. Render it from Advanced or rerun this workflow with rendering enabled.",
        finalReport
      );
    } catch (error) {
      show("Report workflow failed", String(error));
    } finally {
      if (submit) {
        submit.disabled = false;
        submit.textContent = "Create report";
      }
    }
  });
})();
`;

  return (
    <section
      className="guidedManualText"
      data-basic-report-workflow
      data-api-base-url={browserApiBaseUrl}
      data-evidence-count={evidenceCount}
      data-document-count={documentCount}
      data-chunk-count={chunkCount}
    >
      <div className="guidedManualNotice">
        <strong>Basic report workflow</strong>
        <span>
          Current reports are local markdown metadata summaries. They preserve boundaries by counting stored records and do not read raw artifact contents or call external models.
        </span>
      </div>
      <form className="guidedManualForm" data-basic-report-form>
        <label>
          <span>Report title</span>
          <input name="basic_report_title" defaultValue="Evidence inventory report" />
        </label>
        <label>
          <span>Report type</span>
          <select name="basic_report_type" defaultValue="summary">
            <option value="summary">summary</option>
            <option value="incident_review">incident_review</option>
            <option value="status_brief">status_brief</option>
          </select>
        </label>
        <label>
          <span>Render notes</span>
          <textarea name="basic_report_notes" rows={2} placeholder="Optional local note for the rendered markdown report." />
        </label>
        <label className="checkLine">
          <input name="basic_report_render" type="checkbox" defaultChecked /> Render markdown artifact now
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-basic-report-submit disabled={!reportReady}>Create report</button>
          <span>{reportReady ? "Uses existing /reports and /reports/:id/render routes." : "Add supported text and wait for evidence before creating a useful report."}</span>
        </div>
      </form>
      <div className="guidedManualResult" data-basic-report-result>
        <strong>{reports.data.length > 0 ? "Reports are available" : "No reports yet"}</strong>
        <span>{reports.data.length > 0 ? "Create a new metadata report or review recent reports below." : "Create a report after evidence exists, or keep using Ask over evidence."}</span>
      </div>
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function EvidenceFeedbackWorkflow({
  evidenceItems,
  evidenceAnswers,
  reports,
  workItems,
  feedback,
  outcomes,
  improvements
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  reports: ApiResult<ReportRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const feedbackTargets = [
    ...evidenceItems.data.slice(0, 6).map((item) => ({
      type: "evidence_item",
      id: item.id,
      label: `Evidence item ${item.id}`
    })),
    ...evidenceAnswers.data.slice(0, 4).map((answer) => ({
      type: "evidence_answer",
      id: answer.id,
      label: `Answer record ${answer.id}`
    })),
    ...reports.data.slice(0, 3).map((report) => ({
      type: "report",
      id: report.id,
      label: `Report ${report.id}`
    })),
    ...workItems.data.slice(0, 3).map((workItem) => ({
      type: "work_item",
      id: workItem.id,
      label: `Work item ${workItem.id}`
    }))
  ];
  const outcomeTargets = [
    ...reports.data.slice(0, 4).map((report) => ({
      type: "report",
      id: report.id,
      label: `Report ${report.id}`
    })),
    ...workItems.data.slice(0, 4).map((workItem) => ({
      type: "work_item",
      id: workItem.id,
      label: `Work item ${workItem.id}`
    }))
  ];
  const defaultEvidenceId = evidenceItems.data[0]?.id ?? "";
  const improvementFeedbackLabels = new Set(["wrong", "not_useful", "incomplete", "rejected"]);
  const improvementOutcomeStatuses = new Set(["wrong", "not_useful", "partial", "inconclusive"]);
  const feedbackSignals = feedback.data.filter((event) => event.target_type !== "source" && improvementFeedbackLabels.has(event.label));
  const outcomeSignals = outcomes.data.filter((outcome) => improvementOutcomeStatuses.has(outcome.outcome_status));
  const reviewSignals = [
    ...feedbackSignals.slice(0, 6).map((event) => ({
      kind: "feedback",
      id: event.id,
      targetType: event.target_type,
      targetId: event.target_id,
      label: event.label,
      note: event.note ?? "",
      existingImprovement: improvements.data.find((item) => item.metadata_json?.feedback_id === event.id)
    })),
    ...outcomeSignals.slice(0, 6).map((outcome) => ({
      kind: "outcome",
      id: outcome.id,
      targetType: outcome.target_type,
      targetId: outcome.target_id,
      label: outcome.outcome_status,
      note: outcome.summary ?? "",
      existingImprovement: improvements.data.find((item) => item.metadata_json?.outcome_id === outcome.id)
    }))
  ];
  const script = `
(() => {
  const root = document.querySelector("[data-evidence-feedback-workflow]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
	  const result = root.querySelector("[data-evidence-feedback-result]");
	  const improvementResult = root.querySelector("[data-improvement-review-result]");
	  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
	  const selected = (name) => {
    const option = root.querySelector("[name='" + name + "']")?.selectedOptions?.[0];
    return {
      id: option?.value || "",
      type: option?.getAttribute("data-target-type") || ""
	    };
	  };
	  const selectedSignal = () => {
	    const option = root.querySelector("[name='improvement_signal']")?.selectedOptions?.[0];
	    return {
	      id: option?.value || "",
	      kind: option?.getAttribute("data-signal-kind") || "",
	      targetType: option?.getAttribute("data-target-type") || "",
	      targetId: option?.getAttribute("data-target-id") || "",
	      label: option?.getAttribute("data-signal-label") || ""
	    };
	  };
	  const targetAreaFor = (targetType) => {
	    if (targetType === "document") return "parsing";
	    if (targetType === "evidence_item") return "retrieval";
	    if (targetType === "prediction") return "prediction";
	    if (targetType === "report") return "reporting";
	    if (targetType === "work_item") return "safety";
	    return "reasoning";
	  };
	  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-feedback-outcome-status", "");
      [
        ["record", payload.id],
        ["target", (payload.target_type || "") + " " + (payload.target_id || "")],
        ["label", payload.label || payload.outcome_status || "recorded"]
      ].forEach(([label, detail]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = detail || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
	    }
	  };
	  const showImprovement = (state, message, payload) => {
	    if (!improvementResult) return;
	    improvementResult.innerHTML = "";
	    const title = document.createElement("strong");
	    title.textContent = state;
	    const body = document.createElement("span");
	    body.textContent = message;
	    improvementResult.append(title, body);
	    if (payload) {
	      const details = document.createElement("dl");
	      details.setAttribute("data-improvement-review-status", "");
	      [
	        ["improvement", payload.id],
	        ["target area", payload.target_area],
	        ["status", payload.status],
	        ["priority", payload.priority]
	      ].forEach(([label, detail]) => {
	        const term = document.createElement("dt");
	        term.textContent = label;
	        const description = document.createElement("dd");
	        description.textContent = detail || "not returned";
	        details.append(term, description);
	      });
	      improvementResult.appendChild(details);
	    }
	  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  root.querySelector("[data-feedback-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const target = selected("feedback_target");
    try {
      const payload = await postJson("/feedback", {
        target_type: target.type,
        target_id: target.id,
        label: value("feedback_label"),
        note: value("feedback_note") || null,
        metadata_json: { created_from: "results_feedback_outcome_capture" }
      });
      show("Feedback recorded", "IGY6 persisted the review feedback.", payload);
    } catch (error) {
      show("Feedback failed", String(error));
    }
  });
	  root.querySelector("[data-outcome-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const target = selected("outcome_target");
    const evidenceIds = value("outcome_evidence_ids")
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
    try {
      const payload = await postJson("/outcomes", {
        target_type: target.type,
        target_id: target.id,
        outcome_status: value("outcome_status"),
        summary: value("outcome_summary") || null,
        evidence_ids: evidenceIds,
        metadata_json: { created_from: "results_feedback_outcome_capture" }
      });
      show("Outcome recorded", "IGY6 persisted the outcome and updated the supported target.", payload);
    } catch (error) {
	      show("Outcome failed", String(error));
	    }
	  });
	  root.querySelector("[data-improvement-review-form]")?.addEventListener("submit", async (event) => {
	    event.preventDefault();
	    const signal = selectedSignal();
	    if (!signal.id) {
	      showImprovement("No review signal", "Record weak feedback or an outcome before creating an improvement proposal.");
	      return;
	    }
	    try {
	      const payload = await postJson("/improvements", {
	        target_area: targetAreaFor(signal.targetType),
	        objective: value("improvement_objective") || ("Review " + signal.kind + " " + signal.label + " for " + signal.targetType + " " + signal.targetId + "."),
	        priority: value("improvement_priority") || "normal",
	        proposed_by_actor_id: "local-owner",
	        metadata_json: {
	          created_from: "results_improvement_review",
	          signal_kind: signal.kind,
	          signal_label: signal.label,
	          target_type: signal.targetType,
	          target_id: signal.targetId,
	          feedback_id: signal.kind === "feedback" ? signal.id : null,
	          outcome_id: signal.kind === "outcome" ? signal.id : null
	        }
	      });
	      showImprovement("Improvement proposed", "IGY6 created review metadata only. No method changed and no experiment ran.", payload);
	    } catch (error) {
	      showImprovement("Improvement proposal failed", String(error));
	    }
	  });
	})();
	`;

  return (
    <section
      className="guidedManualText"
      data-evidence-feedback-workflow
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="guidedManualNotice">
        <strong>Review outcome capture</strong>
        <span>
          Record feedback on retrieved evidence, saved answer records, or a supported report/work item outcome. Outcomes for answer records are not supported by the current outcome API.
        </span>
      </div>
      <form className="guidedManualForm" data-feedback-form>
        <label>
          <span>Feedback target</span>
          <select name="feedback_target" disabled={feedbackTargets.length === 0}>
            {feedbackTargets.map((target) => (
              <option key={`${target.type}:${target.id}`} value={target.id} data-target-type={target.type}>{target.label}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Feedback label</span>
          <select name="feedback_label" defaultValue="useful">
            <option value="useful">useful</option>
            <option value="verified">verified</option>
            <option value="incomplete">incomplete</option>
            <option value="wrong">wrong</option>
            <option value="not_useful">not_useful</option>
          </select>
        </label>
        <label>
          <span>Feedback note</span>
          <textarea name="feedback_note" rows={2} placeholder="Optional review note." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={feedbackTargets.length === 0}>Record feedback</button>
          <span>{feedbackTargets.length > 0 ? "Targets come from current evidence, saved answer records, reports, and work items." : "No supported feedback target is available yet."}</span>
        </div>
      </form>
      <form className="guidedManualForm" data-outcome-form>
        <label>
          <span>Outcome target</span>
          <select name="outcome_target" disabled={outcomeTargets.length === 0}>
            {outcomeTargets.map((target) => (
              <option key={`${target.type}:${target.id}`} value={target.id} data-target-type={target.type}>{target.label}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Outcome status</span>
          <select name="outcome_status" defaultValue="useful">
            <option value="useful">useful</option>
            <option value="correct">correct</option>
            <option value="partial">partial</option>
            <option value="wrong">wrong</option>
            <option value="not_useful">not_useful</option>
            <option value="inconclusive">inconclusive</option>
          </select>
        </label>
        <label>
          <span>Evidence ids</span>
          <input name="outcome_evidence_ids" defaultValue={defaultEvidenceId} placeholder="Optional comma-separated evidence ids." />
        </label>
        <label>
          <span>Outcome summary</span>
          <textarea name="outcome_summary" rows={2} placeholder="Optional outcome summary." />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={outcomeTargets.length === 0}>Record outcome</button>
          <span>{outcomeTargets.length > 0 ? "Outcomes are only offered for API-supported targets." : "No supported report or work item target is available yet."}</span>
        </div>
      </form>
	      <div className="guidedManualResult" data-evidence-feedback-result>
	        <strong>{feedback.data.length + outcomes.data.length > 0 ? "Review records exist" : "No review record selected"}</strong>
	        <span>{feedback.data.length + outcomes.data.length > 0 ? "Recent feedback and outcomes remain visible in Safety & Audit." : "Record feedback after reviewing retrieved evidence, or record an outcome when a supported target exists."}</span>
	      </div>
	      <section className="improvementReview" data-improvement-review>
	        <div className="guidedManualNotice">
	          <strong>Improvement review</strong>
	          <span>Weak feedback and unresolved outcomes can become proposed improvement items. This is review metadata only; IGY6 does not change methods or run experiments here.</span>
	        </div>
	        <div className="stack">
	          {reviewSignals.slice(0, 6).map((signal) => (
	            <article className="item evidenceItem" key={`${signal.kind}:${signal.id}`} data-improvement-signal>
	              <div>
	                <strong>{signal.kind} · {signal.label}</strong>
	                <span>{signal.targetType} {signal.targetId}</span>
	                <span>{signal.note || "No note recorded."}</span>
	              </div>
	              <div>
	                <StatusPill state={signal.existingImprovement ? "improvement-exists" : "needs-review"} />
	                <span>{signal.existingImprovement?.id ?? "No linked improvement item yet"}</span>
	              </div>
	            </article>
	          ))}
	        </div>
	        {reviewSignals.length === 0 ? <EmptyState label="No weak feedback or unresolved outcome signals are available yet." /> : null}
	        <form className="guidedManualForm" data-improvement-review-form>
	          <label>
	            <span>Review signal</span>
	            <select name="improvement_signal" disabled={reviewSignals.length === 0}>
	              {reviewSignals.map((signal) => (
	                <option
	                  key={`${signal.kind}:${signal.id}`}
	                  value={signal.id}
	                  data-signal-kind={signal.kind}
	                  data-target-type={signal.targetType}
	                  data-target-id={signal.targetId}
	                  data-signal-label={signal.label}
	                >
	                  {signal.kind} {signal.label} · {signal.targetType} {signal.targetId}
	                </option>
	              ))}
	            </select>
	          </label>
	          <label>
	            <span>Improvement objective</span>
	            <textarea name="improvement_objective" rows={2} placeholder="Review why this feedback/outcome was weak and define what should improve." />
	          </label>
	          <label>
	            <span>Priority</span>
	            <select name="improvement_priority" defaultValue="normal">
	              <option value="low">low</option>
	              <option value="normal">normal</option>
	              <option value="high">high</option>
	              <option value="urgent">urgent</option>
	            </select>
	          </label>
	          <div className="guidedManualActions">
	            <button type="submit" disabled={reviewSignals.length === 0}>Propose improvement item</button>
	            <span>{reviewSignals.length > 0 ? "Uses existing /improvements persistence." : "Record a weak signal before proposing improvement work."}</span>
	          </div>
	        </form>
	        <div className="guidedManualResult" data-improvement-review-result>
	          <strong>{improvements.data.length > 0 ? "Improvement items exist" : "No improvement item selected"}</strong>
	          <span>{improvements.data.length > 0 ? "Existing improvement records are listed in Method Review." : "Create a proposal only from a real review signal."}</span>
	        </div>
	      </section>
	      <script dangerouslySetInnerHTML={{ __html: script }} />
	    </section>
  );
}

function EvidenceAnswerHistory({
  evidenceAnswers,
  feedback
}: {
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
}) {
  const recentAnswers = evidenceAnswers.data.slice(0, 8);
  const feedbackForAnswer = (answerId: string) => feedback.data.filter((event) => event.target_type === "evidence_answer" && event.target_id === answerId);

  return (
    <section className="guidedManualText" data-evidence-answer-history>
      <div className="guidedManualNotice">
        <strong>Saved evidence answer records</strong>
        <span>Saved answer records preserve the original retrieval review and evidence identifiers. They do not rewrite evidence, hide superseded evidence, or create full chat memory.</span>
      </div>
      {evidenceAnswers.error ? <p className="errorText">Saved answer records could not be loaded: {evidenceAnswers.error}</p> : null}
      <div className="stack">
        {recentAnswers.map((answer) => {
          const evidenceIds = answer.evidence_item_ids ?? [];
          const labels = answer.safe_labels ?? [];
          const linkedFeedback = feedbackForAnswer(answer.id);
          return (
            <article className="item evidenceItem" key={answer.id}>
              <div>
                <strong>{answer.user_question}</strong>
                <span>{answer.answer_text ?? "Saved answer record without answer text."}</span>
                <span className="messageMeta">Evidence IDs: {evidenceIds.length > 0 ? evidenceIds.slice(0, 5).join(", ") : "none recorded"}</span>
                <span className="messageMeta">Trail: {labels.length > 0 ? labels.slice(0, 6).join(" · ") : "no safe trail labels recorded"}</span>
                <span className="messageMeta">Original evidence, documents, chunks, sources, and raw artifacts remain preserved.</span>
              </div>
              <div>
                <StatusPill state={answer.answer_status} />
                <span>{answer.retrieval_mode} · {answer.retrieval_count} hit(s)</span>
                <span>{answer.local_model_status ?? "local model status not recorded"}</span>
                <span>{formatDate(answer.created_at)}</span>
                <span>{linkedFeedback.length > 0 ? `${linkedFeedback.length} feedback record(s)` : "Feedback can target this answer record."}</span>
              </div>
            </article>
          );
        })}
      </div>
      {recentAnswers.length === 0 ? <EmptyState label="No saved answer records yet. Ask over evidence, then save the answer record." /> : null}
      <p className="messageMeta">Outcomes are not offered for answer records yet because the outcome API only validates reports, work items, predictions, recommendations, hypotheses, and patterns.</p>
    </section>
  );
}

function AgentTaskHistoryReview({
  taskPlans,
  workItems,
  approvals,
  feedback,
  outcomes,
  improvements
}: {
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
  workItems: ApiResult<WorkItemRecord[]>;
  approvals: ApiResult<ApprovalRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  improvements: ApiResult<ImprovementRecord[]>;
}) {
  const recentPlans = taskPlans.data.slice(0, 8);
  const metadataString = (metadata: Record<string, unknown> | null | undefined, key: string): string | null => {
    const value = metadata?.[key];
    return typeof value === "string" && value.trim() ? value : null;
  };
  const planApproval = (planId: string) => approvals.data.find((approval) => (
    approval.request_type === "agent_task_plan"
    && approval.request_payload_json?.task_plan_id === planId
  ));
  const linkedWorkItem = (plan: AgentTaskPlanRecord) => {
    const workItemId = metadataString(plan.metadata_json, "work_item_id");
    return workItemId ? workItems.data.find((item) => item.id === workItemId) ?? null : null;
  };
  const linkedFeedback = (workItemId: string | null) => workItemId
    ? feedback.data.find((item) => item.target_type === "work_item" && item.target_id === workItemId) ?? null
    : null;
  const linkedOutcome = (workItemId: string | null) => workItemId
    ? outcomes.data.find((item) => item.target_type === "work_item" && item.target_id === workItemId) ?? null
    : null;
  const linkedImprovement = (planId: string, workItemId: string | null) => improvements.data.find((item) => (
    item.metadata_json?.agent_task_plan_id === planId
    || (workItemId ? item.metadata_json?.work_item_id === workItemId : false)
  )) ?? null;
  const evidenceSummaryFor = (plan: AgentTaskPlanRecord): Record<string, unknown> | null => {
    const summary = plan.metadata_json?.evidence_summary;
    return summary && typeof summary === "object" ? summary as Record<string, unknown> : null;
  };

  return (
    <section className="panel workflowSection" data-agent-task-history-review>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Task history</p>
          <h2>Agent Task History And Outcomes</h2>
        </div>
        <StatusPill state={recentPlans.length > 0 ? "history-available" : "history-empty"} />
      </div>
      <p className="workflowLead">Review persisted plans, linked work, approvals, outcomes, and improvement records. Missing links are shown honestly; this surface does not create or execute work.</p>
      <section className="agentPlanner" aria-label="Recent agent task history">
        {recentPlans.length === 0 ? (
          <article className="agentPlannerCard">
            <strong>No task history yet</strong>
            <span>Saved agent task plans will appear here after the planner records them.</span>
            <em>empty</em>
          </article>
        ) : recentPlans.map((plan) => {
          const workItem = linkedWorkItem(plan);
          const workItemId = workItem?.id ?? metadataString(plan.metadata_json, "work_item_id");
          const approval = planApproval(plan.id);
          const outcome = linkedOutcome(workItemId);
          const feedbackRecord = linkedFeedback(workItemId);
          const improvement = linkedImprovement(plan.id, workItemId);
          const evidenceSummary = evidenceSummaryFor(plan);
          const evidenceStatus = typeof evidenceSummary?.answer_status === "string" ? evidenceSummary.answer_status : null;
          const evidenceCount = typeof evidenceSummary?.retrieved_count === "number" ? evidenceSummary.retrieved_count : null;
          const safeNextAction = plan.status === "converted_to_work"
            ? "Review the linked work item status before dispatch or outcome review."
            : plan.approval_required && approval?.status !== "approved"
              ? "Review or create a matching approval before creating work."
              : plan.next_safe_action;
          return (
            <article className="agentPlannerCard" key={plan.id} data-agent-task-history-item>
              <strong>{plan.user_request_summary}</strong>
              <span>{safeNextAction}</span>
              <em>{plan.status} · {plan.intent_category} · {formatDate(plan.created_at)}</em>
              <dl className="workStatusIds" aria-label={`Task history for ${plan.id}`}>
                <dt>plan</dt><dd>{plan.id}</dd>
                <dt>work item</dt><dd>{workItem ? `${workItem.id} · ${workItem.status}` : workItemId ?? "not linked"}</dd>
                <dt>approval</dt><dd>{approval ? `${approval.id} · ${approval.status}` : plan.approval_required ? "approval required, not linked" : "not required"}</dd>
                <dt>feedback</dt><dd>{feedbackRecord ? `${feedbackRecord.id} · ${feedbackRecord.label}` : "not linked"}</dd>
                <dt>outcome</dt><dd>{outcome ? `${outcome.id} · ${outcome.outcome_status}` : "not linked"}</dd>
                <dt>improvement</dt><dd>{improvement ? `${improvement.id} · ${improvement.status}` : "not linked"}</dd>
                <dt>evidence</dt><dd>{evidenceStatus ? `${evidenceStatus} · ${evidenceCount ?? 0} hit(s)` : "not checked"}</dd>
              </dl>
            </article>
          );
        })}
      </section>
    </section>
  );
}

function ImprovementExperimentReview({
  improvements,
  experiments
}: {
  improvements: ApiResult<ImprovementRecord[]>;
  experiments: ApiResult<ExperimentRecord[]>;
}) {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const recentImprovements = improvements.data.slice(0, 6);
  const recentExperiments = experiments.data.slice(0, 6);
  const script = `
(() => {
  const root = document.querySelector("[data-improvement-experiment-review]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const result = root.querySelector("[data-experiment-proposal-result]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-experiment-proposal-status", "");
      [
        ["experiment", payload.id],
        ["status", payload.status],
        ["improvement", payload.improvement_item_id || "not linked"],
        ["execution", "metadata only"]
      ].forEach(([label, detail]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = detail || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  root.querySelector("[data-experiment-proposal-form]")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const improvementId = value("experiment_improvement_id");
    if (!improvementId) {
      show("No improvement selected", "Create or select an improvement item before proposing experiment metadata.");
      return;
    }
    try {
      const payload = await postJson("/experiments", {
        improvement_item_id: improvementId,
        status: "planned",
        metrics_json: {
          proposed_success_metric: value("experiment_success_metric") || "manual review required"
        },
        artifacts_json: {},
        metadata_json: {
          created_from: "improvement_experiment_review",
          proposal_scope: value("experiment_scope") || "review only",
          execution_model: "not_started",
          autonomous_method_change: false
        },
        actor_id: "local-owner"
      });
      show("Experiment proposal recorded", "IGY6 created planned experiment metadata only. No runner, MLflow/Optuna execution, or production method change ran.", payload);
    } catch (error) {
      show("Experiment proposal failed", String(error));
    }
  });
})();
`;

  return (
    <section
      className="guidedManualText improvementExperimentReview"
      data-improvement-experiment-review
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="guidedManualNotice">
        <strong>Improvement and experiment review</strong>
        <span>Review proposed improvements and planned experiment metadata. This is controlled review only; no autonomous method changes, MLflow/Optuna run, or Phoenix trace workflow is triggered here.</span>
      </div>
      {[improvements.error, experiments.error].filter(Boolean).length > 0 ? (
        <p className="errorText">Some improvement or experiment endpoints returned errors.</p>
      ) : null}
      <section className="split">
        <div>
          <div className="subHeader"><h3><HelpHeading term="improvementItem">Improvement Items</HelpHeading></h3></div>
          <div className="stack">
            {recentImprovements.map((item) => (
              <article className="item evidenceItem" key={item.id} data-improvement-review-item>
                <div>
                  <strong>{item.target_area}</strong>
                  <span>{excerpt(item.objective, 140)}</span>
                  <span>proposed by {item.proposed_by_actor_id}</span>
                </div>
                <div>
                  <StatusPill state={item.status} />
                  <span>{item.priority}</span>
                </div>
              </article>
            ))}
          </div>
          {recentImprovements.length === 0 ? <EmptyState label="No improvement items recorded yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3><HelpHeading term="experimentRun">Experiment Records</HelpHeading></h3></div>
          <div className="stack">
            {recentExperiments.map((experiment) => (
              <article className="item evidenceItem" key={experiment.id} data-experiment-review-item>
                <div>
                  <strong>{experiment.status}</strong>
                  <span>Improvement: {experiment.improvement_item_id ?? "not linked"}</span>
                  <span>MLflow: {experiment.mlflow_run_id ?? "not executed"}</span>
                  <span>Optuna: {experiment.optuna_study_name ?? "not executed"}</span>
                </div>
                <div>
                  <StatusPill state="review-only" />
                  <span>{formatDate(experiment.created_at)}</span>
                </div>
              </article>
            ))}
          </div>
          {recentExperiments.length === 0 ? <EmptyState label="No experiment records recorded yet." /> : null}
        </div>
      </section>
      <form className="guidedManualForm" data-experiment-proposal-form>
        <label>
          <span>Improvement item</span>
          <select name="experiment_improvement_id" disabled={recentImprovements.length === 0}>
            {recentImprovements.map((item) => (
              <option key={item.id} value={item.id}>{item.target_area} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Review scope</span>
          <textarea name="experiment_scope" rows={2} placeholder="Describe the bounded comparison or review question. No runner starts from this form." />
        </label>
        <label>
          <span>Success metric</span>
          <input name="experiment_success_metric" placeholder="Example: fewer incomplete evidence answers after manual review" />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={recentImprovements.length === 0}>Record planned experiment metadata</button>
          <span>{recentImprovements.length > 0 ? "Creates a planned experiment record only." : "Create or receive an improvement item before proposing an experiment record."}</span>
        </div>
      </form>
      <div className="guidedManualResult" data-experiment-proposal-result>
        <strong>{recentExperiments.length > 0 ? "Experiment metadata exists" : "No experiment proposal selected"}</strong>
        <span>{recentExperiments.length > 0 ? "Recent records are listed above for review." : "Use this only to record metadata for later review."}</span>
      </div>
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

function SourceEvidenceHistory({
  sources,
  collectionRuns,
  artifacts,
  documents,
  chunks,
  evidenceItems
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  artifacts: ApiResult<RawArtifactRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const artifactsByRun = new Map<string, RawArtifactRecord[]>();
  artifacts.data.forEach((artifact) => {
    const key = artifact.collection_run_id ?? "";
    if (!key) return;
    artifactsByRun.set(key, [...(artifactsByRun.get(key) ?? []), artifact]);
  });
  const documentsByArtifact = new Map<string, NormalizedDocumentRecord[]>();
  documents.data.forEach((document) => {
    const key = document.raw_artifact_id ?? "";
    if (!key) return;
    documentsByArtifact.set(key, [...(documentsByArtifact.get(key) ?? []), document]);
  });
  const chunksByDocument = new Map<string, ChunkRecord[]>();
  chunks.data.forEach((chunk) => {
    chunksByDocument.set(chunk.document_id, [...(chunksByDocument.get(chunk.document_id) ?? []), chunk]);
  });
  const evidenceByChunk = new Map<string, EvidenceItemRecord[]>();
  const evidenceByDocument = new Map<string, EvidenceItemRecord[]>();
  evidenceItems.data.forEach((item) => {
    if (item.chunk_id) {
      evidenceByChunk.set(item.chunk_id, [...(evidenceByChunk.get(item.chunk_id) ?? []), item]);
    }
    if (item.document_id) {
      evidenceByDocument.set(item.document_id, [...(evidenceByDocument.get(item.document_id) ?? []), item]);
    }
  });

  const histories = collectionRuns.data.slice(0, 5).map((run) => {
    const runArtifacts = artifactsByRun.get(run.id) ?? [];
    const runDocuments = runArtifacts.flatMap((artifact) => documentsByArtifact.get(artifact.id) ?? []);
    const runChunks = runDocuments.flatMap((document) => chunksByDocument.get(document.id) ?? []);
    const chunkEvidence = runChunks.flatMap((chunk) => evidenceByChunk.get(chunk.id) ?? []);
    const documentEvidence = runDocuments.flatMap((document) => evidenceByDocument.get(document.id) ?? []);
    const uniqueEvidence = [...new Map([...chunkEvidence, ...documentEvidence].map((item) => [item.id, item])).values()];
    return {
      run,
      source: run.source_id ? sourceById.get(run.source_id) : undefined,
      artifacts: runArtifacts,
      documents: runDocuments,
      chunks: runChunks,
      evidence: uniqueEvidence
    };
  });

  return (
    <section className="guidedManualText sourceHistory" data-source-evidence-history>
      <div className="guidedManualNotice">
        <strong>Source and evidence history</strong>
        <span>Recent processing lineage by identifier only. Raw uploaded text and artifact files are not displayed here.</span>
      </div>
      <div className="stack">
        {histories.map((history) => (
          <article className="item evidenceItem" key={history.run.id} data-source-history-item>
            <div>
              <strong>{history.source?.name ?? "Unknown source"}</strong>
              <span>{history.source?.source_type ?? "no source type"} · run {history.run.id}</span>
            </div>
            <dl>
              <dt>source</dt><dd>{history.run.source_id ?? "not linked"}</dd>
              <dt>status</dt><dd>{history.run.status}</dd>
              <dt>artifact</dt><dd>{history.artifacts[0]?.id ?? "none recorded"}</dd>
              <dt>document</dt><dd>{history.documents[0]?.id ?? "none recorded"}</dd>
              <dt>chunks</dt><dd>{history.chunks.length}</dd>
              <dt>evidence</dt><dd>{history.evidence.length}</dd>
            </dl>
          </article>
        ))}
      </div>
      {histories.length === 0 ? <EmptyState label="No source/evidence history is available yet." /> : null}
    </section>
  );
}

function MvpActionConsole() {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const script = `
(() => {
  const root = document.querySelector("[data-mvp-actions]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const output = root.querySelector("[data-action-output]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const show = (label, payload) => {
    if (output) output.textContent = label + "\\n" + (typeof payload === "string" ? payload : JSON.stringify(payload, null, 2));
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  const bind = (selector, handler) => {
    root.querySelector(selector)?.addEventListener("submit", async (event) => {
      event.preventDefault();
      try {
        const result = await handler();
        show("Done", result);
      } catch (error) {
        show("Error", String(error));
      }
    });
  };

  bind("[data-create-source]", () => postJson("/sources", {
    name: value("source_name"),
    source_type: value("source_type"),
    location: value("source_location") || null,
    sensitivity: value("source_sensitivity") || "internal",
    permission: {
      scope_json: value("source_scope_json") ? JSON.parse(value("source_scope_json")) : {},
      allowed_operations: value("source_allowed_operations").split(",").map((item) => item.trim()).filter(Boolean),
      external_model_policy: "blocked",
      approval_required: checked("source_approval_required")
    }
  }));

  bind("[data-approval]", () => postJson("/approvals", {
    request_type: value("approval_request_type"),
    request_payload_json: {
      source_id: value("approval_source_id") || undefined,
      source_permission_id: value("approval_permission_id") || undefined,
      operation: value("approval_operation") || undefined
    }
  }));

  bind("[data-approval-decision]", () => postJson("/approvals/" + value("decision_approval_id") + "/decision", {
    status: value("decision_status"),
    decision_reason: value("decision_reason") || null
  }));

  bind("[data-dry-run]", () => postJson("/collection-runs/dry-run", {
    source_id: value("dry_source_id"),
    source_permission_id: value("dry_permission_id")
  }));

  bind("[data-manual-upload]", () => postJson("/collection-runs/manual-upload", {
    source_id: value("upload_source_id"),
    source_permission_id: value("upload_permission_id"),
    approval_id: value("upload_approval_id") || null,
    filename: value("upload_filename") || "manual-note.txt",
    mime_type: "text/plain",
    content_base64: btoa(unescape(encodeURIComponent(value("upload_text"))))
  }));

  bind("[data-dispatch]", () => postJson("/work-items/" + value("dispatch_work_item_id") + "/dispatch", {}));

  bind("[data-answer]", () => postJson("/chat/evidence-answer", {
    message: value("answer_message"),
    limit: Number(value("answer_limit") || 5)
  }));

  bind("[data-review]", async () => {
    const action = value("review_action");
    if (action === "feedback") {
      return postJson("/feedback", {
        target_type: value("review_target_type"),
        target_id: value("review_target_id"),
        label: value("review_label"),
        note: value("review_note") || null
      });
    }
    if (action === "outcome") {
      return postJson("/outcomes", {
        target_type: value("review_target_type"),
        target_id: value("review_target_id"),
        outcome_status: value("review_label"),
        summary: value("review_note") || null
      });
    }
    return postJson("/analysis/patterns/" + value("review_target_id") + "/review", {
      status: value("review_label"),
      review_note: value("review_note") || null
    });
  });

  bind("[data-pattern-detect]", () => postJson("/analysis/patterns/detect-baseline", {
    recurrence_threshold: Number(value("pattern_threshold") || 3)
  }));

  bind("[data-report]", async () => {
    if (value("report_action") === "render") {
      return postJson("/reports/" + value("report_id") + "/render", { notes: value("report_notes") || null });
    }
    return postJson("/reports", {
      title: value("report_title") || "MVP report",
      report_type: value("report_type") || "summary"
    });
  });
})();
`;

  return (
    <section className="panel actionConsole" data-mvp-actions data-api-base-url={browserApiBaseUrl}>
      <div className="panelHeader">
        <h2>Advanced Route Console</h2>
        <span className="statusText">Advanced route controls</span>
      </div>
      <section className="actionGrid">
        <form className="actionBox" data-create-source>
          <h3><HelpHeading term="source">Source</HelpHeading></h3>
          <p className="actionHint"><TermHelp term="sourcePermission" label="Source Permission" /> controls scope, operations, approval, and external model policy.</p>
          <input name="source_name" placeholder="Source name" />
          <label className="fieldWithHelp">
            <TermHelp term="sourceType" label="Source Type" />
            <select name="source_type" defaultValue="manual_upload">
              <option value="manual_upload">manual_upload</option>
              <option value="local_project">local_project</option>
              <option value="conversation_history">conversation_history</option>
              <option value="user_observation">user_observation</option>
            </select>
          </label>
          <input name="source_location" placeholder="Location" />
          <select name="source_sensitivity" defaultValue="internal">
            <option value="public">public</option>
            <option value="internal">internal</option>
            <option value="sensitive">sensitive</option>
            <option value="secret">secret</option>
          </select>
          <label className="fieldWithHelp"><TermHelp term="allowedOperations" label="Allowed Operations" /><input name="source_allowed_operations" defaultValue="dry_run,read,collect" /></label>
          <label className="fieldWithHelp"><TermHelp term="permissionScope" label="Permission Scope" /><textarea name="source_scope_json" rows={2} defaultValue="{}" /></label>
          <p className="actionHint"><TermHelp term="manualUpload" label="Manual Upload" /> and <TermHelp term="localProject" label="Local Project" /> are the currently useful collection paths.</p>
          <label className="checkLine"><input name="source_approval_required" type="checkbox" defaultChecked /> <TermHelp term="approvalRequired" label="Approval required" /></label>
          <button type="submit">Create Source</button>
        </form>

        <form className="actionBox" data-approval>
          <h3><HelpHeading term="approval">Approval</HelpHeading></h3>
          <input name="approval_request_type" defaultValue="manual_upload_collection" />
          <input name="approval_source_id" placeholder="Source ID" />
          <input name="approval_permission_id" placeholder="Permission ID" />
          <input name="approval_operation" defaultValue="manual_upload_collection" />
          <button type="submit">Request</button>
        </form>

        <form className="actionBox" data-approval-decision>
          <h3>Decision</h3>
          <input name="decision_approval_id" placeholder="Approval ID" />
          <select name="decision_status" defaultValue="approved">
            <option value="approved">approved</option>
            <option value="denied">denied</option>
          </select>
          <input name="decision_reason" placeholder="Reason" />
          <button type="submit">Decide</button>
        </form>

        <form className="actionBox" data-dry-run>
          <h3><HelpHeading term="dryRun">Dry-Run</HelpHeading></h3>
          <input name="dry_source_id" placeholder="Source ID" />
          <input name="dry_permission_id" placeholder="Permission ID" />
          <button type="submit">Run</button>
        </form>

        <form className="actionBox wide" data-manual-upload>
          <h3><HelpHeading term="manualUpload">Manual Upload</HelpHeading></h3>
          <input name="upload_source_id" placeholder="Source ID" />
          <input name="upload_permission_id" placeholder="Permission ID" />
          <input name="upload_approval_id" placeholder="Approval ID" />
          <input name="upload_filename" defaultValue="manual-note.txt" />
          <textarea name="upload_text" rows={4} placeholder="Authorized text content" />
          <button type="submit">Collect</button>
        </form>

        <form className="actionBox" data-dispatch>
          <h3><HelpHeading term="dispatch">Dispatch</HelpHeading></h3>
          <input name="dispatch_work_item_id" placeholder="Queued work item ID" />
          <button type="submit">Dispatch</button>
        </form>

        <form className="actionBox wide" data-answer>
          <h3><HelpHeading term="evidenceAnswer">Evidence Answer</HelpHeading></h3>
          <textarea name="answer_message" rows={3} defaultValue="What does the system know?" />
          <input name="answer_limit" type="number" min="1" max="50" defaultValue="5" />
          <button type="submit">Build Evidence-Grounded Answer</button>
        </form>

        <form className="actionBox" data-review>
          <h3>Review</h3>
          <select name="review_action" defaultValue="feedback">
            <option value="feedback">feedback</option>
            <option value="outcome">outcome</option>
            <option value="pattern_review">pattern_review</option>
          </select>
          <input name="review_target_type" defaultValue="prediction" />
          <input name="review_target_id" placeholder="Target ID" />
          <input name="review_label" defaultValue="useful" />
          <input name="review_note" placeholder="Note" />
          <button type="submit">Record</button>
        </form>

        <form className="actionBox" data-pattern-detect>
          <h3><HelpHeading term="pattern">Patterns</HelpHeading></h3>
          <input name="pattern_threshold" type="number" min="2" max="20" defaultValue="3" />
          <button type="submit">Detect</button>
        </form>

        <form className="actionBox wide" data-report>
          <h3>Reports</h3>
          <select name="report_action" defaultValue="create">
            <option value="create">create</option>
            <option value="render">render</option>
          </select>
          <input name="report_id" placeholder="Report ID for render" />
          <input name="report_title" defaultValue="MVP report" />
          <select name="report_type" defaultValue="summary">
            <option value="summary">summary</option>
            <option value="evidence_review">evidence_review</option>
            <option value="handoff">handoff</option>
            <option value="experiment_summary">experiment_summary</option>
          </select>
          <textarea name="report_notes" rows={2} placeholder="Render notes" />
          <button type="submit">Run</button>
        </form>
      </section>
      <pre className="actionOutput" data-action-output>Action results appear here.</pre>
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

export default async function Home() {
  const [
    health,
    sources,
    collectionRuns,
    artifacts,
    documents,
    chunks,
    evidenceItems,
    evidenceAnswers,
    claims,
    vectorCollection,
    graphSchema,
    patterns,
    hypotheses,
    predictions,
    recommendations,
	    workItems,
	    approvals,
	    feedback,
	    outcomes,
	    improvements,
	    experiments,
	    reports,
	    agentTaskPlans,
    auditEvents,
    envSettings,
    agentCapabilities
  ] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", []),
    getJson<NormalizedDocumentRecord[]>("/evidence/documents", []),
    getJson<ChunkRecord[]>("/evidence/chunks", []),
    getJson<EvidenceItemRecord[]>("/evidence/items", []),
    getJson<EvidenceAnswerRecord[]>("/evidence-answers", []),
    getJson<ClaimRecord[]>("/evidence/claims", []),
    getJson<VectorCollectionStatus>("/memory/vector/chunks", { collection_name: "unknown", exists: false }),
    getJson<GraphSchemaStatus>("/memory/graph/schema", { constraints: [] }),
    getJson<PatternRecord[]>("/analysis/patterns", []),
    getJson<HypothesisRecord[]>("/analysis/hypotheses", []),
    getJson<PredictionRecord[]>("/analysis/predictions", []),
    getJson<RecommendationRecord[]>("/analysis/recommendations", []),
	    getJson<WorkItemRecord[]>("/work-items", []),
	    getJson<ApprovalRecord[]>("/approvals", []),
	    getJson<FeedbackRecord[]>("/feedback", []),
	    getJson<OutcomeRecord[]>("/outcomes", []),
	    getJson<ImprovementRecord[]>("/improvements", []),
	    getJson<ExperimentRecord[]>("/experiments", []),
	    getJson<ReportRecord[]>("/reports", []),
	    getJson<AgentTaskPlanRecord[]>("/agent/task-plans", []),
    getJson<AuditEventRecord[]>("/audit-events", []),
    getJson<EnvSettingsResponse>("/settings/env", {
      file_status: {
        path: "unknown",
        backup_dir: "unknown",
        exists: false,
        writable: false,
        unknown_key_count: 0,
        output_format: "unknown"
      },
      groups: [],
      settings: [],
      unmanaged: [],
      warnings: []
    }),
    getJson<AgentCapabilitiesResponse>("/agent/capabilities", {
      actions: [],
      runtime: {
        repo_root: "unknown",
        docker_cli_available: false,
        docker_compose_available: false,
        docker_socket_available: false,
        docker_host_configured: false,
        docker_control_available: false,
        docker_socket_path: null,
        reason: "Agent capabilities were unavailable."
      }
    })
  ]);

  const checks = health.data.checks ?? {};
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentArtifacts = artifacts.data.slice(0, 6);
  const recentDocuments = documents.data.slice(0, 5);
  const recentChunks = chunks.data.slice(0, 5);
  const recentEvidence = evidenceItems.data.slice(0, 5);
  const recentClaims = claims.data.slice(0, 5);
  const recentPatterns = patterns.data.slice(0, 4);
  const recentHypotheses = hypotheses.data.slice(0, 4);
  const recentPredictions = predictions.data.slice(0, 4);
  const recentRecommendations = recommendations.data.slice(0, 4);
  const recentWorkItems = workItems.data.slice(0, 8);
  const recentApprovals = approvals.data.slice(0, 4);
  const recentFeedback = feedback.data.slice(0, 4);
  const recentOutcomes = outcomes.data.slice(0, 4);
  const recentReports = reports.data.slice(0, 4);
  const recentAuditEvents = auditEvents.data.slice(0, 4);
  const pendingApprovals = approvals.data.filter((approval) => approval.status === "pending");
  const approvedApprovals = approvals.data.filter((approval) => approval.status === "approved");
  const rejectedApprovals = approvals.data.filter((approval) => ["denied", "rejected"].includes(approval.status));
  const queuedWorkItems = workItems.data.filter((item) => item.status === "queued");
  const runningWorkItems = workItems.data.filter((item) => item.status === "running");
  const completedWorkItems = workItems.data.filter((item) => item.status === "completed");
  const failedWorkItems = workItems.data.filter((item) => item.status === "failed");
  const blockedActions = agentCapabilities.data.actions.filter((action) => !action.executable_in_api_runtime);
  const approvalRequiredActions = agentCapabilities.data.actions.filter((action) => action.approval_required);

  return (
    <main className="consoleShell">
      <aside className="leftSidebar" aria-label="IGY6 navigation">
        <div className="brandBlock">
          <div className="brandMark">IG</div>
          <div>
            <strong>IGY6</strong>
            <span>Local evidence app</span>
          </div>
        </div>

        <div className="sidebarActions">
          <label className="sidebarButton primary" htmlFor="tab-results">Ask with evidence</label>
          <label className="sidebarButton" htmlFor="tab-add-data">Add data</label>
          <label className="sidebarButton" htmlFor="tab-work">Check processing</label>
        </div>

        <label className="sidebarSearch">
          <span>Search workspace</span>
          <input readOnly value="" placeholder="Sources, uploads, evidence, reports..." />
        </label>

        <nav className="navSection" aria-label="Workspace tabs">
          <label htmlFor="tab-home">Home</label>
          <label htmlFor="tab-add-data">Add Data</label>
          <label htmlFor="tab-work">Work</label>
          <label htmlFor="tab-results">Results</label>
          <label htmlFor="tab-settings">Settings</label>
          <label htmlFor="tab-advanced">Advanced</label>
        </nav>

        <section className="sidebarList" aria-label="Recent work">
          <div className="sidebarHeading">
            <span>Recent work</span>
            <StatusPill state="rust-worker" />
          </div>
          {recentWorkItems.map((workItem) => (
            <article className="miniRecord" key={workItem.id}>
              <strong>{workItem.work_type}</strong>
              <span>{workItem.status} · {formatDate(workItem.created_at)}</span>
            </article>
          ))}
          {recentWorkItems.length === 0 ? <EmptyState label="No work items yet." /> : null}
        </section>

        <footer className="localFooter">
          <StatusPill state="local-first" />
          <span>Read-only by default · No external model</span>
        </footer>
      </aside>

      <section className="mainConsole">
        <header className="topBar">
          <div>
            <p className="eyebrow">IGY6</p>
            <h1>Local Evidence Dashboard</h1>
          </div>
          <div className="topStatus">
            <StatusPill state="local-first" />
            <StatusPill state="system-ready" />
            <StatusPill state="background-ready" />
            <StatusPill state="no-external-model" />
            <StatusPill state={health.data.status} />
          </div>
        </header>

        <section className="productTabs" aria-label="Main dashboard tabs">
          <input className="tabInput" id="tab-home" name="main-dashboard-tab" type="radio" defaultChecked />
          <input className="tabInput" id="tab-add-data" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-work" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-results" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-settings" name="main-dashboard-tab" type="radio" />
          <input className="tabInput" id="tab-advanced" name="main-dashboard-tab" type="radio" />
          <nav className="tabList" aria-label="Main dashboard">
            <label role="tab" htmlFor="tab-home">Home</label>
            <label role="tab" htmlFor="tab-add-data">Add Data</label>
            <label role="tab" htmlFor="tab-work">Work</label>
            <label role="tab" htmlFor="tab-results">Results</label>
            <label role="tab" htmlFor="tab-settings">Settings</label>
            <label role="tab" htmlFor="tab-advanced">Advanced</label>
          </nav>
        </section>

        <section className="panel workflowHero tabContent" id="home" data-tab-panel="home">
          <div className="panelHeader">
            <div>
              <p className="eyebrow">Home</p>
              <h2>System Ready</h2>
            </div>
            <StatusPill state={health.data.status} />
          </div>
          <section className="readinessStrip" aria-label="Current readiness">
            {USER_READINESS.map((item) => (
              <article key={item.label}>
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <StatusPill state={item.state} />
              </article>
            ))}
          </section>
          <p className="readinessSummary">System ready. Background worker ready. {pendingApprovals.length > 0 ? "Review pending approvals before sensitive collection." : "No approval needs attention right now."}</p>
          <section className="metrics compact" aria-label="Home overview">
            <article><span>Service readiness</span><strong>{Object.keys(checks).length ? `${Object.values(checks).filter((check) => check.status === "ok").length}/${Object.keys(checks).length}` : "Unknown"}</strong></article>
            <article><span>Recent data activity</span><strong>{recentRuns.length + recentArtifacts.length}</strong></article>
            <article><span>Recent work</span><strong>{recentWorkItems.length}</strong></article>
            <article><span>Pending approvals</span><strong>{pendingApprovals.length}</strong></article>
            <article><span>Recent audit events</span><strong>{recentAuditEvents.length}</strong></article>
          </section>
          <div className="primaryWorkflowGrid" aria-label="Primary workflows">
            <article>
              <span>1</span>
              <h3>Add data</h3>
              <p>Create a scoped source and upload approved UTF-8 text such as notes, logs, or exports.</p>
              <label htmlFor="tab-add-data">Open Add Data</label>
            </article>
            <article>
              <span>2</span>
              <h3>Check processing</h3>
              <p>See what is waiting, running, completed, or needs attention.</p>
              <label htmlFor="tab-work">Open Work</label>
            </article>
            <article>
              <span>3</span>
              <h3>Ask with evidence</h3>
              <p>{sources.data.length === 0 ? "Add a data source first." : evidenceItems.data.length === 0 ? "Add approved text and check processing." : "Ask a question over local evidence."}</p>
              <label htmlFor="tab-results">Open Results</label>
            </article>
          </div>
        </section>

        <section className="chatStage workflowSection tabContent" id="assistant" data-tab-panel="results">
          <div className="conversationWindow">
            <article className="message systemMessage">
              <div className="avatar">SYS</div>
              <div className="messageBubble">
                <span className="messageLabel">Assistant</span>
                <p>Ask questions and request safe local actions here. IGY6 previews evidence retrieval and fixed allowlisted actions before anything runs.</p>
                <div className="messageMeta">
                  <TermHelp term="deterministic" label="deterministic" />
                  <StatusPill state="not-generated" />
                  <StatusPill state="read-only-default" />
                </div>
              </div>
            </article>

            <article className="message userMessage">
              <div className="avatar">YOU</div>
              <div className="messageBubble">
                <span className="messageLabel">Example request</span>
                <p>What does this document say I need to do next?</p>
              </div>
            </article>

            <article className="message assistantMessage">
              <div className="avatar">IG</div>
              <div className="messageBubble">
                <span className="messageLabel">Evidence and action status</span>
                <p>Use Ask over evidence for citations and source trails, or Preview action for project health, git status, latest DIFF, work items, stack start/stop, and last healthy stack.</p>
                <div className="retrievalStrip">
                  <span>{evidenceItems.data.length} <TermHelp term="evidenceItem" label="evidence items" /> stored</span>
                  <span>{chunks.data.length} <TermHelp term="chunk" label="chunks" /> indexed in state</span>
                  <span><TermHelp term="vectorMemory" label={vectorCollection.data.exists ? "Vector collection ready" : "Vector collection missing"} /></span>
                </div>
              </div>
            </article>
          </div>

          <LocalLlmStatusPanel envSettings={envSettings} context="assistant" />
          <ChatRetrievalPreview />
          <EvidenceAnswerHistory evidenceAnswers={evidenceAnswers} feedback={feedback} />
	          <AgentCommandPanel capabilities={agentCapabilities} approvals={approvals} taskPlans={agentTaskPlans} />
	          <AgentTaskHistoryReview
	            taskPlans={agentTaskPlans}
	            workItems={workItems}
	            approvals={approvals}
	            feedback={feedback}
	            outcomes={outcomes}
	            improvements={improvements}
	          />
        </section>

        <section className="panel diagnosticsPanel tabContent" id="advanced-diagnostics" data-tab-panel="advanced">
          <div className="panelHeader">
            <div>
              <p className="eyebrow">Advanced</p>
              <h2>Diagnostics</h2>
            </div>
            <StatusPill state={health.data.status} />
          </div>
          <section className="runtimePosture" aria-label="Technical runtime posture">
            {RUNTIME_POSTURE.map((item) => (
              <article key={item.label}>
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <StatusPill state={item.state} />
              </article>
            ))}
          </section>
          <section className="split">
            <article className="panelInset">
              <h3>Service readiness</h3>
              <div className="checkList">
                {Object.entries(checks).map(([name, check]) => (
                  <article className="checkRow" key={name}>
                    <span>{name}</span>
                    <StatusPill state={check.status} />
                  </article>
                ))}
                {Object.keys(checks).length === 0 ? <EmptyState label="No readiness details returned." /> : null}
              </div>
            </article>
            <article className="panelInset">
              <h3>Recent audit</h3>
              <div className="stack">
                {recentAuditEvents.map((event) => (
                  <article className="miniRecord" key={event.id}>
                    <strong>{event.event_type}</strong>
                    <span>{event.decision ?? "recorded"} · {event.actor_id}</span>
                  </article>
                ))}
              </div>
              {recentAuditEvents.length === 0 ? <EmptyState label="No audit events yet." /> : null}
            </article>
          </section>
        </section>

        <section className="panel toolConsole tabContent" aria-label="Advanced route console" data-tab-panel="advanced">
          <details>
            <summary>
              <span>
                <strong>Advanced Route Console</strong>
                <em>Existing API-backed controls · no new workflow behavior</em>
              </span>
              <StatusPill state="advanced" />
            </summary>
            <MvpActionConsole />
          </details>
        </section>

        <section className="workspaceGrid" aria-label="IGY6 workflow records">
          <section className="panel workflowSection tabContent" id="data-knowledge" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data</p>
                <h2>Bring In Authorized Information</h2>
              </div>
              <StatusPill state="local-first" />
            </div>
            <div className="lifecycleFlow" aria-label="Data lifecycle">
              {["Source", "Upload / Collection", "Raw Artifact", "Document", "Chunks", "Evidence", "Memory", "Analysis / Chat Retrieval"].map((step) => (
                <span key={step}>{step}</span>
              ))}
            </div>
            <section className="workflowTabs" aria-label="Add data steps">
              <a href="#data-overview">Overview</a>
              <a href="#sources-panel">Sources</a>
              <a href="#uploads-collection">Uploads</a>
            </section>
            <div className="quickStartGrid" id="data-overview">
              <article>
                <h3>Normal PC user examples</h3>
                <p>Upload warranty notes, router troubleshooting notes, a bill note, or a folder inventory and ask what changed, what expires, or what looks duplicated.</p>
              </article>
              <article>
                <h3>Project examples</h3>
                <p>Upload build notes, verification summaries, or project logs and ask what failed, what changed, or what needs review.</p>
              </article>
            </div>
          </section>

          <section className="panel tabContent" id="sources-panel" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data / Sources</p>
                <h2><HelpHeading term="source">Where Your Data Comes From</HelpHeading></h2>
              </div>
              {sources.error ? <span className="errorText">{sources.error}</span> : <StatusPill state={`${sources.data.length}-sources`} />}
            </div>
            <div className="fieldGuide">
              <article><strong>Source name</strong><span>Everyday: "Router Troubleshooting Notes" · Coder: "IGY6 Build Logs"</span></article>
              <article><strong>Source type</strong><span>Use "manual_upload" for generic pasted text or "conversation_history" for prior conversation/history imports.</span></article>
              <article><strong>Location</strong><span>Everyday: "local notes folder" · Coder: "local repo logs"</span></article>
              <article><strong>Sensitivity</strong><span>Everyday: "private" · Coder: "internal"</span></article>
              <article><strong>Allowed operations</strong><span>Everyday: "read, collect" · Coder: "read, collect, dry_run"</span></article>
            </div>
            <div className="table compactTable">
              {sources.data.map((source) => (
                <div className="row" key={source.id}>
                  <strong>{source.name}</strong>
                  <span><TermHelp term="sourceType" label={source.source_type} /></span>
                  <span>{source.sensitivity}</span>
                  <span>{source.permissions?.length ?? 0} <TermHelp term="sourcePermission" label="permissions" /></span>
                  <StatusPill state={source.enabled ? "enabled" : "disabled"} />
                </div>
              ))}
            </div>
            {sources.data.length === 0 ? <EmptyState label="No sources registered yet." /> : null}
            <SourceTrustSensitivityManagement
              sources={sources}
              collectionRuns={collectionRuns}
              documents={documents}
              evidenceItems={evidenceItems}
            />
            <details className="advancedPanel">
              <summary>Advanced: source IDs, permission IDs, and raw source data</summary>
              <pre>{JSON.stringify(sources.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

          <section className="panel tabContent" id="uploads-collection" data-tab-panel="add-data">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Add Data / Uploads</p>
                <h2><HelpHeading term="manualUpload">Guided Upload</HelpHeading></h2>
              </div>
              <StatusPill state="approval-aware" />
            </div>
            <GuidedManualTextUpload sources={sources} />
            <ol className="workflowSteps">
              <li><strong>Step 1: Select or create source.</strong><span>Use a manual_upload source for notes/logs or a conversation_history source for prior chat/history text.</span></li>
              <li><strong>Step 2: Check approval status.</strong><span>Source permissions show whether approval is required before collection.</span></li>
              <li><strong>Step 3: Request approval if required.</strong><span>Everyday reason: "Allow IGY6 to process this uploaded troubleshooting note." Coder reason: "Approve processing this local build log for evidence extraction."</span></li>
              <li><strong>Step 4: Upload text or a safe file extract.</strong><span>Current manual upload works best with UTF-8 text.</span></li>
              <li><strong>Step 5: Review created records.</strong><span>Check collection run, raw artifact, and work item status.</span></li>
              <li><strong>Step 6: Next action.</strong><span>Check processing, view evidence, or ask Assistant a question.</span></li>
            </ol>
            <ConversationHistoryImport sources={sources} />
            <div className="subHeader"><h3><HelpHeading term="collectionRun">Collection Runs</HelpHeading></h3>{collectionRuns.error ? <span className="errorText">{collectionRuns.error}</span> : null}</div>
            <div className="stack">
              {recentRuns.map((run) => (
                <article className="item evidenceItem" key={run.id}>
                  <div><strong>{run.status}</strong><span>{run.dry_run ? "dry run" : "collection"} · requested by {run.requested_by_actor_id}</span></div>
                  <div><span>{formatDate(run.created_at)}</span><StatusPill state={run.dry_run ? "dry-run" : "collected"} /></div>
                </article>
              ))}
            </div>
            {recentRuns.length === 0 ? <EmptyState label="No collection runs recorded yet." /> : null}
            <details className="advancedPanel">
              <summary>Advanced: raw artifact IDs, collection run IDs, and upload JSON</summary>
              <pre>{JSON.stringify({ collection_runs: collectionRuns.data.slice(0, 10), raw_artifacts: artifacts.data.slice(0, 10) }, null, 2)}</pre>
            </details>
          </section>

          <section className="panel tabContent" id="evidence-panel" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2><HelpHeading term="evidenceItem">Evidence And Documents</HelpHeading></h2>
              </div>
              {[documents.error, chunks.error, evidenceItems.error, claims.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some evidence endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Evidence totals">
              <article><span><TermHelp term="collectionRun" label="Collection runs" /></span><strong>{collectionRuns.data.length}</strong></article>
              <article><span><TermHelp term="rawArtifact" label="Raw artifacts" /></span><strong>{artifacts.data.length}</strong></article>
              <article><span><TermHelp term="normalizedDocument" label="Documents" /></span><strong>{documents.data.length}</strong></article>
              <article><span><TermHelp term="chunk" label="Chunks" /></span><strong>{chunks.data.length}</strong></article>
              <article><span><TermHelp term="evidenceItem" label="Evidence" /></span><strong>{evidenceItems.data.length}</strong></article>
              <article><span><TermHelp term="claim" label="Claims" /></span><strong>{claims.data.length}</strong></article>
            </section>
            <EvidenceCorrectionSupersessionWorkflow evidenceItems={evidenceItems} />
            <SourceEvidenceHistory
              sources={sources}
              collectionRuns={collectionRuns}
              artifacts={artifacts}
              documents={documents}
              chunks={chunks}
              evidenceItems={evidenceItems}
            />
            <section className="quad">
              <div>
                <div className="subHeader"><h3><HelpHeading term="collectionRun">Collection Runs</HelpHeading></h3>{collectionRuns.error ? <span className="errorText">{collectionRuns.error}</span> : null}</div>
                <div className="stack">
                  {recentRuns.map((run) => (
                    <article className="item evidenceItem" key={run.id}>
                      <div><strong>{run.status}</strong><span>{run.dry_run ? "dry run" : "collection"} · requested by {run.requested_by_actor_id}</span></div>
                      <div><span>{formatDate(run.created_at)}</span><StatusPill state={run.source_id ? "source-linked" : "no-source"} /></div>
                    </article>
                  ))}
                </div>
                {recentRuns.length === 0 ? <EmptyState label="No collection runs recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="rawArtifact">Raw Artifacts</HelpHeading></h3>{artifacts.error ? <span className="errorText">{artifacts.error}</span> : null}</div>
                <div className="stack">
                  {recentArtifacts.map((artifact) => (
                    <article className="item evidenceItem" key={artifact.id}>
                      <div><strong>{formatBytes(artifact.size_bytes)}</strong><span>{artifact.mime_type ?? "unknown type"}</span></div>
                      <div><span>{formatDate(artifact.created_at)}</span><StatusPill state={artifact.collection_run_id ? "run-linked" : "no-run"} /></div>
                    </article>
                  ))}
                </div>
                {recentArtifacts.length === 0 ? <EmptyState label="No raw artifacts recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="normalizedDocument">Documents</HelpHeading></h3>{documents.error ? <span className="errorText">{documents.error}</span> : null}</div>
                <div className="stack">
                  {recentDocuments.map((document) => (
                    <article className="item evidenceItem" key={document.id}>
                      <div><strong>{document.title ?? "Untitled document"}</strong><span>{document.document_type} · {document.sensitivity}</span></div>
                      <div><span>{formatDate(document.created_at)}</span><StatusPill state={document.source_id ? "source-linked" : "no-source"} /></div>
                    </article>
                  ))}
                </div>
                {recentDocuments.length === 0 ? <EmptyState label="No normalized documents recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="chunk">Chunks</HelpHeading></h3>{chunks.error ? <span className="errorText">{chunks.error}</span> : null}</div>
                <div className="stack">
                  {recentChunks.map((chunk) => (
                    <article className="item evidenceItem" key={chunk.id}>
                      <div><strong>Chunk {chunk.chunk_index}</strong><span>Normalized document chunk</span></div>
                      <div><StatusPill state={chunk.embedding_status} /><span>index {chunk.chunk_index}</span></div>
                    </article>
                  ))}
                </div>
                {recentChunks.length === 0 ? <EmptyState label="No chunks generated yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="evidenceItem">Evidence Items</HelpHeading></h3>{evidenceItems.error ? <span className="errorText">{evidenceItems.error}</span> : null}</div>
                <div className="stack">
                  {recentEvidence.map((item) => (
                    <article className="item evidenceItem" key={item.id}>
                      <div><strong>{item.evidence_type}</strong><span>{excerpt(item.statement)}</span></div>
                      <div>
                        <span>{item.confidence === null ? "unscored" : `${item.confidence}%`}</span>
                        <StatusPill state={item.chunk_id ? "chunk-linked" : "no-chunk"} />
                        <StatusPill state={evidenceReviewState(item)} />
                      </div>
                    </article>
                  ))}
                </div>
                {recentEvidence.length === 0 ? <EmptyState label="No evidence items recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="claim">Claims</HelpHeading></h3>{claims.error ? <span className="errorText">{claims.error}</span> : null}</div>
                <div className="stack">
                  {recentClaims.map((claim) => (
                    <article className="item evidenceItem" key={claim.id}>
                      <div><strong>{claim.claim_type}</strong><span>{excerpt(claim.claim_text)}</span></div>
                      <div><StatusPill state={claim.status} /><span>{claim.confidence === null ? "unscored" : `${claim.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentClaims.length === 0 ? <EmptyState label="No claims recorded yet." /> : null}
              </div>
            </section>
          </section>

          <section className="panel tabContent" id="memory-panel" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2><HelpHeading term="vectorMemory">Search Memory And Findings</HelpHeading></h2>
              </div>
              {[vectorCollection.error, graphSchema.error, patterns.error, hypotheses.error, predictions.error, recommendations.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some memory or analysis endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Memory and analysis totals">
              <article><span><TermHelp term="vectorMemory" label="Vector collection" /></span><strong>{vectorCollection.data.exists ? "Ready" : "Missing"}</strong></article>
              <article><span><TermHelp term="graphMemory" label="Graph constraints" /></span><strong>{graphSchema.data.constraints.length}</strong></article>
              <article><span><TermHelp term="pattern" label="Patterns" /></span><strong>{patterns.data.length}</strong></article>
              <article><span><TermHelp term="recommendation" label="Recommendations" /></span><strong>{recommendations.data.length}</strong></article>
            </section>
            <section className="split">
              <article className="item evidenceItem">
                <div><strong><TermHelp term="qdrant" label={vectorCollection.data.collection_name} /></strong><span>Configured chunk collection</span></div>
                <div><StatusPill state={vectorCollection.data.exists ? "enabled" : "missing"} /></div>
              </article>
              <article className="item evidenceItem">
                <div><strong><TermHelp term="neo4j" label={`${graphSchema.data.constraints.length} constraints`} /></strong><span>Graph schema inspection only</span></div>
                <div><StatusPill state={graphSchema.error ? "error" : "ok"} /></div>
              </article>
            </section>
	            <section className="quad analysisGrid" id="analysis-panel">
              <div>
                <div className="subHeader"><h3><HelpHeading term="pattern">Patterns</HelpHeading></h3>{patterns.error ? <span className="errorText">{patterns.error}</span> : null}</div>
                <div className="stack">
                  {recentPatterns.map((pattern) => (
                    <article className="item evidenceItem" key={pattern.id}>
                      <div><strong>{pattern.pattern_type}</strong><span>{excerpt(pattern.summary)}</span></div>
                      <div><StatusPill state={pattern.status} /><span>{pattern.confidence === null ? "unscored" : `${pattern.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentPatterns.length === 0 ? <EmptyState label="No patterns recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="hypothesis">Hypotheses</HelpHeading></h3>{hypotheses.error ? <span className="errorText">{hypotheses.error}</span> : null}</div>
                <div className="stack">
                  {recentHypotheses.map((hypothesis) => (
                    <article className="item evidenceItem" key={hypothesis.id}>
                      <div><strong>Hypothesis</strong><span>{excerpt(hypothesis.hypothesis_text)}</span></div>
                      <div><StatusPill state={hypothesis.status} /><span>{hypothesis.confidence === null ? "unscored" : `${hypothesis.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentHypotheses.length === 0 ? <EmptyState label="No hypotheses recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="prediction">Predictions</HelpHeading></h3>{predictions.error ? <span className="errorText">{predictions.error}</span> : null}</div>
                <div className="stack">
                  {recentPredictions.map((prediction) => (
                    <article className="item evidenceItem" key={prediction.id}>
                      <div><strong>{excerpt(prediction.prediction_text, 80)}</strong><span>{excerpt(prediction.expected_result, 90)}</span></div>
                      <div><StatusPill state={prediction.status} /><span>{prediction.confidence === null ? "unscored" : `${prediction.confidence}%`}</span></div>
                    </article>
                  ))}
                </div>
                {recentPredictions.length === 0 ? <EmptyState label="No predictions recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="recommendation">Recommendations</HelpHeading></h3>{recommendations.error ? <span className="errorText">{recommendations.error}</span> : null}</div>
                <div className="stack">
                  {recentRecommendations.map((recommendation) => (
                    <article className="item evidenceItem" key={recommendation.id}>
                      <div><strong>{recommendation.risk_level}</strong><span>{excerpt(recommendation.recommendation_text)}</span></div>
                      <div><StatusPill state={recommendation.status} /><span>{recommendation.approval_required ? "approval" : "no approval"}</span></div>
                    </article>
                  ))}
                </div>
                {recentRecommendations.length === 0 ? <EmptyState label="No recommendations recorded yet." /> : null}
              </div>
	            </section>
	            <ImprovementExperimentReview improvements={improvements} experiments={experiments} />
	            <section className="panelInset" id="data-search">
              <h3>Search Your Data</h3>
              <p>Use Assistant for semantic retrieval now. Filter targets include sources, uploads, evidence, memory, and analysis records. Example question: "What did I upload today?"</p>
              <label className="inlineAction" htmlFor="tab-results">Open Results search</label>
            </section>
          </section>

          <section className="panel workflowSection tabContent" id="work-processing" data-tab-panel="work">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Work</p>
                <h2><HelpHeading term="workItem">Processing Status</HelpHeading></h2>
              </div>
              {[workItems.error, approvals.error, feedback.error, outcomes.error, reports.error, auditEvents.error].filter(Boolean).length > 0 ? (
                <span className="errorText">Some review or operations endpoints returned errors.</span>
              ) : null}
            </div>
            <section className="metrics compact" aria-label="Work and processing totals">
              <article><span>Queued</span><strong>{queuedWorkItems.length}</strong></article>
              <article><span>Running</span><strong>{runningWorkItems.length}</strong></article>
              <article><span>Completed</span><strong>{completedWorkItems.length}</strong></article>
              <article><span>Failed</span><strong>{failedWorkItems.length}</strong></article>
            </section>
            <div className="lifecycleFlow" aria-label="Processing pipeline">
              {["Raw Artifact", "Normalized Document", "Chunks", "Evidence", "Vector Memory", "Graph Memory"].map((step) => (
                <span key={step}>{step}</span>
              ))}
            </div>
            <p className="agentRuntimeReason">Background processing is ready. Supported queued work stays behind system checks and does not run arbitrary user input.</p>
            <section className="quad analysisGrid">
              <div>
                <div className="subHeader"><h3><HelpHeading term="workItem">Work Items</HelpHeading></h3>{workItems.error ? <span className="errorText">{workItems.error}</span> : null}</div>
                <div className="stack">
	                  {recentWorkItems.map((workItem) => {
	                    const guidance = workItemGuidance(workItem);
	                    const relatedIds = workItemRelatedIds(workItem);
	                    const dispatchVisibility = workItemDispatchVisibility(workItem);
	                    return (
	                      <article className="item evidenceItem workStatusItem" key={workItem.id} data-work-status-item>
	                        <div>
	                          <strong>{workItem.work_type}</strong>
	                          <span>Work item: {workItem.id}</span>
	                          <span>{guidance.outcome}</span>
	                          <dl className="workStatusIds" aria-label={`Dispatch visibility for ${workItem.id}`} data-work-dispatch-visibility>
	                            {dispatchVisibility.map((detail) => (
	                              <div key={`${workItem.id}-dispatch-${detail.label}`}>
	                                <dt>{detail.label}</dt>
	                                <dd>{detail.value}</dd>
	                              </div>
	                            ))}
	                          </dl>
	                          {relatedIds.length > 0 ? (
	                            <dl className="workStatusIds" aria-label={`Related records for ${workItem.id}`}>
                              {relatedIds.map((related) => (
                                <div key={`${workItem.id}-${related.label}`}>
                                  <dt>{related.label}</dt>
                                  <dd>{related.values.slice(0, 3).join(", ")}{related.values.length > 3 ? ` +${related.values.length - 3} more` : ""}</dd>
                                </div>
                              ))}
                            </dl>
                          ) : null}
                        </div>
                        <div>
                          <StatusPill state={workItem.status} />
                          <span>created {formatDate(workItem.created_at)}</span>
                          <span>updated {formatDate(workItem.updated_at ?? workItem.created_at)}</span>
                          <span>{guidance.next}</span>
                        </div>
                      </article>
                    );
                  })}
                </div>
                {recentWorkItems.length === 0 ? <EmptyState label="No work items recorded yet." /> : null}
              </div>
            </section>
            <details className="advancedPanel">
              <summary>Advanced: dispatch controls, work item IDs, and raw queue JSON</summary>
              <p>Use Advanced Route Console above for dispatch. Route: POST /work-items/:work_item_id/dispatch.</p>
              <pre>{JSON.stringify(workItems.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

          <section className="panel workflowSection tabContent" id="safety-audit" data-tab-panel="settings">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Settings</p>
                <h2>Safety, Approvals, And Policy</h2>
              </div>
              <StatusPill state="approval-gated" />
            </div>
            <section className="workflowTabs" aria-label="Safety and audit panels">
              <a href="#safety-overview">Overview</a>
              <a href="#approvals">Approvals</a>
              <a href="#audit-log">Audit Log</a>
              <a href="#safety-rules">Safety Rules</a>
              <a href="#safety-advanced">Advanced</a>
            </section>
            <section className="metrics compact" id="safety-overview" aria-label="Safety overview">
              <article><span>Pending approvals</span><strong>{pendingApprovals.length}</strong></article>
              <article><span>Blocked actions</span><strong>{blockedActions.length}</strong></article>
              <article><span>Approval-required actions</span><strong>{approvalRequiredActions.length}</strong></article>
              <article><span>External model policy</span><strong>blocked</strong></article>
            </section>
            <section className="quad analysisGrid">
              <div id="approvals">
                <div className="subHeader"><h3><HelpHeading term="approval">Approvals</HelpHeading></h3>{approvals.error ? <span className="errorText">{approvals.error}</span> : null}</div>
                <div className="messageMeta">
                  <StatusPill state={`${pendingApprovals.length}-pending`} />
                  <StatusPill state={`${approvedApprovals.length}-approved`} />
                  <StatusPill state={`${rejectedApprovals.length}-rejected`} />
                </div>
                <div className="stack">
                  {recentApprovals.map((approval) => (
                    <article className="item evidenceItem" key={approval.id}>
                      <div><strong>{approval.request_type}</strong><span>{approval.decision_reason ?? `requested by ${approval.requested_by_actor_id}`}</span></div>
                      <div><StatusPill state={approval.status} /><span>{approval.decided_by_actor_id ?? "undecided"}</span></div>
                    </article>
                  ))}
                </div>
                {recentApprovals.length === 0 ? <EmptyState label="No approvals recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="feedback">Feedback</HelpHeading></h3>{feedback.error ? <span className="errorText">{feedback.error}</span> : null}</div>
                <div className="stack">
                  {recentFeedback.map((event) => (
                    <article className="item evidenceItem" key={event.id}>
                      <div><strong>{event.label}</strong><span>{event.note ?? `${event.target_type} feedback`}</span></div>
                      <div><span>{event.actor_id}</span><span>{formatDate(event.created_at)}</span></div>
                    </article>
                  ))}
                </div>
                {recentFeedback.length === 0 ? <EmptyState label="No feedback recorded yet." /> : null}
              </div>

              <div>
                <div className="subHeader"><h3><HelpHeading term="outcome">Outcomes</HelpHeading></h3>{outcomes.error ? <span className="errorText">{outcomes.error}</span> : null}</div>
                <div className="stack">
                  {recentOutcomes.map((outcome) => (
                    <article className="item evidenceItem" key={outcome.id}>
                      <div><strong>{outcome.target_type}</strong><span>{outcome.summary ?? "Outcome recorded"}</span></div>
                      <div><StatusPill state={outcome.outcome_status} /><span>{formatDate(outcome.created_at)}</span></div>
                    </article>
                  ))}
                </div>
                {recentOutcomes.length === 0 ? <EmptyState label="No outcomes recorded yet." /> : null}
              </div>

              <div id="safety-rules">
                <div className="subHeader"><h3>Safety Rules</h3></div>
                <div className="stack">
                  <article className="item evidenceItem"><div><strong>Approval-required default</strong><span>System-changing actions require explicit local approval.</span></div><StatusPill state="enabled" /></article>
                  <article className="item evidenceItem"><div><strong>Allowed operation classes</strong><span>Read-only checks, retrieval preview, approved stack controls, approved collection.</span></div><StatusPill state="bounded" /></article>
                  <article className="item evidenceItem"><div><strong>External model policy</strong><span>Local-first evidence workflows do not send data to external models by default.</span></div><StatusPill state="blocked" /></article>
                  <article className="item evidenceItem"><div><strong>Runtime capability</strong><span>{agentCapabilities.data.runtime.reason ?? "Capability status is reported by the system runtime."}</span></div><StatusPill state={agentCapabilities.data.runtime.docker_control_available ? "runtime-ready" : "runtime-blocked"} /></article>
                </div>
              </div>

              <div id="audit-log">
                <div className="subHeader"><h3><HelpHeading term="auditEvent">Audit Log</HelpHeading></h3>{auditEvents.error ? <span className="errorText">{auditEvents.error}</span> : null}</div>
                <div className="fieldGuide">
                  <article><strong>Filters</strong><span>Actor, event type, source, work item, approval, and action filters are preserved as advanced audit controls.</span></article>
                </div>
                <div className="stack">
                  {recentAuditEvents.map((event) => (
                    <article className="item evidenceItem" key={event.id}>
                      <div><strong>{event.event_type}</strong><span>{event.resource_type ?? "resource"} event</span></div>
                      <div><StatusPill state={event.decision ?? "recorded"} /><span>{event.actor_id}</span></div>
                    </article>
                  ))}
                </div>
                {recentAuditEvents.length === 0 ? <EmptyState label="No audit events recorded yet." /> : null}
              </div>
            </section>
            <details className="advancedPanel" id="safety-advanced">
              <summary>Advanced: approval IDs, audit JSON, route filters, and raw safety records</summary>
              <pre>{JSON.stringify({ approvals: approvals.data.slice(0, 10), audit_events: auditEvents.data.slice(0, 10), actions: agentCapabilities.data.actions }, null, 2)}</pre>
            </details>
          </section>

          <section className="panel workflowSection tabContent" id="reports" data-tab-panel="results">
            <div className="panelHeader">
              <div>
                <p className="eyebrow">Results</p>
                <h2>Reports</h2>
              </div>
              {reports.error ? <span className="errorText">{reports.error}</span> : <StatusPill state={`${reports.data.length}-reports`} />}
            </div>
            <div className="fieldGuide">
              <article><strong>Report reason</strong><span>Everyday: "Create a summary of this uploaded bill." · Project: "Summarize the latest verification notes."</span></article>
            </div>
            <EvidenceFeedbackWorkflow evidenceItems={evidenceItems} evidenceAnswers={evidenceAnswers} reports={reports} workItems={workItems} feedback={feedback} outcomes={outcomes} improvements={improvements} />
            <BasicReportWorkflow reports={reports} evidenceCount={evidenceItems.data.length} documentCount={documents.data.length} chunkCount={chunks.data.length} />
            <section className="quad analysisGrid">
              <div>
                <div className="subHeader"><h3>Reports</h3>{reports.error ? <span className="errorText">{reports.error}</span> : null}</div>
                <div className="stack">
                  {recentReports.map((report) => (
                    <article className="item evidenceItem" key={report.id}>
                      <div><strong>{report.title}</strong><span>{report.report_type} · {report.id}</span></div>
                      <div><StatusPill state={report.status} /><span>{report.artifact_path ? "markdown artifact ready" : "metadata only"}</span></div>
                    </article>
                  ))}
                </div>
                {recentReports.length === 0 ? <EmptyState label="No reports recorded yet." /> : null}
              </div>
            </section>
            <details className="advancedPanel">
              <summary>Advanced: report render route, report IDs, output JSON, and export details</summary>
              <p>Use Advanced Route Console above for report create/render. Routes: POST /reports and POST /reports/:report_id/render.</p>
              <pre>{JSON.stringify(reports.data.slice(0, 10), null, 2)}</pre>
            </details>
          </section>

          <SettingsPanel envSettings={envSettings} />
        </section>
      </section>

      <aside className="rightContext" aria-label="IGY6 context">
        <section className="contextCard">
          <div className="panelHeader">
            <h2>Context</h2>
            <StatusPill state={health.data.status} />
          </div>
          {health.error ? <span className="errorText">{health.error}</span> : null}
          <div className="contextStats">
            <article><span><TermHelp term="source" label="Sources" /></span><strong>{sources.data.length}</strong></article>
            <article><span><TermHelp term="evidenceItem" label="Evidence" /></span><strong>{evidenceItems.data.length}</strong></article>
            <article><span><TermHelp term="workItem" label="Work queue" /></span><strong>{workItems.data.length}</strong></article>
            <article><span><TermHelp term="approval" label="Approvals" /></span><strong>{approvals.data.length}</strong></article>
          </div>
        </section>

        <section className="contextCard">
          <h2>Service Readiness</h2>
          <div className="checkList">
            {Object.entries(checks).map(([name, check]) => (
              <article className="checkRow" key={name}>
                <span>{name}</span>
                <StatusPill state={check.status} />
              </article>
            ))}
            {Object.keys(checks).length === 0 ? <EmptyState label="No readiness details returned." /> : null}
          </div>
        </section>

        <section className="contextCard">
          <h2><HelpHeading term="source">Recent Sources</HelpHeading></h2>
          <div className="stack">
            {sources.data.slice(0, 4).map((source) => (
              <article className="miniRecord" key={source.id}>
                <strong>{source.name}</strong>
                <span>{source.source_type} · {source.trust_level}</span>
              </article>
            ))}
          </div>
          {sources.data.length === 0 ? <EmptyState label="No sources yet." /> : null}
        </section>

        <section className="contextCard">
          <h2><HelpHeading term="auditEvent">Recent Audit</HelpHeading></h2>
          <div className="stack">
            {recentAuditEvents.map((event) => (
              <article className="miniRecord" key={event.id}>
                <strong>{event.event_type}</strong>
                <span>{event.decision ?? "recorded"} · {event.actor_id}</span>
              </article>
            ))}
          </div>
          {recentAuditEvents.length === 0 ? <EmptyState label="No audit events yet." /> : null}
        </section>

        <section className="contextCard">
          <h2>Method Review</h2>
          <div className="stack">
            <article className="miniRecord">
              <strong><TermHelp term="improvementItem" label="Improvement Item" /></strong>
              <span>Proposed tuning target; not execution.</span>
            </article>
            <article className="miniRecord">
              <strong><TermHelp term="experimentRun" label="Experiment Run" /></strong>
              <span>Experiment metadata; not active MLflow/Optuna execution.</span>
            </article>
          </div>
        </section>

        <section className="contextCard reminderCard">
          <h2>Uncertainty</h2>
          <p>Retrieval only reflects sources that have been registered, collected, normalized, chunked, and embedded. Missing or disabled sources are not evidence.</p>
        </section>
      </aside>
    </main>
  );
}
