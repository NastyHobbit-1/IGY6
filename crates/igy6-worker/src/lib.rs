use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use igy6_chunking::{plan_document_chunks, ChunkPlan, ChunkingError, EvidencePlan};
use igy6_normalization::{
    build_normalized_document_ref, NormalizedDocumentInput, NormalizedDocumentRef, RawArtifactRef,
};
use igy6_vector_memory::{
    upsert_points_request, ChunkVectorPoint, HttpRequestPlan, QdrantSettings, VectorMemoryError,
};
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
}
