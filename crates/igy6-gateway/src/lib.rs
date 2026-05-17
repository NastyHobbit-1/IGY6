use std::fmt;

use igy6_agent_api::{classify_agent_intent, AgentIntentRequest, ACTION_REGISTRY};
use igy6_evidence_answer::build_evidence_answer_packet;
use igy6_read_only_api::summarize_manifest;
use igy6_retrieval_preview::{build_hydrated_chunk_search_result, build_retrieval_preview};

pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8000";
pub const DEFAULT_FALLBACK_ORIGIN: &str = "http://legacy-api:8000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    EmptyRequest,
    MalformedRequest,
    InvalidContentLength,
    InvalidFallbackOrigin(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(formatter, "request is empty"),
            Self::MalformedRequest => write!(formatter, "request is malformed"),
            Self::InvalidContentLength => write!(formatter, "content-length is invalid"),
            Self::InvalidFallbackOrigin(origin) => {
                write!(formatter, "fallback origin is invalid: {origin}")
            }
        }
    }
}

impl std::error::Error for GatewayError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayResponse {
    pub status_code: u16,
    pub reason: String,
    pub content_type: String,
    pub body: String,
    pub proxied_to_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackProxyPlan {
    pub host: String,
    pub port: u16,
    pub request_target: String,
    pub method: String,
    pub body: String,
}

pub fn parse_gateway_request(raw: &str) -> Result<GatewayRequest, GatewayError> {
    if raw.trim().is_empty() {
        return Err(GatewayError::EmptyRequest);
    }
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let mut lines = head.lines();
    let request_line = lines.next().ok_or(GatewayError::EmptyRequest)?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().ok_or(GatewayError::MalformedRequest)?;
    let path = request_parts.next().ok_or(GatewayError::MalformedRequest)?;
    let version = request_parts.next().ok_or(GatewayError::MalformedRequest)?;
    if !version.starts_with("HTTP/") || request_parts.next().is_some() {
        return Err(GatewayError::MalformedRequest);
    }

    let mut headers = Vec::new();
    let mut content_length = None;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let (name, value) = line.split_once(':').ok_or(GatewayError::MalformedRequest)?;
        let header_name = name.trim().to_string();
        let header_value = value.trim().to_string();
        if header_name.eq_ignore_ascii_case("content-length") {
            content_length = Some(
                header_value
                    .parse::<usize>()
                    .map_err(|_| GatewayError::InvalidContentLength)?,
            );
        }
        headers.push((header_name, header_value));
    }
    if let Some(expected) = content_length {
        if body.len() < expected {
            return Err(GatewayError::InvalidContentLength);
        }
    }

    Ok(GatewayRequest {
        method: method.to_string(),
        path: path.to_string(),
        version: version.to_string(),
        headers,
        body: body.to_string(),
    })
}

pub fn handle_gateway_request(
    request: &GatewayRequest,
    manifest_content: Option<&str>,
    fallback_origin: &str,
) -> GatewayResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health/live") => json_response(
            200,
            "OK",
            "{\"status\":\"ok\",\"service\":\"igy6-gateway\",\"primary_gateway\":true}".to_string(),
            false,
        ),
        ("GET", "/health/ready") => json_response(
            200,
            "OK",
            format!(
                "{{\"status\":\"ok\",\"checks\":{{\"rust_gateway\":{{\"status\":\"ok\"}},\"fastapi_fallback\":{{\"status\":\"configured\",\"origin\":\"{}\"}}}},\"primary_gateway\":\"rust\",\"fallback\":\"fastapi\"}}",
                escape_json(fallback_origin)
            ),
            false,
        ),
        ("GET", "/rust-migration/status") => {
            let summary = summarize_manifest(manifest_content.unwrap_or_default());
            json_response(
                200,
                "OK",
                format!(
                    "{{\"status\":\"ok\",\"cutover_ready\":{},\"complete_phases\":{},\"pending_phases\":{},\"primary_gateway\":\"rust\",\"fallback\":\"fastapi\"}}",
                    summary.cutover_ready, summary.complete_phases, summary.pending_phases
                ),
                false,
            )
        }
        ("GET", "/agent/capabilities") => json_response(200, "OK", agent_capabilities_json(), false),
        ("POST", "/agent/intent") => json_response(200, "OK", agent_intent_json(&request.body), false),
        ("POST", "/chat/retrieval-preview") => {
            json_response(200, "OK", retrieval_preview_json(&request.body), false)
        }
        ("POST", "/chat/evidence-answer") => {
            json_response(200, "OK", evidence_answer_json(&request.body), false)
        }
        _ => match build_fallback_proxy_plan(request, fallback_origin) {
            Ok(plan) => json_response(
                502,
                "Bad Gateway",
                format!(
                    "{{\"detail\":\"Rust gateway fallback proxy is required at runtime\",\"fallback_host\":\"{}\",\"fallback_port\":{},\"fallback_path\":\"{}\"}}",
                    escape_json(&plan.host),
                    plan.port,
                    escape_json(&plan.request_target)
                ),
                true,
            ),
            Err(error) => json_response(
                500,
                "Internal Server Error",
                format!(
                    "{{\"detail\":\"{}\",\"fallback_origin\":\"{}\"}}",
                    escape_json(&error.to_string()),
                    escape_json(fallback_origin)
                ),
                false,
            ),
        },
    }
}

pub fn build_fallback_proxy_plan(
    request: &GatewayRequest,
    fallback_origin: &str,
) -> Result<FallbackProxyPlan, GatewayError> {
    let without_scheme = fallback_origin
        .strip_prefix("http://")
        .ok_or_else(|| GatewayError::InvalidFallbackOrigin(fallback_origin.to_string()))?;
    if without_scheme.is_empty() || without_scheme.contains('/') {
        return Err(GatewayError::InvalidFallbackOrigin(
            fallback_origin.to_string(),
        ));
    }
    let (host, port) = if let Some((host, port)) = without_scheme.rsplit_once(':') {
        let port = port
            .parse::<u16>()
            .map_err(|_| GatewayError::InvalidFallbackOrigin(fallback_origin.to_string()))?;
        (host.to_string(), port)
    } else {
        (without_scheme.to_string(), 80)
    };
    if host.trim().is_empty() {
        return Err(GatewayError::InvalidFallbackOrigin(
            fallback_origin.to_string(),
        ));
    }
    Ok(FallbackProxyPlan {
        host,
        port,
        request_target: request.path.clone(),
        method: request.method.clone(),
        body: request.body.clone(),
    })
}

pub fn render_http_response(response: &GatewayResponse) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_code,
        response.reason,
        response.content_type,
        response.body.len(),
        response.body
    )
}

pub fn render_fallback_http_request(plan: &FallbackProxyPlan, original: &GatewayRequest) -> String {
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        plan.method, plan.request_target, plan.host
    );
    let mut has_content_type = false;
    for (name, value) in &original.headers {
        if name.eq_ignore_ascii_case("host")
            || name.eq_ignore_ascii_case("connection")
            || name.eq_ignore_ascii_case("content-length")
        {
            continue;
        }
        if name.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    if !plan.body.is_empty() {
        if !has_content_type {
            request.push_str("Content-Type: application/json\r\n");
        }
        request.push_str(&format!("Content-Length: {}\r\n", plan.body.len()));
    }
    request.push_str("\r\n");
    request.push_str(&plan.body);
    request
}

pub fn help_text() -> &'static str {
    "igy6-gateway\n\nUsage:\n  igy6-gateway [--bind 0.0.0.0:8000] [--fallback http://legacy-api:8000]\n  igy6-gateway --help\n\nRoutes:\n  GET /health/live\n  GET /health/ready\n  GET /rust-migration/status\n  GET /agent/capabilities\n  POST /agent/intent\n  POST /chat/retrieval-preview\n  POST /chat/evidence-answer\n\nUnsupported routes are proxied to the configured FastAPI fallback at runtime.\n"
}

fn agent_capabilities_json() -> String {
    let actions = ACTION_REGISTRY
        .iter()
        .map(|definition| {
            format!(
                "{{\"name\":\"{}\",\"interpreted_intent\":\"{}\",\"approval_required\":{},\"required_parameters\":{},\"script_backed\":{}}}",
                definition.name,
                escape_json(definition.interpreted_intent),
                definition.approval_required,
                json_string_array(definition.required_parameters),
                definition.script_argv.is_some()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"actions\":[{}],\"runtime\":{{\"gateway\":\"rust\",\"fastapi_fallback\":true}}}}",
        actions
    )
}

fn agent_intent_json(body: &str) -> String {
    let message = extract_json_string(body, "message").unwrap_or_default();
    let response = classify_agent_intent(&AgentIntentRequest {
        message,
        parameters: Vec::new(),
    });
    format!(
        "{{\"original_message\":\"{}\",\"interpreted_intent\":\"{}\",\"proposed_action\":{},\"action_type\":\"{}\",\"approval_required\":{},\"risk_level\":\"{}\",\"required_parameters\":{},\"missing_parameters\":{},\"safety_notes\":{},\"executable_now\":{},\"reason\":{}}}",
        escape_json(&response.original_message),
        escape_json(&response.interpreted_intent),
        option_json(response.proposed_action),
        action_type_json(&response.action_type),
        response.approval_required,
        risk_level_json(&response.risk_level),
        json_string_array(&response.required_parameters),
        json_string_array(&response.missing_parameters),
        json_string_array(&response.safety_notes),
        response.executable_now,
        option_string_json(response.reason.as_deref())
    )
}

fn retrieval_preview_json(body: &str) -> String {
    let message = extract_json_string(body, "message")
        .or_else(|| extract_json_string(body, "query"))
        .unwrap_or_default();
    let collection_name =
        extract_json_string(body, "collection_name").unwrap_or_else(|| "chunks".to_string());
    let context =
        build_hydrated_chunk_search_result(&message, &collection_name, false, Vec::new(), 5);
    let result = build_retrieval_preview(&message, context);
    format!(
        "{{\"query\":\"{}\",\"collection_name\":\"{}\",\"collection_exists\":{},\"answer_status\":\"{}\",\"items\":[],\"message\":\"{}\"}}",
        escape_json(&result.retrieval_context.query),
        escape_json(&result.retrieval_context.collection_name),
        result.retrieval_context.collection_exists,
        result.answer_status,
        escape_json("Rust gateway preview is contract-only without live Qdrant/PostgreSQL hydration.")
    )
}

fn evidence_answer_json(body: &str) -> String {
    let message = extract_json_string(body, "message")
        .or_else(|| extract_json_string(body, "query"))
        .unwrap_or_default();
    let retrieval_context = build_hydrated_chunk_search_result(
        &message,
        "chunks",
        false,
        Vec::<igy6_retrieval_preview::HydratedChunkSearchHit>::new(),
        5,
    );
    let answer = build_evidence_answer_packet(retrieval_context);
    format!(
        "{{\"message\":\"{}\",\"answer_status\":\"{}\",\"facts\":[],\"source_trails\":[],\"assumptions\":{},\"uncertainty\":{},\"missing_information\":{}}}",
        escape_json(&answer.message),
        answer.answer_status,
        json_owned_string_array(&answer.assumptions),
        json_owned_string_array(&answer.uncertainty),
        json_owned_string_array(&answer.missing_information)
    )
}

fn json_response(
    status_code: u16,
    reason: &str,
    body: String,
    proxied_to_fallback: bool,
) -> GatewayResponse {
    GatewayResponse {
        status_code,
        reason: reason.to_string(),
        content_type: "application/json".to_string(),
        body,
        proxied_to_fallback,
    }
}

fn extract_json_string(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = body.find(&needle)?;
    let after_key = &body[start + needle.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let value = after_colon.strip_prefix('"')?;
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(output);
        } else {
            output.push(character);
        }
    }
    None
}

fn option_json(value: Option<&str>) -> String {
    value
        .map(|item| format!("\"{}\"", escape_json(item)))
        .unwrap_or_else(|| "null".to_string())
}

fn option_string_json(value: Option<&str>) -> String {
    option_json(value)
}

fn json_string_array(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", escape_json(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_owned_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", escape_json(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn action_type_json(value: &igy6_agent_api::ActionType) -> &'static str {
    match value {
        igy6_agent_api::ActionType::ReadOnly => "read_only",
        igy6_agent_api::ActionType::SystemChanging => "system_changing",
        igy6_agent_api::ActionType::Unknown => "unknown",
    }
}

fn risk_level_json(value: &igy6_agent_api::RiskLevel) -> &'static str {
    match value {
        igy6_agent_api::RiskLevel::Low => "low",
        igy6_agent_api::RiskLevel::High => "high",
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_request_with_body() {
        let request = parse_gateway_request(
            "POST /agent/intent HTTP/1.1\r\nHost: local\r\nContent-Length: 20\r\n\r\n{\"message\":\"health\"}",
        )
        .expect("request");
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/agent/intent");
        assert_eq!(request.body, "{\"message\":\"health\"}");
    }

    #[test]
    fn health_routes_identify_rust_as_primary_gateway() {
        let response = handle_gateway_request(
            &request("GET", "/health/live", ""),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"primary_gateway\":true"));

        let response = handle_gateway_request(
            &request("GET", "/health/ready", ""),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"primary_gateway\":\"rust\""));
        assert!(response.body.contains("\"fallback\":\"fastapi\""));
    }

    #[test]
    fn migration_status_uses_manifest_summary() {
        let response = handle_gateway_request(
            &request("GET", "/rust-migration/status", ""),
            Some("{\"cutover_ready\": true, \"status\": \"complete\"}"),
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"cutover_ready\":true"));
        assert!(response.body.contains("\"complete_phases\":1"));
    }

    #[test]
    fn agent_capabilities_are_served_by_rust() {
        let response = handle_gateway_request(
            &request("GET", "/agent/capabilities", ""),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("show_project_health"));
        assert!(!response.proxied_to_fallback);
    }

    #[test]
    fn agent_intent_uses_rust_classifier() {
        let response = handle_gateway_request(
            &request("POST", "/agent/intent", "{\"message\":\"show health\"}"),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"proposed_action\":\"show_project_health\""));
        assert!(response.body.contains("\"approval_required\":false"));
    }

    #[test]
    fn retrieval_preview_and_evidence_answer_are_contract_only() {
        let response = handle_gateway_request(
            &request(
                "POST",
                "/chat/retrieval-preview",
                "{\"message\":\"what changed?\"}",
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"answer_status\":\"not_generated\""));

        let response = handle_gateway_request(
            &request(
                "POST",
                "/chat/evidence-answer",
                "{\"message\":\"what changed?\"}",
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"answer_status\":\"insufficient_evidence\""));
    }

    #[test]
    fn unsupported_routes_plan_fastapi_fallback() {
        let request = request("POST", "/sources", "{\"name\":\"x\"}");
        let plan = build_fallback_proxy_plan(&request, "http://legacy-api:8000").expect("plan");
        assert_eq!(plan.host, "legacy-api");
        assert_eq!(plan.port, 8000);
        assert_eq!(plan.request_target, "/sources");

        let response = handle_gateway_request(&request, None, "http://legacy-api:8000");
        assert_eq!(response.status_code, 502);
        assert!(response.proxied_to_fallback);
    }

    #[test]
    fn fallback_origin_is_local_http_only() {
        let error = build_fallback_proxy_plan(&request("GET", "/sources", ""), "https://api:8000")
            .expect_err("https is rejected");
        assert!(matches!(error, GatewayError::InvalidFallbackOrigin(_)));
    }

    #[test]
    fn renders_fallback_request_without_host_or_content_length_duplication() {
        let request = parse_gateway_request(
            "POST /sources HTTP/1.1\r\nHost: api\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
        )
        .expect("request");
        let plan = build_fallback_proxy_plan(&request, "http://legacy-api:8000").expect("plan");
        let rendered = render_fallback_http_request(&plan, &request);
        assert!(rendered.starts_with("POST /sources HTTP/1.1\r\nHost: legacy-api\r\n"));
        assert_eq!(rendered.matches("Content-Length").count(), 1);
        assert!(rendered.ends_with("{}"));
    }

    fn request(method: &str, path: &str, body: &str) -> GatewayRequest {
        GatewayRequest {
            method: method.to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: if body.is_empty() {
                Vec::new()
            } else {
                vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Content-Length".to_string(), body.len().to_string()),
                ]
            },
            body: body.to_string(),
        }
    }
}
