pub const CHUNK_VECTOR_SEARCH_MAX_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalChunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub text_content: String,
    pub embedding_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalDocument {
    pub id: String,
    pub raw_artifact_id: Option<String>,
    pub source_id: Option<String>,
    pub title: Option<String>,
    pub document_type: String,
    pub sensitivity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalSource {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub trust_level: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalRawArtifact {
    pub id: String,
    pub source_id: Option<String>,
    pub content_hash: String,
    pub storage_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalEvidenceItem {
    pub id: String,
    pub source_id: Option<String>,
    pub document_id: Option<String>,
    pub chunk_id: Option<String>,
    pub evidence_type: String,
    pub statement: String,
    pub confidence: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HydratedChunkSearchHit {
    pub score: f64,
    pub qdrant_payload_summary: String,
    pub chunk: RetrievalChunk,
    pub document: RetrievalDocument,
    pub source: Option<RetrievalSource>,
    pub raw_artifact: Option<RetrievalRawArtifact>,
    pub evidence_items: Vec<RetrievalEvidenceItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HydratedChunkSearchResult {
    pub query: String,
    pub collection_name: String,
    pub collection_exists: bool,
    pub hits: Vec<HydratedChunkSearchHit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatRetrievalPreviewResponse {
    pub message: String,
    pub answer_status: String,
    pub retrieval_context: HydratedChunkSearchResult,
}

pub fn bounded_limit(limit: usize) -> usize {
    limit.clamp(1, CHUNK_VECTOR_SEARCH_MAX_LIMIT)
}

pub fn is_source_allowed_for_retrieval(source: Option<&RetrievalSource>) -> bool {
    source.map(|source| source.enabled).unwrap_or(true)
}

pub fn build_hydrated_chunk_search_result(
    query: &str,
    collection_name: &str,
    collection_exists: bool,
    hits: Vec<HydratedChunkSearchHit>,
    limit: usize,
) -> HydratedChunkSearchResult {
    let bounded = bounded_limit(limit);
    let filtered_hits = hits
        .into_iter()
        .filter(|hit| is_source_allowed_for_retrieval(hit.source.as_ref()))
        .take(bounded)
        .collect();

    HydratedChunkSearchResult {
        query: query.to_string(),
        collection_name: collection_name.to_string(),
        collection_exists,
        hits: filtered_hits,
    }
}

pub fn build_retrieval_preview(
    message: &str,
    retrieval_context: HydratedChunkSearchResult,
) -> ChatRetrievalPreviewResponse {
    ChatRetrievalPreviewResponse {
        message: message.to_string(),
        answer_status: "not_generated".to_string(),
        retrieval_context,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: &str, source_enabled: Option<bool>) -> HydratedChunkSearchHit {
        HydratedChunkSearchHit {
            score: 0.9,
            qdrant_payload_summary: format!("payload-{id}"),
            chunk: RetrievalChunk {
                id: format!("chunk-{id}"),
                document_id: format!("doc-{id}"),
                chunk_index: 0,
                text_content: format!("text {id}"),
                embedding_status: "completed".to_string(),
            },
            document: RetrievalDocument {
                id: format!("doc-{id}"),
                raw_artifact_id: Some(format!("raw-{id}")),
                source_id: source_enabled.map(|_| format!("source-{id}")),
                title: Some(format!("title {id}")),
                document_type: "text".to_string(),
                sensitivity: "internal".to_string(),
            },
            source: source_enabled.map(|enabled| RetrievalSource {
                id: format!("source-{id}"),
                name: format!("source {id}"),
                source_type: "manual_upload".to_string(),
                trust_level: "standard".to_string(),
                enabled,
            }),
            raw_artifact: Some(RetrievalRawArtifact {
                id: format!("raw-{id}"),
                source_id: source_enabled.map(|_| format!("source-{id}")),
                content_hash: format!("hash-{id}"),
                storage_path: format!("sha256/aa/bb/hash-{id}"),
            }),
            evidence_items: vec![RetrievalEvidenceItem {
                id: format!("evidence-{id}"),
                source_id: source_enabled.map(|_| format!("source-{id}")),
                document_id: Some(format!("doc-{id}")),
                chunk_id: Some(format!("chunk-{id}")),
                evidence_type: "document_chunk".to_string(),
                statement: format!("text {id}"),
                confidence: Some(80),
            }],
        }
    }

    #[test]
    fn preview_preserves_not_generated_status() {
        let context = build_hydrated_chunk_search_result(
            "question",
            "igy6_chunks",
            true,
            vec![hit("1", Some(true))],
            10,
        );
        let preview = build_retrieval_preview("question", context);
        assert_eq!(preview.answer_status, "not_generated");
        assert_eq!(preview.message, "question");
    }

    #[test]
    fn limit_is_bounded() {
        assert_eq!(bounded_limit(0), 1);
        assert_eq!(bounded_limit(99), 50);
        let hits = (0..60)
            .map(|index| hit(&index.to_string(), Some(true)))
            .collect();
        let result = build_hydrated_chunk_search_result("q", "c", true, hits, 99);
        assert_eq!(result.hits.len(), 50);
    }

    #[test]
    fn disabled_sources_are_filtered() {
        let result = build_hydrated_chunk_search_result(
            "q",
            "c",
            true,
            vec![hit("enabled", Some(true)), hit("disabled", Some(false))],
            10,
        );
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].chunk.id, "chunk-enabled");
    }

    #[test]
    fn missing_source_is_allowed() {
        let result =
            build_hydrated_chunk_search_result("q", "c", true, vec![hit("no-source", None)], 10);
        assert_eq!(result.hits.len(), 1);
        assert!(result.hits[0].source.is_none());
    }

    #[test]
    fn source_trail_metadata_is_preserved() {
        let result =
            build_hydrated_chunk_search_result("q", "c", true, vec![hit("1", Some(true))], 10);
        let first = &result.hits[0];
        assert_eq!(first.chunk.id, "chunk-1");
        assert_eq!(first.document.raw_artifact_id.as_deref(), Some("raw-1"));
        assert_eq!(first.source.as_ref().expect("source").name, "source 1");
        assert_eq!(
            first.raw_artifact.as_ref().expect("raw").content_hash,
            "hash-1"
        );
        assert_eq!(first.evidence_items[0].id, "evidence-1");
        assert_eq!(first.score, 0.9);
    }
}
