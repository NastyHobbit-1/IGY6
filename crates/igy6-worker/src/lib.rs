use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use igy6_chunking::{plan_document_chunks, ChunkPlan, ChunkingError, EvidencePlan};
use igy6_normalization::{
    build_normalized_document_ref, NormalizedDocumentInput, NormalizedDocumentRef, RawArtifactRef,
};
use igy6_vector_memory::{
    collection_status_request, embed_text_local, ensure_collection_request, upsert_points_request,
    ChunkVectorPoint, HttpMethod, HttpRequestPlan, QdrantSettings, VectorMemoryError,
};
use postgres::{Client, NoTls};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerError {
    NonUtf8Artifact,
    Chunking(ChunkingError),
    VectorMemory(VectorMemoryError),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUtf8Artifact => write!(
                formatter,
                "artifact is not UTF-8 text; this worker plan supports UTF-8 text normalization only"
            ),
            Self::Chunking(error) => write!(formatter, "{error}"),
            Self::VectorMemory(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for WorkerError {}

impl From<ChunkingError> for WorkerError {
    fn from(error: ChunkingError) -> Self {
        Self::Chunking(error)
    }
}

impl From<VectorMemoryError> for WorkerError {
    fn from(error: VectorMemoryError) -> Self {
        Self::VectorMemory(error)
    }
}

pub const WORKER_EMBEDDING_METHOD: &str = "local_hash_v1";
pub const WORKER_VECTOR_GENERATED_BY: &str = "DIFF-053";
pub const DEFAULT_DATABASE_URL: &str =
    "postgresql+psycopg://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence";
pub const DEFAULT_QDRANT_URL: &str = "http://qdrant:6333";
pub const DEFAULT_IGY6_DATA_ROOT: &str = "../IGY6_Data";
pub const DEFAULT_QDRANT_CHUNK_COLLECTION: &str = "igy6_chunks";
pub const DEFAULT_QDRANT_CHUNK_VECTOR_SIZE: usize = 384;
pub const DEFAULT_WORKER_CLAIM_LIMIT: usize = 1;
pub const DEFAULT_WORKER_POLL_INTERVAL_MS: u64 = 1000;
pub const MAX_WORKER_CLAIM_LIMIT: usize = 16;
pub const MIN_WORKER_POLL_INTERVAL_MS: u64 = 100;
pub const MAX_WORKER_POLL_INTERVAL_MS: u64 = 60000;
static GENERATED_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerTaskKind {
    CollectionNormalization,
    DocumentChunking,
    ChunkVectorUpsert,
}

impl WorkerTaskKind {
    pub fn from_work_type(work_type: &str) -> Option<Self> {
        match work_type {
            "collection_normalization" => Some(Self::CollectionNormalization),
            "document_chunking" => Some(Self::DocumentChunking),
            "chunk_vector_upsert" => Some(Self::ChunkVectorUpsert),
            _ => None,
        }
    }

    pub fn work_type(self) -> &'static str {
        match self {
            Self::CollectionNormalization => "collection_normalization",
            Self::DocumentChunking => "document_chunking",
            Self::ChunkVectorUpsert => "chunk_vector_upsert",
        }
    }

    pub fn celery_task_name(self) -> &'static str {
        match self {
            Self::CollectionNormalization => "collection.normalize_collection_run",
            Self::DocumentChunking => "evidence.generate_document_chunks",
            Self::ChunkVectorUpsert => "memory.vector.upsert_chunks",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueClaimError {
    UnsupportedWorkType(String),
    NotQueued(String),
    MissingIntentVerification,
    InvalidPayload(String),
    EmptyActorId,
    InvalidClaimLimit(usize),
}

impl fmt::Display for QueueClaimError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedWorkType(work_type) => {
                write!(formatter, "unsupported worker work type: {work_type}")
            }
            Self::NotQueued(status) => write!(formatter, "work item is not queued: {status}"),
            Self::MissingIntentVerification => {
                write!(formatter, "work item requires recorded intent verification")
            }
            Self::InvalidPayload(message) => write!(formatter, "{message}"),
            Self::EmptyActorId => write!(formatter, "claim actor id is required"),
            Self::InvalidClaimLimit(limit) => {
                write!(
                    formatter,
                    "claim limit must be between 1 and 16, got {limit}"
                )
            }
        }
    }
}

impl std::error::Error for QueueClaimError {}

#[derive(Debug, Clone, PartialEq)]
pub struct QueueClaimCandidate {
    pub id: String,
    pub work_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueClaimPlan {
    pub work_item_id: String,
    pub work_type: String,
    pub task_name: String,
    pub previous_status: String,
    pub next_status: String,
    pub claimed_by_actor_id: String,
    pub audit_event_type: String,
    pub audit_decision: String,
    pub execution_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueClaimQueryPlan {
    pub allowed_work_types: Vec<&'static str>,
    pub claim_limit: usize,
    pub select_sql: &'static str,
    pub update_sql: &'static str,
    pub audit_event_type: &'static str,
    pub execution_status: &'static str,
}

pub fn queue_claim_query_plan(claim_limit: usize) -> Result<QueueClaimQueryPlan, QueueClaimError> {
    if !(1..=16).contains(&claim_limit) {
        return Err(QueueClaimError::InvalidClaimLimit(claim_limit));
    }
    Ok(QueueClaimQueryPlan {
        allowed_work_types: vec![
            WorkerTaskKind::CollectionNormalization.work_type(),
            WorkerTaskKind::DocumentChunking.work_type(),
            WorkerTaskKind::ChunkVectorUpsert.work_type(),
        ],
        claim_limit,
        select_sql: "SELECT id, work_type, status, requested_by_actor_id, payload_json FROM work_items WHERE status = 'queued' AND work_type = ANY($1) ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT $2",
        update_sql: "UPDATE work_items SET status = 'running', error_message = NULL, updated_at = now() WHERE id = $1 AND status = 'queued'",
        audit_event_type: "work_item.claimed",
        execution_status: "claimed_without_execution",
    })
}

pub fn plan_queue_claim(
    candidate: QueueClaimCandidate,
    claimed_by_actor_id: &str,
) -> Result<QueueClaimPlan, QueueClaimError> {
    let claimed_by_actor_id = claimed_by_actor_id.trim();
    if claimed_by_actor_id.is_empty() {
        return Err(QueueClaimError::EmptyActorId);
    }
    if candidate.status != "queued" {
        return Err(QueueClaimError::NotQueued(candidate.status));
    }
    if !has_intent_verification(&candidate.payload_json) {
        return Err(QueueClaimError::MissingIntentVerification);
    }
    let task_kind = WorkerTaskKind::from_work_type(&candidate.work_type)
        .ok_or_else(|| QueueClaimError::UnsupportedWorkType(candidate.work_type.clone()))?;
    validate_claim_payload(task_kind, &candidate.payload_json)?;
    Ok(QueueClaimPlan {
        work_item_id: candidate.id,
        work_type: task_kind.work_type().to_string(),
        task_name: task_kind.celery_task_name().to_string(),
        previous_status: "queued".to_string(),
        next_status: "running".to_string(),
        claimed_by_actor_id: claimed_by_actor_id.to_string(),
        audit_event_type: "work_item.claimed".to_string(),
        audit_decision: "running".to_string(),
        execution_status: "claimed_without_execution".to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerRuntimeMode {
    Check,
    DryRun,
    Once,
    Help,
}

impl WorkerRuntimeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::DryRun => "dry-run",
            Self::Once => "once",
            Self::Help => "help",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerRuntimeError {
    UnknownArgument(String),
    MissingArgumentValue(&'static str),
    InvalidCanaryMode(String),
    InvalidClaimLimit(String),
    InvalidPollInterval(String),
    InvalidDatabaseUrl(String),
    InvalidQdrantUrl(String),
    InvalidDataRoot(String),
    InvalidVectorSize(String),
    InvalidCollectionName(String),
    LiveExecution(String),
}

impl fmt::Display for WorkerRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownArgument(argument) => write!(formatter, "unknown argument: {argument}"),
            Self::MissingArgumentValue(argument) => {
                write!(formatter, "missing value for argument: {argument}")
            }
            Self::InvalidCanaryMode(message)
            | Self::InvalidClaimLimit(message)
            | Self::InvalidPollInterval(message)
            | Self::InvalidDatabaseUrl(message)
            | Self::InvalidQdrantUrl(message)
            | Self::InvalidDataRoot(message)
            | Self::InvalidVectorSize(message)
            | Self::InvalidCollectionName(message)
            | Self::LiveExecution(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for WorkerRuntimeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerRuntimeArgs {
    pub mode: WorkerRuntimeMode,
    pub claim_limit: usize,
    pub poll_interval_ms: u64,
    pub explicit_live_execution: bool,
    pub canary_live: bool,
    pub canary_work_item_id: Option<String>,
}

impl Default for WorkerRuntimeArgs {
    fn default() -> Self {
        Self {
            mode: WorkerRuntimeMode::Check,
            claim_limit: DEFAULT_WORKER_CLAIM_LIMIT,
            poll_interval_ms: DEFAULT_WORKER_POLL_INTERVAL_MS,
            explicit_live_execution: false,
            canary_live: false,
            canary_work_item_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerRuntimeConfig {
    pub database_url: String,
    pub qdrant_url: String,
    pub igy6_data_root: String,
    pub qdrant_chunk_collection: String,
    pub qdrant_chunk_vector_size: usize,
    pub claim_limit: usize,
    pub poll_interval_ms: u64,
    pub live_execution_enabled: bool,
}

impl WorkerRuntimeConfig {
    pub fn safe_default() -> Self {
        Self {
            database_url: DEFAULT_DATABASE_URL.to_string(),
            qdrant_url: DEFAULT_QDRANT_URL.to_string(),
            igy6_data_root: DEFAULT_IGY6_DATA_ROOT.to_string(),
            qdrant_chunk_collection: DEFAULT_QDRANT_CHUNK_COLLECTION.to_string(),
            qdrant_chunk_vector_size: DEFAULT_QDRANT_CHUNK_VECTOR_SIZE,
            claim_limit: DEFAULT_WORKER_CLAIM_LIMIT,
            poll_interval_ms: DEFAULT_WORKER_POLL_INTERVAL_MS,
            live_execution_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkerRuntimePlan {
    pub mode: WorkerRuntimeMode,
    pub status: String,
    pub mutates_runtime_data: bool,
    pub live_execution_enabled: bool,
    pub claim_query: QueueClaimQueryPlan,
    pub allowed_work_types: Vec<&'static str>,
    pub planned_steps: Vec<String>,
    pub blocked_side_effects: Vec<String>,
    pub canary_plan: Option<WorkerCanaryPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkerLiveCanaryResult {
    pub service: &'static str,
    pub diff: &'static str,
    pub mode: &'static str,
    pub status: String,
    pub work_item_id: String,
    pub work_type: Option<String>,
    pub result_state: String,
    pub mutates_runtime_data: bool,
    pub live_execution_enabled: bool,
    pub side_effects_executed: Vec<String>,
    pub side_effects_planned: Vec<String>,
    pub error_message: Option<String>,
    pub output_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerCanaryPlan {
    pub work_item_id: String,
    pub status: String,
    pub max_jobs: usize,
    pub supported_result_states: Vec<&'static str>,
    pub side_effects_executed: Vec<&'static str>,
    pub side_effects_planned: Vec<WorkerSideEffectPlan>,
    pub rollback_posture: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerSideEffectPlan {
    pub name: &'static str,
    pub execution_status: &'static str,
    pub verification: &'static str,
}

pub fn worker_runtime_help() -> &'static str {
    "igy6-worker\n\nUsage:\n  igy6-worker [--check]\n  igy6-worker --dry-run [--claim-limit N] [--poll-interval-ms MS]\n  igy6-worker --once [--claim-limit 1]\n  IGY6_WORKER_LIVE_CANARY=DIFF-148 igy6-worker --once --canary-live --canary-work-item ID\n  igy6-worker --help\n\nModes:\n  --check            Validate safe runtime configuration without touching runtime data. This is the default.\n  --dry-run          Plan queue polling and one bounded claim batch without DB, artifact, audit, or Qdrant side effects.\n  --once             Plan a single bounded worker iteration without live execution.\n  --canary-live      Opt-in DIFF-149 live canary; bounded to one named work item and requires IGY6_WORKER_LIVE_CANARY=DIFF-148.\n  --canary-work-item Work item id for the canary gate.\n  --claim-limit N    Bounded claim limit, 1 through 16.\n  --poll-interval-ms Bounded modeled poll interval, 100 through 60000.\n\nDIFF-149 safety: default mode is non-mutating, canary-live is explicit and one-job bounded, Python/Celery worker and beat remain active, and Rust-only runtime is not claimed.\n"
}

pub fn parse_worker_runtime_args<I, S>(args: I) -> Result<WorkerRuntimeArgs, WorkerRuntimeError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut parsed = WorkerRuntimeArgs::default();
    let mut iterator = args.into_iter();
    while let Some(argument) = iterator.next() {
        match argument.as_ref() {
            "--help" | "-h" => parsed.mode = WorkerRuntimeMode::Help,
            "--check" => parsed.mode = WorkerRuntimeMode::Check,
            "--dry-run" => parsed.mode = WorkerRuntimeMode::DryRun,
            "--once" => parsed.mode = WorkerRuntimeMode::Once,
            "--canary-live" => {
                parsed.canary_live = true;
                parsed.explicit_live_execution = true;
            }
            "--canary-work-item" => {
                let value = iterator
                    .next()
                    .ok_or(WorkerRuntimeError::MissingArgumentValue(
                        "--canary-work-item",
                    ))?;
                let value = value.as_ref().trim();
                if value.is_empty() || value.contains(char::is_whitespace) {
                    return Err(WorkerRuntimeError::InvalidCanaryMode(
                        "--canary-work-item must be a non-empty single token".to_string(),
                    ));
                }
                parsed.canary_work_item_id = Some(value.to_string());
            }
            "--claim-limit" => {
                let value = iterator
                    .next()
                    .ok_or(WorkerRuntimeError::MissingArgumentValue("--claim-limit"))?;
                parsed.claim_limit = parse_claim_limit(value.as_ref())?;
            }
            "--poll-interval-ms" => {
                let value = iterator
                    .next()
                    .ok_or(WorkerRuntimeError::MissingArgumentValue(
                        "--poll-interval-ms",
                    ))?;
                parsed.poll_interval_ms = parse_poll_interval_ms(value.as_ref())?;
            }
            other => return Err(WorkerRuntimeError::UnknownArgument(other.to_string())),
        }
    }
    if matches!(parsed.mode, WorkerRuntimeMode::Once) && parsed.claim_limit != 1 {
        return Err(WorkerRuntimeError::InvalidClaimLimit(
            "--once requires claim limit 1".to_string(),
        ));
    }
    if parsed.canary_live {
        if !matches!(parsed.mode, WorkerRuntimeMode::Once) {
            return Err(WorkerRuntimeError::InvalidCanaryMode(
                "--canary-live requires --once".to_string(),
            ));
        }
        if parsed.claim_limit != 1 {
            return Err(WorkerRuntimeError::InvalidCanaryMode(
                "--canary-live is bounded to claim limit 1".to_string(),
            ));
        }
        if parsed.canary_work_item_id.is_none() {
            return Err(WorkerRuntimeError::InvalidCanaryMode(
                "--canary-live requires --canary-work-item".to_string(),
            ));
        }
    } else if parsed.canary_work_item_id.is_some() {
        return Err(WorkerRuntimeError::InvalidCanaryMode(
            "--canary-work-item requires --canary-live".to_string(),
        ));
    }
    Ok(parsed)
}

pub fn validate_worker_runtime_config(
    config: WorkerRuntimeConfig,
) -> Result<WorkerRuntimeConfig, WorkerRuntimeError> {
    validate_database_url(&config.database_url)?;
    validate_qdrant_url(&config.qdrant_url)?;
    validate_data_root(&config.igy6_data_root)?;
    validate_collection_name(&config.qdrant_chunk_collection)?;
    if config.qdrant_chunk_vector_size == 0 {
        return Err(WorkerRuntimeError::InvalidVectorSize(
            "QDRANT_CHUNK_VECTOR_SIZE must be at least 1".to_string(),
        ));
    }
    if !(1..=MAX_WORKER_CLAIM_LIMIT).contains(&config.claim_limit) {
        return Err(WorkerRuntimeError::InvalidClaimLimit(format!(
            "WORKER_CLAIM_LIMIT must be between 1 and {MAX_WORKER_CLAIM_LIMIT}"
        )));
    }
    if !(MIN_WORKER_POLL_INTERVAL_MS..=MAX_WORKER_POLL_INTERVAL_MS)
        .contains(&config.poll_interval_ms)
    {
        return Err(WorkerRuntimeError::InvalidPollInterval(format!(
            "WORKER_POLL_INTERVAL_MS must be between {MIN_WORKER_POLL_INTERVAL_MS} and {MAX_WORKER_POLL_INTERVAL_MS}"
        )));
    }
    Ok(config)
}

pub fn plan_worker_runtime(
    args: WorkerRuntimeArgs,
    config: WorkerRuntimeConfig,
) -> Result<WorkerRuntimePlan, WorkerRuntimeError> {
    let config = validate_worker_runtime_config(config)?;
    let claim_query = queue_claim_query_plan(args.claim_limit).map_err(|error| {
        WorkerRuntimeError::InvalidClaimLimit(format!("invalid claim limit: {error}"))
    })?;
    let planned_steps = match args.mode {
        WorkerRuntimeMode::Help => vec!["render help text".to_string()],
        WorkerRuntimeMode::Check => vec![
            "validate DATABASE_URL, QDRANT_URL, IGY6_DATA_ROOT, Qdrant collection, vector size, claim limit, and poll interval".to_string(),
            "render non-mutating status output".to_string(),
        ],
        WorkerRuntimeMode::DryRun => vec![
            "build bounded queued-work SELECT plan".to_string(),
            "build bounded claim UPDATE plan".to_string(),
            "report supported job families without executing them".to_string(),
        ],
        WorkerRuntimeMode::Once => vec![
            "build one-job queued-work SELECT plan".to_string(),
            "build one-job claim UPDATE plan".to_string(),
            "stop before DB, artifact, audit, or Qdrant side effects".to_string(),
        ],
    };
    let canary_plan = if args.canary_live {
        let work_item_id = args.canary_work_item_id.clone().ok_or_else(|| {
            WorkerRuntimeError::InvalidCanaryMode(
                "--canary-live requires --canary-work-item".to_string(),
            )
        })?;
        Some(plan_worker_canary(&work_item_id))
    } else {
        None
    };
    Ok(WorkerRuntimePlan {
        mode: args.mode,
        status: if args.canary_live && config.live_execution_enabled {
            "canary_live_ready".to_string()
        } else if args.canary_live {
            "canary_ready_side_effects_planned".to_string()
        } else {
            "planned_without_execution".to_string()
        },
        mutates_runtime_data: false,
        live_execution_enabled: config.live_execution_enabled && args.explicit_live_execution,
        allowed_work_types: claim_query.allowed_work_types.clone(),
        claim_query,
        planned_steps,
        blocked_side_effects: vec![
            "PostgreSQL connection".to_string(),
            "runtime queue mutation".to_string(),
            "artifact store reads".to_string(),
            "audit writes".to_string(),
            "Qdrant HTTP calls".to_string(),
            "Celery or beat control".to_string(),
            "arbitrary shell command execution".to_string(),
        ],
        canary_plan,
    })
}

pub fn plan_worker_canary(work_item_id: &str) -> WorkerCanaryPlan {
    WorkerCanaryPlan {
        work_item_id: work_item_id.to_string(),
        status: "side_effects_planned_not_executed".to_string(),
        max_jobs: 1,
        supported_result_states: vec!["claimed", "skipped", "completed", "failed", "unsupported"],
        side_effects_executed: Vec::new(),
        side_effects_planned: vec![
            WorkerSideEffectPlan {
                name: "postgres_work_item_claim",
                execution_status: "planned",
                verification: "work_items row transitions queued to running for one canary id",
            },
            WorkerSideEffectPlan {
                name: "postgres_job_family_writes",
                execution_status: "planned",
                verification: "normalized_documents/chunks/evidence_items/chained work_items match parity contract",
            },
            WorkerSideEffectPlan {
                name: "audit_events",
                execution_status: "planned",
                verification: "audit_events contains claimed/completed/failed event for canary correlation id",
            },
            WorkerSideEffectPlan {
                name: "artifact_store_read",
                execution_status: "planned",
                verification: "artifact path remains relative and resolves under IGY6_DATA_ROOT artifact store",
            },
            WorkerSideEffectPlan {
                name: "qdrant_collection_and_points",
                execution_status: "planned",
                verification: "Qdrant collection exists and point ids match canary chunk ids",
            },
        ],
        rollback_posture:
            "Do not remove Python/Celery; inspect canary work item and audit trail before any retry"
                .to_string(),
    }
}

pub fn render_worker_runtime_status(
    plan: &WorkerRuntimePlan,
    config: &WorkerRuntimeConfig,
) -> Value {
    json!({
        "service": "igy6-worker",
        "diff": "DIFF-149",
        "mode": plan.mode.as_str(),
        "status": plan.status,
        "mutates_runtime_data": plan.mutates_runtime_data,
        "live_execution_enabled": plan.live_execution_enabled,
        "python_celery_worker_required": true,
        "python_celery_beat_required": true,
        "rust_only_runtime_claimed": false,
        "claim_limit": config.claim_limit,
        "poll_interval_ms": config.poll_interval_ms,
        "qdrant_chunk_collection": config.qdrant_chunk_collection,
        "qdrant_chunk_vector_size": config.qdrant_chunk_vector_size,
        "database_url_configured": !config.database_url.trim().is_empty(),
        "qdrant_url_configured": !config.qdrant_url.trim().is_empty(),
        "igy6_data_root_configured": !config.igy6_data_root.trim().is_empty(),
        "allowed_work_types": plan.allowed_work_types,
        "planned_steps": plan.planned_steps,
        "blocked_side_effects": plan.blocked_side_effects,
        "canary": plan.canary_plan.as_ref().map(|canary| json!({
            "work_item_id": canary.work_item_id,
            "status": canary.status,
            "max_jobs": canary.max_jobs,
            "supported_result_states": canary.supported_result_states,
            "side_effects_executed": canary.side_effects_executed,
            "side_effects_planned": canary.side_effects_planned.iter().map(|effect| json!({
                "name": effect.name,
                "execution_status": effect.execution_status,
                "verification": effect.verification,
            })).collect::<Vec<Value>>(),
            "rollback_posture": canary.rollback_posture,
        })),
    })
}

pub fn render_worker_live_canary_result(
    result: &WorkerLiveCanaryResult,
    config: &WorkerRuntimeConfig,
) -> Value {
    json!({
        "service": result.service,
        "diff": result.diff,
        "mode": result.mode,
        "status": result.status,
        "work_item_id": result.work_item_id,
        "work_type": result.work_type,
        "result_state": result.result_state,
        "mutates_runtime_data": result.mutates_runtime_data,
        "live_execution_enabled": result.live_execution_enabled,
        "python_celery_worker_required": true,
        "python_celery_beat_required": true,
        "rust_only_runtime_claimed": false,
        "qdrant_chunk_collection": config.qdrant_chunk_collection,
        "qdrant_chunk_vector_size": config.qdrant_chunk_vector_size,
        "database_url_configured": !config.database_url.trim().is_empty(),
        "qdrant_url_configured": !config.qdrant_url.trim().is_empty(),
        "igy6_data_root_configured": !config.igy6_data_root.trim().is_empty(),
        "side_effects_executed": result.side_effects_executed,
        "side_effects_planned": result.side_effects_planned,
        "error_message": result.error_message,
        "output": result.output_json,
    })
}

pub fn execute_worker_live_canary(
    args: &WorkerRuntimeArgs,
    config: &WorkerRuntimeConfig,
) -> Result<WorkerLiveCanaryResult, WorkerRuntimeError> {
    if !args.canary_live || !matches!(args.mode, WorkerRuntimeMode::Once) || args.claim_limit != 1 {
        return Err(WorkerRuntimeError::InvalidCanaryMode(
            "live execution requires --once --canary-live with claim limit 1".to_string(),
        ));
    }
    if !config.live_execution_enabled {
        return Err(WorkerRuntimeError::InvalidCanaryMode(
            "live execution requires IGY6_WORKER_LIVE_CANARY=DIFF-148".to_string(),
        ));
    }
    let work_item_id = args.canary_work_item_id.as_deref().ok_or_else(|| {
        WorkerRuntimeError::InvalidCanaryMode(
            "live execution requires --canary-work-item".to_string(),
        )
    })?;
    let validated_config = validate_worker_runtime_config(config.clone())?;
    execute_worker_live_canary_inner(work_item_id, &validated_config)
}

fn execute_worker_live_canary_inner(
    work_item_id: &str,
    config: &WorkerRuntimeConfig,
) -> Result<WorkerLiveCanaryResult, WorkerRuntimeError> {
    let postgres_url = postgres_client_url(&config.database_url);
    let mut client = Client::connect(&postgres_url, NoTls).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("failed to connect to PostgreSQL: {error}"))
    })?;

    let Some(claimed) = claim_one_canary_work_item(&mut client, work_item_id)? else {
        return Ok(live_result(
            work_item_id,
            None,
            "skipped",
            false,
            vec![],
            vec![],
            Some("work item was not found, was locked, or was not queued".to_string()),
            json!({}),
        ));
    };

    let execution = match claimed.task_kind {
        WorkerTaskKind::CollectionNormalization => {
            execute_collection_normalization_canary(&mut client, &claimed, config)
        }
        WorkerTaskKind::DocumentChunking => {
            execute_document_chunking_canary(&mut client, &claimed, config)
        }
        WorkerTaskKind::ChunkVectorUpsert => {
            execute_chunk_vector_upsert_canary(&mut client, &claimed, config)
        }
    };

    match execution {
        Ok(mut result) => {
            result
                .side_effects_executed
                .insert(0, "audit_work_item_started".to_string());
            result
                .side_effects_executed
                .insert(0, "audit_work_item_claimed".to_string());
            result
                .side_effects_executed
                .insert(0, "postgres_work_item_claim".to_string());
            Ok(result)
        }
        Err(error) => {
            let error_message = error.to_string();
            mark_canary_failed(&mut client, &claimed, &error_message)?;
            Ok(live_result(
                &claimed.work_item_id,
                Some(claimed.task_kind.work_type().to_string()),
                "failed",
                true,
                vec![
                    "postgres_work_item_claim".to_string(),
                    "audit_work_item_claimed".to_string(),
                    "audit_work_item_started".to_string(),
                    "postgres_work_item_failed".to_string(),
                    "audit_worker_failure".to_string(),
                ],
                vec![],
                Some(error_message),
                json!({}),
            ))
        }
    }
}

#[derive(Debug, Clone)]
struct ClaimedWorkItem {
    work_item_id: String,
    task_kind: WorkerTaskKind,
    requested_by_actor_id: String,
    payload_json: Value,
}

fn claim_one_canary_work_item(
    client: &mut Client,
    work_item_id: &str,
) -> Result<Option<ClaimedWorkItem>, WorkerRuntimeError> {
    let mut transaction = client.transaction().map_err(live_error)?;
    let row = transaction
        .query_opt(
            "SELECT id, work_type, status, requested_by_actor_id, payload_json FROM work_items WHERE id = $1 FOR UPDATE SKIP LOCKED",
            &[&work_item_id],
        )
        .map_err(live_error)?;
    let Some(row) = row else {
        transaction.commit().map_err(live_error)?;
        return Ok(None);
    };
    let candidate = QueueClaimCandidate {
        id: row.get("id"),
        work_type: row.get("work_type"),
        status: row.get("status"),
        requested_by_actor_id: row.get("requested_by_actor_id"),
        payload_json: row.get("payload_json"),
    };
    if candidate.status != "queued" {
        transaction.commit().map_err(live_error)?;
        return Ok(None);
    }
    let claim_plan = plan_queue_claim(candidate.clone(), &candidate.requested_by_actor_id)
        .map_err(|error| {
            WorkerRuntimeError::LiveExecution(format!("canary claim rejected: {error}"))
        })?;
    transaction
        .execute(
            "UPDATE work_items SET status = 'running', error_message = NULL, updated_at = now() WHERE id = $1 AND status = 'queued'",
            &[&claim_plan.work_item_id],
        )
        .map_err(live_error)?;
    insert_audit_event_tx(
        &mut transaction,
        &candidate.requested_by_actor_id,
        "work_item.claimed",
        "running",
        "work_item",
        &claim_plan.work_item_id,
        &claim_plan.work_item_id,
        json!({
            "work_type": claim_plan.work_type,
            "task_name": claim_plan.task_name,
            "generated_by": "DIFF-149",
        }),
    )?;
    insert_audit_event_tx(
        &mut transaction,
        &candidate.requested_by_actor_id,
        "work_item.started",
        "running",
        "work_item",
        &claim_plan.work_item_id,
        &claim_plan.work_item_id,
        json!({
            "work_type": claim_plan.work_type,
            "generated_by": "DIFF-149",
        }),
    )?;
    transaction.commit().map_err(live_error)?;

    Ok(Some(ClaimedWorkItem {
        work_item_id: claim_plan.work_item_id,
        task_kind: WorkerTaskKind::from_work_type(&claim_plan.work_type).expect("validated"),
        requested_by_actor_id: candidate.requested_by_actor_id,
        payload_json: candidate.payload_json,
    }))
}

fn execute_collection_normalization_canary(
    client: &mut Client,
    claimed: &ClaimedWorkItem,
    config: &WorkerRuntimeConfig,
) -> Result<WorkerLiveCanaryResult, WorkerRuntimeError> {
    let collection_run_id = required_payload_string(&claimed.payload_json, "collection_run_id")?;
    let raw_artifact_ids =
        required_payload_string_array(&claimed.payload_json, "raw_artifact_ids")?;
    let rows = client
        .query(
            "SELECT id, source_id, collection_run_id, content_hash, storage_path, metadata_json FROM raw_artifacts WHERE id = ANY($1)",
            &[&raw_artifact_ids],
        )
        .map_err(live_error)?;
    let mut raw_artifacts = Vec::new();
    for row in rows {
        let storage_path: String = row.get("storage_path");
        raw_artifacts.push(RawArtifactRecord {
            id: row.get("id"),
            source_id: row
                .get::<_, Option<String>>("source_id")
                .unwrap_or_default(),
            collection_run_id: row
                .get::<_, Option<String>>("collection_run_id")
                .unwrap_or_default(),
            content_hash: row.get("content_hash"),
            storage_path: storage_path.clone(),
            metadata_json: row.get("metadata_json"),
            bytes: read_artifact_bytes_under_data_root(&config.igy6_data_root, &storage_path)?,
        });
    }
    let collection_run = client
        .query_opt(
            "SELECT id FROM collection_runs WHERE id = $1",
            &[&collection_run_id],
        )
        .map_err(live_error)?
        .map(|row| CollectionRunRecord { id: row.get("id") });
    let existing_documents = client
        .query(
            "SELECT id, raw_artifact_id FROM normalized_documents WHERE raw_artifact_id = ANY($1)",
            &[&raw_artifact_ids],
        )
        .map_err(live_error)?
        .into_iter()
        .map(|row| ExistingNormalizedDocument {
            id: row.get("id"),
            raw_artifact_id: row.get("raw_artifact_id"),
        })
        .collect::<Vec<_>>();
    let existing_raw_artifact_ids: BTreeSet<String> = existing_documents
        .iter()
        .map(|document| document.raw_artifact_id.clone())
        .collect();
    let generated_document_ids = raw_artifact_ids
        .iter()
        .filter(|id| !existing_raw_artifact_ids.contains(*id))
        .map(|id| GeneratedDocumentId {
            raw_artifact_id: id.clone(),
            document_id: generated_id("document"),
        })
        .collect::<Vec<_>>();
    let plan = plan_collection_normalization_execution(CollectionNormalizationExecutionInput {
        work_item: Some(CollectionNormalizationWorkItem {
            id: claimed.work_item_id.clone(),
            work_type: claimed.task_kind.work_type().to_string(),
            status: "running".to_string(),
            requested_by_actor_id: claimed.requested_by_actor_id.clone(),
            payload_json: claimed.payload_json.clone(),
        }),
        requested_collection_run_id: collection_run_id.clone(),
        requested_raw_artifact_ids: raw_artifact_ids.clone(),
        collection_run,
        raw_artifacts,
        existing_documents,
        generated_document_ids,
    })
    .map_err(|error| WorkerRuntimeError::LiveExecution(error.to_string()))?;

    let mut transaction = client.transaction().map_err(live_error)?;
    for document in &plan.normalized_documents {
        transaction.execute(
            "INSERT INTO normalized_documents (id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json) VALUES ($1, $2, $3, $4, 'text', NULL, $5, 'internal', $6)",
            &[&document.id, &document.raw_artifact_id, &document.source_id, &document.title, &document.text_content, &document.metadata_json],
        ).map_err(live_error)?;
    }
    let chained_id = if let Some(chained) = &plan.document_chunking_work_item {
        let id = generated_id("work-item");
        transaction.execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json) VALUES ($1, 'document_chunking', 'queued', $2, $3)",
            &[&id, &chained.requested_by_actor_id, &chained.payload_json],
        ).map_err(live_error)?;
        insert_audit_event_tx(
            &mut transaction,
            &chained.audit_event.actor_id,
            &chained.audit_event.event_type,
            &chained.audit_event.decision,
            &chained.audit_event.resource_type,
            &id,
            &chained.audit_event.correlation_id,
            replace_placeholder_id(&chained.audit_event.details_json, &id),
        )?;
        Some(id)
    } else {
        None
    };
    transaction
        .execute(
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
            &[&claimed.work_item_id],
        )
        .map_err(live_error)?;
    insert_audit_event_tx(
        &mut transaction,
        &plan.completion_audit_event.actor_id,
        &plan.completion_audit_event.event_type,
        &plan.completion_audit_event.decision,
        &plan.completion_audit_event.resource_type,
        &plan.completion_audit_event.resource_id,
        &plan.completion_audit_event.correlation_id,
        replace_placeholder_id(
            &plan.completion_audit_event.details_json,
            chained_id.as_deref().unwrap_or(""),
        ),
    )?;
    transaction.commit().map_err(live_error)?;

    Ok(live_result(
        &claimed.work_item_id,
        Some(claimed.task_kind.work_type().to_string()),
        "completed",
        true,
        vec![
            "artifact_store_read".to_string(),
            "postgres_normalized_document_writes".to_string(),
            "postgres_chained_work_item_write".to_string(),
            "postgres_work_item_completed".to_string(),
            "audit_worker_success".to_string(),
        ],
        vec!["qdrant_collection_and_points".to_string()],
        None,
        json!({
            "created_document_ids": plan.normalized_documents.iter().map(|document| document.id.clone()).collect::<Vec<_>>(),
            "skipped_raw_artifact_ids": plan.skipped_raw_artifact_ids,
            "document_chunking_work_item_id": chained_id,
        }),
    ))
}

fn execute_document_chunking_canary(
    client: &mut Client,
    claimed: &ClaimedWorkItem,
    _config: &WorkerRuntimeConfig,
) -> Result<WorkerLiveCanaryResult, WorkerRuntimeError> {
    let document_ids = payload_document_ids(&claimed.payload_json)?;
    let chunk_size = claimed
        .payload_json
        .get("chunk_size")
        .and_then(Value::as_u64)
        .unwrap_or(1000) as usize;
    let documents = client
        .query(
            "SELECT id, source_id, text_content FROM normalized_documents WHERE id = ANY($1)",
            &[&document_ids],
        )
        .map_err(live_error)?
        .into_iter()
        .map(|row| NormalizedDocumentRecord {
            id: row.get("id"),
            source_id: row.get("source_id"),
            text_content: row.get("text_content"),
        })
        .collect::<Vec<_>>();
    let existing_chunks = client
        .query(
            "SELECT id, document_id FROM chunks WHERE document_id = ANY($1)",
            &[&document_ids],
        )
        .map_err(live_error)?
        .into_iter()
        .map(|row| ExistingChunkRecord {
            id: row.get("id"),
            document_id: row.get("document_id"),
        })
        .collect::<Vec<_>>();
    let documents_with_chunks: BTreeSet<String> = existing_chunks
        .iter()
        .map(|chunk| chunk.document_id.clone())
        .collect();
    let mut generated_chunk_ids = Vec::new();
    let mut generated_evidence_ids = Vec::new();
    for document in &documents {
        if documents_with_chunks.contains(&document.id) || document.text_content.is_empty() {
            continue;
        }
        let chunking_plan = plan_document_chunks(
            &document.id,
            document.source_id.as_deref(),
            &document.text_content,
            chunk_size,
        )
        .map_err(|error| WorkerRuntimeError::LiveExecution(error.to_string()))?;
        for chunk in chunking_plan.chunks {
            generated_chunk_ids.push(GeneratedChunkId {
                document_id: document.id.clone(),
                chunk_index: chunk.chunk_index,
                chunk_id: generated_id("chunk"),
            });
            generated_evidence_ids.push(GeneratedEvidenceId {
                document_id: document.id.clone(),
                chunk_index: chunk.chunk_index,
                evidence_id: generated_id("evidence"),
            });
        }
    }
    let plan = plan_document_chunking_execution(DocumentChunkingExecutionInput {
        work_item: Some(DocumentChunkingWorkItem {
            id: claimed.work_item_id.clone(),
            work_type: claimed.task_kind.work_type().to_string(),
            status: "running".to_string(),
            requested_by_actor_id: claimed.requested_by_actor_id.clone(),
            payload_json: claimed.payload_json.clone(),
        }),
        requested_document_ids: document_ids.clone(),
        chunk_size,
        documents,
        existing_chunks,
        generated_chunk_ids,
        generated_evidence_ids,
    })
    .map_err(|error| WorkerRuntimeError::LiveExecution(error.to_string()))?;

    let mut transaction = client.transaction().map_err(live_error)?;
    for chunk in &plan.chunks {
        transaction.execute(
            "INSERT INTO chunks (id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json) VALUES ($1, $2, $3, $4, $5, 'not_started', $6)",
            &[&chunk.id, &chunk.document_id, &(chunk.chunk_index as i32), &chunk.text_content, &chunk.location_json, &chunk.metadata_json],
        ).map_err(live_error)?;
    }
    for evidence in &plan.evidence_items {
        transaction.execute(
            "INSERT INTO evidence_items (id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json) VALUES ($1, $2, $3, $4, 'document_chunk', $5, NULL, NULL, $6)",
            &[&evidence.id, &evidence.source_id, &evidence.document_id, &evidence.chunk_id, &evidence.statement, &evidence.metadata_json],
        ).map_err(live_error)?;
    }
    let chained_id = if let Some(chained) = &plan.chunk_vector_upsert_work_item {
        let id = generated_id("work-item");
        transaction.execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json) VALUES ($1, 'chunk_vector_upsert', 'queued', $2, $3)",
            &[&id, &chained.requested_by_actor_id, &chained.payload_json],
        ).map_err(live_error)?;
        insert_audit_event_tx(
            &mut transaction,
            &chained.audit_event.actor_id,
            &chained.audit_event.event_type,
            &chained.audit_event.decision,
            &chained.audit_event.resource_type,
            &id,
            &chained.audit_event.correlation_id,
            replace_placeholder_id(&chained.audit_event.details_json, &id),
        )?;
        Some(id)
    } else {
        None
    };
    transaction
        .execute(
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
            &[&claimed.work_item_id],
        )
        .map_err(live_error)?;
    insert_audit_event_tx(
        &mut transaction,
        &plan.completion_audit_event.actor_id,
        &plan.completion_audit_event.event_type,
        &plan.completion_audit_event.decision,
        &plan.completion_audit_event.resource_type,
        &plan.completion_audit_event.resource_id,
        &plan.completion_audit_event.correlation_id,
        replace_placeholder_id(
            &plan.completion_audit_event.details_json,
            chained_id.as_deref().unwrap_or(""),
        ),
    )?;
    transaction.commit().map_err(live_error)?;

    Ok(live_result(
        &claimed.work_item_id,
        Some(claimed.task_kind.work_type().to_string()),
        "completed",
        true,
        vec![
            "postgres_chunk_writes".to_string(),
            "postgres_evidence_item_writes".to_string(),
            "postgres_chained_work_item_write".to_string(),
            "postgres_work_item_completed".to_string(),
            "audit_worker_success".to_string(),
        ],
        vec![
            "artifact_store_read".to_string(),
            "qdrant_collection_and_points".to_string(),
        ],
        None,
        json!({
            "created_chunk_ids": plan.chunks.iter().map(|chunk| chunk.id.clone()).collect::<Vec<_>>(),
            "created_evidence_ids": plan.evidence_items.iter().map(|evidence| evidence.id.clone()).collect::<Vec<_>>(),
            "skipped_document_ids": plan.skipped_document_ids,
            "chunk_vector_upsert_work_item_id": chained_id,
        }),
    ))
}

fn execute_chunk_vector_upsert_canary(
    client: &mut Client,
    claimed: &ClaimedWorkItem,
    config: &WorkerRuntimeConfig,
) -> Result<WorkerLiveCanaryResult, WorkerRuntimeError> {
    let limit = claimed
        .payload_json
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(100) as usize;
    let requested_chunk_ids = validate_chunk_vector_upsert_payload(&claimed.payload_json)
        .map_err(|error| WorkerRuntimeError::LiveExecution(error.to_string()))?;
    let rows = if let Some(chunk_ids) = &requested_chunk_ids {
        client.query(
            "SELECT id, document_id, chunk_index, text_content, embedding_status, metadata_json FROM chunks WHERE embedding_status != 'completed' AND id = ANY($1) ORDER BY id ASC LIMIT $2",
            &[chunk_ids, &(limit as i64)],
        )
    } else {
        client.query(
            "SELECT id, document_id, chunk_index, text_content, embedding_status, metadata_json FROM chunks WHERE embedding_status != 'completed' ORDER BY id ASC LIMIT $1",
            &[&(limit as i64)],
        )
    }
    .map_err(live_error)?;
    let candidate_chunks = rows
        .into_iter()
        .map(|row| ChunkForVectorRecord {
            id: row.get("id"),
            document_id: row.get("document_id"),
            chunk_index: row.get::<_, i32>("chunk_index") as usize,
            text_content: row.get("text_content"),
            embedding_status: row.get("embedding_status"),
            metadata_json: row.get("metadata_json"),
        })
        .collect::<Vec<_>>();
    let qdrant_settings = QdrantSettings {
        base_url: config.qdrant_url.clone(),
        collection_name: config.qdrant_chunk_collection.clone(),
        vector_size: config.qdrant_chunk_vector_size,
    };
    let plan = plan_chunk_vector_upsert_execution(ChunkVectorUpsertExecutionInput {
        work_item: Some(ChunkVectorUpsertWorkItem {
            id: claimed.work_item_id.clone(),
            work_type: claimed.task_kind.work_type().to_string(),
            status: "running".to_string(),
            requested_by_actor_id: claimed.requested_by_actor_id.clone(),
            payload_json: claimed.payload_json.clone(),
        }),
        limit,
        candidate_chunks,
        qdrant_settings,
    })
    .map_err(|error| WorkerRuntimeError::LiveExecution(error.to_string()))?;

    let mut qdrant_effects = Vec::new();
    if !plan.points.is_empty() {
        execute_qdrant_vector_plan(&plan)?;
        qdrant_effects.push("qdrant_collection_ensure".to_string());
        qdrant_effects.push("qdrant_points_upsert".to_string());
    }

    let mut transaction = client.transaction().map_err(live_error)?;
    for update in &plan.chunk_updates {
        transaction
            .execute(
                "UPDATE chunks SET embedding_status = 'completed', metadata_json = $2 WHERE id = $1",
                &[&update.chunk_id, &update.metadata_json],
            )
            .map_err(live_error)?;
    }
    transaction
        .execute(
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
            &[&claimed.work_item_id],
        )
        .map_err(live_error)?;
    insert_audit_event_tx(
        &mut transaction,
        &plan.completion_audit_event.actor_id,
        &plan.completion_audit_event.event_type,
        &plan.completion_audit_event.decision,
        &plan.completion_audit_event.resource_type,
        &plan.completion_audit_event.resource_id,
        &plan.completion_audit_event.correlation_id,
        plan.completion_audit_event.details_json.clone(),
    )?;
    transaction.commit().map_err(live_error)?;

    let mut side_effects = qdrant_effects;
    side_effects.extend([
        "postgres_chunk_embedding_updates".to_string(),
        "postgres_work_item_completed".to_string(),
        "audit_worker_success".to_string(),
    ]);
    Ok(live_result(
        &claimed.work_item_id,
        Some(claimed.task_kind.work_type().to_string()),
        "completed",
        true,
        side_effects,
        vec!["artifact_store_read".to_string()],
        None,
        json!({
            "chunks_selected": plan.selected_chunk_ids.len(),
            "chunks_upserted": plan.points.len(),
            "chunk_ids": plan.selected_chunk_ids,
        }),
    ))
}

#[allow(clippy::too_many_arguments)]
fn live_result(
    work_item_id: &str,
    work_type: Option<String>,
    result_state: &str,
    mutates_runtime_data: bool,
    side_effects_executed: Vec<String>,
    side_effects_planned: Vec<String>,
    error_message: Option<String>,
    output_json: Value,
) -> WorkerLiveCanaryResult {
    WorkerLiveCanaryResult {
        service: "igy6-worker",
        diff: "DIFF-149",
        mode: "once",
        status: format!("canary_{result_state}"),
        work_item_id: work_item_id.to_string(),
        work_type,
        result_state: result_state.to_string(),
        mutates_runtime_data,
        live_execution_enabled: true,
        side_effects_executed,
        side_effects_planned,
        error_message,
        output_json,
    }
}

fn mark_canary_failed(
    client: &mut Client,
    claimed: &ClaimedWorkItem,
    error_message: &str,
) -> Result<(), WorkerRuntimeError> {
    let mut transaction = client.transaction().map_err(live_error)?;
    transaction
        .execute(
            "UPDATE work_items SET status = 'failed', error_message = $2, updated_at = now() WHERE id = $1",
            &[&claimed.work_item_id, &error_message],
        )
        .map_err(live_error)?;
    let event_type = match claimed.task_kind {
        WorkerTaskKind::CollectionNormalization => "collection_normalization.failed",
        WorkerTaskKind::DocumentChunking => "document_chunks.failed",
        WorkerTaskKind::ChunkVectorUpsert => "chunk_vectors.failed",
    };
    insert_audit_event_tx(
        &mut transaction,
        &claimed.requested_by_actor_id,
        event_type,
        "failed",
        "work_item",
        &claimed.work_item_id,
        &claimed.work_item_id,
        json!({
            "work_type": claimed.task_kind.work_type(),
            "error_message": error_message,
            "generated_by": "DIFF-149",
        }),
    )?;
    transaction.commit().map_err(live_error)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_audit_event_tx(
    transaction: &mut postgres::Transaction<'_>,
    actor_id: &str,
    event_type: &str,
    decision: &str,
    resource_type: &str,
    resource_id: &str,
    correlation_id: &str,
    details_json: Value,
) -> Result<(), WorkerRuntimeError> {
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&actor_id, &event_type, &decision, &resource_type, &resource_id, &correlation_id, &details_json],
        )
        .map_err(live_error)?;
    Ok(())
}

fn required_payload_string(payload: &Value, key: &str) -> Result<String, WorkerRuntimeError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| WorkerRuntimeError::LiveExecution(format!("payload requires {key}")))
}

fn required_payload_string_array(
    payload: &Value,
    key: &str,
) -> Result<Vec<String>, WorkerRuntimeError> {
    let values = payload
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| WorkerRuntimeError::LiveExecution(format!("payload requires {key}")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|text| !text.trim().is_empty())
                .map(ToString::to_string)
                .ok_or_else(|| {
                    WorkerRuntimeError::LiveExecution(format!(
                        "payload {key} must contain only non-empty strings"
                    ))
                })
        })
        .collect()
}

fn payload_document_ids(payload: &Value) -> Result<Vec<String>, WorkerRuntimeError> {
    if payload.get("document_ids").is_some() {
        return required_payload_string_array(payload, "document_ids");
    }
    required_payload_string(payload, "document_id").map(|document_id| vec![document_id])
}

fn read_artifact_bytes_under_data_root(
    data_root: &str,
    storage_path: &str,
) -> Result<Vec<u8>, WorkerRuntimeError> {
    let relative_path = Path::new(storage_path);
    if relative_path.is_absolute()
        || storage_path.contains('\0')
        || relative_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(WorkerRuntimeError::LiveExecution(
            "artifact storage path must be relative and stay under IGY6_DATA_ROOT".to_string(),
        ));
    }
    let root = PathBuf::from(data_root);
    let artifact_root = root.join("artifacts");
    let canonical_root = artifact_root.canonicalize().map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("artifact root is not readable: {error}"))
    })?;
    let target = artifact_root.join(relative_path);
    let canonical_target = target.canonicalize().map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("artifact file is not readable: {error}"))
    })?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err(WorkerRuntimeError::LiveExecution(
            "artifact storage path escapes IGY6_DATA_ROOT/artifacts".to_string(),
        ));
    }
    fs::read(canonical_target).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("artifact file read failed: {error}"))
    })
}

fn execute_qdrant_vector_plan(
    plan: &ChunkVectorUpsertExecutionPlan,
) -> Result<(), WorkerRuntimeError> {
    let status = execute_http_request_plan(&plan.collection_status_request)?;
    if status == 404 {
        let Some(ensure_request) = &plan.ensure_collection_request else {
            return Err(WorkerRuntimeError::LiveExecution(
                "Qdrant collection is missing and no ensure request was planned".to_string(),
            ));
        };
        let ensure_status = execute_http_request_plan(ensure_request)?;
        if !(200..300).contains(&ensure_status) {
            return Err(WorkerRuntimeError::LiveExecution(format!(
                "Qdrant collection ensure failed with HTTP {ensure_status}"
            )));
        }
    } else if !(200..300).contains(&status) {
        return Err(WorkerRuntimeError::LiveExecution(format!(
            "Qdrant collection status failed with HTTP {status}"
        )));
    }
    if let Some(upsert_request) = &plan.upsert_points_request {
        let upsert_status = execute_http_request_plan(upsert_request)?;
        if !(200..300).contains(&upsert_status) {
            return Err(WorkerRuntimeError::LiveExecution(format!(
                "Qdrant point upsert failed with HTTP {upsert_status}"
            )));
        }
    }
    Ok(())
}

fn execute_http_request_plan(request: &HttpRequestPlan) -> Result<u16, WorkerRuntimeError> {
    let (host, port) = host_port_from_http_origin(&request.origin)?;
    let mut stream = TcpStream::connect((host.as_str(), port)).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("Qdrant connection failed: {error}"))
    })?;
    let timeout = Duration::from_secs(request.timeout_seconds.clamp(1, 30));
    stream.set_read_timeout(Some(timeout)).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("Qdrant timeout failed: {error}"))
    })?;
    stream.set_write_timeout(Some(timeout)).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("Qdrant timeout failed: {error}"))
    })?;
    let method = match request.method {
        HttpMethod::Get => "GET",
        HttpMethod::Put => "PUT",
        HttpMethod::Post => "POST",
    };
    let body = request.body.as_deref().unwrap_or("");
    let request_text = format!(
        "{method} {} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        request.path,
        body.len()
    );
    stream.write_all(request_text.as_bytes()).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("Qdrant request failed: {error}"))
    })?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|error| {
        WorkerRuntimeError::LiveExecution(format!("Qdrant response failed: {error}"))
    })?;
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| {
            WorkerRuntimeError::LiveExecution(
                "Qdrant response did not include an HTTP status".to_string(),
            )
        })?;
    Ok(status)
}

fn host_port_from_http_origin(origin: &str) -> Result<(String, u16), WorkerRuntimeError> {
    let host_port = origin.strip_prefix("http://").ok_or_else(|| {
        WorkerRuntimeError::LiveExecution("Qdrant live canary requires an http:// URL".to_string())
    })?;
    if host_port.contains('/') || host_port.contains('@') || host_port.contains("..") {
        return Err(WorkerRuntimeError::LiveExecution(
            "Qdrant URL must be a bare http://host[:port] origin".to_string(),
        ));
    }
    let (host, port) = if let Some((host, port)) = host_port.rsplit_once(':') {
        let port = port.parse::<u16>().map_err(|_| {
            WorkerRuntimeError::LiveExecution("Qdrant URL port is invalid".to_string())
        })?;
        (host.to_string(), port)
    } else {
        (host_port.to_string(), 80)
    };
    if host.trim().is_empty() {
        return Err(WorkerRuntimeError::LiveExecution(
            "Qdrant URL host is required".to_string(),
        ));
    }
    Ok((host, port))
}

fn replace_placeholder_id(details_json: &Value, id: &str) -> Value {
    match details_json {
        Value::Object(map) => {
            let mut map = map.clone();
            for value in map.values_mut() {
                if value == "<generated-document-chunking-work-item-id>"
                    || value == "<generated-chunk-vector-upsert-work-item-id>"
                {
                    *value = if id.is_empty() {
                        Value::Null
                    } else {
                        Value::String(id.to_string())
                    };
                }
            }
            Value::Object(map)
        }
        _ => details_json.clone(),
    }
}

fn generated_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = GENERATED_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{nanos:x}-{sequence:x}")
}

fn postgres_client_url(database_url: &str) -> String {
    for prefix in ["postgresql+", "postgres+"] {
        if let Some(driver_url) = database_url.strip_prefix(prefix) {
            if let Some((_, rest)) = driver_url.split_once("://") {
                return format!("{}://{}", prefix.trim_end_matches('+'), rest);
            }
        }
    }
    database_url.to_string()
}

fn live_error(error: impl fmt::Display) -> WorkerRuntimeError {
    WorkerRuntimeError::LiveExecution(error.to_string())
}

pub fn parse_usize_setting(value: Option<&str>, default: usize) -> Result<usize, String> {
    match value {
        Some(raw) if !raw.trim().is_empty() => raw
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("invalid usize setting: {raw}")),
        _ => Ok(default),
    }
}

pub fn parse_u64_setting(value: Option<&str>, default: u64) -> Result<u64, String> {
    match value {
        Some(raw) if !raw.trim().is_empty() => raw
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid u64 setting: {raw}")),
        _ => Ok(default),
    }
}

fn parse_claim_limit(value: &str) -> Result<usize, WorkerRuntimeError> {
    let parsed = value.parse::<usize>().map_err(|_| {
        WorkerRuntimeError::InvalidClaimLimit(format!("invalid claim limit: {value}"))
    })?;
    if !(1..=MAX_WORKER_CLAIM_LIMIT).contains(&parsed) {
        return Err(WorkerRuntimeError::InvalidClaimLimit(format!(
            "claim limit must be between 1 and {MAX_WORKER_CLAIM_LIMIT}, got {parsed}"
        )));
    }
    Ok(parsed)
}

fn parse_poll_interval_ms(value: &str) -> Result<u64, WorkerRuntimeError> {
    let parsed = value.parse::<u64>().map_err(|_| {
        WorkerRuntimeError::InvalidPollInterval(format!("invalid poll interval: {value}"))
    })?;
    if !(MIN_WORKER_POLL_INTERVAL_MS..=MAX_WORKER_POLL_INTERVAL_MS).contains(&parsed) {
        return Err(WorkerRuntimeError::InvalidPollInterval(format!(
            "poll interval must be between {MIN_WORKER_POLL_INTERVAL_MS} and {MAX_WORKER_POLL_INTERVAL_MS}, got {parsed}"
        )));
    }
    Ok(parsed)
}

fn validate_database_url(value: &str) -> Result<(), WorkerRuntimeError> {
    let trimmed = value.trim();
    if trimmed.starts_with("postgresql://")
        || trimmed.starts_with("postgresql+")
        || trimmed.starts_with("postgres://")
    {
        Ok(())
    } else {
        Err(WorkerRuntimeError::InvalidDatabaseUrl(
            "DATABASE_URL must be a PostgreSQL URL".to_string(),
        ))
    }
}

fn validate_qdrant_url(value: &str) -> Result<(), WorkerRuntimeError> {
    let trimmed = value.trim();
    if (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
        && !trimmed.contains('@')
        && !trimmed.contains(char::is_whitespace)
    {
        Ok(())
    } else {
        Err(WorkerRuntimeError::InvalidQdrantUrl(
            "QDRANT_URL must be an http(s) URL without credentials".to_string(),
        ))
    }
}

fn validate_data_root(value: &str) -> Result<(), WorkerRuntimeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "/" || trimmed.contains('\0') {
        return Err(WorkerRuntimeError::InvalidDataRoot(
            "IGY6_DATA_ROOT must be a non-root local path".to_string(),
        ));
    }
    if trimmed.contains("..") && trimmed != DEFAULT_IGY6_DATA_ROOT {
        return Err(WorkerRuntimeError::InvalidDataRoot(
            "IGY6_DATA_ROOT must not contain parent traversal".to_string(),
        ));
    }
    Ok(())
}

fn validate_collection_name(value: &str) -> Result<(), WorkerRuntimeError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || !trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
    {
        Err(WorkerRuntimeError::InvalidCollectionName(
            "QDRANT_CHUNK_COLLECTION must contain only ASCII letters, numbers, underscore, or dash"
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionNormalizationError {
    WorkItemNotFound,
    WrongWorkType(String),
    PayloadMismatch(String),
    CollectionRunNotFound,
    MissingRawArtifacts(Vec<String>),
    RawArtifactCollectionMismatch(String),
    NonUtf8Artifact(String),
    MissingGeneratedDocumentId(String),
}

impl fmt::Display for CollectionNormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkItemNotFound => write!(formatter, "Work item not found"),
            Self::WrongWorkType(work_type) => {
                write!(formatter, "Work item is not a collection_normalization item: {work_type}")
            }
            Self::PayloadMismatch(message) => write!(formatter, "{message}"),
            Self::CollectionRunNotFound => write!(formatter, "Collection run not found"),
            Self::MissingRawArtifacts(ids) => write!(formatter, "Raw artifacts not found: {}", ids.join(", ")),
            Self::RawArtifactCollectionMismatch(id) => write!(
                formatter,
                "Raw artifact does not belong to the collection run: {id}"
            ),
            Self::NonUtf8Artifact(id) => write!(
                formatter,
                "Artifact is not UTF-8 text; this phase supports UTF-8 text normalization only: {id}"
            ),
            Self::MissingGeneratedDocumentId(id) => {
                write!(formatter, "missing generated document id for raw artifact: {id}")
            }
        }
    }
}

impl std::error::Error for CollectionNormalizationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentChunkingError {
    InvalidChunkSize(usize),
    WorkItemNotFound,
    WrongWorkType(String),
    PayloadMismatch(String),
    MissingDocuments(Vec<String>),
    EmptyDocumentText(String),
    MissingGeneratedChunkId {
        document_id: String,
        chunk_index: usize,
    },
    MissingGeneratedEvidenceId {
        document_id: String,
        chunk_index: usize,
    },
    Chunking(ChunkingError),
}

impl fmt::Display for DocumentChunkingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChunkSize(size) => {
                write!(
                    formatter,
                    "Chunk size must be between 100 and 5000, got {size}"
                )
            }
            Self::WorkItemNotFound => write!(formatter, "Work item not found"),
            Self::WrongWorkType(work_type) => {
                write!(
                    formatter,
                    "Work item is not a document_chunking item: {work_type}"
                )
            }
            Self::PayloadMismatch(message) => write!(formatter, "{message}"),
            Self::MissingDocuments(ids) => {
                write!(formatter, "Documents not found: {}", ids.join(", "))
            }
            Self::EmptyDocumentText(id) => write!(formatter, "Document text is empty: {id}"),
            Self::MissingGeneratedChunkId {
                document_id,
                chunk_index,
            } => write!(
                formatter,
                "missing generated chunk id for document {document_id} chunk {chunk_index}"
            ),
            Self::MissingGeneratedEvidenceId {
                document_id,
                chunk_index,
            } => write!(
                formatter,
                "missing generated evidence id for document {document_id} chunk {chunk_index}"
            ),
            Self::Chunking(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for DocumentChunkingError {}

impl From<ChunkingError> for DocumentChunkingError {
    fn from(error: ChunkingError) -> Self {
        Self::Chunking(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkVectorUpsertError {
    InvalidLimit(usize),
    WorkItemNotFound,
    WrongWorkType(String),
    InvalidPayload(String),
    VectorMemory(VectorMemoryError),
}

impl fmt::Display for ChunkVectorUpsertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimit(limit) => {
                write!(formatter, "limit must be between 1 and 1000, got {limit}")
            }
            Self::WorkItemNotFound => write!(formatter, "Work item not found"),
            Self::WrongWorkType(work_type) => {
                write!(
                    formatter,
                    "Work item is not a chunk_vector_upsert item: {work_type}"
                )
            }
            Self::InvalidPayload(message) => write!(formatter, "{message}"),
            Self::VectorMemory(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ChunkVectorUpsertError {}

impl From<VectorMemoryError> for ChunkVectorUpsertError {
    fn from(error: VectorMemoryError) -> Self {
        Self::VectorMemory(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionNormalizationWorkItem {
    pub id: String,
    pub work_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionRunRecord {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawArtifactRecord {
    pub id: String,
    pub source_id: String,
    pub collection_run_id: String,
    pub content_hash: String,
    pub storage_path: String,
    pub metadata_json: Value,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingNormalizedDocument {
    pub id: String,
    pub raw_artifact_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedDocumentId {
    pub raw_artifact_id: String,
    pub document_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionNormalizationExecutionInput {
    pub work_item: Option<CollectionNormalizationWorkItem>,
    pub requested_collection_run_id: String,
    pub requested_raw_artifact_ids: Vec<String>,
    pub collection_run: Option<CollectionRunRecord>,
    pub raw_artifacts: Vec<RawArtifactRecord>,
    pub existing_documents: Vec<ExistingNormalizedDocument>,
    pub generated_document_ids: Vec<GeneratedDocumentId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDocumentDraft {
    pub id: String,
    pub raw_artifact_id: String,
    pub source_id: String,
    pub title: Option<String>,
    pub document_type: String,
    pub language: Option<String>,
    pub text_content: String,
    pub sensitivity: String,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainedWorkItemDraft {
    pub work_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub payload_json: Value,
    pub audit_event: AuditEventDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemStatusDraft {
    pub work_item_id: String,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraft {
    pub actor_id: String,
    pub event_type: String,
    pub decision: String,
    pub resource_type: String,
    pub resource_id: String,
    pub correlation_id: String,
    pub details_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentChunkingWorkItem {
    pub id: String,
    pub work_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDocumentRecord {
    pub id: String,
    pub source_id: Option<String>,
    pub text_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingChunkRecord {
    pub id: String,
    pub document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedChunkId {
    pub document_id: String,
    pub chunk_index: usize,
    pub chunk_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedEvidenceId {
    pub document_id: String,
    pub chunk_index: usize,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentChunkingExecutionInput {
    pub work_item: Option<DocumentChunkingWorkItem>,
    pub requested_document_ids: Vec<String>,
    pub chunk_size: usize,
    pub documents: Vec<NormalizedDocumentRecord>,
    pub existing_chunks: Vec<ExistingChunkRecord>,
    pub generated_chunk_ids: Vec<GeneratedChunkId>,
    pub generated_evidence_ids: Vec<GeneratedEvidenceId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkRecordDraft {
    pub id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub text_content: String,
    pub location_json: Value,
    pub embedding_status: String,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceItemDraft {
    pub id: String,
    pub source_id: Option<String>,
    pub document_id: String,
    pub chunk_id: String,
    pub evidence_type: String,
    pub statement: String,
    pub observed_at: Option<String>,
    pub confidence: Option<i32>,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentChunkingExecutionPlan {
    pub status: String,
    pub actor_id: String,
    pub work_item_id: String,
    pub document_ids: Vec<String>,
    pub chunks: Vec<ChunkRecordDraft>,
    pub evidence_items: Vec<EvidenceItemDraft>,
    pub skipped_document_ids: Vec<String>,
    pub completion_status_update: WorkItemStatusDraft,
    pub chunk_vector_upsert_work_item: Option<ChainedWorkItemDraft>,
    pub completion_audit_event: AuditEventDraft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVectorUpsertWorkItem {
    pub id: String,
    pub work_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkForVectorRecord {
    pub id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub text_content: String,
    pub embedding_status: String,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVectorUpsertExecutionInput {
    pub work_item: Option<ChunkVectorUpsertWorkItem>,
    pub limit: usize,
    pub candidate_chunks: Vec<ChunkForVectorRecord>,
    pub qdrant_settings: QdrantSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVectorPointDraft {
    pub id: String,
    pub vector: Vec<f64>,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMetadataUpdateDraft {
    pub chunk_id: String,
    pub embedding_status: String,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVectorUpsertExecutionPlan {
    pub status: String,
    pub actor_id: String,
    pub work_item_id: String,
    pub requested_chunk_ids: Option<Vec<String>>,
    pub selected_chunk_ids: Vec<String>,
    pub points: Vec<ChunkVectorPointDraft>,
    pub collection_status_request: HttpRequestPlan,
    pub ensure_collection_request: Option<HttpRequestPlan>,
    pub upsert_points_request: Option<HttpRequestPlan>,
    pub chunk_updates: Vec<ChunkMetadataUpdateDraft>,
    pub completion_status_update: WorkItemStatusDraft,
    pub completion_audit_event: AuditEventDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkVectorUpsertSqlPlan {
    pub mark_running_sql: &'static str,
    pub select_chunks_sql: &'static str,
    pub select_requested_chunks_sql: &'static str,
    pub update_chunk_completed_sql: &'static str,
    pub mark_completed_sql: &'static str,
    pub mark_failed_sql: &'static str,
    pub insert_audit_event_sql: &'static str,
}

pub fn chunk_vector_upsert_sql_plan() -> ChunkVectorUpsertSqlPlan {
    ChunkVectorUpsertSqlPlan {
        mark_running_sql:
            "UPDATE work_items SET status = 'running', error_message = NULL, updated_at = now() WHERE id = $1",
        select_chunks_sql:
            "SELECT id, document_id, chunk_index, text_content, embedding_status, metadata_json FROM chunks WHERE embedding_status != 'completed' ORDER BY id ASC LIMIT $1",
        select_requested_chunks_sql:
            "SELECT id, document_id, chunk_index, text_content, embedding_status, metadata_json FROM chunks WHERE embedding_status != 'completed' AND id = ANY($1) ORDER BY id ASC LIMIT $2",
        update_chunk_completed_sql:
            "UPDATE chunks SET embedding_status = 'completed', metadata_json = $2 WHERE id = $1",
        mark_completed_sql:
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
        mark_failed_sql:
            "UPDATE work_items SET status = 'failed', error_message = $2, updated_at = now() WHERE id = $1",
        insert_audit_event_sql:
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChunkingSqlPlan {
    pub mark_running_sql: &'static str,
    pub insert_chunk_sql: &'static str,
    pub insert_evidence_item_sql: &'static str,
    pub mark_completed_sql: &'static str,
    pub mark_failed_sql: &'static str,
    pub insert_chained_work_item_sql: &'static str,
    pub insert_audit_event_sql: &'static str,
}

pub fn document_chunking_sql_plan() -> DocumentChunkingSqlPlan {
    DocumentChunkingSqlPlan {
        mark_running_sql:
            "UPDATE work_items SET status = 'running', error_message = NULL, updated_at = now() WHERE id = $1",
        insert_chunk_sql:
            "INSERT INTO chunks (id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json) VALUES ($1, $2, $3, $4, $5, 'not_started', $6)",
        insert_evidence_item_sql:
            "INSERT INTO evidence_items (id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json) VALUES ($1, $2, $3, $4, 'document_chunk', $5, NULL, NULL, $6)",
        mark_completed_sql:
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
        mark_failed_sql:
            "UPDATE work_items SET status = 'failed', error_message = $2, updated_at = now() WHERE id = $1",
        insert_chained_work_item_sql:
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json) VALUES ($1, 'chunk_vector_upsert', 'queued', $2, $3)",
        insert_audit_event_sql:
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    }
}

pub fn plan_document_chunking_execution(
    input: DocumentChunkingExecutionInput,
) -> Result<DocumentChunkingExecutionPlan, DocumentChunkingError> {
    if !(100..=5000).contains(&input.chunk_size) {
        return Err(DocumentChunkingError::InvalidChunkSize(input.chunk_size));
    }
    let work_item = input
        .work_item
        .ok_or(DocumentChunkingError::WorkItemNotFound)?;
    if work_item.work_type != WorkerTaskKind::DocumentChunking.work_type() {
        return Err(DocumentChunkingError::WrongWorkType(work_item.work_type));
    }
    validate_document_chunking_payload(&work_item.payload_json, &input.requested_document_ids)?;

    let documents_by_id: BTreeMap<String, NormalizedDocumentRecord> = input
        .documents
        .into_iter()
        .map(|document| (document.id.clone(), document))
        .collect();
    let missing_document_ids: Vec<String> = input
        .requested_document_ids
        .iter()
        .filter(|id| !documents_by_id.contains_key(*id))
        .cloned()
        .collect();
    if !missing_document_ids.is_empty() {
        return Err(DocumentChunkingError::MissingDocuments(
            missing_document_ids,
        ));
    }

    let documents_with_chunks: BTreeSet<String> = input
        .existing_chunks
        .iter()
        .map(|chunk| chunk.document_id.clone())
        .collect();
    let generated_chunk_ids: BTreeMap<(String, usize), String> = input
        .generated_chunk_ids
        .into_iter()
        .map(|generated| {
            (
                (generated.document_id, generated.chunk_index),
                generated.chunk_id,
            )
        })
        .collect();
    let generated_evidence_ids: BTreeMap<(String, usize), String> = input
        .generated_evidence_ids
        .into_iter()
        .map(|generated| {
            (
                (generated.document_id, generated.chunk_index),
                generated.evidence_id,
            )
        })
        .collect();

    let mut chunks = Vec::new();
    let mut evidence_items = Vec::new();
    let mut skipped_document_ids = Vec::new();
    for document_id in &input.requested_document_ids {
        if documents_with_chunks.contains(document_id) {
            skipped_document_ids.push(document_id.clone());
            continue;
        }
        let document = documents_by_id.get(document_id).expect("validated above");
        if document.text_content.is_empty() {
            return Err(DocumentChunkingError::EmptyDocumentText(
                document_id.clone(),
            ));
        }
        let chunking_plan = plan_document_chunks(
            document_id,
            document.source_id.as_deref(),
            &document.text_content,
            input.chunk_size,
        )?;
        for chunk in chunking_plan.chunks {
            let key = (document_id.clone(), chunk.chunk_index);
            let chunk_id = generated_chunk_ids
                .get(&key)
                .ok_or_else(|| DocumentChunkingError::MissingGeneratedChunkId {
                    document_id: document_id.clone(),
                    chunk_index: chunk.chunk_index,
                })?
                .clone();
            chunks.push(ChunkRecordDraft {
                id: chunk_id.clone(),
                document_id: document_id.clone(),
                chunk_index: chunk.chunk_index,
                text_content: chunk.text_content.clone(),
                location_json: json!({
                    "char_start": chunk.char_start,
                    "char_end": chunk.char_end,
                }),
                embedding_status: "not_started".to_string(),
                metadata_json: json!({
                    "generated_by": "DIFF-052",
                    "chunk_size": input.chunk_size,
                    "work_item_id": work_item.id,
                }),
            });
            let evidence_id = generated_evidence_ids
                .get(&key)
                .ok_or_else(|| DocumentChunkingError::MissingGeneratedEvidenceId {
                    document_id: document_id.clone(),
                    chunk_index: chunk.chunk_index,
                })?
                .clone();
            evidence_items.push(EvidenceItemDraft {
                id: evidence_id,
                source_id: document.source_id.clone(),
                document_id: document_id.clone(),
                chunk_id,
                evidence_type: "document_chunk".to_string(),
                statement: chunk.text_content,
                observed_at: None,
                confidence: None,
                metadata_json: json!({
                    "generated_by": "DIFF-052",
                    "chunk_index": chunk.chunk_index,
                    "work_item_id": work_item.id,
                }),
            });
        }
    }

    let created_chunk_ids: Vec<String> = chunks.iter().map(|chunk| chunk.id.clone()).collect();
    let created_evidence_ids: Vec<String> = evidence_items
        .iter()
        .map(|evidence| evidence.id.clone())
        .collect();
    let chunk_vector_upsert_work_item = if created_chunk_ids.is_empty() {
        None
    } else {
        Some(ChainedWorkItemDraft {
            work_type: "chunk_vector_upsert".to_string(),
            status: "queued".to_string(),
            requested_by_actor_id: work_item.requested_by_actor_id.clone(),
            payload_json: chained_vector_upsert_payload(&created_chunk_ids, Some(&work_item.id)),
            audit_event: AuditEventDraft {
                actor_id: work_item.requested_by_actor_id.clone(),
                event_type: "work_item.created".to_string(),
                decision: "queued".to_string(),
                resource_type: "work_item".to_string(),
                resource_id: "<generated-chunk-vector-upsert-work-item-id>".to_string(),
                correlation_id: work_item.id.clone(),
                details_json: json!({
                    "work_type": "chunk_vector_upsert",
                    "parent_work_item_id": work_item.id,
                    "generated_by": "DIFF-066",
                }),
            },
        })
    };

    Ok(DocumentChunkingExecutionPlan {
        status: "completed".to_string(),
        actor_id: work_item.requested_by_actor_id.clone(),
        work_item_id: work_item.id.clone(),
        document_ids: input.requested_document_ids.clone(),
        chunks,
        evidence_items,
        skipped_document_ids: skipped_document_ids.clone(),
        completion_status_update: WorkItemStatusDraft {
            work_item_id: work_item.id.clone(),
            status: "completed".to_string(),
            error_message: None,
        },
        completion_audit_event: AuditEventDraft {
            actor_id: work_item.requested_by_actor_id,
            event_type: "document_chunks.generated".to_string(),
            decision: "completed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item.id.clone(),
            correlation_id: work_item.id,
            details_json: json!({
                "document_ids": input.requested_document_ids,
                "chunk_count": created_chunk_ids.len(),
                "evidence_count": created_evidence_ids.len(),
                "skipped_document_ids": skipped_document_ids,
                "chunk_vector_upsert_work_item_id": if chunk_vector_upsert_work_item.is_some() {
                    Value::String("<generated-chunk-vector-upsert-work-item-id>".to_string())
                } else {
                    Value::Null
                },
            }),
        },
        chunk_vector_upsert_work_item,
    })
}

pub fn plan_document_chunking_failure(
    work_item_id: &str,
    document_ids: &[String],
    actor_id: &str,
    error_message: &str,
) -> (WorkItemStatusDraft, AuditEventDraft) {
    (
        WorkItemStatusDraft {
            work_item_id: work_item_id.to_string(),
            status: "failed".to_string(),
            error_message: Some(error_message.to_string()),
        },
        AuditEventDraft {
            actor_id: actor_id.to_string(),
            event_type: "document_chunks.failed".to_string(),
            decision: "failed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item_id.to_string(),
            correlation_id: work_item_id.to_string(),
            details_json: json!({
                "document_ids": document_ids,
                "error_message": error_message,
            }),
        },
    )
}

pub fn plan_chunk_vector_upsert_execution(
    input: ChunkVectorUpsertExecutionInput,
) -> Result<ChunkVectorUpsertExecutionPlan, ChunkVectorUpsertError> {
    if !(1..=1000).contains(&input.limit) {
        return Err(ChunkVectorUpsertError::InvalidLimit(input.limit));
    }
    let work_item = input
        .work_item
        .ok_or(ChunkVectorUpsertError::WorkItemNotFound)?;
    if work_item.work_type != WorkerTaskKind::ChunkVectorUpsert.work_type() {
        return Err(ChunkVectorUpsertError::WrongWorkType(work_item.work_type));
    }
    let requested_chunk_ids = validate_chunk_vector_upsert_payload(&work_item.payload_json)?;
    let requested_chunk_set: Option<BTreeSet<String>> = requested_chunk_ids
        .as_ref()
        .map(|ids| ids.iter().cloned().collect());

    let mut selectable_chunks: Vec<ChunkForVectorRecord> = input
        .candidate_chunks
        .into_iter()
        .filter(|chunk| chunk.embedding_status != "completed")
        .filter(|chunk| {
            requested_chunk_set
                .as_ref()
                .is_none_or(|ids| ids.contains(&chunk.id))
        })
        .collect();
    selectable_chunks.sort_by(|left, right| left.id.cmp(&right.id));
    selectable_chunks.truncate(input.limit);

    let collection_status_request = collection_status_request(&input.qdrant_settings)?;
    let ensure_collection_request = if selectable_chunks.is_empty() {
        None
    } else {
        Some(ensure_collection_request(&input.qdrant_settings)?)
    };

    let mut points = Vec::with_capacity(selectable_chunks.len());
    let mut vector_points = Vec::with_capacity(selectable_chunks.len());
    let mut chunk_updates = Vec::with_capacity(selectable_chunks.len());
    for chunk in &selectable_chunks {
        let vector = embed_text_local(&chunk.text_content, input.qdrant_settings.vector_size)?;
        points.push(ChunkVectorPointDraft {
            id: chunk.id.clone(),
            vector: vector.clone(),
            payload_json: json!({
                "chunk_id": chunk.id,
                "document_id": chunk.document_id,
                "chunk_index": chunk.chunk_index,
                "embedding_method": WORKER_EMBEDDING_METHOD,
                "generated_by": WORKER_VECTOR_GENERATED_BY,
            }),
        });
        vector_points.push(ChunkVectorPoint {
            id: chunk.id.clone(),
            vector,
            chunk_id: chunk.id.clone(),
            document_id: chunk.document_id.clone(),
            chunk_index: chunk.chunk_index,
            embedding_method: WORKER_EMBEDDING_METHOD.to_string(),
        });
        chunk_updates.push(ChunkMetadataUpdateDraft {
            chunk_id: chunk.id.clone(),
            embedding_status: "completed".to_string(),
            metadata_json: merge_chunk_vector_metadata(
                &chunk.metadata_json,
                &input.qdrant_settings.collection_name,
            ),
        });
    }
    let upsert_points_request = if vector_points.is_empty() {
        None
    } else {
        Some(upsert_points_request(
            &input.qdrant_settings,
            &vector_points,
        )?)
    };
    let selected_chunk_ids: Vec<String> = selectable_chunks
        .iter()
        .map(|chunk| chunk.id.clone())
        .collect();

    Ok(ChunkVectorUpsertExecutionPlan {
        status: "completed".to_string(),
        actor_id: work_item.requested_by_actor_id.clone(),
        work_item_id: work_item.id.clone(),
        requested_chunk_ids,
        selected_chunk_ids: selected_chunk_ids.clone(),
        points,
        collection_status_request,
        ensure_collection_request,
        upsert_points_request,
        chunk_updates,
        completion_status_update: WorkItemStatusDraft {
            work_item_id: work_item.id.clone(),
            status: "completed".to_string(),
            error_message: None,
        },
        completion_audit_event: AuditEventDraft {
            actor_id: work_item.requested_by_actor_id,
            event_type: "chunk_vectors.upserted".to_string(),
            decision: "completed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item.id.clone(),
            correlation_id: work_item.id,
            details_json: json!({
                "chunks_selected": selected_chunk_ids.len(),
                "chunks_upserted": selected_chunk_ids.len(),
                "chunk_ids": selected_chunk_ids,
                "vector_collection": input.qdrant_settings.collection_name,
                "embedding_method": WORKER_EMBEDDING_METHOD,
            }),
        },
    })
}

pub fn plan_chunk_vector_upsert_failure(
    work_item_id: &str,
    limit: usize,
    actor_id: &str,
    error_message: &str,
) -> (WorkItemStatusDraft, AuditEventDraft) {
    (
        WorkItemStatusDraft {
            work_item_id: work_item_id.to_string(),
            status: "failed".to_string(),
            error_message: Some(error_message.to_string()),
        },
        AuditEventDraft {
            actor_id: actor_id.to_string(),
            event_type: "chunk_vectors.failed".to_string(),
            decision: "failed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item_id.to_string(),
            correlation_id: work_item_id.to_string(),
            details_json: json!({
                "limit": limit,
                "error_message": error_message,
            }),
        },
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionNormalizationExecutionPlan {
    pub status: String,
    pub actor_id: String,
    pub work_item_id: String,
    pub collection_run_id: String,
    pub normalized_documents: Vec<NormalizedDocumentDraft>,
    pub skipped_raw_artifact_ids: Vec<String>,
    pub completion_status_update: WorkItemStatusDraft,
    pub document_chunking_work_item: Option<ChainedWorkItemDraft>,
    pub completion_audit_event: AuditEventDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionNormalizationSqlPlan {
    pub mark_running_sql: &'static str,
    pub insert_normalized_document_sql: &'static str,
    pub mark_completed_sql: &'static str,
    pub mark_failed_sql: &'static str,
    pub insert_chained_work_item_sql: &'static str,
    pub insert_audit_event_sql: &'static str,
}

pub fn collection_normalization_sql_plan() -> CollectionNormalizationSqlPlan {
    CollectionNormalizationSqlPlan {
        mark_running_sql:
            "UPDATE work_items SET status = 'running', error_message = NULL, updated_at = now() WHERE id = $1",
        insert_normalized_document_sql:
            "INSERT INTO normalized_documents (id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json) VALUES ($1, $2, $3, $4, 'text', NULL, $5, 'internal', $6)",
        mark_completed_sql:
            "UPDATE work_items SET status = 'completed', error_message = NULL, updated_at = now() WHERE id = $1",
        mark_failed_sql:
            "UPDATE work_items SET status = 'failed', error_message = $2, updated_at = now() WHERE id = $1",
        insert_chained_work_item_sql:
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json) VALUES ($1, 'document_chunking', 'queued', $2, $3)",
        insert_audit_event_sql:
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    }
}

pub fn plan_collection_normalization_execution(
    input: CollectionNormalizationExecutionInput,
) -> Result<CollectionNormalizationExecutionPlan, CollectionNormalizationError> {
    let work_item = input
        .work_item
        .ok_or(CollectionNormalizationError::WorkItemNotFound)?;
    if work_item.work_type != WorkerTaskKind::CollectionNormalization.work_type() {
        return Err(CollectionNormalizationError::WrongWorkType(
            work_item.work_type,
        ));
    }
    validate_collection_normalization_payload(
        &work_item.payload_json,
        &input.requested_collection_run_id,
        &input.requested_raw_artifact_ids,
    )?;
    if input.collection_run.is_none() {
        return Err(CollectionNormalizationError::CollectionRunNotFound);
    }

    let artifacts_by_id: BTreeMap<String, RawArtifactRecord> = input
        .raw_artifacts
        .into_iter()
        .map(|artifact| (artifact.id.clone(), artifact))
        .collect();
    let missing_artifact_ids: Vec<String> = input
        .requested_raw_artifact_ids
        .iter()
        .filter(|id| !artifacts_by_id.contains_key(*id))
        .cloned()
        .collect();
    if !missing_artifact_ids.is_empty() {
        return Err(CollectionNormalizationError::MissingRawArtifacts(
            missing_artifact_ids,
        ));
    }

    let existing_raw_artifact_ids: BTreeSet<String> = input
        .existing_documents
        .iter()
        .map(|document| document.raw_artifact_id.clone())
        .collect();
    let generated_document_ids: BTreeMap<String, String> = input
        .generated_document_ids
        .into_iter()
        .map(|generated| (generated.raw_artifact_id, generated.document_id))
        .collect();

    let mut normalized_documents = Vec::new();
    let mut skipped_raw_artifact_ids = Vec::new();
    for artifact_id in &input.requested_raw_artifact_ids {
        let artifact = artifacts_by_id.get(artifact_id).expect("validated above");
        if artifact.collection_run_id != input.requested_collection_run_id {
            return Err(CollectionNormalizationError::RawArtifactCollectionMismatch(
                artifact.id.clone(),
            ));
        }
        if existing_raw_artifact_ids.contains(artifact_id) {
            skipped_raw_artifact_ids.push(artifact_id.clone());
            continue;
        }
        let text_content = std::str::from_utf8(&artifact.bytes)
            .map_err(|_| CollectionNormalizationError::NonUtf8Artifact(artifact.id.clone()))?
            .to_string();
        let document_id = generated_document_ids
            .get(artifact_id)
            .ok_or_else(|| {
                CollectionNormalizationError::MissingGeneratedDocumentId(artifact_id.clone())
            })?
            .clone();
        normalized_documents.push(NormalizedDocumentDraft {
            id: document_id,
            raw_artifact_id: artifact.id.clone(),
            source_id: artifact.source_id.clone(),
            title: document_title_from_metadata(&artifact.metadata_json, &artifact.id),
            document_type: "text".to_string(),
            language: None,
            text_content,
            sensitivity: "internal".to_string(),
            metadata_json: json!({
                "generated_by": "DIFF-051",
                "raw_content_hash": artifact.content_hash,
                "raw_storage_path": artifact.storage_path,
                "work_item_id": work_item.id,
            }),
        });
    }

    let created_document_ids: Vec<String> = normalized_documents
        .iter()
        .map(|document| document.id.clone())
        .collect();
    let document_chunking_work_item = if created_document_ids.is_empty() {
        None
    } else {
        Some(ChainedWorkItemDraft {
            work_type: "document_chunking".to_string(),
            status: "queued".to_string(),
            requested_by_actor_id: work_item.requested_by_actor_id.clone(),
            payload_json: chained_document_chunking_payload(&created_document_ids, &work_item.id),
            audit_event: AuditEventDraft {
                actor_id: work_item.requested_by_actor_id.clone(),
                event_type: "work_item.created".to_string(),
                decision: "queued".to_string(),
                resource_type: "work_item".to_string(),
                resource_id: "<generated-document-chunking-work-item-id>".to_string(),
                correlation_id: work_item.id.clone(),
                details_json: json!({
                    "work_type": "document_chunking",
                    "parent_work_item_id": work_item.id,
                    "generated_by": "DIFF-066",
                }),
            },
        })
    };

    Ok(CollectionNormalizationExecutionPlan {
        status: "completed".to_string(),
        actor_id: work_item.requested_by_actor_id.clone(),
        work_item_id: work_item.id.clone(),
        collection_run_id: input.requested_collection_run_id.clone(),
        normalized_documents,
        skipped_raw_artifact_ids: skipped_raw_artifact_ids.clone(),
        completion_status_update: WorkItemStatusDraft {
            work_item_id: work_item.id.clone(),
            status: "completed".to_string(),
            error_message: None,
        },
        completion_audit_event: AuditEventDraft {
            actor_id: work_item.requested_by_actor_id,
            event_type: "collection_normalization.completed".to_string(),
            decision: "completed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item.id,
            correlation_id: input.requested_collection_run_id.clone(),
            details_json: json!({
                "collection_run_id": input.requested_collection_run_id,
                "created_document_ids": created_document_ids,
                "skipped_raw_artifact_ids": skipped_raw_artifact_ids,
                "document_chunking_work_item_id": if document_chunking_work_item.is_some() {
                    Value::String("<generated-document-chunking-work-item-id>".to_string())
                } else {
                    Value::Null
                },
            }),
        },
        document_chunking_work_item,
    })
}

pub fn plan_collection_normalization_failure(
    work_item_id: &str,
    collection_run_id: &str,
    raw_artifact_ids: &[String],
    actor_id: &str,
    error_message: &str,
) -> (WorkItemStatusDraft, AuditEventDraft) {
    (
        WorkItemStatusDraft {
            work_item_id: work_item_id.to_string(),
            status: "failed".to_string(),
            error_message: Some(error_message.to_string()),
        },
        AuditEventDraft {
            actor_id: actor_id.to_string(),
            event_type: "collection_normalization.failed".to_string(),
            decision: "failed".to_string(),
            resource_type: "work_item".to_string(),
            resource_id: work_item_id.to_string(),
            correlation_id: collection_run_id.to_string(),
            details_json: json!({
                "collection_run_id": collection_run_id,
                "raw_artifact_ids": raw_artifact_ids,
                "error_message": error_message,
            }),
        },
    )
}

fn validate_collection_normalization_payload(
    payload_json: &Value,
    collection_run_id: &str,
    raw_artifact_ids: &[String],
) -> Result<(), CollectionNormalizationError> {
    if payload_json
        .get("collection_run_id")
        .and_then(Value::as_str)
        != Some(collection_run_id)
    {
        return Err(CollectionNormalizationError::PayloadMismatch(
            "Work item collection_run_id does not match task request".to_string(),
        ));
    }
    let expected_artifact_ids = payload_json
        .get("raw_artifact_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CollectionNormalizationError::PayloadMismatch(
                "Work item raw_artifact_ids do not match task request".to_string(),
            )
        })?;
    let expected_artifact_ids: Option<Vec<String>> = expected_artifact_ids
        .iter()
        .map(|value| value.as_str().map(ToString::to_string))
        .collect();
    if expected_artifact_ids.as_deref() != Some(raw_artifact_ids) {
        return Err(CollectionNormalizationError::PayloadMismatch(
            "Work item raw_artifact_ids do not match task request".to_string(),
        ));
    }
    Ok(())
}

fn validate_document_chunking_payload(
    payload_json: &Value,
    document_ids: &[String],
) -> Result<(), DocumentChunkingError> {
    let expected_document_ids =
        if let Some(values) = payload_json.get("document_ids").and_then(Value::as_array) {
            values
                .iter()
                .map(|value| value.as_str().map(ToString::to_string))
                .collect::<Option<Vec<String>>>()
        } else {
            payload_json
                .get("document_id")
                .and_then(Value::as_str)
                .map(|document_id| vec![document_id.to_string()])
        };
    if expected_document_ids.as_deref() != Some(document_ids) {
        return Err(DocumentChunkingError::PayloadMismatch(
            "Work item document IDs do not match task request".to_string(),
        ));
    }
    Ok(())
}

fn validate_chunk_vector_upsert_payload(
    payload_json: &Value,
) -> Result<Option<Vec<String>>, ChunkVectorUpsertError> {
    if let Some(limit) = payload_json.get("limit").and_then(Value::as_i64) {
        if !(1..=1000).contains(&limit) {
            return Err(ChunkVectorUpsertError::InvalidPayload(
                "chunk_vector_upsert limit must be between 1 and 1000".to_string(),
            ));
        }
    }
    let Some(chunk_ids) = payload_json.get("chunk_ids") else {
        return Ok(None);
    };
    let Some(values) = chunk_ids.as_array() else {
        return Err(ChunkVectorUpsertError::InvalidPayload(
            "chunk_vector_upsert chunk_ids must be an array".to_string(),
        ));
    };
    let ids: Option<Vec<String>> = values
        .iter()
        .map(|value| value.as_str().map(ToString::to_string))
        .collect();
    ids.ok_or_else(|| {
        ChunkVectorUpsertError::InvalidPayload(
            "chunk_vector_upsert chunk_ids must contain only strings".to_string(),
        )
    })
    .map(Some)
}

fn merge_chunk_vector_metadata(metadata_json: &Value, collection_name: &str) -> Value {
    let mut metadata = match metadata_json {
        Value::Object(map) => map.clone(),
        _ => serde_json::Map::new(),
    };
    metadata.insert(
        "embedding_method".to_string(),
        Value::String(WORKER_EMBEDDING_METHOD.to_string()),
    );
    metadata.insert(
        "vector_collection".to_string(),
        Value::String(collection_name.to_string()),
    );
    Value::Object(metadata)
}

fn chained_document_chunking_payload(document_ids: &[String], parent_work_item_id: &str) -> Value {
    json!({
        "document_ids": document_ids,
        "chunk_size": 1000,
        "parent_work_item_id": parent_work_item_id,
        "worker_task_name": "evidence.generate_document_chunks",
        "generated_by": "DIFF-066",
        "intent_verification_recorded": true,
        "intent_verification": {
            "original_request": "Continue deterministic post-normalization evidence processing.",
            "interpretation": "Chunk normalized UTF-8 text documents created by the approved collection pipeline.",
            "proposed_work_type": "document_chunking",
            "sources_likely_used": [],
            "expected_output": "Chunk and evidence item records for normalized documents.",
            "safety_requirements": [
                "Use only local normalized documents from the parent work item.",
                "Do not perform external model calls or system-changing actions."
            ],
            "assumptions": ["Parent normalization work item completed successfully."],
            "missing_information": [],
            "recorded_by": "DIFF-074 worker chained governance"
        }
    })
}

fn chained_vector_upsert_payload(chunk_ids: &[String], parent_work_item_id: Option<&str>) -> Value {
    json!({
        "chunk_ids": chunk_ids,
        "limit": chunk_ids.len().max(1),
        "parent_work_item_id": parent_work_item_id,
        "worker_task_name": "memory.vector.upsert_chunks",
        "generated_by": "DIFF-066",
        "intent_verification_recorded": true,
        "intent_verification": {
            "original_request": "Continue deterministic post-chunking vector memory processing.",
            "interpretation": "Upsert local deterministic embeddings for chunks created by the approved pipeline.",
            "proposed_work_type": "chunk_vector_upsert",
            "sources_likely_used": [],
            "expected_output": "Qdrant points for local chunk embeddings.",
            "safety_requirements": [
                "Use only local chunk text from the parent work item.",
                "Do not perform external model calls or system-changing actions."
            ],
            "assumptions": ["Parent chunking work item completed successfully."],
            "missing_information": [],
            "recorded_by": "DIFF-074 worker chained governance"
        }
    })
}

fn document_title_from_metadata(metadata_json: &Value, artifact_id: &str) -> Option<String> {
    for key in ["filename", "relative_path", "source_path"] {
        if let Some(value) = metadata_json.get(key).and_then(Value::as_str) {
            if !value.is_empty() {
                return Some(value.chars().take(255).collect());
            }
        }
    }
    Some(artifact_id.to_string())
}

fn has_intent_verification(payload_json: &Value) -> bool {
    payload_json
        .get("intent_verification")
        .is_some_and(Value::is_object)
        || payload_json
            .get("intent_verification_recorded")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn validate_claim_payload(
    task_kind: WorkerTaskKind,
    payload_json: &Value,
) -> Result<(), QueueClaimError> {
    match task_kind {
        WorkerTaskKind::CollectionNormalization => {
            if !payload_json
                .get("collection_run_id")
                .is_some_and(Value::is_string)
            {
                return Err(QueueClaimError::InvalidPayload(
                    "collection_normalization requires collection_run_id".to_string(),
                ));
            }
            if !payload_json
                .get("raw_artifact_ids")
                .is_some_and(Value::is_array)
            {
                return Err(QueueClaimError::InvalidPayload(
                    "collection_normalization requires raw_artifact_ids".to_string(),
                ));
            }
        }
        WorkerTaskKind::DocumentChunking => {
            let has_document_ids = payload_json
                .get("document_ids")
                .is_some_and(Value::is_array);
            let has_document_id = payload_json
                .get("document_id")
                .is_some_and(Value::is_string);
            if !has_document_ids && !has_document_id {
                return Err(QueueClaimError::InvalidPayload(
                    "document_chunking requires document_ids or document_id".to_string(),
                ));
            }
        }
        WorkerTaskKind::ChunkVectorUpsert => {
            if let Some(limit) = payload_json.get("limit").and_then(Value::as_i64) {
                if !(1..=1000).contains(&limit) {
                    return Err(QueueClaimError::InvalidPayload(
                        "chunk_vector_upsert limit must be between 1 and 1000".to_string(),
                    ));
                }
            }
            if let Some(chunk_ids) = payload_json.get("chunk_ids") {
                if !chunk_ids.is_array() {
                    return Err(QueueClaimError::InvalidPayload(
                        "chunk_vector_upsert chunk_ids must be an array".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlanInput {
    pub work_item_id: String,
    pub collection_run_id: String,
    pub raw_artifact: RawArtifactRef,
    pub artifact_bytes: Vec<u8>,
    pub chunk_size: usize,
    pub qdrant_settings: QdrantSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkerPlan {
    pub status: String,
    pub work_item_id: String,
    pub collection_run_id: String,
    pub normalized_document: NormalizedDocumentRef,
    pub chunks: Vec<ChunkPlan>,
    pub evidence_items: Vec<EvidencePlan>,
    pub vector_points: Vec<ChunkVectorPoint>,
    pub vector_upsert_request: HttpRequestPlan,
}

pub fn plan_utf8_pipeline(input: WorkerPlanInput) -> Result<WorkerPlan, WorkerError> {
    let text = std::str::from_utf8(&input.artifact_bytes)
        .map_err(|_| WorkerError::NonUtf8Artifact)?
        .to_string();
    let normalized_document = build_normalized_document_ref(
        &input.raw_artifact,
        NormalizedDocumentInput {
            text_content: text,
            title: document_title(&input.raw_artifact),
            document_type: "text".to_string(),
            language: None,
            sensitivity: Some("internal".to_string()),
            metadata: BTreeMap::from([
                ("generated_by".to_string(), "DIFF-095".to_string()),
                ("work_item_id".to_string(), input.work_item_id.clone()),
            ]),
        },
    );

    let chunking_plan = plan_document_chunks(
        &normalized_document.id,
        Some(&normalized_document.source_id),
        &normalized_document.text_content,
        input.chunk_size,
    )?;
    let chunks = with_worker_chunk_ids(&chunking_plan.chunks);
    let evidence_items = with_worker_evidence_ids(&chunks, &chunking_plan.evidence_items);

    let mut vector_points = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        vector_points.push(igy6_vector_memory::plan_chunk_vector_point(
            &chunk_id(&chunk.document_id, chunk.chunk_index),
            &chunk.document_id,
            chunk.chunk_index,
            &chunk.text_content,
            input.qdrant_settings.vector_size,
        )?);
    }
    let vector_upsert_request = upsert_points_request(&input.qdrant_settings, &vector_points)?;

    Ok(WorkerPlan {
        status: "planned".to_string(),
        work_item_id: input.work_item_id,
        collection_run_id: input.collection_run_id,
        normalized_document,
        chunks,
        evidence_items,
        vector_points,
        vector_upsert_request,
    })
}

pub fn chunk_id(document_id: &str, chunk_index: usize) -> String {
    format!("{document_id}:chunk:{chunk_index}")
}

pub fn evidence_id(document_id: &str, chunk_index: usize) -> String {
    format!("{document_id}:evidence:{chunk_index}")
}

fn document_title(raw_artifact: &RawArtifactRef) -> Option<String> {
    for key in ["filename", "relative_path", "source_path"] {
        if let Some(value) = raw_artifact.metadata.get(key) {
            if !value.is_empty() {
                return Some(value.chars().take(255).collect());
            }
        }
    }
    Some(raw_artifact.id.clone())
}

fn with_worker_chunk_ids(chunks: &[ChunkPlan]) -> Vec<ChunkPlan> {
    chunks
        .iter()
        .map(|chunk| ChunkPlan {
            document_id: chunk.document_id.clone(),
            chunk_index: chunk.chunk_index,
            text_content: chunk.text_content.clone(),
            char_start: chunk.char_start,
            char_end: chunk.char_end,
            chunk_size: chunk.chunk_size,
        })
        .collect()
}

fn with_worker_evidence_ids(chunks: &[ChunkPlan], evidence: &[EvidencePlan]) -> Vec<EvidencePlan> {
    evidence
        .iter()
        .zip(chunks)
        .map(|(item, chunk)| EvidencePlan {
            source_id: item.source_id.clone(),
            document_id: item.document_id.clone(),
            chunk_index: chunk.chunk_index,
            evidence_type: item.evidence_type.clone(),
            statement: item.statement.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn runtime_args_default_to_safe_check_mode() {
        let args = parse_worker_runtime_args(Vec::<String>::new()).expect("args");
        assert_eq!(args.mode, WorkerRuntimeMode::Check);
        assert_eq!(args.claim_limit, 1);
        assert!(!args.explicit_live_execution);
    }

    #[test]
    fn runtime_args_validate_modes_and_bounds() {
        let args =
            parse_worker_runtime_args(["--dry-run", "--claim-limit", "4"]).expect("dry run args");
        assert_eq!(args.mode, WorkerRuntimeMode::DryRun);
        assert_eq!(args.claim_limit, 4);

        let once = parse_worker_runtime_args(["--once"]).expect("once");
        assert_eq!(once.mode, WorkerRuntimeMode::Once);
        assert_eq!(
            parse_worker_runtime_args(["--once", "--claim-limit", "2"]).expect_err("limit"),
            WorkerRuntimeError::InvalidClaimLimit("--once requires claim limit 1".to_string())
        );
        assert!(parse_worker_runtime_args(["--claim-limit", "0"]).is_err());
        assert!(parse_worker_runtime_args(["--unknown"]).is_err());
    }

    #[test]
    fn canary_live_requires_explicit_once_and_work_item() {
        assert!(matches!(
            parse_worker_runtime_args(["--canary-live", "--canary-work-item", "work-1"]),
            Err(WorkerRuntimeError::InvalidCanaryMode(_))
        ));
        assert!(matches!(
            parse_worker_runtime_args(["--once", "--canary-live"]),
            Err(WorkerRuntimeError::InvalidCanaryMode(_))
        ));
        assert!(matches!(
            parse_worker_runtime_args(["--once", "--canary-work-item", "work-1"]),
            Err(WorkerRuntimeError::InvalidCanaryMode(_))
        ));

        let args =
            parse_worker_runtime_args(["--once", "--canary-live", "--canary-work-item", "work-1"])
                .expect("canary");
        assert_eq!(args.mode, WorkerRuntimeMode::Once);
        assert!(args.canary_live);
        assert!(args.explicit_live_execution);
        assert_eq!(args.canary_work_item_id.as_deref(), Some("work-1"));
        assert_eq!(args.claim_limit, 1);
    }

    #[test]
    fn runtime_config_validation_rejects_unsafe_values() {
        assert!(validate_worker_runtime_config(WorkerRuntimeConfig::safe_default()).is_ok());

        let mut invalid_database = WorkerRuntimeConfig::safe_default();
        invalid_database.database_url = "sqlite:///tmp.db".to_string();
        assert!(matches!(
            validate_worker_runtime_config(invalid_database),
            Err(WorkerRuntimeError::InvalidDatabaseUrl(_))
        ));

        let mut invalid_qdrant = WorkerRuntimeConfig::safe_default();
        invalid_qdrant.qdrant_url = "http://user:secret@qdrant:6333".to_string();
        assert!(matches!(
            validate_worker_runtime_config(invalid_qdrant),
            Err(WorkerRuntimeError::InvalidQdrantUrl(_))
        ));

        let mut invalid_root = WorkerRuntimeConfig::safe_default();
        invalid_root.igy6_data_root = "/".to_string();
        assert!(matches!(
            validate_worker_runtime_config(invalid_root),
            Err(WorkerRuntimeError::InvalidDataRoot(_))
        ));
    }

    #[test]
    fn runtime_plan_is_non_mutating_and_blocks_side_effects() {
        let args = parse_worker_runtime_args(["--once"]).expect("args");
        let config = WorkerRuntimeConfig::safe_default();
        let plan = plan_worker_runtime(args, config.clone()).expect("plan");
        let status = render_worker_runtime_status(&plan, &config);

        assert_eq!(plan.status, "planned_without_execution");
        assert!(!plan.mutates_runtime_data);
        assert!(!plan.live_execution_enabled);
        assert!(plan
            .claim_query
            .select_sql
            .contains("FOR UPDATE SKIP LOCKED"));
        assert!(plan
            .allowed_work_types
            .contains(&"collection_normalization"));
        assert!(plan.allowed_work_types.contains(&"document_chunking"));
        assert!(plan.allowed_work_types.contains(&"chunk_vector_upsert"));
        assert!(plan
            .blocked_side_effects
            .contains(&"Qdrant HTTP calls".to_string()));
        assert_eq!(status["rust_only_runtime_claimed"], json!(false));
        assert_eq!(status["python_celery_worker_required"], json!(true));
    }

    #[test]
    fn canary_plan_is_one_job_and_side_effects_are_planned_only() {
        let args =
            parse_worker_runtime_args(["--once", "--canary-live", "--canary-work-item", "work-1"])
                .expect("canary args");
        let config = WorkerRuntimeConfig::safe_default();
        let plan = plan_worker_runtime(args, config.clone()).expect("plan");
        let canary = plan.canary_plan.as_ref().expect("canary");
        let status = render_worker_runtime_status(&plan, &config);

        assert_eq!(plan.status, "canary_ready_side_effects_planned");
        assert!(!plan.mutates_runtime_data);
        assert!(!plan.live_execution_enabled);
        assert_eq!(canary.max_jobs, 1);
        assert_eq!(canary.work_item_id, "work-1");
        assert!(canary.side_effects_executed.is_empty());
        assert!(canary.supported_result_states.contains(&"claimed"));
        assert!(canary.supported_result_states.contains(&"unsupported"));
        assert!(canary
            .side_effects_planned
            .iter()
            .any(|effect| effect.name == "audit_events"));
        assert_eq!(
            status["canary"]["status"],
            json!("side_effects_planned_not_executed")
        );
        assert_eq!(status["canary"]["side_effects_executed"], json!([]));
    }

    #[test]
    fn live_canary_requires_env_gate_before_executor_is_ready() {
        let args =
            parse_worker_runtime_args(["--once", "--canary-live", "--canary-work-item", "work-1"])
                .expect("canary args");
        let mut config = WorkerRuntimeConfig::safe_default();
        config.live_execution_enabled = true;
        let plan = plan_worker_runtime(args, config).expect("plan");

        assert_eq!(plan.status, "canary_live_ready");
        assert!(plan.live_execution_enabled);
        assert!(!plan.mutates_runtime_data);
    }

    #[test]
    fn live_canary_result_reports_real_side_effect_shape() {
        let config = WorkerRuntimeConfig::safe_default();
        let result = live_result(
            "work-1",
            Some("chunk_vector_upsert".to_string()),
            "completed",
            true,
            vec![
                "postgres_work_item_claim".to_string(),
                "audit_work_item_started".to_string(),
                "qdrant_points_upsert".to_string(),
            ],
            vec!["artifact_store_read".to_string()],
            None,
            json!({"chunks_upserted": 1}),
        );
        let rendered = render_worker_live_canary_result(&result, &config);

        assert_eq!(rendered["diff"], json!("DIFF-149"));
        assert_eq!(rendered["result_state"], json!("completed"));
        assert_eq!(rendered["mutates_runtime_data"], json!(true));
        assert_eq!(rendered["rust_only_runtime_claimed"], json!(false));
        assert!(rendered["side_effects_executed"]
            .as_array()
            .expect("side effects")
            .contains(&json!("qdrant_points_upsert")));
    }

    #[test]
    fn artifact_path_safety_rejects_absolute_and_traversal_paths() {
        assert!(matches!(
            read_artifact_bytes_under_data_root("/tmp/igy6-test-root", "/etc/passwd"),
            Err(WorkerRuntimeError::LiveExecution(_))
        ));
        assert!(matches!(
            read_artifact_bytes_under_data_root("/tmp/igy6-test-root", "../escape.txt"),
            Err(WorkerRuntimeError::LiveExecution(_))
        ));
    }

    #[test]
    fn qdrant_live_http_origin_is_bounded_to_http_origin() {
        assert_eq!(
            host_port_from_http_origin("http://qdrant:6333").expect("origin"),
            ("qdrant".to_string(), 6333)
        );
        assert!(matches!(
            host_port_from_http_origin("https://qdrant:6333"),
            Err(WorkerRuntimeError::LiveExecution(_))
        ));
        assert!(matches!(
            host_port_from_http_origin("http://qdrant:6333/collections/x"),
            Err(WorkerRuntimeError::LiveExecution(_))
        ));
    }

    fn input(bytes: Vec<u8>) -> WorkerPlanInput {
        let mut metadata = BTreeMap::new();
        metadata.insert("filename".to_string(), "notes.txt".to_string());
        WorkerPlanInput {
            work_item_id: "work-1".to_string(),
            collection_run_id: "run-1".to_string(),
            raw_artifact: RawArtifactRef {
                id: "raw-1".to_string(),
                source_id: "source-1".to_string(),
                content_hash: "abc123".to_string(),
                storage_path: "sha256/ab/c1/abc123".to_string(),
                mime_type: Some("text/plain".to_string()),
                size_bytes: Some(bytes.len() as u64),
                metadata,
            },
            artifact_bytes: bytes,
            chunk_size: 100,
            qdrant_settings: QdrantSettings {
                base_url: "http://localhost:6333".to_string(),
                collection_name: "igy6_chunks".to_string(),
                vector_size: 16,
            },
        }
    }

    #[test]
    fn plans_utf8_pipeline_end_to_end() {
        let plan = plan_utf8_pipeline(input("alpha beta ".repeat(20).into_bytes())).expect("plan");
        assert_eq!(plan.status, "planned");
        assert_eq!(plan.normalized_document.id, "normalized-raw-1");
        assert_eq!(plan.normalized_document.title.as_deref(), Some("notes.txt"));
        assert_eq!(plan.chunks.len(), plan.evidence_items.len());
        assert_eq!(plan.chunks.len(), plan.vector_points.len());
        assert_eq!(
            plan.vector_upsert_request.path,
            "/collections/igy6_chunks/points"
        );
        assert!(plan
            .vector_upsert_request
            .body
            .expect("body")
            .contains("\"points\""));
    }

    #[test]
    fn rejects_non_utf8_artifacts() {
        assert_eq!(
            plan_utf8_pipeline(input(vec![0xff, b'a'])).expect_err("error"),
            WorkerError::NonUtf8Artifact
        );
    }

    #[test]
    fn rejects_invalid_chunk_size() {
        let mut input = input(b"hello".to_vec());
        input.chunk_size = 99;
        assert!(matches!(
            plan_utf8_pipeline(input).expect_err("error"),
            WorkerError::Chunking(ChunkingError::InvalidChunkSize { size: 99 })
        ));
    }

    #[test]
    fn rejects_empty_document_text() {
        assert!(matches!(
            plan_utf8_pipeline(input(Vec::new())).expect_err("error"),
            WorkerError::Chunking(ChunkingError::EmptyText)
        ));
    }

    #[test]
    fn ids_are_deterministic() {
        assert_eq!(chunk_id("doc-1", 3), "doc-1:chunk:3");
        assert_eq!(evidence_id("doc-1", 3), "doc-1:evidence:3");
        let first = plan_utf8_pipeline(input("x".repeat(205).into_bytes())).expect("first");
        let second = plan_utf8_pipeline(input("x".repeat(205).into_bytes())).expect("second");
        assert_eq!(
            chunk_id(&first.normalized_document.id, first.chunks[1].chunk_index),
            chunk_id(&second.normalized_document.id, second.chunks[1].chunk_index)
        );
    }

    #[test]
    fn vector_errors_are_reported() {
        let mut input = input("alpha beta".repeat(30).into_bytes());
        input.qdrant_settings.vector_size = 0;
        assert!(matches!(
            plan_utf8_pipeline(input).expect_err("error"),
            WorkerError::VectorMemory(VectorMemoryError::InvalidVectorSize)
        ));
    }

    fn candidate(work_type: &str, payload_json: Value) -> QueueClaimCandidate {
        QueueClaimCandidate {
            id: "work-1".to_string(),
            work_type: work_type.to_string(),
            status: "queued".to_string(),
            requested_by_actor_id: "local-owner".to_string(),
            payload_json,
        }
    }

    #[test]
    fn queue_claim_query_plan_is_bounded_and_local() {
        let plan = queue_claim_query_plan(4).expect("plan");
        assert_eq!(plan.claim_limit, 4);
        assert_eq!(
            plan.allowed_work_types,
            vec![
                "collection_normalization",
                "document_chunking",
                "chunk_vector_upsert"
            ]
        );
        assert!(plan.select_sql.contains("FOR UPDATE SKIP LOCKED"));
        assert!(plan.update_sql.contains("status = 'running'"));
        assert_eq!(plan.audit_event_type, "work_item.claimed");
        assert_eq!(plan.execution_status, "claimed_without_execution");
        assert!(queue_claim_query_plan(0).is_err());
        assert!(queue_claim_query_plan(17).is_err());
    }

    #[test]
    fn queue_claim_validates_collection_normalization_contract() {
        let plan = plan_queue_claim(
            candidate(
                "collection_normalization",
                json!({
                    "collection_run_id": "run-1",
                    "raw_artifact_ids": ["raw-1"],
                    "intent_verification_recorded": true
                }),
            ),
            "rust-worker",
        )
        .expect("claim");
        assert_eq!(plan.work_type, "collection_normalization");
        assert_eq!(plan.task_name, "collection.normalize_collection_run");
        assert_eq!(plan.previous_status, "queued");
        assert_eq!(plan.next_status, "running");
        assert_eq!(plan.audit_event_type, "work_item.claimed");
        assert_eq!(plan.execution_status, "claimed_without_execution");
    }

    #[test]
    fn queue_claim_validates_document_chunking_contract() {
        let plan = plan_queue_claim(
            candidate(
                "document_chunking",
                json!({
                    "document_ids": ["doc-1"],
                    "intent_verification": {"original_request": "chunk docs"}
                }),
            ),
            "rust-worker",
        )
        .expect("claim");
        assert_eq!(plan.task_name, "evidence.generate_document_chunks");
    }

    #[test]
    fn queue_claim_validates_vector_upsert_contract() {
        let plan = plan_queue_claim(
            candidate(
                "chunk_vector_upsert",
                json!({
                    "chunk_ids": ["chunk-1"],
                    "limit": 1,
                    "intent_verification_recorded": true
                }),
            ),
            "rust-worker",
        )
        .expect("claim");
        assert_eq!(plan.task_name, "memory.vector.upsert_chunks");
    }

    #[test]
    fn queue_claim_rejects_unsafe_or_unready_items() {
        assert!(matches!(
            plan_queue_claim(
                QueueClaimCandidate {
                    status: "pending_intent_verification".to_string(),
                    ..candidate(
                        "document_chunking",
                        json!({"document_id": "doc-1", "intent_verification_recorded": true})
                    )
                },
                "rust-worker"
            )
            .expect_err("status"),
            QueueClaimError::NotQueued(_)
        ));
        assert_eq!(
            plan_queue_claim(
                candidate("document_chunking", json!({"document_id": "doc-1"})),
                "rust-worker"
            )
            .expect_err("intent"),
            QueueClaimError::MissingIntentVerification
        );
        assert!(matches!(
            plan_queue_claim(
                candidate(
                    "shell_command",
                    json!({"intent_verification_recorded": true})
                ),
                "rust-worker"
            )
            .expect_err("unsupported"),
            QueueClaimError::UnsupportedWorkType(_)
        ));
        assert!(matches!(
            plan_queue_claim(
                candidate(
                    "collection_normalization",
                    json!({"collection_run_id": "run-1", "intent_verification_recorded": true})
                ),
                "rust-worker"
            )
            .expect_err("payload"),
            QueueClaimError::InvalidPayload(_)
        ));
        assert_eq!(
            plan_queue_claim(
                candidate(
                    "chunk_vector_upsert",
                    json!({"limit": 1001, "intent_verification_recorded": true})
                ),
                "rust-worker"
            )
            .expect_err("limit"),
            QueueClaimError::InvalidPayload(
                "chunk_vector_upsert limit must be between 1 and 1000".to_string()
            )
        );
        assert_eq!(
            plan_queue_claim(
                candidate(
                    "chunk_vector_upsert",
                    json!({"intent_verification_recorded": true})
                ),
                ""
            )
            .expect_err("actor"),
            QueueClaimError::EmptyActorId
        );
    }

    fn normalization_work_item(payload_json: Value) -> CollectionNormalizationWorkItem {
        CollectionNormalizationWorkItem {
            id: "work-1".to_string(),
            work_type: "collection_normalization".to_string(),
            status: "running".to_string(),
            requested_by_actor_id: "local-owner".to_string(),
            payload_json,
        }
    }

    fn raw_artifact(id: &str, bytes: Vec<u8>) -> RawArtifactRecord {
        raw_artifact_for_run(id, "run-1", bytes)
    }

    fn raw_artifact_for_run(
        id: &str,
        collection_run_id: &str,
        bytes: Vec<u8>,
    ) -> RawArtifactRecord {
        RawArtifactRecord {
            id: id.to_string(),
            source_id: "source-1".to_string(),
            collection_run_id: collection_run_id.to_string(),
            content_hash: format!("hash-{id}"),
            storage_path: format!("sha256/{id}"),
            metadata_json: json!({"filename": format!("{id}.txt")}),
            bytes,
        }
    }

    fn normalization_input() -> CollectionNormalizationExecutionInput {
        CollectionNormalizationExecutionInput {
            work_item: Some(normalization_work_item(json!({
                "collection_run_id": "run-1",
                "raw_artifact_ids": ["raw-1", "raw-2"],
                "intent_verification_recorded": true
            }))),
            requested_collection_run_id: "run-1".to_string(),
            requested_raw_artifact_ids: vec!["raw-1".to_string(), "raw-2".to_string()],
            collection_run: Some(CollectionRunRecord {
                id: "run-1".to_string(),
            }),
            raw_artifacts: vec![
                raw_artifact("raw-1", b"alpha".to_vec()),
                raw_artifact("raw-2", b"beta".to_vec()),
            ],
            existing_documents: Vec::new(),
            generated_document_ids: vec![
                GeneratedDocumentId {
                    raw_artifact_id: "raw-1".to_string(),
                    document_id: "doc-1".to_string(),
                },
                GeneratedDocumentId {
                    raw_artifact_id: "raw-2".to_string(),
                    document_id: "doc-2".to_string(),
                },
            ],
        }
    }

    #[test]
    fn collection_normalization_plans_python_equivalent_success() {
        let plan = plan_collection_normalization_execution(normalization_input()).expect("success");

        assert_eq!(plan.status, "completed");
        assert_eq!(plan.actor_id, "local-owner");
        assert_eq!(plan.normalized_documents.len(), 2);
        assert_eq!(plan.normalized_documents[0].id, "doc-1");
        assert_eq!(plan.normalized_documents[0].raw_artifact_id, "raw-1");
        assert_eq!(
            plan.normalized_documents[0].title.as_deref(),
            Some("raw-1.txt")
        );
        assert_eq!(plan.normalized_documents[0].document_type, "text");
        assert_eq!(plan.normalized_documents[0].language, None);
        assert_eq!(plan.normalized_documents[0].text_content, "alpha");
        assert_eq!(plan.normalized_documents[0].sensitivity, "internal");
        assert_eq!(
            plan.normalized_documents[0].metadata_json,
            json!({
                "generated_by": "DIFF-051",
                "raw_content_hash": "hash-raw-1",
                "raw_storage_path": "sha256/raw-1",
                "work_item_id": "work-1"
            })
        );
        assert_eq!(plan.completion_status_update.status, "completed");
        assert_eq!(plan.completion_status_update.error_message, None);
        assert_eq!(
            plan.completion_audit_event.event_type,
            "collection_normalization.completed"
        );
        assert_eq!(plan.completion_audit_event.decision, "completed");
        assert_eq!(plan.completion_audit_event.correlation_id, "run-1");
        assert_eq!(
            plan.completion_audit_event.details_json["created_document_ids"],
            json!(["doc-1", "doc-2"])
        );
    }

    #[test]
    fn collection_normalization_creates_chained_document_chunking_item_only_when_docs_created() {
        let plan = plan_collection_normalization_execution(normalization_input()).expect("success");
        let chained = plan
            .document_chunking_work_item
            .expect("document chunking item");
        assert_eq!(chained.work_type, "document_chunking");
        assert_eq!(chained.status, "queued");
        assert_eq!(chained.requested_by_actor_id, "local-owner");
        assert_eq!(
            chained.payload_json["document_ids"],
            json!(["doc-1", "doc-2"])
        );
        assert_eq!(chained.payload_json["chunk_size"], json!(1000));
        assert_eq!(
            chained.payload_json["worker_task_name"],
            json!("evidence.generate_document_chunks")
        );
        assert_eq!(
            chained.payload_json["intent_verification"]["recorded_by"],
            json!("DIFF-074 worker chained governance")
        );
        assert_eq!(chained.audit_event.event_type, "work_item.created");
        assert_eq!(chained.audit_event.decision, "queued");
        assert_eq!(
            chained.audit_event.details_json["generated_by"],
            json!("DIFF-066")
        );
    }

    #[test]
    fn collection_normalization_skips_existing_documents_without_chaining_when_no_new_docs() {
        let mut input = normalization_input();
        input.existing_documents = vec![
            ExistingNormalizedDocument {
                id: "existing-1".to_string(),
                raw_artifact_id: "raw-1".to_string(),
            },
            ExistingNormalizedDocument {
                id: "existing-2".to_string(),
                raw_artifact_id: "raw-2".to_string(),
            },
        ];

        let plan = plan_collection_normalization_execution(input).expect("skip");

        assert!(plan.normalized_documents.is_empty());
        assert_eq!(plan.skipped_raw_artifact_ids, vec!["raw-1", "raw-2"]);
        assert!(plan.document_chunking_work_item.is_none());
        assert_eq!(
            plan.completion_audit_event.details_json["document_chunking_work_item_id"],
            Value::Null
        );
    }

    #[test]
    fn collection_normalization_rejects_missing_artifacts() {
        let mut input = normalization_input();
        input.raw_artifacts.pop();

        assert_eq!(
            plan_collection_normalization_execution(input).expect_err("missing"),
            CollectionNormalizationError::MissingRawArtifacts(vec!["raw-2".to_string()])
        );
    }

    #[test]
    fn collection_normalization_rejects_invalid_payload() {
        let mut input = normalization_input();
        input.work_item = Some(normalization_work_item(json!({
            "collection_run_id": "run-1",
            "raw_artifact_ids": ["raw-2", "raw-1"],
            "intent_verification_recorded": true
        })));

        assert_eq!(
            plan_collection_normalization_execution(input).expect_err("payload"),
            CollectionNormalizationError::PayloadMismatch(
                "Work item raw_artifact_ids do not match task request".to_string()
            )
        );
    }

    #[test]
    fn collection_normalization_rejects_collection_mismatch_and_non_utf8() {
        let mut mismatch = normalization_input();
        mismatch.raw_artifacts[0] = raw_artifact_for_run("raw-1", "other-run", b"alpha".to_vec());
        assert_eq!(
            plan_collection_normalization_execution(mismatch).expect_err("mismatch"),
            CollectionNormalizationError::RawArtifactCollectionMismatch("raw-1".to_string())
        );

        let mut non_utf8 = normalization_input();
        non_utf8.raw_artifacts[0] = raw_artifact("raw-1", vec![0xff, b'a']);
        assert_eq!(
            plan_collection_normalization_execution(non_utf8).expect_err("utf8"),
            CollectionNormalizationError::NonUtf8Artifact("raw-1".to_string())
        );
    }

    #[test]
    fn collection_normalization_failure_plan_matches_python_audit_shape() {
        let raw_ids = vec!["raw-1".to_string(), "raw-2".to_string()];
        let (status, audit) = plan_collection_normalization_failure(
            "work-1",
            "run-1",
            &raw_ids,
            "local-owner",
            "Raw artifacts not found: raw-2",
        );

        assert_eq!(status.status, "failed");
        assert_eq!(
            status.error_message.as_deref(),
            Some("Raw artifacts not found: raw-2")
        );
        assert_eq!(audit.event_type, "collection_normalization.failed");
        assert_eq!(audit.decision, "failed");
        assert_eq!(audit.resource_type, "work_item");
        assert_eq!(audit.resource_id, "work-1");
        assert_eq!(audit.correlation_id, "run-1");
        assert_eq!(audit.details_json["raw_artifact_ids"], json!(raw_ids));
        assert_eq!(
            audit.details_json["error_message"],
            json!("Raw artifacts not found: raw-2")
        );
    }

    #[test]
    fn collection_normalization_sql_plan_covers_status_inserts_and_audit() {
        let sql = collection_normalization_sql_plan();
        assert!(sql.mark_running_sql.contains("status = 'running'"));
        assert!(sql
            .insert_normalized_document_sql
            .contains("normalized_documents"));
        assert!(sql.mark_completed_sql.contains("status = 'completed'"));
        assert!(sql.mark_failed_sql.contains("status = 'failed'"));
        assert!(sql
            .insert_chained_work_item_sql
            .contains("'document_chunking'"));
        assert!(sql.insert_audit_event_sql.contains("audit_events"));
    }

    fn chunking_work_item(payload_json: Value) -> DocumentChunkingWorkItem {
        DocumentChunkingWorkItem {
            id: "chunk-work-1".to_string(),
            work_type: "document_chunking".to_string(),
            status: "running".to_string(),
            requested_by_actor_id: "local-owner".to_string(),
            payload_json,
        }
    }

    fn chunking_input() -> DocumentChunkingExecutionInput {
        DocumentChunkingExecutionInput {
            work_item: Some(chunking_work_item(json!({
                "document_ids": ["doc-1"],
                "chunk_size": 100,
                "intent_verification_recorded": true
            }))),
            requested_document_ids: vec!["doc-1".to_string()],
            chunk_size: 100,
            documents: vec![NormalizedDocumentRecord {
                id: "doc-1".to_string(),
                source_id: Some("source-1".to_string()),
                text_content: "a".repeat(205),
            }],
            existing_chunks: Vec::new(),
            generated_chunk_ids: vec![
                GeneratedChunkId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 0,
                    chunk_id: "chunk-1".to_string(),
                },
                GeneratedChunkId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 1,
                    chunk_id: "chunk-2".to_string(),
                },
                GeneratedChunkId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 2,
                    chunk_id: "chunk-3".to_string(),
                },
            ],
            generated_evidence_ids: vec![
                GeneratedEvidenceId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 0,
                    evidence_id: "evidence-1".to_string(),
                },
                GeneratedEvidenceId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 1,
                    evidence_id: "evidence-2".to_string(),
                },
                GeneratedEvidenceId {
                    document_id: "doc-1".to_string(),
                    chunk_index: 2,
                    evidence_id: "evidence-3".to_string(),
                },
            ],
        }
    }

    #[test]
    fn document_chunking_plans_python_equivalent_success() {
        let plan = plan_document_chunking_execution(chunking_input()).expect("success");

        assert_eq!(plan.status, "completed");
        assert_eq!(plan.actor_id, "local-owner");
        assert_eq!(plan.work_item_id, "chunk-work-1");
        assert_eq!(plan.chunks.len(), 3);
        assert_eq!(plan.evidence_items.len(), 3);
        assert_eq!(plan.chunks[0].id, "chunk-1");
        assert_eq!(plan.chunks[0].document_id, "doc-1");
        assert_eq!(plan.chunks[0].chunk_index, 0);
        assert_eq!(plan.chunks[0].text_content.len(), 100);
        assert_eq!(
            plan.chunks[0].location_json,
            json!({"char_start": 0, "char_end": 100})
        );
        assert_eq!(plan.chunks[0].embedding_status, "not_started");
        assert_eq!(
            plan.chunks[0].metadata_json,
            json!({
                "generated_by": "DIFF-052",
                "chunk_size": 100,
                "work_item_id": "chunk-work-1"
            })
        );
        assert_eq!(plan.evidence_items[0].id, "evidence-1");
        assert_eq!(
            plan.evidence_items[0].source_id.as_deref(),
            Some("source-1")
        );
        assert_eq!(plan.evidence_items[0].chunk_id, "chunk-1");
        assert_eq!(plan.evidence_items[0].evidence_type, "document_chunk");
        assert_eq!(plan.evidence_items[0].observed_at, None);
        assert_eq!(plan.evidence_items[0].confidence, None);
        assert_eq!(
            plan.evidence_items[0].metadata_json,
            json!({
                "generated_by": "DIFF-052",
                "chunk_index": 0,
                "work_item_id": "chunk-work-1"
            })
        );
        assert_eq!(plan.completion_status_update.status, "completed");
        assert_eq!(plan.completion_status_update.error_message, None);
        assert_eq!(
            plan.completion_audit_event.event_type,
            "document_chunks.generated"
        );
        assert_eq!(plan.completion_audit_event.decision, "completed");
        assert_eq!(plan.completion_audit_event.resource_type, "work_item");
        assert_eq!(
            plan.completion_audit_event.details_json["chunk_count"],
            json!(3)
        );
        assert_eq!(
            plan.completion_audit_event.details_json["evidence_count"],
            json!(3)
        );
    }

    #[test]
    fn document_chunking_creates_chained_vector_work_item_without_qdrant_work() {
        let plan = plan_document_chunking_execution(chunking_input()).expect("success");
        let chained = plan
            .chunk_vector_upsert_work_item
            .expect("chunk vector work item");

        assert_eq!(chained.work_type, "chunk_vector_upsert");
        assert_eq!(chained.status, "queued");
        assert_eq!(chained.requested_by_actor_id, "local-owner");
        assert_eq!(
            chained.payload_json["chunk_ids"],
            json!(["chunk-1", "chunk-2", "chunk-3"])
        );
        assert_eq!(chained.payload_json["limit"], json!(3));
        assert_eq!(
            chained.payload_json["worker_task_name"],
            json!("memory.vector.upsert_chunks")
        );
        assert_eq!(
            chained.payload_json["intent_verification"]["recorded_by"],
            json!("DIFF-074 worker chained governance")
        );
        assert_eq!(chained.audit_event.event_type, "work_item.created");
        assert_eq!(chained.audit_event.decision, "queued");
        assert_eq!(
            chained.audit_event.details_json["work_type"],
            json!("chunk_vector_upsert")
        );
    }

    #[test]
    fn document_chunking_skips_documents_with_existing_chunks() {
        let mut input = chunking_input();
        input.existing_chunks = vec![ExistingChunkRecord {
            id: "existing-chunk".to_string(),
            document_id: "doc-1".to_string(),
        }];

        let plan = plan_document_chunking_execution(input).expect("skip");

        assert!(plan.chunks.is_empty());
        assert!(plan.evidence_items.is_empty());
        assert_eq!(plan.skipped_document_ids, vec!["doc-1"]);
        assert!(plan.chunk_vector_upsert_work_item.is_none());
        assert_eq!(
            plan.completion_audit_event.details_json["chunk_vector_upsert_work_item_id"],
            Value::Null
        );
    }

    #[test]
    fn document_chunking_rejects_missing_document_invalid_payload_and_empty_text() {
        let mut missing = chunking_input();
        missing.documents.clear();
        assert_eq!(
            plan_document_chunking_execution(missing).expect_err("missing"),
            DocumentChunkingError::MissingDocuments(vec!["doc-1".to_string()])
        );

        let mut invalid_payload = chunking_input();
        invalid_payload.work_item = Some(chunking_work_item(json!({
            "document_ids": ["doc-2"],
            "intent_verification_recorded": true
        })));
        assert_eq!(
            plan_document_chunking_execution(invalid_payload).expect_err("payload"),
            DocumentChunkingError::PayloadMismatch(
                "Work item document IDs do not match task request".to_string()
            )
        );

        let mut empty = chunking_input();
        empty.documents[0].text_content = String::new();
        assert_eq!(
            plan_document_chunking_execution(empty).expect_err("empty"),
            DocumentChunkingError::EmptyDocumentText("doc-1".to_string())
        );
    }

    #[test]
    fn document_chunking_rejects_invalid_chunk_size_and_missing_generated_ids() {
        let mut invalid_size = chunking_input();
        invalid_size.chunk_size = 99;
        assert_eq!(
            plan_document_chunking_execution(invalid_size).expect_err("size"),
            DocumentChunkingError::InvalidChunkSize(99)
        );

        let mut missing_chunk_id = chunking_input();
        missing_chunk_id.generated_chunk_ids.pop();
        assert_eq!(
            plan_document_chunking_execution(missing_chunk_id).expect_err("chunk id"),
            DocumentChunkingError::MissingGeneratedChunkId {
                document_id: "doc-1".to_string(),
                chunk_index: 2
            }
        );

        let mut missing_evidence_id = chunking_input();
        missing_evidence_id.generated_evidence_ids.pop();
        assert_eq!(
            plan_document_chunking_execution(missing_evidence_id).expect_err("evidence id"),
            DocumentChunkingError::MissingGeneratedEvidenceId {
                document_id: "doc-1".to_string(),
                chunk_index: 2
            }
        );
    }

    #[test]
    fn document_chunking_single_document_id_payload_is_supported() {
        let mut input = chunking_input();
        input.work_item = Some(chunking_work_item(json!({
            "document_id": "doc-1",
            "intent_verification_recorded": true
        })));

        let plan = plan_document_chunking_execution(input).expect("single id");

        assert_eq!(plan.document_ids, vec!["doc-1"]);
        assert_eq!(plan.chunks.len(), 3);
    }

    #[test]
    fn document_chunking_failure_plan_matches_python_audit_shape() {
        let document_ids = vec!["doc-1".to_string()];
        let (status, audit) = plan_document_chunking_failure(
            "chunk-work-1",
            &document_ids,
            "local-owner",
            "Documents not found: doc-1",
        );

        assert_eq!(status.status, "failed");
        assert_eq!(
            status.error_message.as_deref(),
            Some("Documents not found: doc-1")
        );
        assert_eq!(audit.event_type, "document_chunks.failed");
        assert_eq!(audit.decision, "failed");
        assert_eq!(audit.resource_type, "work_item");
        assert_eq!(audit.resource_id, "chunk-work-1");
        assert_eq!(audit.correlation_id, "chunk-work-1");
        assert_eq!(audit.details_json["document_ids"], json!(document_ids));
        assert_eq!(
            audit.details_json["error_message"],
            json!("Documents not found: doc-1")
        );
    }

    #[test]
    fn document_chunking_sql_plan_covers_status_inserts_and_audit() {
        let sql = document_chunking_sql_plan();
        assert!(sql.mark_running_sql.contains("status = 'running'"));
        assert!(sql.insert_chunk_sql.contains("INSERT INTO chunks"));
        assert!(sql
            .insert_evidence_item_sql
            .contains("INSERT INTO evidence_items"));
        assert!(sql.mark_completed_sql.contains("status = 'completed'"));
        assert!(sql.mark_failed_sql.contains("status = 'failed'"));
        assert!(sql
            .insert_chained_work_item_sql
            .contains("'chunk_vector_upsert'"));
        assert!(sql.insert_audit_event_sql.contains("audit_events"));
    }

    fn vector_work_item(payload_json: Value) -> ChunkVectorUpsertWorkItem {
        ChunkVectorUpsertWorkItem {
            id: "vector-work-1".to_string(),
            work_type: "chunk_vector_upsert".to_string(),
            status: "running".to_string(),
            requested_by_actor_id: "local-owner".to_string(),
            payload_json,
        }
    }

    fn vector_settings() -> QdrantSettings {
        QdrantSettings {
            base_url: "http://qdrant:6333".to_string(),
            collection_name: "igy6_chunks".to_string(),
            vector_size: 8,
        }
    }

    fn chunk_for_vector(id: &str, embedding_status: &str) -> ChunkForVectorRecord {
        ChunkForVectorRecord {
            id: id.to_string(),
            document_id: format!("doc-{id}"),
            chunk_index: 0,
            text_content: format!("alpha beta {id}"),
            embedding_status: embedding_status.to_string(),
            metadata_json: json!({"generated_by": "DIFF-052", "chunk_size": 100}),
        }
    }

    fn vector_input() -> ChunkVectorUpsertExecutionInput {
        ChunkVectorUpsertExecutionInput {
            work_item: Some(vector_work_item(json!({
                "chunk_ids": ["chunk-2", "chunk-1"],
                "limit": 2,
                "intent_verification_recorded": true
            }))),
            limit: 2,
            candidate_chunks: vec![
                chunk_for_vector("chunk-3", "not_started"),
                chunk_for_vector("chunk-1", "not_started"),
                chunk_for_vector("chunk-2", "completed"),
                chunk_for_vector("chunk-2", "not_started"),
            ],
            qdrant_settings: vector_settings(),
        }
    }

    #[test]
    fn chunk_vector_upsert_plans_python_equivalent_success() {
        let plan = plan_chunk_vector_upsert_execution(vector_input()).expect("success");

        assert_eq!(plan.status, "completed");
        assert_eq!(plan.actor_id, "local-owner");
        assert_eq!(plan.work_item_id, "vector-work-1");
        assert_eq!(
            plan.requested_chunk_ids,
            Some(vec!["chunk-2".to_string(), "chunk-1".to_string()])
        );
        assert_eq!(plan.selected_chunk_ids, vec!["chunk-1", "chunk-2"]);
        assert_eq!(plan.points.len(), 2);
        assert_eq!(plan.points[0].id, "chunk-1");
        assert_eq!(plan.points[0].vector.len(), 8);
        assert_eq!(plan.points[0].payload_json["chunk_id"], json!("chunk-1"));
        assert_eq!(
            plan.points[0].payload_json["embedding_method"],
            json!(WORKER_EMBEDDING_METHOD)
        );
        assert_eq!(
            plan.points[0].payload_json["generated_by"],
            json!(WORKER_VECTOR_GENERATED_BY)
        );
        assert_eq!(
            plan.collection_status_request.path,
            "/collections/igy6_chunks"
        );
        assert_eq!(
            plan.ensure_collection_request
                .as_ref()
                .expect("ensure")
                .path,
            "/collections/igy6_chunks"
        );
        assert_eq!(
            plan.upsert_points_request.as_ref().expect("upsert").path,
            "/collections/igy6_chunks/points"
        );
        assert_eq!(plan.chunk_updates.len(), 2);
        assert_eq!(plan.chunk_updates[0].chunk_id, "chunk-1");
        assert_eq!(plan.chunk_updates[0].embedding_status, "completed");
        assert_eq!(
            plan.chunk_updates[0].metadata_json["embedding_method"],
            json!(WORKER_EMBEDDING_METHOD)
        );
        assert_eq!(
            plan.chunk_updates[0].metadata_json["vector_collection"],
            json!("igy6_chunks")
        );
        assert_eq!(plan.completion_status_update.status, "completed");
        assert_eq!(
            plan.completion_audit_event.event_type,
            "chunk_vectors.upserted"
        );
        assert_eq!(
            plan.completion_audit_event.details_json["chunks_selected"],
            json!(2)
        );
        assert_eq!(
            plan.completion_audit_event.details_json["chunks_upserted"],
            json!(2)
        );
        assert_eq!(
            plan.completion_audit_event.details_json["chunk_ids"],
            json!(["chunk-1", "chunk-2"])
        );
    }

    #[test]
    fn chunk_vector_upsert_without_requested_ids_selects_oldest_uncompleted_to_limit() {
        let mut input = vector_input();
        input.work_item = Some(vector_work_item(json!({
            "limit": 1,
            "intent_verification_recorded": true
        })));
        input.limit = 1;

        let plan = plan_chunk_vector_upsert_execution(input).expect("success");

        assert_eq!(plan.requested_chunk_ids, None);
        assert_eq!(plan.selected_chunk_ids, vec!["chunk-1"]);
        assert_eq!(plan.points.len(), 1);
    }

    #[test]
    fn chunk_vector_upsert_empty_selection_completes_without_upsert_request() {
        let mut input = vector_input();
        input.candidate_chunks = vec![chunk_for_vector("chunk-1", "completed")];

        let plan = plan_chunk_vector_upsert_execution(input).expect("empty");

        assert!(plan.selected_chunk_ids.is_empty());
        assert!(plan.points.is_empty());
        assert!(plan.ensure_collection_request.is_none());
        assert!(plan.upsert_points_request.is_none());
        assert!(plan.chunk_updates.is_empty());
        assert_eq!(
            plan.completion_audit_event.details_json["chunks_selected"],
            json!(0)
        );
    }

    #[test]
    fn chunk_vector_upsert_rejects_invalid_limit_payload_type_and_settings() {
        let mut invalid_limit = vector_input();
        invalid_limit.limit = 0;
        assert_eq!(
            plan_chunk_vector_upsert_execution(invalid_limit).expect_err("limit"),
            ChunkVectorUpsertError::InvalidLimit(0)
        );

        let mut invalid_payload = vector_input();
        invalid_payload.work_item = Some(vector_work_item(json!({
            "chunk_ids": "chunk-1",
            "intent_verification_recorded": true
        })));
        assert_eq!(
            plan_chunk_vector_upsert_execution(invalid_payload).expect_err("payload"),
            ChunkVectorUpsertError::InvalidPayload(
                "chunk_vector_upsert chunk_ids must be an array".to_string()
            )
        );

        let mut invalid_settings = vector_input();
        invalid_settings.qdrant_settings.vector_size = 0;
        assert_eq!(
            plan_chunk_vector_upsert_execution(invalid_settings).expect_err("settings"),
            ChunkVectorUpsertError::VectorMemory(VectorMemoryError::InvalidVectorSize)
        );
    }

    #[test]
    fn chunk_vector_upsert_rejects_missing_or_wrong_work_item() {
        let mut missing = vector_input();
        missing.work_item = None;
        assert_eq!(
            plan_chunk_vector_upsert_execution(missing).expect_err("missing"),
            ChunkVectorUpsertError::WorkItemNotFound
        );

        let mut wrong = vector_input();
        wrong.work_item = Some(ChunkVectorUpsertWorkItem {
            work_type: "document_chunking".to_string(),
            ..vector_work_item(json!({"intent_verification_recorded": true}))
        });
        assert_eq!(
            plan_chunk_vector_upsert_execution(wrong).expect_err("wrong"),
            ChunkVectorUpsertError::WrongWorkType("document_chunking".to_string())
        );
    }

    #[test]
    fn chunk_vector_upsert_failure_plan_matches_python_audit_shape() {
        let (status, audit) = plan_chunk_vector_upsert_failure(
            "vector-work-1",
            100,
            "local-owner",
            "qdrant unavailable",
        );

        assert_eq!(status.status, "failed");
        assert_eq!(status.error_message.as_deref(), Some("qdrant unavailable"));
        assert_eq!(audit.event_type, "chunk_vectors.failed");
        assert_eq!(audit.decision, "failed");
        assert_eq!(audit.resource_type, "work_item");
        assert_eq!(audit.resource_id, "vector-work-1");
        assert_eq!(audit.correlation_id, "vector-work-1");
        assert_eq!(audit.details_json["limit"], json!(100));
        assert_eq!(
            audit.details_json["error_message"],
            json!("qdrant unavailable")
        );
    }

    #[test]
    fn chunk_vector_upsert_sql_plan_covers_selection_updates_and_audit() {
        let sql = chunk_vector_upsert_sql_plan();
        assert!(sql.mark_running_sql.contains("status = 'running'"));
        assert!(sql
            .select_chunks_sql
            .contains("embedding_status != 'completed'"));
        assert!(sql.select_requested_chunks_sql.contains("id = ANY($1)"));
        assert!(sql
            .update_chunk_completed_sql
            .contains("embedding_status = 'completed'"));
        assert!(sql.mark_completed_sql.contains("status = 'completed'"));
        assert!(sql.mark_failed_sql.contains("status = 'failed'"));
        assert!(sql.insert_audit_event_sql.contains("audit_events"));
    }
}
