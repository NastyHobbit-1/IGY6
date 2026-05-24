use std::fmt;

pub const CHUNK_VECTOR_SEARCH_MAX_LIMIT: usize = 50;
pub const EMBEDDING_METHOD: &str = "rust_local_hash_v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorMemoryError {
    InvalidVectorSize,
    InvalidCollectionName,
    InvalidBaseUrl,
}

impl fmt::Display for VectorMemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVectorSize => write!(formatter, "vector_size must be at least 1"),
            Self::InvalidCollectionName => write!(
                formatter,
                "collection name must contain only ASCII letters, digits, underscores, or hyphens"
            ),
            Self::InvalidBaseUrl => write!(
                formatter,
                "Qdrant base URL must be http://host[:port] without path traversal"
            ),
        }
    }
}

impl std::error::Error for VectorMemoryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QdrantSettings {
    pub base_url: String,
    pub collection_name: String,
    pub vector_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequestPlan {
    pub method: HttpMethod,
    pub origin: String,
    pub path: String,
    pub body: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVectorPoint {
    pub id: String,
    pub vector: Vec<f64>,
    pub chunk_id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub embedding_method: String,
}

pub fn embed_text_local(text: &str, vector_size: usize) -> Result<Vec<f64>, VectorMemoryError> {
    if vector_size < 1 {
        return Err(VectorMemoryError::InvalidVectorSize);
    }

    let mut vector = vec![0.0; vector_size];
    let tokens = text.split_whitespace().map(str::to_lowercase);
    for token in tokens {
        let hash = stable_token_hash(token.as_bytes());
        let index = (hash as usize) % vector_size;
        let sign = if ((hash >> 63) & 1) == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }

    let magnitude = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
    if magnitude == 0.0 {
        return Ok(vector);
    }

    Ok(vector.into_iter().map(|value| value / magnitude).collect())
}

pub fn plan_chunk_vector_point(
    id: &str,
    document_id: &str,
    chunk_index: usize,
    text: &str,
    vector_size: usize,
) -> Result<ChunkVectorPoint, VectorMemoryError> {
    Ok(ChunkVectorPoint {
        id: id.to_string(),
        vector: embed_text_local(text, vector_size)?,
        chunk_id: id.to_string(),
        document_id: document_id.to_string(),
        chunk_index,
        embedding_method: EMBEDDING_METHOD.to_string(),
    })
}

pub fn qdrant_collection_payload(vector_size: usize) -> Result<String, VectorMemoryError> {
    if vector_size < 1 {
        return Err(VectorMemoryError::InvalidVectorSize);
    }
    Ok(format!(
        "{{\"vectors\":{{\"size\":{vector_size},\"distance\":\"Cosine\"}}}}"
    ))
}

pub fn qdrant_points_payload(points: &[ChunkVectorPoint]) -> String {
    let encoded_points = points
        .iter()
        .map(point_payload)
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"points\":[{encoded_points}]}}")
}

pub fn qdrant_search_payload(vector: &[f64], limit: usize) -> Result<String, VectorMemoryError> {
    if vector.is_empty() {
        return Err(VectorMemoryError::InvalidVectorSize);
    }

    let bounded_limit = limit.clamp(1, CHUNK_VECTOR_SEARCH_MAX_LIMIT);
    Ok(format!(
        "{{\"vector\":[{}],\"limit\":{bounded_limit},\"with_payload\":true,\"with_vector\":false}}",
        encode_vector(vector)
    ))
}

pub fn collection_status_request(
    settings: &QdrantSettings,
) -> Result<HttpRequestPlan, VectorMemoryError> {
    request_plan(settings, HttpMethod::Get, "collections", None, 5)
}

pub fn ensure_collection_request(
    settings: &QdrantSettings,
) -> Result<HttpRequestPlan, VectorMemoryError> {
    request_plan(
        settings,
        HttpMethod::Put,
        "collections",
        Some(qdrant_collection_payload(settings.vector_size)?),
        10,
    )
}

pub fn upsert_points_request(
    settings: &QdrantSettings,
    points: &[ChunkVectorPoint],
) -> Result<HttpRequestPlan, VectorMemoryError> {
    request_plan(
        settings,
        HttpMethod::Put,
        "points",
        Some(qdrant_points_payload(points)),
        15,
    )
}

pub fn search_points_request(
    settings: &QdrantSettings,
    query: &str,
    limit: usize,
) -> Result<HttpRequestPlan, VectorMemoryError> {
    let vector = embed_text_local(query, settings.vector_size)?;
    request_plan(
        settings,
        HttpMethod::Post,
        "points/search",
        Some(qdrant_search_payload(&vector, limit)?),
        10,
    )
}

fn stable_token_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn request_plan(
    settings: &QdrantSettings,
    method: HttpMethod,
    route: &str,
    body: Option<String>,
    timeout_seconds: u64,
) -> Result<HttpRequestPlan, VectorMemoryError> {
    validate_collection_name(&settings.collection_name)?;
    if settings.vector_size < 1 {
        return Err(VectorMemoryError::InvalidVectorSize);
    }
    let origin = normalize_http_origin(&settings.base_url)?;
    let path = match route {
        "collections" => format!("/collections/{}", settings.collection_name),
        "points" => format!("/collections/{}/points", settings.collection_name),
        "points/search" => format!("/collections/{}/points/search", settings.collection_name),
        _ => return Err(VectorMemoryError::InvalidBaseUrl),
    };

    Ok(HttpRequestPlan {
        method,
        origin,
        path,
        body,
        timeout_seconds,
    })
}

fn validate_collection_name(collection_name: &str) -> Result<(), VectorMemoryError> {
    if !collection_name.is_empty()
        && collection_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        Ok(())
    } else {
        Err(VectorMemoryError::InvalidCollectionName)
    }
}

fn normalize_http_origin(base_url: &str) -> Result<String, VectorMemoryError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let Some(rest) = trimmed.strip_prefix("http://") else {
        return Err(VectorMemoryError::InvalidBaseUrl);
    };
    if rest.is_empty() || rest.contains('/') || rest.contains('\\') || rest.contains("..") {
        return Err(VectorMemoryError::InvalidBaseUrl);
    }
    Ok(format!("http://{rest}"))
}

fn point_payload(point: &ChunkVectorPoint) -> String {
    format!(
        "{{\"id\":\"{}\",\"vector\":[{}],\"payload\":{{\"chunk_id\":\"{}\",\"document_id\":\"{}\",\"chunk_index\":{},\"embedding_method\":\"{}\"}}}}",
        json_escape(&qdrant_point_id(&point.id)),
        encode_vector(&point.vector),
        json_escape(&point.chunk_id),
        json_escape(&point.document_id),
        point.chunk_index,
        json_escape(&point.embedding_method),
    )
}

fn qdrant_point_id(source_id: &str) -> String {
    let seed = source_id.as_bytes();
    let mut bytes = [0_u8; 16];
    let salts = [0xcbf29ce484222325_u64, 0x9e3779b97f4a7c15_u64];
    for (index, salt) in salts.iter().enumerate() {
        let hash = stable_token_hash_with_seed(seed, *salt);
        bytes[index * 8..(index + 1) * 8].copy_from_slice(&hash.to_be_bytes());
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

fn stable_token_hash_with_seed(bytes: &[u8], seed: u64) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn encode_vector(vector: &[f64]) -> String {
    vector
        .iter()
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{value:.1}")
            } else {
                value.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> QdrantSettings {
        QdrantSettings {
            base_url: "http://localhost:6333/".to_string(),
            collection_name: "igy6_chunks".to_string(),
            vector_size: 16,
        }
    }

    #[test]
    fn embedding_is_deterministic_and_normalized() {
        let first = embed_text_local("Hello world hello", 16).expect("vector");
        let second = embed_text_local("hello WORLD hello", 16).expect("vector");
        assert_eq!(first, second);
        let magnitude = first.iter().map(|value| value * value).sum::<f64>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.000001);
    }

    #[test]
    fn empty_text_returns_zero_vector() {
        let vector = embed_text_local(" \n\t", 4).expect("vector");
        assert_eq!(vector, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn invalid_vector_size_is_rejected() {
        assert_eq!(
            embed_text_local("hello", 0),
            Err(VectorMemoryError::InvalidVectorSize)
        );
        assert_eq!(
            qdrant_collection_payload(0),
            Err(VectorMemoryError::InvalidVectorSize)
        );
    }

    #[test]
    fn collection_payload_uses_cosine_distance() {
        let payload = qdrant_collection_payload(384).expect("payload");
        assert_eq!(
            payload,
            "{\"vectors\":{\"size\":384,\"distance\":\"Cosine\"}}"
        );
    }

    #[test]
    fn points_payload_contains_chunk_metadata() {
        let point =
            plan_chunk_vector_point("chunk-1", "document-1", 7, "alpha beta", 8).expect("point");
        let payload = qdrant_points_payload(&[point]);
        assert!(payload.contains(&format!("\"id\":\"{}\"", qdrant_point_id("chunk-1"))));
        assert!(payload.contains("\"chunk_id\":\"chunk-1\""));
        assert!(payload.contains("\"document_id\":\"document-1\""));
        assert!(payload.contains("\"chunk_index\":7"));
        assert!(payload.contains("\"embedding_method\":\"rust_local_hash_v1\""));
    }

    #[test]
    fn qdrant_point_ids_are_uuid_shaped_and_deterministic() {
        let first = qdrant_point_id("chunk-18b25ee83a4d8467-0");
        let second = qdrant_point_id("chunk-18b25ee83a4d8467-0");
        assert_eq!(first, second);
        assert_eq!(first.len(), 36);
        assert_eq!(
            first.chars().filter(|character| *character == '-').count(),
            4
        );
        assert_eq!(first.as_bytes()[14], b'5');
    }

    #[test]
    fn search_payload_clamps_limit_and_excludes_vectors() {
        let payload = qdrant_search_payload(&[0.25, -0.25], 99).expect("payload");
        assert!(payload.contains("\"limit\":50"));
        assert!(payload.contains("\"with_payload\":true"));
        assert!(payload.contains("\"with_vector\":false"));

        let payload = qdrant_search_payload(&[0.25], 0).expect("payload");
        assert!(payload.contains("\"limit\":1"));
    }

    #[test]
    fn request_plans_are_bounded_to_collection_paths() {
        let plan = search_points_request(&settings(), "alpha beta", 5).expect("plan");
        assert_eq!(plan.method, HttpMethod::Post);
        assert_eq!(plan.origin, "http://localhost:6333");
        assert_eq!(plan.path, "/collections/igy6_chunks/points/search");
        assert_eq!(plan.timeout_seconds, 10);
        assert!(plan.body.expect("body").contains("\"limit\":5"));
    }

    #[test]
    fn unsafe_collection_names_are_rejected() {
        let mut settings = settings();
        settings.collection_name = "../secret".to_string();
        assert_eq!(
            collection_status_request(&settings),
            Err(VectorMemoryError::InvalidCollectionName)
        );
    }

    #[test]
    fn invalid_base_urls_are_rejected() {
        let mut settings = settings();
        settings.base_url = "https://localhost:6333".to_string();
        assert_eq!(
            ensure_collection_request(&settings),
            Err(VectorMemoryError::InvalidBaseUrl)
        );

        settings.base_url = "http://localhost:6333/collections/other".to_string();
        assert_eq!(
            ensure_collection_request(&settings),
            Err(VectorMemoryError::InvalidBaseUrl)
        );
    }
}
