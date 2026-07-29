use std::collections::BTreeMap;

use igy6_media_extract::{extract_or_utf8, MediaExtractResult};

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

/// Normalize artifact bytes. Media types use local tools (pdftotext/tesseract/ffmpeg/whisper).
/// Extracted text stays inside IGY6; this function does not transmit data externally.
pub fn normalize_artifact_bytes(
    bytes: &[u8],
    mime_type: Option<&str>,
    filename: Option<&str>,
) -> (TextNormalization, MediaExtractResult) {
    let extracted = extract_or_utf8(bytes, mime_type, filename);
    let normalization = TextNormalization {
        text: normalize_text(&extracted.text),
        used_replacement: extracted.method == "utf8_lossy",
    };
    (normalization, extracted)
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

/// Build a normalized document from raw bytes, running media extraction when needed.
pub fn normalize_raw_artifact(
    raw: &RawArtifactRef,
    bytes: &[u8],
    filename: Option<&str>,
    title: Option<String>,
    sensitivity: Option<String>,
) -> NormalizedDocumentRef {
    let (text_norm, extract) =
        normalize_artifact_bytes(bytes, raw.mime_type.as_deref(), filename);
    let document_type = match extract.method.as_str() {
        "pdf_text_layer" => "pdf_extracted",
        "image_ocr" => "image_ocr",
        m if m.ends_with("_transcription") => "av_transcript",
        "utf8_passthrough" | "utf8_lossy" => "text",
        _ => "media",
    }
    .to_string();
    let mut metadata = BTreeMap::new();
    metadata.insert("extract_method".to_string(), extract.method.clone());
    metadata.insert("extract_tool".to_string(), extract.tool.clone());
    metadata.insert("extract_success".to_string(), extract.success.to_string());
    metadata.insert("extract_detail".to_string(), extract.detail.clone());
    if let Some(mime) = &raw.mime_type {
        metadata.insert("source_mime_type".to_string(), mime.clone());
    }
    build_normalized_document_ref(
        raw,
        NormalizedDocumentInput {
            text_content: text_norm.text,
            title,
            document_type,
            language: Some("en".to_string()),
            sensitivity,
            metadata,
        },
    )
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

    #[test]
    fn normalize_artifact_bytes_handles_plain_text() {
        let (norm, extract) =
            normalize_artifact_bytes(b"hello media", Some("text/plain"), Some("note.txt"));
        assert_eq!(norm.text, "hello media");
        assert!(extract.success);
    }

    #[test]
    fn normalize_raw_artifact_records_extract_metadata() {
        let doc = normalize_raw_artifact(
            &raw_ref(),
            b"sample text",
            Some("note.txt"),
            Some("Title".to_string()),
            Some("internal".to_string()),
        );
        assert_eq!(doc.title.as_deref(), Some("Title"));
        assert_eq!(doc.text_content, "sample text");
        assert!(doc.metadata.contains_key("extract_method"));
        assert!(doc.metadata.contains_key("extract_tool"));
    }
}
