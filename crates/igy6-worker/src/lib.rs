use std::collections::BTreeMap;
use std::fmt;

use igy6_chunking::{plan_document_chunks, ChunkPlan, ChunkingError, EvidencePlan};
use igy6_normalization::{
    build_normalized_document_ref, NormalizedDocumentInput, NormalizedDocumentRef, RawArtifactRef,
};
use igy6_vector_memory::{
    upsert_points_request, ChunkVectorPoint, HttpRequestPlan, QdrantSettings, VectorMemoryError,
};
use serde_json::Value;

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
}
