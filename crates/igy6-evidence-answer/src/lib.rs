use std::collections::BTreeSet;

use igy6_retrieval_preview::HydratedChunkSearchResult;

#[derive(Debug, Clone, PartialEq)]
pub struct AnswerCitation {
    pub citation_id: String,
    pub citation_type: String,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub document_id: String,
    pub document_title: Option<String>,
    pub chunk_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnswerStatement {
    pub text: String,
    pub confidence: Option<i32>,
    pub citations: Vec<AnswerCitation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnswerSourceTrail {
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub source_type: Option<String>,
    pub trust_level: Option<String>,
    pub document_id: String,
    pub document_title: Option<String>,
    pub raw_artifact_id: Option<String>,
    pub chunk_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAnswerResponse {
    pub message: String,
    pub answer_status: String,
    pub facts: Vec<AnswerStatement>,
    pub assumptions: Vec<String>,
    pub inferences: Vec<AnswerStatement>,
    pub uncertainty: Vec<String>,
    pub missing_information: Vec<String>,
    pub source_trails: Vec<AnswerSourceTrail>,
    pub retrieval_context: HydratedChunkSearchResult,
}

pub fn build_evidence_answer_packet(
    retrieval_context: HydratedChunkSearchResult,
) -> EvidenceAnswerResponse {
    let mut facts = Vec::new();
    let mut inferences = Vec::new();
    let mut source_trails = Vec::new();
    let mut seen_fact_keys = BTreeSet::new();
    let mut seen_trail_keys = BTreeSet::new();

    for hit in &retrieval_context.hits {
        let source_trail = AnswerSourceTrail {
            source_id: hit.source.as_ref().map(|source| source.id.clone()),
            source_name: hit.source.as_ref().map(|source| source.name.clone()),
            source_type: hit.source.as_ref().map(|source| source.source_type.clone()),
            trust_level: hit.source.as_ref().map(|source| source.trust_level.clone()),
            document_id: hit.document.id.clone(),
            document_title: hit.document.title.clone(),
            raw_artifact_id: hit.raw_artifact.as_ref().map(|raw| raw.id.clone()),
            chunk_id: hit.chunk.id.clone(),
            score: hit.score,
        };
        let trail_key = format!("{}:{}", source_trail.document_id, source_trail.chunk_id);
        if seen_trail_keys.insert(trail_key) {
            source_trails.push(source_trail);
        }

        if hit.evidence_items.is_empty() {
            if seen_fact_keys.insert(hit.chunk.id.clone()) {
                facts.push(AnswerStatement {
                    text: excerpt(&hit.chunk.text_content, 220),
                    confidence: Some(confidence_from_hit(hit.score, None)),
                    citations: vec![AnswerCitation {
                        citation_id: hit.chunk.id.clone(),
                        citation_type: "chunk".to_string(),
                        source_id: hit.source.as_ref().map(|source| source.id.clone()),
                        source_name: hit.source.as_ref().map(|source| source.name.clone()),
                        document_id: hit.document.id.clone(),
                        document_title: hit.document.title.clone(),
                        chunk_id: hit.chunk.id.clone(),
                        score: hit.score,
                    }],
                });
            }
            continue;
        }

        for evidence_item in &hit.evidence_items {
            if !seen_fact_keys.insert(evidence_item.id.clone()) {
                continue;
            }
            facts.push(AnswerStatement {
                text: evidence_item.statement.clone(),
                confidence: Some(confidence_from_hit(hit.score, evidence_item.confidence)),
                citations: vec![AnswerCitation {
                    citation_id: evidence_item.id.clone(),
                    citation_type: "evidence_item".to_string(),
                    source_id: hit
                        .source
                        .as_ref()
                        .map(|source| source.id.clone())
                        .or_else(|| evidence_item.source_id.clone()),
                    source_name: hit.source.as_ref().map(|source| source.name.clone()),
                    document_id: hit.document.id.clone(),
                    document_title: hit.document.title.clone(),
                    chunk_id: hit.chunk.id.clone(),
                    score: hit.score,
                }],
            });
        }
    }

    if !facts.is_empty() {
        let cited_ids = facts
            .iter()
            .take(3)
            .filter_map(|statement| statement.citations.first())
            .map(|citation| citation.citation_id.clone())
            .collect::<Vec<_>>();
        let confidence = facts
            .iter()
            .take(3)
            .filter_map(|statement| statement.confidence)
            .min()
            .unwrap_or(0);
        let citations = facts
            .iter()
            .take(3)
            .flat_map(|statement| statement.citations.clone())
            .collect();
        inferences.push(AnswerStatement {
            text: format!(
                "The available answer is limited to the retrieved local evidence. The strongest cited records are: {}.",
                cited_ids.join(", ")
            ),
            confidence: Some(confidence),
            citations,
        });
    }

    let mut missing_information = Vec::new();
    if facts.is_empty() {
        missing_information.push(
            "No matching chunks or evidence items were retrieved for the message.".to_string(),
        );
    } else {
        missing_information.push(
            "Any relevant source not yet ingested, chunked, and embedded is absent from this answer."
                .to_string(),
        );
    }

    EvidenceAnswerResponse {
        message: retrieval_context.query.clone(),
        answer_status: if facts.is_empty() {
            "insufficient_evidence".to_string()
        } else {
            "evidence_summary".to_string()
        },
        facts,
        assumptions: vec![
            "Registered source metadata and stored evidence records are treated as local records of what was collected.".to_string(),
            "Retrieval scores are similarity signals, not proof of correctness.".to_string(),
        ],
        inferences,
        uncertainty: vec![
            "This deterministic answer packet uses local retrieval scores and stored evidence only.".to_string(),
            "No external model, hidden reasoning, or graph inference was used.".to_string(),
        ],
        missing_information,
        source_trails,
        retrieval_context,
    }
}

pub fn confidence_from_hit(score: f64, evidence_confidence: Option<i32>) -> i32 {
    let score_confidence = (score * 100.0).round().clamp(0.0, 100.0) as i32;
    match evidence_confidence {
        Some(confidence) => ((score_confidence + confidence.clamp(0, 100)) as f64 / 2.0)
            .round()
            .clamp(0.0, 100.0) as i32,
        None => score_confidence,
    }
}

pub fn excerpt(value: &str, max_length: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= max_length {
        return normalized;
    }
    if max_length <= 3 {
        return "...".chars().take(max_length).collect();
    }
    let prefix = normalized.chars().take(max_length - 3).collect::<String>();
    format!("{prefix}...")
}

#[cfg(test)]
mod tests {
    use super::*;
    use igy6_retrieval_preview::{
        HydratedChunkSearchHit, HydratedChunkSearchResult, RetrievalChunk, RetrievalDocument,
        RetrievalEvidenceItem, RetrievalRawArtifact, RetrievalSource,
    };

    fn context(with_evidence: bool) -> HydratedChunkSearchResult {
        HydratedChunkSearchResult {
            query: "what is known?".to_string(),
            collection_name: "igy6_chunks".to_string(),
            collection_exists: true,
            hits: vec![HydratedChunkSearchHit {
                score: 0.82,
                qdrant_payload_summary: "payload".to_string(),
                chunk: RetrievalChunk {
                    id: "chunk-1".to_string(),
                    document_id: "doc-1".to_string(),
                    chunk_index: 0,
                    text_content: "chunk text with several words".to_string(),
                    embedding_status: "completed".to_string(),
                },
                document: RetrievalDocument {
                    id: "doc-1".to_string(),
                    raw_artifact_id: Some("raw-1".to_string()),
                    source_id: Some("source-1".to_string()),
                    title: Some("Doc".to_string()),
                    document_type: "text".to_string(),
                    sensitivity: "internal".to_string(),
                },
                source: Some(RetrievalSource {
                    id: "source-1".to_string(),
                    name: "Source".to_string(),
                    source_type: "manual_upload".to_string(),
                    trust_level: "standard".to_string(),
                    enabled: true,
                }),
                raw_artifact: Some(RetrievalRawArtifact {
                    id: "raw-1".to_string(),
                    source_id: Some("source-1".to_string()),
                    content_hash: "hash".to_string(),
                    storage_path: "sha256/aa/bb/hash".to_string(),
                }),
                evidence_items: if with_evidence {
                    vec![RetrievalEvidenceItem {
                        id: "evidence-1".to_string(),
                        source_id: Some("source-1".to_string()),
                        document_id: Some("doc-1".to_string()),
                        chunk_id: Some("chunk-1".to_string()),
                        evidence_type: "document_chunk".to_string(),
                        statement: "stored evidence statement".to_string(),
                        confidence: Some(90),
                    }]
                } else {
                    Vec::new()
                },
            }],
        }
    }

    #[test]
    fn evidence_items_are_used_for_facts() {
        let answer = build_evidence_answer_packet(context(true));
        assert_eq!(answer.answer_status, "evidence_summary");
        assert_eq!(answer.facts[0].text, "stored evidence statement");
        assert_eq!(answer.facts[0].citations[0].citation_type, "evidence_item");
    }

    #[test]
    fn chunks_are_used_when_evidence_is_absent() {
        let answer = build_evidence_answer_packet(context(false));
        assert_eq!(answer.answer_status, "evidence_summary");
        assert_eq!(answer.facts[0].text, "chunk text with several words");
        assert_eq!(answer.facts[0].citations[0].citation_type, "chunk");
    }

    #[test]
    fn insufficient_evidence_when_no_hits_exist() {
        let mut context = context(false);
        context.hits.clear();
        let answer = build_evidence_answer_packet(context);
        assert_eq!(answer.answer_status, "insufficient_evidence");
        assert!(answer.facts.is_empty());
        assert!(answer.missing_information[0].contains("No matching chunks"));
    }

    #[test]
    fn source_trails_are_deduplicated() {
        let mut context = context(false);
        context.hits.push(context.hits[0].clone());
        let answer = build_evidence_answer_packet(context);
        assert_eq!(answer.source_trails.len(), 1);
    }

    #[test]
    fn confidence_is_bounded_and_combined() {
        assert_eq!(confidence_from_hit(1.5, None), 100);
        assert_eq!(confidence_from_hit(-0.5, None), 0);
        assert_eq!(confidence_from_hit(0.8, Some(100)), 90);
    }

    #[test]
    fn inference_references_strongest_citations() {
        let answer = build_evidence_answer_packet(context(true));
        assert_eq!(answer.inferences.len(), 1);
        assert!(answer.inferences[0].text.contains("evidence-1"));
    }
}
