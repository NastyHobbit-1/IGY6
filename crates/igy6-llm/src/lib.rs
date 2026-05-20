use std::env;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

pub const DEFAULT_PROVIDER: &str = "none";
pub const OLLAMA_PROVIDER: &str = "ollama";
pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://host.docker.internal:11434";
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
pub const DEFAULT_MAX_EVIDENCE_BYTES: usize = 32 * 1024;
pub const DEFAULT_TASK_NAME: &str = "chat_default";
pub const LOCAL_LLM_ROUTING_CONFIG_PATH: &str = "configs/local-llm-routing.json";

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
                message: "Local Ollama provider configured; evidence-answer generation uses task routing when enabled"
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

#[derive(Debug, Clone, PartialEq)]
pub struct LlmGenerateRequest {
    pub prompt: String,
    pub evidence_bytes: usize,
    pub system_instruction: Option<String>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmGenerateResponse {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub done: bool,
    pub redacted_output_preview: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlmRoutingConfig {
    pub schema_version: i64,
    pub provider: String,
    pub default_provider: String,
    pub hardware_target: String,
    pub default_models: Vec<String>,
    pub optional_models: Vec<String>,
    pub blocked_default_models: Vec<String>,
    pub tasks: Vec<LocalLlmTaskRoute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlmTaskRoute {
    pub task_name: String,
    pub model: String,
    pub optional_model: Option<String>,
    pub purpose: String,
    pub system_instruction: String,
    pub temperature: f64,
    pub evidence_required: bool,
    pub max_context_note: String,
}

impl LocalLlmRoutingConfig {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, LlmError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|error| {
            LlmError::InvalidConfig(format!(
                "local LLM routing config could not be read: {error}"
            ))
        })?;
        Self::from_json_str(&content)
    }

    pub fn from_json_str(content: &str) -> Result<Self, LlmError> {
        let value: serde_json::Value = serde_json::from_str(content).map_err(|error| {
            LlmError::InvalidConfig(format!("local LLM routing JSON is invalid: {error}"))
        })?;
        Self::from_value(&value)
    }

    pub fn from_value(value: &serde_json::Value) -> Result<Self, LlmError> {
        let object = value.as_object().ok_or_else(|| {
            LlmError::InvalidConfig("local LLM routing config must be a JSON object".to_string())
        })?;
        let tasks_value = object
            .get("tasks")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                LlmError::InvalidConfig("local LLM routing config requires tasks array".to_string())
            })?;
        let mut tasks = Vec::new();
        for task_value in tasks_value {
            tasks.push(parse_task_route(task_value)?);
        }
        let config = Self {
            schema_version: required_i64(object, "schema_version")?,
            provider: required_string(object, "provider")?,
            default_provider: required_string(object, "default_provider")?,
            hardware_target: required_string(object, "hardware_target")?,
            default_models: required_string_array(object, "default_models")?,
            optional_models: optional_string_array(object, "optional_models")?,
            blocked_default_models: optional_string_array(object, "blocked_default_models")?,
            tasks,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), LlmError> {
        if self.schema_version != 1 {
            return Err(LlmError::InvalidConfig(
                "local LLM routing schema_version must be 1".to_string(),
            ));
        }
        if self.provider != OLLAMA_PROVIDER {
            return Err(LlmError::InvalidConfig(
                "local LLM routing provider must be ollama".to_string(),
            ));
        }
        if self.default_provider != DEFAULT_PROVIDER {
            return Err(LlmError::InvalidConfig(
                "local LLM routing default_provider must remain none".to_string(),
            ));
        }
        let required_tasks = [
            "code_repo",
            "evidence_summary",
            "fast_triage",
            "report_draft",
            "action_explanation",
            DEFAULT_TASK_NAME,
        ];
        for required in required_tasks {
            if self.tasks.iter().all(|task| task.task_name != required) {
                return Err(LlmError::InvalidConfig(format!(
                    "local LLM routing missing required task {required}"
                )));
            }
        }
        for model in &self.default_models {
            if self
                .blocked_default_models
                .iter()
                .any(|blocked| blocked == model)
            {
                return Err(LlmError::InvalidConfig(format!(
                    "blocked model {model} cannot be a default pull"
                )));
            }
        }
        for task in &self.tasks {
            if task.task_name.trim().is_empty() {
                return Err(LlmError::InvalidConfig(
                    "task_name must not be empty".to_string(),
                ));
            }
            if task.model.trim().is_empty() {
                return Err(LlmError::InvalidConfig(format!(
                    "task {} model must not be empty",
                    task.task_name
                )));
            }
            if task.system_instruction.trim().is_empty() {
                return Err(LlmError::InvalidConfig(format!(
                    "task {} system_instruction must not be empty",
                    task.task_name
                )));
            }
            if !task.evidence_required {
                return Err(LlmError::InvalidConfig(format!(
                    "task {} must keep evidence_required=true",
                    task.task_name
                )));
            }
            if !(0.0..=1.0).contains(&task.temperature) {
                return Err(LlmError::InvalidConfig(format!(
                    "task {} temperature must be between 0 and 1",
                    task.task_name
                )));
            }
        }
        Ok(())
    }

    pub fn route_for_task(&self, task_name: &str) -> Option<&LocalLlmTaskRoute> {
        let wanted = task_name.trim();
        self.tasks
            .iter()
            .find(|task| task.task_name == wanted)
            .or_else(|| {
                self.tasks
                    .iter()
                    .find(|task| task.task_name == DEFAULT_TASK_NAME)
            })
    }

    pub fn select_route(&self, task_name: &str) -> Result<SelectedLocalLlmRoute, LlmError> {
        let route = self.route_for_task(task_name).ok_or_else(|| {
            LlmError::InvalidConfig("local LLM routing missing chat_default route".to_string())
        })?;
        Ok(SelectedLocalLlmRoute {
            task_name: route.task_name.clone(),
            model: route.model.clone(),
            system_instruction: route.system_instruction.clone(),
            temperature: route.temperature,
            evidence_required: route.evidence_required,
        })
    }

    pub fn default_pull_models(&self) -> Vec<String> {
        self.default_models.clone()
    }

    pub fn recommended_models(&self) -> Vec<String> {
        let mut models = self.default_models.clone();
        for model in &self.optional_models {
            if !models.iter().any(|existing| existing == model) {
                models.push(model.clone());
            }
        }
        models
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedLocalLlmRoute {
    pub task_name: String,
    pub model: String,
    pub system_instruction: String,
    pub temperature: f64,
    pub evidence_required: bool,
}

impl LlmConfig {
    pub fn with_selected_route(&self, route: &SelectedLocalLlmRoute) -> Self {
        let mut routed = self.clone();
        routed.model = Some(route.model.clone());
        routed.evidence_required = route.evidence_required;
        routed
    }
}

pub fn load_local_llm_routing_config() -> Result<LocalLlmRoutingConfig, LlmError> {
    LocalLlmRoutingConfig::from_path(LOCAL_LLM_ROUTING_CONFIG_PATH)
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

fn parse_task_route(value: &serde_json::Value) -> Result<LocalLlmTaskRoute, LlmError> {
    let object = value.as_object().ok_or_else(|| {
        LlmError::InvalidConfig("local LLM task route must be an object".to_string())
    })?;
    Ok(LocalLlmTaskRoute {
        task_name: required_string(object, "task_name")?,
        model: required_string(object, "model")?,
        optional_model: optional_string(object, "optional_model")?,
        purpose: required_string(object, "purpose")?,
        system_instruction: required_string(object, "system_instruction")?,
        temperature: required_f64(object, "temperature")?,
        evidence_required: required_bool(object, "evidence_required")?,
        max_context_note: required_string(object, "max_context_note")?,
    })
}

fn required_string(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, LlmError> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be a non-empty string")))
}

fn optional_string(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<String>, LlmError> {
    match object.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be a string"))),
    }
}

fn required_i64(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<i64, LlmError> {
    object
        .get(key)
        .and_then(|value| value.as_i64())
        .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be an integer")))
}

fn required_f64(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<f64, LlmError> {
    object
        .get(key)
        .and_then(|value| value.as_f64())
        .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be a number")))
}

fn required_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<bool, LlmError> {
    object
        .get(key)
        .and_then(|value| value.as_bool())
        .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be a boolean")))
}

fn required_string_array(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Vec<String>, LlmError> {
    let values = object
        .get(key)
        .and_then(|value| value.as_array())
        .ok_or_else(|| LlmError::InvalidConfig(format!("{key} must be an array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|value| value.to_string())
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    LlmError::InvalidConfig(format!("{key} values must be non-empty strings"))
                })
        })
        .collect()
}

fn optional_string_array(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Vec<String>, LlmError> {
    match object.get(key) {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(_) => required_string_array(object, key),
    }
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
    let mut body = serde_json::json!({
        "model": model,
        "prompt": request.prompt,
        "stream": false
    });
    if let Some(system_instruction) = request.system_instruction.as_deref() {
        body["system"] = serde_json::Value::String(system_instruction.to_string());
    }
    if let Some(temperature) = request.temperature {
        if !(0.0..=1.0).contains(&temperature) {
            return Err(LlmError::InvalidRequest(
                "temperature must be between 0 and 1".to_string(),
            ));
        }
        body["options"] = serde_json::json!({ "temperature": temperature });
    }
    let body = body.to_string();

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

    fn routing_json() -> &'static str {
        include_str!("../../../configs/local-llm-routing.json")
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
                system_instruction: None,
                temperature: None,
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
    fn generate_sends_selected_route_system_instruction_and_temperature() {
        let transport = FakeTransport::new(HttpResponse {
            status_code: 200,
            body: "{\"response\":\"Routed answer [evidence-1]\",\"done\":true}".to_string(),
        });
        let routing = LocalLlmRoutingConfig::from_json_str(routing_json()).expect("routing config");
        let selected = routing.select_route("code_repo").expect("selected route");
        let config = ollama_config().with_selected_route(&selected);

        let response = generate(
            &config,
            &LlmGenerateRequest {
                prompt: "Use citation [evidence-1].".to_string(),
                evidence_bytes: 28,
                system_instruction: Some(selected.system_instruction.clone()),
                temperature: Some(selected.temperature),
            },
            &transport,
        )
        .expect("generate should pass");

        assert_eq!(response.model, "qwen2.5-coder:7b");
        let requests = transport.requests.borrow();
        let body = requests[0].body.as_ref().expect("body expected");
        let value: serde_json::Value = serde_json::from_str(body).expect("request JSON");
        assert_eq!(value["model"], "qwen2.5-coder:7b");
        assert_eq!(value["system"], selected.system_instruction);
        assert_eq!(value["options"]["temperature"], selected.temperature);
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
                system_instruction: None,
                temperature: None,
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
                    system_instruction: None,
                    temperature: None,
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
                system_instruction: None,
                temperature: None,
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
                system_instruction: None,
                temperature: None,
            },
            &transport,
        )
        .expect_err("missing response field should fail");

        assert!(matches!(error, LlmError::InvalidProviderResponse(_)));
    }

    #[test]
    fn local_routing_config_validates_required_tasks_and_models() {
        let config = LocalLlmRoutingConfig::from_json_str(routing_json()).expect("routing config");

        assert_eq!(config.default_provider, DEFAULT_PROVIDER);
        assert_eq!(
            config.default_pull_models(),
            vec![
                "qwen2.5-coder:7b".to_string(),
                "llama3.1:8b".to_string(),
                "gemma3:4b".to_string()
            ]
        );
        assert!(config
            .recommended_models()
            .contains(&"gemma3:12b".to_string()));
        assert_eq!(
            config
                .route_for_task("code_repo")
                .expect("code route")
                .model,
            "qwen2.5-coder:7b"
        );
        assert_eq!(
            config
                .route_for_task("unknown_task")
                .expect("fallback route")
                .task_name,
            DEFAULT_TASK_NAME
        );
        assert!(config.tasks.iter().all(|task| task.evidence_required));
    }

    #[test]
    fn local_routing_selects_chat_default_for_unknown_tasks() {
        let config = LocalLlmRoutingConfig::from_json_str(routing_json()).expect("routing config");

        let selected = config
            .select_route("unexpected_task")
            .expect("fallback route");

        assert_eq!(selected.task_name, DEFAULT_TASK_NAME);
        assert_eq!(selected.model, "llama3.1:8b");
        assert!(selected.evidence_required);
    }

    #[test]
    fn local_routing_rejects_unsafe_defaults_and_missing_evidence_gate() {
        let unsafe_default = routing_json().replace("\"gemma3:4b\"", "\"qwen2.5-coder:32b\"");
        assert!(matches!(
            LocalLlmRoutingConfig::from_json_str(&unsafe_default),
            Err(LlmError::InvalidConfig(_))
        ));

        let no_evidence_gate = routing_json().replacen(
            "\"evidence_required\": true",
            "\"evidence_required\": false",
            1,
        );
        assert!(matches!(
            LocalLlmRoutingConfig::from_json_str(&no_evidence_gate),
            Err(LlmError::InvalidConfig(_))
        ));
    }
}
