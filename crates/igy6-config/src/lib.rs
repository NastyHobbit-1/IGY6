use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use igy6_core::validate_non_empty;

pub const REQUIRED_KEYS: &[&str] = &[
    "APP_ENV",
    "APP_HOST",
    "APP_PORT",
    "API_BASE_URL",
    "WEB_BASE_URL",
    "POSTGRES_HOST",
    "POSTGRES_PORT",
    "POSTGRES_DB",
    "POSTGRES_USER",
    "POSTGRES_PASSWORD",
    "DATABASE_URL",
    "REDIS_HOST",
    "REDIS_PORT",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
    "QDRANT_HOST",
    "QDRANT_PORT",
    "QDRANT_URL",
    "QDRANT_CHUNK_COLLECTION",
    "QDRANT_CHUNK_VECTOR_SIZE",
    "NEO4J_HOST",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "NEO4J_USER",
    "NEO4J_PASSWORD",
    "NEO4J_URI",
    "MLFLOW_TRACKING_URI",
    "MLFLOW_ARTIFACT_ROOT",
    "PHOENIX_HOST",
    "PHOENIX_PORT",
    "PHOENIX_COLLECTOR_ENDPOINT",
    "ARTIFACT_STORE_PATH",
    "EXPORT_STORE_PATH",
    "ENV_FILE_PATH",
    "ENV_BACKUP_DIR",
    "IGY6_DATA_ROOT",
    "EXTERNAL_MODEL_POLICY_DEFAULT",
    "SINGLE_USER_MODE",
    "AUDIT_LOG_LEVEL",
    "APPROVAL_REQUIRED_DEFAULT",
];

pub const ENV_REQUIRED_KEYS: &[&str] = &[
    "APP_ENV",
    "APP_HOST",
    "APP_PORT",
    "API_BASE_URL",
    "WEB_BASE_URL",
    "POSTGRES_HOST",
    "POSTGRES_PORT",
    "POSTGRES_DB",
    "POSTGRES_USER",
    "POSTGRES_PASSWORD",
    "DATABASE_URL",
    "REDIS_HOST",
    "REDIS_PORT",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
    "QDRANT_HOST",
    "QDRANT_PORT",
    "QDRANT_URL",
    "QDRANT_CHUNK_COLLECTION",
    "QDRANT_CHUNK_VECTOR_SIZE",
    "NEO4J_HOST",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "NEO4J_USER",
    "NEO4J_PASSWORD",
    "NEO4J_URI",
    "MLFLOW_TRACKING_URI",
    "MLFLOW_ARTIFACT_ROOT",
    "PHOENIX_HOST",
    "PHOENIX_PORT",
    "PHOENIX_COLLECTOR_ENDPOINT",
    "ARTIFACT_STORE_PATH",
    "EXPORT_STORE_PATH",
    "IGY6_DATA_ROOT",
    "EXTERNAL_MODEL_POLICY_DEFAULT",
    "SINGLE_USER_MODE",
    "AUDIT_LOG_LEVEL",
    "APPROVAL_REQUIRED_DEFAULT",
];

const BOOLEAN_KEYS: &[&str] = &["SINGLE_USER_MODE", "APPROVAL_REQUIRED_DEFAULT"];
const PORT_KEYS: &[&str] = &[
    "APP_PORT",
    "POSTGRES_PORT",
    "REDIS_PORT",
    "QDRANT_PORT",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "PHOENIX_PORT",
];
const URL_KEYS: &[&str] = &[
    "API_BASE_URL",
    "WEB_BASE_URL",
    "DATABASE_URL",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
    "QDRANT_URL",
    "NEO4J_URI",
    "MLFLOW_TRACKING_URI",
    "PHOENIX_COLLECTOR_ENDPOINT",
];
const STORAGE_PATH_KEYS: &[&str] = &[
    "ARTIFACT_STORE_PATH",
    "EXPORT_STORE_PATH",
    "ENV_FILE_PATH",
    "ENV_BACKUP_DIR",
];
const HOST_PATH_KEYS: &[&str] = &["IGY6_DATA_ROOT"];
const EXTERNAL_MODEL_POLICIES: &[&str] = &["blocked", "metadata_only", "allowed_with_approval"];
const AUDIT_LOG_LEVELS: &[&str] = &["debug", "info", "warning", "error"];
const DEFAULT_IGY6_DATA_ROOT: &str = "../IGY6_Data";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvPair {
    pub key: String,
    pub value: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEnv {
    pub values: BTreeMap<String, String>,
    pub order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFinding {
    pub severity: FindingSeverity,
    pub source: String,
    pub key: Option<String>,
    pub line_number: Option<usize>,
    pub message: String,
}

impl ConfigFinding {
    pub fn error(
        source: impl Into<String>,
        key: Option<String>,
        line_number: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: FindingSeverity::Error,
            source: source.into(),
            key,
            line_number,
            message: message.into(),
        }
    }

    pub fn warning(
        source: impl Into<String>,
        key: Option<String>,
        line_number: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: FindingSeverity::Warning,
            source: source.into(),
            key,
            line_number,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFileReport {
    pub label: String,
    pub path: PathBuf,
    pub exists: bool,
    pub key_count: usize,
    pub errors: Vec<ConfigFinding>,
    pub warnings: Vec<ConfigFinding>,
}

impl ConfigFileReport {
    pub fn passed(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValidationReport {
    pub env_example: ConfigFileReport,
    pub env: ConfigFileReport,
}

impl ConfigValidationReport {
    pub fn passed(&self) -> bool {
        self.env_example.passed() && self.env.passed()
    }

    pub fn error_messages(&self) -> Vec<String> {
        self.env_example
            .errors
            .iter()
            .chain(self.env.errors.iter())
            .map(render_finding)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyKey,
    MissingSeparator,
    EmptyRequiredValue { key: String },
    InvalidBoolean { key: String, value: String },
    InvalidPort { key: String, value: String },
    ReadFailed { path: PathBuf, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => write!(formatter, "environment key must not be empty"),
            Self::MissingSeparator => write!(formatter, "environment line must contain '='"),
            Self::EmptyRequiredValue { key } => write!(formatter, "{key} must not be empty"),
            Self::InvalidBoolean { key, value } => {
                write!(formatter, "{key} has invalid boolean value {value:?}")
            }
            Self::InvalidPort { key, value } => {
                write!(formatter, "{key} has invalid port {value:?}")
            }
            Self::ReadFailed { path, reason } => {
                write!(formatter, "could not read {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn parse_env_line(line: &str) -> Result<Option<EnvPair>, ConfigError> {
    parse_env_line_with_number(line, 0)
}

pub fn parse_env_line_with_number(
    line: &str,
    line_number: usize,
) -> Result<Option<EnvPair>, ConfigError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }
    let Some((key, value)) = trimmed.split_once('=') else {
        return Err(ConfigError::MissingSeparator);
    };
    let key = key.trim();
    if key.is_empty() {
        return Err(ConfigError::EmptyKey);
    }
    Ok(Some(EnvPair {
        key: key.to_string(),
        value: strip_matching_quotes(value.trim()).to_string(),
        line_number,
    }))
}

pub fn parse_env_content(content: &str, source: &str) -> (ParsedEnv, Vec<ConfigFinding>) {
    let mut values = BTreeMap::new();
    let mut order = Vec::new();
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        match parse_env_line_with_number(line, line_number) {
            Ok(Some(pair)) => {
                if !seen.insert(pair.key.clone()) {
                    findings.push(ConfigFinding::error(
                        source,
                        Some(pair.key.clone()),
                        Some(line_number),
                        "Duplicate key.",
                    ));
                }
                if !values.contains_key(&pair.key) {
                    order.push(pair.key.clone());
                }
                values.insert(pair.key, pair.value);
            }
            Ok(None) => {}
            Err(error) => findings.push(ConfigFinding::error(
                source,
                None,
                Some(line_number),
                error.to_string(),
            )),
        }
    }

    (ParsedEnv { values, order }, findings)
}

pub fn validate_repo_config(repo_root: &Path) -> Result<ConfigValidationReport, ConfigError> {
    let env_example = validate_env_file(&repo_root.join(".env.example"), ".env.example", true)?;
    let env = validate_env_file(&repo_root.join(".env"), ".env", false)?;
    Ok(ConfigValidationReport { env_example, env })
}

pub fn validate_env_file(
    path: &Path,
    label: &str,
    required: bool,
) -> Result<ConfigFileReport, ConfigError> {
    if !path.exists() {
        let warnings = if required {
            Vec::new()
        } else {
            vec![ConfigFinding::warning(
                label,
                None,
                None,
                ".env is absent; this is allowed for repo-visible validation.",
            )]
        };
        let errors = if required {
            vec![ConfigFinding::error(
                label,
                None,
                None,
                "Required config file is missing.",
            )]
        } else {
            Vec::new()
        };
        return Ok(ConfigFileReport {
            label: label.to_string(),
            path: path.to_path_buf(),
            exists: false,
            key_count: 0,
            errors,
            warnings,
        });
    }

    let content = fs::read_to_string(path).map_err(|error| ConfigError::ReadFailed {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    Ok(validate_env_content(&content, label, path, required))
}

pub fn validate_env_content(
    content: &str,
    label: &str,
    path: &Path,
    required: bool,
) -> ConfigFileReport {
    let (parsed, mut errors) = parse_env_content(content, label);
    let mut warnings = Vec::new();
    let required_keys = if label == ".env" {
        ENV_REQUIRED_KEYS
    } else {
        REQUIRED_KEYS
    };

    for key in required_keys {
        if !parsed.values.contains_key(*key) {
            errors.push(ConfigFinding::error(
                label,
                Some((*key).to_string()),
                None,
                "Required setting is missing.",
            ));
        }
    }

    validate_values(label, &parsed, &mut errors, &mut warnings);

    if !required && parsed.values.is_empty() {
        warnings.push(ConfigFinding::warning(
            label,
            None,
            None,
            ".env exists but contains no managed keys.",
        ));
    }

    ConfigFileReport {
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: true,
        key_count: parsed.values.len(),
        errors,
        warnings,
    }
}

pub fn render_cli_report(report: &ConfigValidationReport) -> String {
    let mut output = String::from("IGY6 config check\n");
    output.push_str(&render_file_summary(&report.env_example));
    output.push_str(&render_file_summary(&report.env));
    output.push_str("values: redacted\n");
    output.push_str("runtime_data_read: false\n");

    for finding in report
        .env_example
        .warnings
        .iter()
        .chain(report.env.warnings.iter())
    {
        output.push_str("warning: ");
        output.push_str(&render_finding(finding));
        output.push('\n');
    }
    for finding in report
        .env_example
        .errors
        .iter()
        .chain(report.env.errors.iter())
    {
        output.push_str("error: ");
        output.push_str(&render_finding(finding));
        output.push('\n');
    }

    output.push_str(if report.passed() {
        "status: ok\n"
    } else {
        "status: failed\n"
    });
    output
}

pub fn require_value(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_non_empty("value", value).map_err(|_| ConfigError::EmptyRequiredValue {
        key: key.to_string(),
    })
}

pub fn parse_bool(key: &str, value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::InvalidBoolean {
            key: key.to_string(),
            value: value.to_string(),
        }),
    }
}

pub fn parse_port(key: &str, value: &str) -> Result<u16, ConfigError> {
    let parsed = value
        .trim()
        .parse::<u16>()
        .map_err(|_| ConfigError::InvalidPort {
            key: key.to_string(),
            value: value.to_string(),
        })?;
    if parsed == 0 {
        Err(ConfigError::InvalidPort {
            key: key.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(parsed)
    }
}

fn validate_values(
    source: &str,
    parsed: &ParsedEnv,
    errors: &mut Vec<ConfigFinding>,
    warnings: &mut Vec<ConfigFinding>,
) {
    for (key, value) in &parsed.values {
        if (REQUIRED_KEYS.contains(&key.as_str()) || ENV_REQUIRED_KEYS.contains(&key.as_str()))
            && value.trim().is_empty()
        {
            errors.push(ConfigFinding::error(
                source,
                Some(key.clone()),
                None,
                "Required setting must not be empty.",
            ));
        }
    }

    for key in BOOLEAN_KEYS {
        if let Some(value) = parsed.values.get(*key) {
            if parse_bool(key, value).is_err() {
                errors.push(ConfigFinding::error(
                    source,
                    Some((*key).to_string()),
                    None,
                    "Boolean must be true or false.",
                ));
            }
        }
    }

    for key in PORT_KEYS {
        if let Some(value) = parsed.values.get(*key) {
            if parse_port(key, value).is_err() {
                errors.push(ConfigFinding::error(
                    source,
                    Some((*key).to_string()),
                    None,
                    "Port must be between 1 and 65535.",
                ));
            }
        }
    }

    if let Some(value) = parsed.values.get("QDRANT_CHUNK_VECTOR_SIZE") {
        if value
            .trim()
            .parse::<u32>()
            .map_or(true, |number| number == 0)
        {
            errors.push(ConfigFinding::error(
                source,
                Some("QDRANT_CHUNK_VECTOR_SIZE".to_string()),
                None,
                "Vector size must be a positive integer.",
            ));
        }
    }

    for key in URL_KEYS {
        if let Some(value) = parsed.values.get(*key) {
            if !is_plausible_url(key, value) {
                errors.push(ConfigFinding::error(
                    source,
                    Some((*key).to_string()),
                    None,
                    "URL or URI is not syntactically plausible.",
                ));
            }
        }
    }

    if let Some(value) = parsed.values.get("EXTERNAL_MODEL_POLICY_DEFAULT") {
        if !EXTERNAL_MODEL_POLICIES.contains(&value.trim()) {
            errors.push(ConfigFinding::error(
                source,
                Some("EXTERNAL_MODEL_POLICY_DEFAULT".to_string()),
                None,
                "External model policy is not recognized.",
            ));
        }
    }

    if let Some(value) = parsed.values.get("AUDIT_LOG_LEVEL") {
        if !AUDIT_LOG_LEVELS.contains(&value.trim()) {
            errors.push(ConfigFinding::error(
                source,
                Some("AUDIT_LOG_LEVEL".to_string()),
                None,
                "Audit log level is not recognized.",
            ));
        }
    }

    for key in STORAGE_PATH_KEYS {
        if let Some(value) = parsed.values.get(*key) {
            if !storage_path_string_is_safe(value) {
                errors.push(ConfigFinding::error(
                    source,
                    Some((*key).to_string()),
                    None,
                    "Storage path must be absolute and must not contain traversal.",
                ));
            }
        }
    }

    for key in HOST_PATH_KEYS {
        if let Some(value) = parsed.values.get(*key) {
            if let Some(issue) = host_data_root_issue(value) {
                errors.push(ConfigFinding::error(
                    source,
                    Some((*key).to_string()),
                    None,
                    issue,
                ));
            }
        }
    }

    for key in parsed.values.keys() {
        if !REQUIRED_KEYS.contains(&key.as_str()) {
            warnings.push(ConfigFinding::warning(
                source,
                Some(key.clone()),
                None,
                "Unmanaged key is present; value was not printed.",
            ));
        }
    }
}

fn is_plausible_url(key: &str, value: &str) -> bool {
    let value = value.trim();
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };
    if scheme.is_empty() || rest.is_empty() {
        return false;
    }
    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty() {
        return false;
    }
    let host_port = authority.rsplit('@').next().unwrap_or_default();
    let host = host_port
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or(host_port)
        .split(':')
        .next()
        .unwrap_or_default();
    if host.is_empty() {
        return false;
    }

    match key {
        "NEO4J_URI" => matches!(scheme, "bolt" | "neo4j"),
        "DATABASE_URL" => scheme.starts_with("postgresql") && rest.contains('/'),
        "REDIS_URL" | "CELERY_BROKER_URL" | "CELERY_RESULT_BACKEND" => scheme == "redis",
        _ => matches!(scheme, "http" | "https"),
    }
}

fn storage_path_string_is_safe(value: &str) -> bool {
    let trimmed = value.trim().replace('\\', "/");
    trimmed.starts_with('/') && !trimmed.split('/').any(|part| part == "..")
}

fn host_data_root_issue(value: &str) -> Option<String> {
    let stripped = value.trim();
    if stripped.is_empty() {
        return Some("IGY6_DATA_ROOT must not be empty.".to_string());
    }
    let normalized = stripped.replace('\\', "/");
    if normalized == DEFAULT_IGY6_DATA_ROOT {
        return None;
    }
    if normalized == "/" || normalized == "~" {
        return Some("IGY6_DATA_ROOT must point to a dedicated folder.".to_string());
    }
    let windows_drive_root =
        normalized.len() == 3 && normalized.as_bytes()[1] == b':' && normalized.ends_with('/');
    if windows_drive_root {
        return Some("IGY6_DATA_ROOT must not be a drive root.".to_string());
    }
    if stripped.contains('\\') {
        return Some("Use forward slashes in IGY6_DATA_ROOT.".to_string());
    }
    let windows_absolute = normalized.len() > 3
        && normalized.as_bytes()[1] == b':'
        && normalized.as_bytes()[2] == b'/'
        && normalized.as_bytes()[0].is_ascii_alphabetic();
    let linux_absolute = normalized.starts_with('/');
    if !windows_absolute && !linux_absolute {
        return Some(
            "Use ../IGY6_Data or an absolute path such as D:/Projects/IGY6_Data or /home/user/IGY6_Data."
                .to_string(),
        );
    }
    if normalized.split('/').any(|part| part == "..") {
        return Some("IGY6_DATA_ROOT must not contain path traversal.".to_string());
    }
    None
}

fn strip_matching_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn render_file_summary(report: &ConfigFileReport) -> String {
    if report.exists {
        format!(
            "{}: {} ({} keys, values redacted)\n",
            report.label,
            if report.passed() { "valid" } else { "invalid" },
            report.key_count
        )
    } else {
        format!(
            "{}: {} (values redacted)\n",
            report.label,
            if report.passed() { "absent" } else { "missing" }
        )
    }
}

fn render_finding(finding: &ConfigFinding) -> String {
    let mut rendered = finding.source.clone();
    if let Some(key) = &finding.key {
        rendered.push(' ');
        rendered.push_str(key);
    }
    if let Some(line_number) = finding.line_number {
        rendered.push_str(&format!(" line {line_number}"));
    }
    rendered.push_str(": ");
    rendered.push_str(&finding.message);
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    const VALID_ENV: &str = "APP_ENV=local
APP_HOST=127.0.0.1
APP_PORT=8000
API_BASE_URL=http://127.0.0.1:8000
WEB_BASE_URL=http://127.0.0.1:3000
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_DB=adaptive_intelligence
POSTGRES_USER=adaptive
POSTGRES_PASSWORD=change-me-local-only
DATABASE_URL=postgresql+psycopg://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_URL=redis://redis:6379/0
CELERY_BROKER_URL=redis://redis:6379/0
CELERY_RESULT_BACKEND=redis://redis:6379/1
QDRANT_HOST=qdrant
QDRANT_PORT=6333
QDRANT_URL=http://qdrant:6333
QDRANT_CHUNK_COLLECTION=igy6_chunks
QDRANT_CHUNK_VECTOR_SIZE=384
NEO4J_HOST=neo4j
NEO4J_HTTP_PORT=7474
NEO4J_BOLT_PORT=7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=change-me-local-only
NEO4J_URI=bolt://neo4j:7687
MLFLOW_TRACKING_URI=http://mlflow:5000
MLFLOW_ARTIFACT_ROOT=/mlflow/artifacts
PHOENIX_HOST=phoenix
PHOENIX_PORT=6006
PHOENIX_COLLECTOR_ENDPOINT=http://phoenix:6006
ARTIFACT_STORE_PATH=/workspace/storage/artifacts
EXPORT_STORE_PATH=/workspace/storage/exports
ENV_FILE_PATH=/workspace/project/.env
ENV_BACKUP_DIR=/workspace/storage/env_backups
IGY6_DATA_ROOT=../IGY6_Data
EXTERNAL_MODEL_POLICY_DEFAULT=blocked
SINGLE_USER_MODE=true
AUDIT_LOG_LEVEL=info
APPROVAL_REQUIRED_DEFAULT=true
";

    #[test]
    fn parses_simple_env_line() {
        let pair = parse_env_line("APP_PORT=8000")
            .expect("line parses")
            .expect("not skipped");
        assert_eq!(pair.key, "APP_PORT");
        assert_eq!(pair.value, "8000");
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        assert_eq!(parse_env_line("# comment").expect("comment ok"), None);
        assert_eq!(parse_env_line("  ").expect("blank ok"), None);
    }

    #[test]
    fn validates_boolean_and_port() {
        assert!(parse_bool("SINGLE_USER_MODE", "true").expect("bool"));
        assert_eq!(parse_port("APP_PORT", "8000").expect("port"), 8000);
        assert!(parse_port("APP_PORT", "0").is_err());
    }

    #[test]
    fn valid_env_example_passes() {
        let report =
            validate_env_content(VALID_ENV, ".env.example", Path::new(".env.example"), true);
        assert!(report.passed(), "{:?}", report.errors);
        assert_eq!(report.key_count, REQUIRED_KEYS.len());
    }

    #[test]
    fn malformed_line_is_error() {
        let report = validate_env_content(
            &format!("{VALID_ENV}\nBROKEN_LINE\n"),
            ".env.example",
            Path::new(".env.example"),
            true,
        );
        assert!(report
            .errors
            .iter()
            .any(|finding| finding.message.contains("must contain '='")));
    }

    #[test]
    fn empty_key_is_error() {
        let report = validate_env_content(
            &format!("{VALID_ENV}\n=missing\n"),
            ".env.example",
            Path::new(".env.example"),
            true,
        );
        assert!(report
            .errors
            .iter()
            .any(|finding| finding.message.contains("key must not be empty")));
    }

    #[test]
    fn duplicate_key_is_error() {
        let report = validate_env_content(
            &format!("{VALID_ENV}\nAPP_ENV=other\n"),
            ".env.example",
            Path::new(".env.example"),
            true,
        );
        assert!(report
            .errors
            .iter()
            .any(|finding| finding.message.contains("Duplicate key")));
    }

    #[test]
    fn missing_required_key_is_error() {
        let without_app_env = VALID_ENV
            .lines()
            .filter(|line| !line.starts_with("APP_ENV="))
            .collect::<Vec<_>>()
            .join("\n");
        let report = validate_env_content(
            &without_app_env,
            ".env.example",
            Path::new(".env.example"),
            true,
        );
        assert!(report
            .errors
            .iter()
            .any(|finding| finding.key.as_deref() == Some("APP_ENV")));
    }

    #[test]
    fn cli_report_redacts_values() {
        let env_example =
            validate_env_content(VALID_ENV, ".env.example", Path::new(".env.example"), true);
        let env = validate_env_content(VALID_ENV, ".env", Path::new(".env"), false);
        let output = render_cli_report(&ConfigValidationReport { env_example, env });
        assert!(output.contains("values redacted"));
        assert!(!output.contains("change-me-local-only"));
        assert!(!output.contains("postgresql+psycopg"));
    }

    #[test]
    fn env_absent_does_not_fail() {
        let mut root = env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        root.push(format!("igy6-config-test-{unique}"));
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join(".env.example"), VALID_ENV).expect("write example");

        let report = validate_repo_config(&root).expect("validate repo config");
        fs::remove_dir_all(&root).expect("cleanup");
        assert!(report.passed(), "{:?}", report.error_messages());
        assert!(!report.env.exists);
    }
}
