use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactWritePlan {
    pub content_hash: String,
    pub relative_path: PathBuf,
    pub target_path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredArtifact {
    pub content_hash: String,
    pub storage_path: String,
    pub size_bytes: u64,
    pub existed: bool,
}

/// On grok branch (full access mode): rich media / content kind detection.
/// Uses magic bytes (infer) + filename hint. Everything stays local.
#[derive(Debug, Clone)]
pub struct ContentKind {
    pub mime: String,
    pub kind: String, // "text", "image", "pdf", "audio", "video", "binary", "archive", etc.
    pub metadata: serde_json::Value,
}

pub fn detect_content_kind(data: &[u8], filename: Option<&str>) -> ContentKind {
    let mut mime = "application/octet-stream".to_string();
    let mut kind = "binary".to_string();
    let mut meta = serde_json::json!({});

    if let Some(kind_infer) = infer::get(data) {
        mime = kind_infer.mime_type().to_string();
        kind = match kind_infer.matcher_type() {
            infer::MatcherType::Image => "image".to_string(),
            infer::MatcherType::Video => "video".to_string(),
            infer::MatcherType::Audio => "audio".to_string(),
            infer::MatcherType::Archive => "archive".to_string(),
            infer::MatcherType::Doc => {
                if mime.contains("pdf") { "pdf".to_string() } else { "document".to_string() }
            }
            _ => "binary".to_string(),
        };
        meta["magic"] = serde_json::json!(kind_infer.to_string());
    }

    if let Some(name) = filename {
        meta["filename"] = serde_json::json!(name);
        let lower = name.to_lowercase();
        if lower.ends_with(".txt") || lower.ends_with(".md") || lower.ends_with(".log") {
            mime = "text/plain".to_string();
            kind = "text".to_string();
        } else if lower.ends_with(".json") {
            mime = "application/json".to_string();
            kind = "text".to_string();
        } else if lower.ends_with(".html") || lower.ends_with(".htm") {
            mime = "text/html".to_string();
            kind = "text".to_string();
        }
    }

    // Try UTF-8 detection for text
    if kind == "binary" && std::str::from_utf8(data).is_ok() {
        kind = "text".to_string();
        if mime == "application/octet-stream" {
            mime = "text/plain".to_string();
        }
    }

    meta["size"] = serde_json::json!(data.len());
    ContentKind { mime, kind, metadata: meta }
}

/// On grok branch: deep PDF (and text) extraction.
/// Returns the extracted text content when possible (real content for evidence/normalized docs),
/// instead of placeholders. This enables "deep pdf collection" and similar for other media types.
pub fn extract_text_if_possible(data: &[u8], kind: &ContentKind) -> Option<String> {
    if kind.kind == "pdf" || kind.mime.to_lowercase().contains("pdf") {
        // Use pdf-extract for real text extraction from PDF (no external binaries needed for basic)
        match pdf_extract::extract_text_from_mem(data) {
            Ok(text) if !text.trim().is_empty() => {
                return Some(text);
            }
            _ => {
                // Fallback to raw if extraction fails (still better than nothing for some PDFs)
                if let Ok(s) = std::str::from_utf8(data) {
                    if s.len() > 20 {
                        return Some(format!("[PDF raw fallback]\n{}", s.chars().take(4000).collect::<String>()));
                    }
                }
            }
        }
    }

    if kind.kind == "text" || kind.mime.starts_with("text/") || kind.mime == "application/json" || kind.mime == "text/html" {
        return std::str::from_utf8(data).ok().map(|s| s.to_string());
    }

    // For images/audio/video etc., we can return a rich metadata summary as "text" for evidence mining
    // This covers "some of that nature" – at least description/metadata is extracted and usable for claims/graph.
    if kind.kind == "image" || kind.kind == "video" || kind.kind == "audio" {
        let mut desc = format!("Binary {} ({} bytes)", kind.kind, data.len());
        if let Some(magic) = kind.metadata.get("magic") {
            desc.push_str(&format!(", magic: {}", magic));
        }
        if let Some(fname) = kind.metadata.get("filename") {
            desc.push_str(&format!(", file: {}", fname));
        }
        return Some(desc);
    }

    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactStoreError {
    InvalidDataRoot(String),
    InvalidHash(String),
    PathEscapesRoot(String),
    Io(String),
    HashMismatch,
}

impl fmt::Display for ArtifactStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDataRoot(reason) => write!(formatter, "invalid data root: {reason}"),
            Self::InvalidHash(hash) => write!(formatter, "invalid artifact hash: {hash}"),
            Self::PathEscapesRoot(path) => write!(formatter, "artifact path escapes root: {path}"),
            Self::Io(reason) => write!(formatter, "artifact store I/O error: {reason}"),
            Self::HashMismatch => write!(formatter, "artifact content hash mismatch"),
        }
    }
}

impl std::error::Error for ArtifactStoreError {}

pub struct ArtifactStore {
    data_root: PathBuf,
    artifact_root: PathBuf,
}

impl ArtifactStore {
    pub fn new(data_root: impl AsRef<Path>) -> Result<Self, ArtifactStoreError> {
        let data_root = normalize_data_root(data_root.as_ref())?;
        let artifact_root = data_root.join("artifacts");
        Ok(Self {
            data_root,
            artifact_root,
        })
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    pub fn artifact_root(&self) -> &Path {
        &self.artifact_root
    }

    pub fn plan_write(&self, content: &[u8]) -> Result<ArtifactWritePlan, ArtifactStoreError> {
        let content_hash = sha256_hex(content);
        let relative_path = relative_hash_path(&content_hash)?;
        let target_path = self.bounded_target(&relative_path)?;
        Ok(ArtifactWritePlan {
            content_hash,
            relative_path,
            target_path,
            size_bytes: content.len() as u64,
        })
    }

    pub fn write_bytes(&self, content: &[u8]) -> Result<StoredArtifact, ArtifactStoreError> {
        let plan = self.plan_write(content)?;
        fs::create_dir_all(
            plan.target_path.parent().ok_or_else(|| {
                ArtifactStoreError::Io("artifact target has no parent".to_string())
            })?,
        )
        .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        self.ensure_parent_bounded(&plan.target_path)?;

        if plan.target_path.exists() {
            self.verify_existing(&plan.target_path, &plan.content_hash)?;
            return Ok(StoredArtifact {
                content_hash: plan.content_hash,
                storage_path: path_to_storage_string(&plan.relative_path),
                size_bytes: plan.size_bytes,
                existed: true,
            });
        }

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&plan.target_path)
        {
            Ok(mut file) => {
                file.write_all(content)
                    .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                self.verify_existing(&plan.target_path, &plan.content_hash)?;
                return Ok(StoredArtifact {
                    content_hash: plan.content_hash,
                    storage_path: path_to_storage_string(&plan.relative_path),
                    size_bytes: plan.size_bytes,
                    existed: true,
                });
            }
            Err(error) => return Err(ArtifactStoreError::Io(error.to_string())),
        }

        Ok(StoredArtifact {
            content_hash: plan.content_hash,
            storage_path: path_to_storage_string(&plan.relative_path),
            size_bytes: plan.size_bytes,
            existed: false,
        })
    }

    pub fn read_by_hash(&self, content_hash: &str) -> Result<Vec<u8>, ArtifactStoreError> {
        let relative_path = relative_hash_path(content_hash)?;
        let target_path = self.bounded_target(&relative_path)?;
        self.ensure_parent_bounded(&target_path)?;
        let mut bytes = Vec::new();
        let mut file = fs::File::open(&target_path)
            .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        file.read_to_end(&mut bytes)
            .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        if sha256_hex(&bytes) != content_hash {
            return Err(ArtifactStoreError::HashMismatch);
        }
        Ok(bytes)
    }

    fn bounded_target(&self, relative_path: &Path) -> Result<PathBuf, ArtifactStoreError> {
        if relative_path.is_absolute() || relative_path.components().any(is_parent_dir_component) {
            return Err(ArtifactStoreError::PathEscapesRoot(
                relative_path.display().to_string(),
            ));
        }
        Ok(self.artifact_root.join(relative_path))
    }

    fn ensure_parent_bounded(&self, target_path: &Path) -> Result<(), ArtifactStoreError> {
        fs::create_dir_all(&self.artifact_root)
            .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        let root = self
            .artifact_root
            .canonicalize()
            .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        let parent = target_path
            .parent()
            .ok_or_else(|| ArtifactStoreError::Io("artifact target has no parent".to_string()))?
            .canonicalize()
            .map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        if parent.starts_with(&root) {
            Ok(())
        } else {
            Err(ArtifactStoreError::PathEscapesRoot(
                target_path.display().to_string(),
            ))
        }
    }

    fn verify_existing(
        &self,
        target_path: &Path,
        expected_hash: &str,
    ) -> Result<(), ArtifactStoreError> {
        let bytes =
            fs::read(target_path).map_err(|error| ArtifactStoreError::Io(error.to_string()))?;
        if sha256_hex(&bytes) == expected_hash {
            Ok(())
        } else {
            Err(ArtifactStoreError::HashMismatch)
        }
    }
}

pub fn hash_bytes(content: &[u8]) -> String {
    sha256_hex(content)
}

pub fn relative_hash_path(content_hash: &str) -> Result<PathBuf, ArtifactStoreError> {
    validate_sha256_hash(content_hash)?;
    Ok(Path::new("sha256")
        .join(&content_hash[0..2])
        .join(&content_hash[2..4])
        .join(content_hash))
}

fn normalize_data_root(path: &Path) -> Result<PathBuf, ArtifactStoreError> {
    if path.as_os_str().is_empty() {
        return Err(ArtifactStoreError::InvalidDataRoot(
            "data root must not be empty".to_string(),
        ));
    }
    if path.components().any(is_parent_dir_component) {
        return Err(ArtifactStoreError::InvalidDataRoot(
            "data root must not contain path traversal".to_string(),
        ));
    }
    Ok(path.to_path_buf())
}

fn validate_sha256_hash(content_hash: &str) -> Result<(), ArtifactStoreError> {
    if content_hash.len() == 64 && content_hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ArtifactStoreError::InvalidHash(content_hash.to_string()))
    }
}

fn is_parent_dir_component(component: std::path::Component<'_>) -> bool {
    matches!(component, std::path::Component::ParentDir)
}

fn path_to_storage_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn sha256_hex(content: &[u8]) -> String {
    let digest = sha256(content);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256(content: &[u8]) -> [u8; 32] {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut message = content.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while (message.len() % 64) != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = H0;
    for chunk in message.chunks(64) {
        let mut w = [0u32; 64];
        for (index, word) in w.iter_mut().enumerate().take(16) {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut digest = [0u8; 32];
    for (index, word) in h.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("igy6-artifacts-{label}-{unique}"))
    }

    #[test]
    fn hashes_are_stable() {
        assert_eq!(
            hash_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(hash_bytes(b"same"), hash_bytes(b"same"));
    }

    #[test]
    fn write_plan_is_content_addressed() {
        let root = temp_root("plan");
        let store = ArtifactStore::new(&root).expect("store");
        let plan = store.plan_write(b"hello").expect("plan");
        assert!(plan.relative_path.starts_with("sha256"));
        assert!(plan.target_path.starts_with(root.join("artifacts")));
        assert_eq!(plan.size_bytes, 5);
    }

    #[test]
    fn duplicate_writes_are_avoided() {
        let root = temp_root("duplicate");
        let store = ArtifactStore::new(&root).expect("store");
        let first = store.write_bytes(b"hello").expect("first write");
        let second = store.write_bytes(b"hello").expect("second write");
        fs::remove_dir_all(&root).expect("cleanup");
        assert!(!first.existed);
        assert!(second.existed);
        assert_eq!(first.content_hash, second.content_hash);
        assert_eq!(first.storage_path, second.storage_path);
    }

    #[test]
    fn writes_stay_inside_root() {
        let root = temp_root("bounded");
        let store = ArtifactStore::new(&root).expect("store");
        let stored = store.write_bytes(b"bounded").expect("write");
        let target = root.join("artifacts").join(&stored.storage_path);
        assert!(target.is_file());
        assert!(target.starts_with(root.join("artifacts")));
        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn read_by_hash_returns_expected_bytes() {
        let root = temp_root("read");
        let store = ArtifactStore::new(&root).expect("store");
        let stored = store.write_bytes(b"read me").expect("write");
        let bytes = store.read_by_hash(&stored.content_hash).expect("read");
        fs::remove_dir_all(&root).expect("cleanup");
        assert_eq!(bytes, b"read me");
    }

    #[test]
    fn invalid_hash_is_rejected() {
        let root = temp_root("invalid-hash");
        let store = ArtifactStore::new(&root).expect("store");
        assert!(matches!(
            store.read_by_hash("../escape"),
            Err(ArtifactStoreError::InvalidHash(_))
        ));
    }

    #[test]
    fn path_traversal_data_root_is_rejected() {
        assert!(matches!(
            ArtifactStore::new(Path::new("..").join("outside")),
            Err(ArtifactStoreError::InvalidDataRoot(_))
        ));
    }
}
