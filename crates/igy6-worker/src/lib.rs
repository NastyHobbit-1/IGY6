use std::collections::BTreeMap;
use std::fmt;

use igy6_chunking::{plan_document_chunks, ChunkPlan, ChunkingError, EvidencePlan};
use igy6_normalization::{
    build_normalized_document_ref, NormalizedDocumentInput, NormalizedDocumentRef, RawArtifactRef,
};
use igy6_vector_memory::{
    upsert_points_request, ChunkVectorPoint, HttpRequestPlan, QdrantSettings, VectorMemoryError,
};

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
}
