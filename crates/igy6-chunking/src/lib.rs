use std::fmt;

pub const MIN_CHUNK_SIZE: usize = 100;
pub const MAX_CHUNK_SIZE: usize = 5000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPlan {
    pub document_id: String,
    pub chunk_index: usize,
    pub text_content: String,
    pub char_start: usize,
    pub char_end: usize,
    pub chunk_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidencePlan {
    pub source_id: Option<String>,
    pub document_id: String,
    pub chunk_index: usize,
    pub evidence_type: String,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkingPlan {
    pub chunks: Vec<ChunkPlan>,
    pub evidence_items: Vec<EvidencePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkingError {
    EmptyText,
    InvalidChunkSize { size: usize },
}

impl fmt::Display for ChunkingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText => write!(formatter, "document text is empty"),
            Self::InvalidChunkSize { size } => write!(
                formatter,
                "chunk size must be between {MIN_CHUNK_SIZE} and {MAX_CHUNK_SIZE}, got {size}"
            ),
        }
    }
}

impl std::error::Error for ChunkingError {}

pub fn split_text_chunks(text: &str, chunk_size: usize) -> Result<Vec<String>, ChunkingError> {
    validate_chunk_size(chunk_size)?;
    if text.is_empty() {
        return Err(ChunkingError::EmptyText);
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Err(ChunkingError::EmptyText);
    }

    Ok(chars
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect::<String>())
        .filter(|chunk| !chunk.is_empty())
        .collect())
}

pub fn plan_document_chunks(
    document_id: &str,
    source_id: Option<&str>,
    text: &str,
    chunk_size: usize,
) -> Result<ChunkingPlan, ChunkingError> {
    let text_chunks = split_text_chunks(text, chunk_size)?;
    let mut char_start = 0usize;
    let mut chunks = Vec::with_capacity(text_chunks.len());
    let mut evidence_items = Vec::with_capacity(text_chunks.len());

    for (chunk_index, text_content) in text_chunks.into_iter().enumerate() {
        let length = text_content.chars().count();
        let char_end = char_start + length;
        chunks.push(ChunkPlan {
            document_id: document_id.to_string(),
            chunk_index,
            text_content: text_content.clone(),
            char_start,
            char_end,
            chunk_size,
        });
        evidence_items.push(EvidencePlan {
            source_id: source_id.map(ToString::to_string),
            document_id: document_id.to_string(),
            chunk_index,
            evidence_type: "document_chunk".to_string(),
            statement: text_content,
        });
        char_start = char_end;
    }

    Ok(ChunkingPlan {
        chunks,
        evidence_items,
    })
}

fn validate_chunk_size(chunk_size: usize) -> Result<(), ChunkingError> {
    if (MIN_CHUNK_SIZE..=MAX_CHUNK_SIZE).contains(&chunk_size) {
        Ok(())
    } else {
        Err(ChunkingError::InvalidChunkSize { size: chunk_size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_text_deterministically() {
        let text = "a".repeat(250);
        let chunks = split_text_chunks(&text, 100).expect("chunks");
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
        assert_eq!(chunks[2].len(), 50);
    }

    #[test]
    fn plans_chunk_boundaries_and_evidence() {
        let text = "x".repeat(205);
        let plan = plan_document_chunks("doc-1", Some("source-1"), &text, 100).expect("plan");
        assert_eq!(plan.chunks.len(), 3);
        assert_eq!(plan.evidence_items.len(), 3);
        assert_eq!(plan.chunks[0].char_start, 0);
        assert_eq!(plan.chunks[0].char_end, 100);
        assert_eq!(plan.chunks[2].char_start, 200);
        assert_eq!(plan.chunks[2].char_end, 205);
        assert_eq!(plan.evidence_items[0].evidence_type, "document_chunk");
    }

    #[test]
    fn empty_text_is_rejected() {
        assert!(matches!(
            split_text_chunks("", 100),
            Err(ChunkingError::EmptyText)
        ));
    }

    #[test]
    fn invalid_chunk_sizes_are_rejected() {
        assert!(matches!(
            split_text_chunks("abc", 99),
            Err(ChunkingError::InvalidChunkSize { size: 99 })
        ));
        assert!(matches!(
            split_text_chunks("abc", 5001),
            Err(ChunkingError::InvalidChunkSize { size: 5001 })
        ));
    }

    #[test]
    fn non_ascii_boundaries_are_character_based() {
        let text = "é".repeat(101);
        let chunks = split_text_chunks(&text, 100).expect("chunks");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chars().count(), 100);
        assert_eq!(chunks[1].chars().count(), 1);
        let plan = plan_document_chunks("doc-1", None, &text, 100).expect("plan");
        assert_eq!(plan.chunks[1].char_start, 100);
        assert_eq!(plan.chunks[1].char_end, 101);
    }
}
