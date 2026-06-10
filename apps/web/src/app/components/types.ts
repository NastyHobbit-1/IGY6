export type HealthResponse = {
  status: string;
  checks?: Record<string, { status: string; detail?: string }>;
};

export type SourcePermission = {
  id: string;
  allowed_operations: string[];
  approval_required: boolean;
  external_model_policy: string;
};

export type SourceRecord = {
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

export type CollectionRunRecord = {
  id: string;
  source_id: string | null;
  status: string;
  dry_run: boolean;
  requested_by_actor_id: string;
  created_at: string;
  summary_json: Record<string, unknown>;
};

export type RawArtifactRecord = {
  id: string;
  source_id: string | null;
  collection_run_id: string | null;
  content_hash: string;
  mime_type: string | null;
  size_bytes: number | null;
  created_at: string;
};

export type NormalizedDocumentRecord = {
  id: string;
  raw_artifact_id: string | null;
  source_id: string | null;
  title: string | null;
  document_type: string;
  language: string | null;
  sensitivity: string;
  created_at: string;
};

export type ChunkRecord = {
  id: string;
  document_id: string;
  chunk_index: number;
  embedding_status: string;
  created_at: string;
};

export type EvidenceItemRecord = {
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

export type EvidenceAnswerRecord = {
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

export type ClaimRecord = {
  id: string;
  claim_text: string;
  claim_type: string;
  status: string;
  evidence_ids?: string[];
  confidence: number | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

export type VectorCollectionStatus = {
  collection_name: string;
  exists: boolean;
  detail?: {
    tcp_reachable?: boolean;
    collection_existence_verified?: boolean;
    note?: string;
  };
};

export type GraphSchemaStatus = {
  constraints: Array<Record<string, unknown>>;
};

export type PatternRecord = {
  id: string;
  pattern_type: string;
  status: string;
  summary: string;
  evidence_ids?: string[];
  confidence: number | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
  updated_at?: string | null;
};

export type HypothesisRecord = {
  id: string;
  hypothesis_text: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

export type PredictionRecord = {
  id: string;
  prediction_text: string;
  expected_result: string;
  disproof_condition?: string | null;
  evidence_ids?: string[];
  status: string;
  confidence: number | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

export type RecommendationRecord = {
  id: string;
  recommendation_text: string;
  risk_level: string;
  approval_required: boolean;
  expected_result?: string | null;
  evidence_ids?: string[];
  status: string;
  confidence: number | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

export type CalibrationSummary = {
  schema_version: string;
  record_counts: {
    predictions: number;
    recommendations: number;
    total: number;
    evidence_linked: number;
    with_outcome: number;
  };
  outcome_counts: {
    correct: number;
    wrong: number;
    partial: number;
    useful: number;
    not_useful: number;
    inconclusive: number;
    total: number;
  };
  by_kind: {
    prediction: { records: number; outcomes: number };
    recommendation: { records: number; outcomes: number };
  };
  confidence_bands: Record<string, { records: number; outcomes: number }>;
  calibration_status: string;
  limitations?: string[];
  forecasting_engine: boolean;
  auto_execute_recommendations: boolean;
  advanced_calibration: boolean;
};

export type WorkItemRecord = {
  id: string;
  work_type: string;
  status: string;
  requested_by_actor_id: string;
  payload_json?: Record<string, unknown> | null;
  error_message: string | null;
  created_at: string;
  updated_at?: string | null;
};

export type AgentTaskPlanRecord = {
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

export type ApprovalRecord = {
  id: string;
  request_type: string;
  status: string;
  requested_by_actor_id: string;
  decided_by_actor_id: string | null;
  decision_reason: string | null;
  request_payload_json?: Record<string, unknown> | null;
  created_at: string;
};

export type FeedbackRecord = {
  id: string;
  target_type: string;
  target_id: string;
  label: string;
  actor_id: string;
  note: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

export type OutcomeRecord = {
  id: string;
  target_type: string;
  target_id: string;
  outcome_status: string;
  summary: string | null;
  metadata_json?: Record<string, unknown> | null;
  created_at: string;
};

export type ImprovementRecord = {
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

export type ExperimentRecord = {
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

export type ReportRecord = {
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

export type AuditEventRecord = {
  id: number;
  actor_id: string;
  event_type: string;
  decision: string | null;
  resource_type: string | null;
  resource_id: string | null;
  created_at: string;
};

export type EnvSettingRecord = {
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

export type EnvUnmanagedRecord = {
  key: string;
  masked_value: string;
  has_value: boolean;
  secret: boolean;
  read_only: boolean;
};

export type EnvSettingsResponse = {
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

export type AgentActionCapability = {
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

export type AgentCapabilitiesResponse = {
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
  policy?: {
    local_first: boolean;
    hosted_ai_enabled: boolean;
    external_model_policy: string;
    arbitrary_command_execution: boolean;
    prompt_injection_filter: string;
    approval_required_for_system_changing: boolean;
    blocked_request_classes: string[];
  };
};

export type ApiResult<T> = {
  data: T;
  error: string | null;
};

export type ConnectorContractStep = {
  key: string;
  label: string;
  requirement: string;
};

export type SourceConnectorStatus = {
  sourceType: string;
  status: string;
  defaultScope: string;
  dryRun: string;
  collect: string;
  sensitivity: string;
  cleanupAudit: string;
};

export type BrowserWebRouterImportType = {
  key: string;
  label: string;
  scopePrompt: string;
  collected: string;
  excluded: string;
  sensitivity: string;
};

export type MediaImportType = {
  key: string;
  label: string;
  status: string;
  acceptedInput: string;
  unsupportedReason: string;
  safeNext: string;
};

export type LocalProjectDiagnosticsMode = {
  key: string;
  label: string;
  scope: string;
  collect: string;
  excluded: string;
};

export type TermHelpContent = {
  title: string;
  explanation: string;
  manage: string;
  purpose: string;
  examples?: string;
  warning?: string;
};
