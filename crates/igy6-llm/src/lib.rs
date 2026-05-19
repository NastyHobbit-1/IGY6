use std::env;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const DEFAULT_PROVIDER: &str = "none";
pub const OLLAMA_PROVIDER: &str = "ollama";
pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://host.docker.internal:11434";
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
pub const DEFAULT_MAX_EVIDENCE_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: Option<String>,
    pub timeout: Duration,
    pub evidence_required: bool,
    pub max_evidence_bytes: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Disabled,
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            model: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            evidence_required: true,
            max_evidence_bytes: DEFAULT_MAX_EVIDENCE_BYTES,
        }
    }
}

impl LlmConfig {
    pub fn from_env() -> Result<Self, LlmError> {
        let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| DEFAULT_PROVIDER.to_string());
        let base_url =
            env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_BASE_URL.to_string());
        let model = env::var("OLLAMA_MODEL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let timeout_seconds = env::var("LLM_TIMEOUT_SECONDS")
            .ok()
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    LlmError::InvalidConfig(
                        "LLM_TIMEOUT_SECONDS must be a positive integer".to_string(),
                    )
                })
            })
            .transpose()?
            .unwrap_or(DEFAULT_TIMEOUT_SECONDS);
        let evidence_required = env::var("LLM_EVIDENCE_REQUIRED")
            .ok()
            .map(|value| parse_bool(&value, "LLM_EVIDENCE_REQUIRED"))
            .transpose()?
            .unwrap_or(true);

        let provider = LlmProvider::parse(&provider)?;
        let config = Self {
            provider,
            base_url,
            model,
            timeout: Duration::from_secs(timeout_seconds),
            evidence_required,
            max_evidence_bytes: DEFAULT_MAX_EVIDENCE_BYTES,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), LlmError> {
        if self.timeout.is_zero() {
            return Err(LlmError::InvalidConfig(
                "LLM_TIMEOUT_SECONDS must be greater than zero".to_string(),
            ));
        }
        match self.provider {
            LlmProvider::Disabled => Ok(()),
            LlmProvider::Ollama => {
                parse_local_http_base_url(&self.base_url)?;
                if self.model.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(LlmError::InvalidConfig(
                        "OLLAMA_MODEL is required when LLM_PROVIDER=ollama".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    pub fn status(&self) -> LlmStatus {
        match self.provider {
            LlmProvider::Disabled => LlmStatus {
                provider: DEFAULT_PROVIDER.to_string(),
                state: LlmStatusState::Disabled,
                model: None,
                base_url: redact_url(&self.base_url),
                evidence_required: self.evidence_required,
                timeout_seconds: self.timeout.as_secs(),
                message: "LLM provider disabled; deterministic evidence fallback is active"
                    .to_string(),
            },
            LlmProvider::Ollama => LlmStatus {
                provider: OLLAMA_PROVIDER.to_string(),
                state: LlmStatusState::Configured,
                model: self.model.clone(),
                base_url: redact_url(&self.base_url),
                evidence_required: self.evidence_required,
                timeout_seconds: self.timeout.as_secs(),
                message: "Local Ollama provider configured but not wired into Assistant generation"
                    .to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Disabled,
    Ollama,
}

impl LlmProvider {
    fn parse(value: &str) -> Result<Self, LlmError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | DEFAULT_PROVIDER => Ok(Self::Disabled),
            OLLAMA_PROVIDER => Ok(Self::Ollama),
            other => Err(LlmError::InvalidConfig(format!(
                "unsupported LLM_PROVIDER {other:?}; supported values are none and ollama"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmStatus {
    pub provider: String,
    pub state: LlmStatusState,
    pub model: Option<String>,
    pub base_url: String,
    pub evidence_required: bool,
    pub timeout_seconds: u64,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmStatusState {
    Disabled,
    Configured,
    Healthy,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmGenerateRequest {
    pub prompt: String,
    pub evidence_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmGenerateResponse {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub done: bool,
    pub redacted_output_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
}

pub trait LlmHttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, LlmError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdHttpTransport;

impl LlmHttpTransport for StdHttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, LlmError> {
        send_local_http(request)
    }
}

pub fn check_health<T: LlmHttpTransport>(
    config: &LlmConfig,
    transport: &T,
) -> Result<LlmStatus, LlmError> {
    config.validate()?;
    if config.provider == LlmProvider::Disabled {
        return Ok(config.status());
    }

    let response = transport.send(&HttpRequest {
        method: "GET".to_string(),
        url: join_url(&config.base_url, "/api/tags")?,
        body: None,
        timeout: config.timeout,
    })?;
    if response.status_code != 200 {
        return Err(LlmError::ProviderUnavailable(format!(
            "Ollama health check returned HTTP {}",
            response.status_code
        )));
    }

    let mut status = config.status();
    status.state = LlmStatusState::Healthy;
    status.message = "Local Ollama provider health check passed".to_string();
    Ok(status)
}

pub fn generate<T: LlmHttpTransport>(
    config: &LlmConfig,
    request: &LlmGenerateRequest,
    transport: &T,
) -> Result<LlmGenerateResponse, LlmError> {
    config.validate()?;
    if config.provider == LlmProvider::Disabled {
        return Err(LlmError::ProviderDisabled);
    }
    if config.evidence_required && request.evidence_bytes == 0 {
        return Err(LlmError::InsufficientEvidence);
    }
    if request.evidence_bytes > config.max_evidence_bytes {
        return Err(LlmError::EvidenceBudgetExceeded {
            max_bytes: config.max_evidence_bytes,
            actual_bytes: request.evidence_bytes,
        });
    }
    if request.prompt.trim().is_empty() {
        return Err(LlmError::InvalidRequest(
            "prompt must not be empty".to_string(),
        ));
    }

    let model = config
        .model
        .as_deref()
        .ok_or_else(|| LlmError::InvalidConfig("OLLAMA_MODEL is required".to_string()))?;
    let body = serde_json::json!({
        "model": model,
        "prompt": request.prompt,
        "stream": false
    })
    .to_string();

    let response = transport.send(&HttpRequest {
        method: "POST".to_string(),
        url: join_url(&config.base_url, "/api/generate")?,
        body: Some(body),
        timeout: config.timeout,
    })?;
    if response.status_code != 200 {
        return Err(LlmError::ProviderUnavailable(format!(
            "Ollama generate returned HTTP {}",
            response.status_code
        )));
    }

    parse_ollama_generate_response(model, &response.body)
}

fn parse_ollama_generate_response(
    model: &str,
    body: &str,
) -> Result<LlmGenerateResponse, LlmError> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        LlmError::InvalidProviderResponse("Ollama response was not valid JSON".to_string())
    })?;
    let text = value
        .get("response")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            LlmError::InvalidProviderResponse(
                "Ollama response did not include response text".to_string(),
            )
        })?
        .to_string();
    let done = value
        .get("done")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    Ok(LlmGenerateResponse {
        provider: OLLAMA_PROVIDER.to_string(),
        model: model.to_string(),
        redacted_output_preview: redact_sensitive_text(&text),
        text,
        done,
    })
}

fn send_local_http(request: &HttpRequest) -> Result<HttpResponse, LlmError> {
    let endpoint = parse_local_http_url(&request.url)?;
    let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
        .map_err(|error| LlmError::Transport(error.to_string()))?;
    stream
        .set_read_timeout(Some(request.timeout))
        .map_err(|error| LlmError::Transport(error.to_string()))?;
    stream
        .set_write_timeout(Some(request.timeout))
        .map_err(|error| LlmError::Transport(error.to_string()))?;
    let body = request.body.as_deref().unwrap_or("");
    let rendered = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        request.method,
        endpoint.path,
        endpoint.host_header,
        body.len(),
        body
    );
    stream
        .write_all(rendered.as_bytes())
        .map_err(|error| LlmError::Transport(error.to_string()))?;

    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|error| LlmError::Transport(error.to_string()))?;
    parse_http_response(&raw)
}

fn parse_http_response(raw: &str) -> Result<HttpResponse, LlmError> {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let status_line = head.lines().next().unwrap_or_default();
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| LlmError::Transport("provider response missing HTTP status".to_string()))?;
    Ok(HttpResponse {
        status_code,
        body: body.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
    host_header: String,
}

fn join_url(base_url: &str, path: &str) -> Result<String, LlmError> {
    let base = base_url.trim().trim_end_matches('/');
    if !path.starts_with('/') {
        return Err(LlmError::InvalidConfig(
            "provider path must start with /".to_string(),
        ));
    }
    parse_local_http_base_url(base)?;
    Ok(format!("{base}{path}"))
}

fn parse_local_http_base_url(base_url: &str) -> Result<(), LlmError> {
    let parsed = parse_local_http_url(base_url)?;
    if parsed.path != "/" {
        return Err(LlmError::InvalidConfig(
            "OLLAMA_BASE_URL must not include a path".to_string(),
        ));
    }
    Ok(())
}

fn parse_local_http_url(url: &str) -> Result<ParsedUrl, LlmError> {
    let without_scheme = url.strip_prefix("http://").ok_or_else(|| {
        LlmError::InvalidConfig("only local http:// providers are supported".to_string())
    })?;
    if without_scheme.contains('@') {
        return Err(LlmError::InvalidConfig(
            "provider URLs must not include credentials".to_string(),
        ));
    }
    let (authority, path) = without_scheme
        .split_once('/')
        .map(|(authority, path)| (authority, format!("/{path}")))
        .unwrap_or((without_scheme, "/".to_string()));
    if authority.is_empty() {
        return Err(LlmError::InvalidConfig(
            "provider URL host is required".to_string(),
        ));
    }
    let (host, port) = parse_host_port(authority)?;
    validate_local_host(&host)?;
    let host_header = if port == 80 {
        host.clone()
    } else {
        format!("{host}:{port}")
    };
    Ok(ParsedUrl {
        host,
        port,
        path,
        host_header,
    })
}

fn parse_host_port(authority: &str) -> Result<(String, u16), LlmError> {
    let (host, port) = authority
        .rsplit_once(':')
        .map(|(host, port)| {
            let parsed = port
                .parse::<u16>()
                .map_err(|_| LlmError::InvalidConfig("provider URL port is invalid".to_string()))?;
            Ok((host.to_string(), parsed))
        })
        .transpose()?
        .unwrap_or_else(|| (authority.to_string(), 80));
    if host.trim().is_empty() {
        return Err(LlmError::InvalidConfig(
            "provider URL host is required".to_string(),
        ));
    }
    Ok((host, port))
}

fn validate_local_host(host: &str) -> Result<(), LlmError> {
    let normalized = host.trim().to_ascii_lowercase();
    let allowed = matches!(
        normalized.as_str(),
        "localhost" | "127.0.0.1" | "::1" | "host.docker.internal"
    );
    if !allowed {
        return Err(LlmError::InvalidConfig(
            "only localhost, 127.0.0.1, ::1, or host.docker.internal providers are supported"
                .to_string(),
        ));
    }
    Ok(())
}

pub fn redact_sensitive_text(value: &str) -> String {
    let mut redacted = Vec::new();
    for token in value.split_whitespace().take(32) {
        let lower = token.to_ascii_lowercase();
        if lower.contains("secret")
            || lower.contains("token")
            || lower.contains("password")
            || lower.contains("api_key")
            || lower.contains("apikey")
            || lower.contains("private_key")
        {
            redacted.push("[redacted]");
        } else {
            redacted.push(token);
        }
    }
    let mut preview = redacted.join(" ");
    if value.split_whitespace().count() > 32 {
        preview.push_str(" ...");
    }
    preview
}

fn redact_url(url: &str) -> String {
    if url.contains('@') {
        return "http://[redacted]".to_string();
    }
    url.to_string()
}

fn parse_bool(value: &str, name: &str) -> Result<bool, LlmError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(LlmError::InvalidConfig(format!(
            "{name} must be true or false"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmError {
    ProviderDisabled,
    InsufficientEvidence,
    EvidenceBudgetExceeded {
        max_bytes: usize,
        actual_bytes: usize,
    },
    InvalidConfig(String),
    InvalidRequest(String),
    InvalidProviderResponse(String),
    ProviderUnavailable(String),
    Transport(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderDisabled => write!(formatter, "LLM provider is disabled"),
            Self::InsufficientEvidence => write!(formatter, "insufficient evidence for LLM answer"),
            Self::EvidenceBudgetExceeded {
                max_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "evidence budget exceeded: max {max_bytes} bytes, got {actual_bytes} bytes"
            ),
            Self::InvalidConfig(error) => write!(formatter, "invalid LLM config: {error}"),
            Self::InvalidRequest(error) => write!(formatter, "invalid LLM request: {error}"),
            Self::InvalidProviderResponse(error) => {
                write!(formatter, "invalid LLM provider response: {error}")
            }
            Self::ProviderUnavailable(error) => {
                write!(formatter, "LLM provider unavailable: {error}")
            }
            Self::Transport(error) => write!(formatter, "LLM transport error: {error}"),
        }
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    struct FakeTransport {
        response: HttpResponse,
        requests: RefCell<Vec<HttpRequest>>,
    }

    impl FakeTransport {
        fn new(response: HttpResponse) -> Self {
            Self {
                response,
                requests: RefCell::new(Vec::new()),
            }
        }
    }

    impl LlmHttpTransport for FakeTransport {
        fn send(&self, request: &HttpRequest) -> Result<HttpResponse, LlmError> {
            self.requests.borrow_mut().push(request.clone());
            Ok(self.response.clone())
        }
    }

    fn ollama_config() -> LlmConfig {
        LlmConfig {
            provider: LlmProvider::Ollama,
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            model: Some("llama3.2:latest".to_string()),
            timeout: Duration::from_secs(7),
            evidence_required: true,
            max_evidence_bytes: 256,
        }
    }

    #[test]
    fn default_config_is_disabled() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, LlmProvider::Disabled);
        assert_eq!(config.status().state, LlmStatusState::Disabled);
    }

    #[test]
    fn rejects_external_or_secret_bearing_provider_urls() {
        let mut config = ollama_config();
        config.base_url = "https://api.example.com".to_string();
        assert!(matches!(config.validate(), Err(LlmError::InvalidConfig(_))));

        config.base_url = "http://token@example@127.0.0.1:11434".to_string();
        assert!(matches!(config.validate(), Err(LlmError::InvalidConfig(_))));

        config.base_url = "http://example.com:11434".to_string();
        assert!(matches!(config.validate(), Err(LlmError::InvalidConfig(_))));
    }

    #[test]
    fn health_check_uses_safe_local_tags_endpoint() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{\"models\":[]}".to_string(),
        });

        let status = check_health(&ollama_config(), &transport).expect("health should pass");

        assert_eq!(status.state, LlmStatusState::Healthy);
        let requests = transport.requests.borrow();
        assert_eq!(requests[0].method, "GET");
        assert_eq!(
            requests[0].url,
            "http://host.docker.internal:11434/api/tags"
        );
        assert!(requests[0].body.is_none());
        assert_eq!(requests[0].timeout, Duration::from_secs(7));
    }

    #[test]
    fn generate_is_timeout_bound_and_structured() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{\"response\":\"Answer with password abc\",\"done\":true}".to_string(),
        });
        let response = generate(
            &ollama_config(),
            &LlmGenerateRequest {
                prompt: "Use citation [1] only.".to_string(),
                evidence_bytes: 42,
            },
            &transport,
        )
        .expect("generate should pass");

        assert_eq!(response.provider, OLLAMA_PROVIDER);
        assert_eq!(response.model, "llama3.2:latest");
        assert_eq!(response.text, "Answer with password abc");
        assert_eq!(
            response.redacted_output_preview,
            "Answer with [redacted] abc"
        );

        let requests = transport.requests.borrow();
        assert_eq!(requests[0].method, "POST");
        assert_eq!(
            requests[0].url,
            "http://host.docker.internal:11434/api/generate"
        );
        assert_eq!(requests[0].timeout, Duration::from_secs(7));
        let body = requests[0].body.as_ref().expect("body expected");
        assert!(body.contains("\"stream\":false"));
        assert!(body.contains("\"model\":\"llama3.2:latest\""));
    }

    #[test]
    fn generate_fails_closed_without_evidence_when_required() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{}".to_string(),
        });
        let error = generate(
            &ollama_config(),
            &LlmGenerateRequest {
                prompt: "No evidence.".to_string(),
                evidence_bytes: 0,
            },
            &transport,
        )
        .expect_err("missing evidence should fail");

        assert_eq!(error, LlmError::InsufficientEvidence);
        assert!(transport.requests.borrow().is_empty());
    }

    #[test]
    fn generate_fails_closed_when_disabled_or_over_budget() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{}".to_string(),
        });
        let disabled = LlmConfig::default();
        assert_eq!(
            generate(
                &disabled,
                &LlmGenerateRequest {
                    prompt: "test".to_string(),
                    evidence_bytes: 1,
                },
                &transport,
            )
            .expect_err("disabled should fail"),
            LlmError::ProviderDisabled
        );

        let error = generate(
            &ollama_config(),
            &LlmGenerateRequest {
                prompt: "test".to_string(),
                evidence_bytes: 257,
            },
            &transport,
        )
        .expect_err("over budget should fail");
        assert_eq!(
            error,
            LlmError::EvidenceBudgetExceeded {
                max_bytes: 256,
                actual_bytes: 257
            }
        );
    }

    #[test]
    fn invalid_provider_responses_are_explicit_errors() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{\"done\":true}".to_string(),
        });
        let error = generate(
            &ollama_config(),
            &LlmGenerateRequest {
                prompt: "test".to_string(),
                evidence_bytes: 1,
            },
            &transport,
        )
        .expect_err("missing response field should fail");

        assert!(matches!(error, LlmError::InvalidProviderResponse(_)));
    }
}
