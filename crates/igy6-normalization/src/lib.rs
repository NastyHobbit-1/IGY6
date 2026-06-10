use std::collections::BTreeMap;

const KNOWN_SENSITIVITY_LABELS: &[&str] = &["public", "internal", "sensitive", "secret"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawArtifactRef {
    pub id: String,
    pub source_id: String,
    pub content_hash: String,
    pub storage_path: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDocumentRef {
    pub id: String,
    pub raw_artifact_id: String,
    pub source_id: String,
    pub text_content: String,
    pub title: Option<String>,
    pub document_type: String,
    pub language: Option<String>,
    pub sensitivity: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextNormalization {
    pub text: String,
    pub used_replacement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDocumentInput {
    pub text_content: String,
    pub title: Option<String>,
    pub document_type: String,
    pub language: Option<String>,
    pub sensitivity: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

impl Default for NormalizedDocumentInput {
    fn default() -> Self {
        Self {
            text_content: String::new(),
            title: None,
            document_type: "unknown".to_string(),
            language: None,
            sensitivity: None,
            metadata: BTreeMap::new(),
        }
    }
}

pub fn classify_sensitivity_label(value: Option<&str>, fallback: &str) -> String {
    match value {
        Some(label) if KNOWN_SENSITIVITY_LABELS.contains(&label) => label.to_string(),
        _ => fallback.to_string(),
    }
}

pub fn normalize_utf8_bytes(bytes: &[u8]) -> TextNormalization {
    match std::str::from_utf8(bytes) {
        Ok(text) => TextNormalization {
            text: normalize_text(text),
            used_replacement: false,
        },
        Err(_) => {
            let text = String::from_utf8_lossy(bytes);
            TextNormalization {
                text: normalize_text(&text),
                used_replacement: true,
            }
        }
    }
}

pub fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn build_normalized_document_ref(
    raw: &RawArtifactRef,
    input: NormalizedDocumentInput,
) -> NormalizedDocumentRef {
    let mut metadata = BTreeMap::from([
        (
            "normalized_from_raw_artifact_id".to_string(),
            raw.id.clone(),
        ),
        ("raw_content_hash".to_string(), raw.content_hash.clone()),
        ("raw_storage_path".to_string(), raw.storage_path.clone()),
    ]);
    for (key, value) in input.metadata {
        metadata.insert(key, value);
    }

    NormalizedDocumentRef {
        id: format!("normalized-{}", raw.id),
        raw_artifact_id: raw.id.clone(),
        source_id: raw.source_id.clone(),
        text_content: normalize_text(&input.text_content),
        title: input.title,
        document_type: input.document_type,
        language: input.language,
        sensitivity: classify_sensitivity_label(input.sensitivity.as_deref(), "internal"),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_ref() -> RawArtifactRef {
        RawArtifactRef {
            id: "raw-1".to_string(),
            source_id: "source-1".to_string(),
            content_hash: "abc123".to_string(),
            storage_path: "sha256/ab/c1/abc123".to_string(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: Some(12),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn sensitivity_known_label_is_preserved() {
        assert_eq!(
            classify_sensitivity_label(Some("sensitive"), "internal"),
            "sensitive"
        );
    }

    #[test]
    fn sensitivity_unknown_label_falls_back() {
        assert_eq!(
            classify_sensitivity_label(Some("private"), "internal"),
            "internal"
        );
        assert_eq!(classify_sensitivity_label(None, "public"), "public");
    }

    #[test]
    fn normalized_document_preserves_raw_lineage() {
        let document =
            build_normalized_document_ref(&raw_ref(), NormalizedDocumentInput::default());
        assert_eq!(document.id, "normalized-raw-1");
        assert_eq!(document.raw_artifact_id, "raw-1");
        assert_eq!(
            document
                .metadata
                .get("normalized_from_raw_artifact_id")
                .expect("lineage"),
            "raw-1"
        );
        assert_eq!(
            document.metadata.get("raw_content_hash").expect("hash"),
            "abc123"
        );
    }

    #[test]
    fn caller_metadata_is_merged() {
        let mut metadata = BTreeMap::new();
        metadata.insert("connector_name".to_string(), "manual_upload".to_string());
        let document = build_normalized_document_ref(
            &raw_ref(),
            NormalizedDocumentInput {
                metadata,
                ..NormalizedDocumentInput::default()
            },
        );
        assert_eq!(
            document.metadata.get("connector_name").expect("connector"),
            "manual_upload"
        );
    }

    #[test]
    fn valid_utf8_is_normalized_without_replacement() {
        let normalized = normalize_utf8_bytes(b"line1\r\nline2\rline3");
        assert_eq!(normalized.text, "line1\nline2\nline3");
        assert!(!normalized.used_replacement);
    }

    #[test]
    fn lossy_utf8_reports_replacement() {
        let normalized = normalize_utf8_bytes(&[0xff, b'a']);
        assert_eq!(normalized.text, "\u{fffd}a");
        assert!(normalized.used_replacement);
    }
}
