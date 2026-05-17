use std::env;
use std::fmt;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use igy6_agent_api::{classify_agent_intent, AgentIntentRequest, ACTION_REGISTRY};
use igy6_evidence_answer::build_evidence_answer_packet;
use igy6_read_only_api::summarize_manifest;
use igy6_retrieval_preview::{build_hydrated_chunk_search_result, build_retrieval_preview};
use postgres::{Client, NoTls};
use serde_json::Value;

pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8000";
pub const DEFAULT_FALLBACK_ORIGIN: &str = "http://legacy-api:8000";
pub const RUST_NATIVE_ROUTES: &[(&str, &str)] = &[
    ("GET", "/health/live"),
    ("GET", "/health/ready"),
    ("GET", "/rust-migration/status"),
    ("GET", "/agent/capabilities"),
    ("GET", "/analysis/hypotheses"),
    ("GET", "/analysis/hypotheses/{hypothesis_id}"),
    ("GET", "/analysis/patterns"),
    ("GET", "/analysis/patterns/{pattern_id}"),
    ("GET", "/analysis/predictions"),
    ("GET", "/analysis/predictions/{prediction_id}"),
    ("GET", "/analysis/recommendations"),
    ("GET", "/analysis/recommendations/{recommendation_id}"),
    ("POST", "/agent/intent"),
    ("POST", "/chat/retrieval-preview"),
    ("POST", "/chat/evidence-answer"),
    ("GET", "/approvals"),
    ("GET", "/approvals/{approval_id}"),
    ("POST", "/approvals"),
    ("GET", "/artifacts"),
    ("GET", "/artifacts/{artifact_id}"),
    ("GET", "/audit-events"),
    ("GET", "/audit-events/{audit_event_id}"),
    ("GET", "/collection-runs"),
    ("GET", "/collection-runs/{collection_run_id}"),
    ("GET", "/evidence/documents"),
    ("GET", "/evidence/documents/{document_id}"),
    ("GET", "/evidence/items"),
    ("GET", "/evidence/items/{evidence_item_id}"),
    ("GET", "/evidence/chunks"),
    ("GET", "/evidence/chunks/{chunk_id}"),
    ("GET", "/evidence/claims"),
    ("GET", "/evidence/claims/{claim_id}"),
    ("GET", "/feedback"),
    ("GET", "/feedback/{feedback_id}"),
    ("POST", "/feedback"),
    ("GET", "/memory/graph/schema"),
    ("GET", "/memory/vector/chunks"),
    ("GET", "/outcomes"),
    ("GET", "/outcomes/{outcome_id}"),
    ("POST", "/outcomes"),
    ("GET", "/reports"),
    ("GET", "/reports/{report_id}"),
    ("GET", "/settings/env"),
    ("GET", "/sources"),
    ("GET", "/sources/{source_id}"),
    ("GET", "/sources/{source_id}/permissions"),
    ("GET", "/work-items"),
    ("GET", "/work-items/{work_item_id}"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    EmptyRequest,
    MalformedRequest,
    InvalidContentLength,
    InvalidFallbackOrigin(String),
    MissingDatabaseUrl,
    Database(String),
    Validation(String),
    NotFound(String),
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
            Self::MissingDatabaseUrl => {
                write!(formatter, "DATABASE_URL is required for this route")
            }
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Validation(error) => write!(formatter, "validation error: {error}"),
            Self::NotFound(error) => write!(formatter, "{error}"),
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
    handle_gateway_request_with_db(request, manifest_content, fallback_origin, None)
}

pub fn handle_gateway_request_with_db(
    request: &GatewayRequest,
    manifest_content: Option<&str>,
    fallback_origin: &str,
    database_url: Option<&str>,
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
        ("POST", "/approvals") => approval_create_response(&request.body, database_url),
        ("POST", "/feedback") => feedback_create_response(&request.body, database_url),
        ("POST", "/outcomes") => outcome_create_response(&request.body, database_url),
        ("GET", "/settings/env") => {
            json_response(200, "OK", settings_env_status_json(), false)
        }
        ("GET", "/memory/vector/chunks") => {
            json_response(200, "OK", vector_collection_status_json(), false)
        }
        ("GET", "/memory/graph/schema") => {
            json_response(200, "OK", graph_schema_status_json(), false)
        }
        ("GET", _) => {
            if let Some(route) = db_read_route(&request.path) {
                db_read_response(route, database_url)
            } else {
                fallback_or_error(request, fallback_origin)
            }
        }
        _ => fallback_or_error(request, fallback_origin),
    }
}

fn fallback_or_error(request: &GatewayRequest, fallback_origin: &str) -> GatewayResponse {
    match build_fallback_proxy_plan(request, fallback_origin) {
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
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DbReadRoute {
    List {
        sql: &'static str,
    },
    Detail {
        id: String,
        sql: &'static str,
        _not_found_detail: &'static str,
    },
}

fn db_read_route(path: &str) -> Option<DbReadRoute> {
    match path {
        "/approvals" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, request_type, status, requested_by_actor_id, decided_by_actor_id, decision_reason, request_payload_json, decided_at, created_at, updated_at FROM approvals ORDER BY created_at DESC) t), '[]')",
        }),
        "/analysis/patterns" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, pattern_type, status, summary, evidence_ids, confidence, metadata_json, created_at, updated_at FROM patterns ORDER BY created_at DESC) t), '[]')",
        }),
        "/analysis/hypotheses" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, hypothesis_text, status, supporting_evidence_ids, missing_evidence_json, confidence, metadata_json, created_at, updated_at FROM hypotheses ORDER BY created_at DESC) t), '[]')",
        }),
        "/analysis/predictions" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, prediction_text, expected_result, disproof_condition, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM predictions ORDER BY created_at DESC) t), '[]')",
        }),
        "/analysis/recommendations" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, recommendation_text, risk_level, approval_required, expected_result, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM recommendations ORDER BY created_at DESC) t), '[]')",
        }),
        "/artifacts" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json, created_at, updated_at FROM raw_artifacts ORDER BY created_at DESC) t), '[]')",
        }),
        "/audit-events" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, created_at, actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json FROM audit_events ORDER BY created_at DESC) t), '[]')",
        }),
        "/collection-runs" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs ORDER BY created_at DESC) t), '[]')",
        }),
        "/evidence/documents" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json, created_at, updated_at FROM normalized_documents ORDER BY created_at DESC) t), '[]')",
        }),
        "/evidence/items" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json, created_at, updated_at FROM evidence_items ORDER BY created_at DESC) t), '[]')",
        }),
        "/evidence/chunks" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json, created_at, updated_at FROM chunks ORDER BY created_at DESC) t), '[]')",
        }),
        "/evidence/claims" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, claim_text, claim_type, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM claims ORDER BY created_at DESC) t), '[]')",
        }),
        "/feedback" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, target_type, target_id, label, actor_id, note, metadata_json, created_at, updated_at FROM feedback_events ORDER BY created_at DESC) t), '[]')",
        }),
        "/outcomes" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, target_type, target_id, outcome_status, summary, occurred_at, evidence_ids, metadata_json, created_at, updated_at FROM outcomes ORDER BY created_at DESC) t), '[]')",
        }),
        "/reports" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, title, report_type, status, requested_by_actor_id, artifact_path, metadata_json, created_at, updated_at FROM reports ORDER BY created_at DESC) t), '[]')",
        }),
        "/sources" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT s.id, s.name, s.source_type, s.location, s.owner_actor_id, s.sensitivity, s.trust_level, s.enabled, s.metadata_json, s.created_at, s.updated_at, COALESCE((SELECT json_agg(row_to_json(p)) FROM (SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id, created_at, updated_at FROM source_permissions WHERE source_id = s.id ORDER BY created_at ASC) p), '[]'::json) AS permissions FROM sources s ORDER BY s.created_at DESC) t), '[]')",
        }),
        "/work-items" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, work_type, status, requested_by_actor_id, payload_json, error_message, created_at, updated_at FROM work_items ORDER BY created_at DESC) t), '[]')",
        }),
        _ => db_detail_route(path),
    }
}

fn db_detail_route(path: &str) -> Option<DbReadRoute> {
    let segments = path.trim_matches('/').split('/').collect::<Vec<_>>();
    match segments.as_slice() {
        ["approvals", id] => Some(detail(
            id,
            "Approval not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, request_type, status, requested_by_actor_id, decided_by_actor_id, decision_reason, request_payload_json, decided_at, created_at, updated_at FROM approvals WHERE id = $1) t), '')",
        )),
        ["analysis", "patterns", id] => Some(detail(
            id,
            "Pattern not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, pattern_type, status, summary, evidence_ids, confidence, metadata_json, created_at, updated_at FROM patterns WHERE id = $1) t), '')",
        )),
        ["analysis", "hypotheses", id] => Some(detail(
            id,
            "Hypothesis not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, hypothesis_text, status, supporting_evidence_ids, missing_evidence_json, confidence, metadata_json, created_at, updated_at FROM hypotheses WHERE id = $1) t), '')",
        )),
        ["analysis", "predictions", id] => Some(detail(
            id,
            "Prediction not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, prediction_text, expected_result, disproof_condition, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM predictions WHERE id = $1) t), '')",
        )),
        ["analysis", "recommendations", id] => Some(detail(
            id,
            "Recommendation not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, recommendation_text, risk_level, approval_required, expected_result, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM recommendations WHERE id = $1) t), '')",
        )),
        ["artifacts", id] => Some(detail(
            id,
            "Artifact not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json, created_at, updated_at FROM raw_artifacts WHERE id = $1) t), '')",
        )),
        ["audit-events", id] => Some(detail(
            id,
            "Audit event not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, created_at, actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json FROM audit_events WHERE id = $1::integer) t), '')",
        )),
        ["collection-runs", id] => Some(detail(
            id,
            "Collection run not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs WHERE id = $1) t), '')",
        )),
        ["evidence", "documents", id] => Some(detail(
            id,
            "Document not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json, created_at, updated_at FROM normalized_documents WHERE id = $1) t), '')",
        )),
        ["evidence", "items", id] => Some(detail(
            id,
            "Evidence item not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json, created_at, updated_at FROM evidence_items WHERE id = $1) t), '')",
        )),
        ["evidence", "chunks", id] => Some(detail(
            id,
            "Chunk not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json, created_at, updated_at FROM chunks WHERE id = $1) t), '')",
        )),
        ["evidence", "claims", id] => Some(detail(
            id,
            "Claim not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, claim_text, claim_type, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM claims WHERE id = $1) t), '')",
        )),
        ["feedback", id] => Some(detail(
            id,
            "Feedback event not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, target_type, target_id, label, actor_id, note, metadata_json, created_at, updated_at FROM feedback_events WHERE id = $1) t), '')",
        )),
        ["outcomes", id] => Some(detail(
            id,
            "Outcome not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, target_type, target_id, outcome_status, summary, occurred_at, evidence_ids, metadata_json, created_at, updated_at FROM outcomes WHERE id = $1) t), '')",
        )),
        ["reports", id] => Some(detail(
            id,
            "Report not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, title, report_type, status, requested_by_actor_id, artifact_path, metadata_json, created_at, updated_at FROM reports WHERE id = $1) t), '')",
        )),
        ["sources", id] => Some(detail(
            id,
            "Source not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT s.id, s.name, s.source_type, s.location, s.owner_actor_id, s.sensitivity, s.trust_level, s.enabled, s.metadata_json, s.created_at, s.updated_at, COALESCE((SELECT json_agg(row_to_json(p)) FROM (SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id, created_at, updated_at FROM source_permissions WHERE source_id = s.id ORDER BY created_at ASC) p), '[]'::json) AS permissions FROM sources s WHERE s.id = $1) t), '')",
        )),
        ["sources", source_id, "permissions"] => Some(detail(
            source_id,
            "Source not found",
            "SELECT CASE WHEN EXISTS (SELECT 1 FROM sources WHERE id = $1) THEN COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id, created_at, updated_at FROM source_permissions WHERE source_id = $1 ORDER BY created_at ASC) t), '[]') ELSE '' END",
        )),
        ["work-items", id] => Some(detail(
            id,
            "Work item not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, work_type, status, requested_by_actor_id, payload_json, error_message, created_at, updated_at FROM work_items WHERE id = $1) t), '')",
        )),
        _ => None,
    }
}

fn detail(id: &str, not_found_detail: &'static str, sql: &'static str) -> DbReadRoute {
    DbReadRoute::Detail {
        id: id.to_string(),
        sql,
        _not_found_detail: not_found_detail,
    }
}

fn db_read_response(route: DbReadRoute, database_url: Option<&str>) -> GatewayResponse {
    match query_db_route(route, database_url) {
        Ok(Some(body)) => json_response(200, "OK", body, false),
        Ok(None) => json_response(
            404,
            "Not Found",
            "{\"detail\":\"Not found\"}".to_string(),
            false,
        ),
        Err(GatewayError::MissingDatabaseUrl) => json_response(
            503,
            "Service Unavailable",
            "{\"detail\":\"DATABASE_URL is required for Rust DB route\"}".to_string(),
            false,
        ),
        Err(error) => json_response(
            502,
            "Bad Gateway",
            format!("{{\"detail\":\"{}\"}}", escape_json(&error.to_string())),
            false,
        ),
    }
}

fn query_db_route(
    route: DbReadRoute,
    database_url: Option<&str>,
) -> Result<Option<String>, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    match route {
        DbReadRoute::List { sql } => client
            .query_one(sql, &[])
            .map(|row| Some(row.get::<_, String>(0)))
            .map_err(|error| GatewayError::Database(error.to_string())),
        DbReadRoute::Detail { id, sql, .. } => {
            let body = client
                .query_one(sql, &[&id])
                .map(|row| row.get::<_, String>(0))
                .map_err(|error| GatewayError::Database(error.to_string()))?;
            if body.is_empty() {
                Ok(None)
            } else {
                Ok(Some(body))
            }
        }
    }
}

fn approval_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    match create_approval(body, database_url) {
        Ok(response_body) => json_response(201, "Created", response_body, false),
        Err(GatewayError::Validation(message)) => json_response(
            422,
            "Unprocessable Entity",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
            false,
        ),
        Err(GatewayError::MissingDatabaseUrl) => json_response(
            503,
            "Service Unavailable",
            "{\"detail\":\"DATABASE_URL is required for Rust DB route\"}".to_string(),
            false,
        ),
        Err(error) => json_response(
            502,
            "Bad Gateway",
            format!("{{\"detail\":\"{}\"}}", escape_json(&error.to_string())),
            false,
        ),
    }
}

fn write_route_response(result: Result<String, GatewayError>) -> GatewayResponse {
    match result {
        Ok(response_body) => json_response(201, "Created", response_body, false),
        Err(GatewayError::Validation(message)) => json_response(
            422,
            "Unprocessable Entity",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
            false,
        ),
        Err(GatewayError::NotFound(message)) => json_response(
            404,
            "Not Found",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
            false,
        ),
        Err(GatewayError::MissingDatabaseUrl) => json_response(
            503,
            "Service Unavailable",
            "{\"detail\":\"DATABASE_URL is required for Rust DB route\"}".to_string(),
            false,
        ),
        Err(error) => json_response(
            502,
            "Bad Gateway",
            format!("{{\"detail\":\"{}\"}}", escape_json(&error.to_string())),
            false,
        ),
    }
}

fn feedback_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_feedback(body, database_url))
}

fn outcome_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_outcome(body, database_url))
}

fn create_approval(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_approval_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let approval_id = generated_record_id("approval");
    transaction
        .execute(
            "INSERT INTO approvals (id, request_type, status, requested_by_actor_id, decided_by_actor_id, decision_reason, request_payload_json, decided_at) VALUES ($1, $2, 'pending', $3, NULL, NULL, $4::jsonb, NULL)",
            &[
                &approval_id,
                &payload.request_type,
                &payload.requested_by_actor_id,
                &payload.request_payload_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "request_type": payload.request_type,
        "status": "pending"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'approval.requested', 'pending', 'approval', $2, NULL, $3::jsonb)",
            &[&payload.requested_by_actor_id, &approval_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, request_type, status, requested_by_actor_id, decided_by_actor_id, decision_reason, request_payload_json, decided_at, created_at, updated_at FROM approvals WHERE id = $1) t",
            &[&approval_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_feedback(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_feedback_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let feedback_id = generated_record_id("feedback");

    if payload.target_type == "source" {
        if let Some((_, _)) = source_trust_update(&payload.label) {
            let exists = transaction
                .query_one(
                    "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
                    &[&payload.target_id],
                )
                .map(|row| row.get::<_, bool>(0))
                .map_err(|error| GatewayError::Database(error.to_string()))?;
            if !exists {
                return Err(GatewayError::NotFound("Source not found".to_string()));
            }
        }
    }

    transaction
        .execute(
            "INSERT INTO feedback_events (id, target_type, target_id, label, actor_id, note, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb)",
            &[
                &feedback_id,
                &payload.target_type,
                &payload.target_id,
                &payload.label,
                &payload.actor_id,
                &payload.note,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let feedback_audit_details = serde_json::json!({
        "feedback_id": feedback_id,
        "label": payload.label
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'feedback.created', 'recorded', $2, $3, NULL, $4::jsonb)",
            &[
                &payload.actor_id,
                &payload.target_type,
                &payload.target_id,
                &feedback_audit_details,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    if payload.target_type == "source" {
        if let Some((trust_level, enabled)) = source_trust_update(&payload.label) {
            let row = transaction
                .query_one(
                    "SELECT trust_level, enabled FROM sources WHERE id = $1",
                    &[&payload.target_id],
                )
                .map_err(|error| GatewayError::Database(error.to_string()))?;
            let previous_trust_level = row.get::<_, String>(0);
            let previous_enabled = row.get::<_, bool>(1);
            transaction
                .execute(
                    "UPDATE sources SET trust_level = $1, enabled = $2 WHERE id = $3",
                    &[&trust_level, &enabled, &payload.target_id],
                )
                .map_err(|error| GatewayError::Database(error.to_string()))?;
            let details = serde_json::json!({
                "feedback_id": feedback_id,
                "previous_trust_level": previous_trust_level,
                "new_trust_level": trust_level,
                "previous_enabled": previous_enabled,
                "new_enabled": enabled
            });
            transaction
                .execute(
                    "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'source.trust_feedback_applied', $2, 'source', $3, NULL, $4::jsonb)",
                    &[&payload.actor_id, &payload.label, &payload.target_id, &details],
                )
                .map_err(|error| GatewayError::Database(error.to_string()))?;
        }
    }

    if is_weak_feedback_label(&payload.label) && payload.target_type != "source" {
        let improvement_id = generated_record_id("improvement");
        let target_area = improvement_target_area(&payload.target_type);
        let objective = format!(
            "Investigate {} feedback for {} {}.",
            payload.label, payload.target_type, payload.target_id
        );
        let improvement_metadata = serde_json::json!({
            "generated_by": "DIFF-068",
            "feedback_id": feedback_id,
            "feedback_label": payload.label,
            "target_type": payload.target_type,
            "target_id": payload.target_id,
            "note": payload.note
        });
        transaction
            .execute(
                "INSERT INTO improvement_items (id, target_area, status, objective, proposed_by_actor_id, priority, metadata_json) VALUES ($1, $2, 'proposed', $3, $4, 'normal', $5::jsonb)",
                &[
                    &improvement_id,
                    &target_area,
                    &objective,
                    &payload.actor_id,
                    &improvement_metadata,
                ],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        let details = serde_json::json!({
            "target_area": target_area,
            "priority": "normal",
            "source_feedback_id": feedback_id
        });
        transaction
            .execute(
                "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'improvement_item.created', 'proposed', 'improvement_item', $2, $3, $4::jsonb)",
                &[&payload.actor_id, &improvement_id, &feedback_id, &details],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    }

    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, target_type, target_id, label, actor_id, note, metadata_json, created_at, updated_at FROM feedback_events WHERE id = $1) t",
            &[&feedback_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_outcome(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_outcome_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let target_table = outcome_target_table(&payload.target_type)?;
    let target_exists = transaction
        .query_one(
            &format!("SELECT EXISTS (SELECT 1 FROM {target_table} WHERE id = $1)"),
            &[&payload.target_id],
        )
        .map(|row| row.get::<_, bool>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if !target_exists {
        return Err(GatewayError::Validation(
            "Outcome target record does not exist".to_string(),
        ));
    }
    for evidence_id in &payload.evidence_ids {
        let exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM evidence_items WHERE id = $1)",
                &[evidence_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !exists {
            return Err(GatewayError::Validation(
                "Outcome records must reference existing evidence items".to_string(),
            ));
        }
    }

    let outcome_id = generated_record_id("outcome");
    let evidence_ids_json = Value::Array(
        payload
            .evidence_ids
            .iter()
            .map(|id| Value::String(id.clone()))
            .collect(),
    );
    transaction
        .execute(
            "INSERT INTO outcomes (id, target_type, target_id, outcome_status, summary, occurred_at, evidence_ids, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::jsonb, $8::jsonb)",
            &[
                &outcome_id,
                &payload.target_type,
                &payload.target_id,
                &payload.outcome_status,
                &payload.summary,
                &payload.occurred_at,
                &evidence_ids_json,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "outcome_id": outcome_id,
        "outcome_status": payload.outcome_status
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ('local-owner', 'outcome.created', 'recorded', $1, $2, NULL, $3::jsonb)",
            &[&payload.target_type, &payload.target_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    let previous_status = transaction
        .query_one(
            &format!("SELECT status FROM {target_table} WHERE id = $1"),
            &[&payload.target_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let new_status = outcome_target_status(&payload.outcome_status);
    let metadata_patch = serde_json::json!({
        "latest_outcome_id": outcome_id,
        "latest_outcome_status": payload.outcome_status
    });
    transaction
        .execute(
            &format!("UPDATE {target_table} SET status = $1, metadata_json = COALESCE(metadata_json, '{{}}'::jsonb) || $2::jsonb WHERE id = $3"),
            &[&new_status, &metadata_patch, &payload.target_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let target_audit_details = serde_json::json!({
        "previous_status": previous_status,
        "new_status": new_status,
        "outcome_status": payload.outcome_status
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ('local-owner', 'outcome.target_updated', $1, $2, $3, $4, $5::jsonb)",
            &[
                &new_status,
                &payload.target_type,
                &payload.target_id,
                &outcome_id,
                &target_audit_details,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, target_type, target_id, outcome_status, summary, occurred_at, evidence_ids, metadata_json, created_at, updated_at FROM outcomes WHERE id = $1) t",
            &[&outcome_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

struct ApprovalCreatePayload {
    request_type: String,
    requested_by_actor_id: String,
    request_payload_json: Value,
}

struct FeedbackCreatePayload {
    target_type: String,
    target_id: String,
    label: String,
    actor_id: String,
    note: Option<String>,
    metadata_json: Value,
}

struct OutcomeCreatePayload {
    target_type: String,
    target_id: String,
    outcome_status: String,
    summary: Option<String>,
    occurred_at: Option<String>,
    evidence_ids: Vec<String>,
    metadata_json: Value,
}

fn parse_approval_create(body: &str) -> Result<ApprovalCreatePayload, GatewayError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| GatewayError::Validation("Request body must be valid JSON.".to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        GatewayError::Validation("Approval request body must be a JSON object.".to_string())
    })?;
    let request_type = object
        .get("request_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if request_type.is_empty() {
        return Err(GatewayError::Validation(
            "request_type is required.".to_string(),
        ));
    }
    if request_type.len() > 64 {
        return Err(GatewayError::Validation(
            "request_type must be 64 characters or fewer.".to_string(),
        ));
    }
    let requested_by_actor_id = object
        .get("requested_by_actor_id")
        .and_then(Value::as_str)
        .unwrap_or("local-owner")
        .trim()
        .to_string();
    if requested_by_actor_id.is_empty() {
        return Err(GatewayError::Validation(
            "requested_by_actor_id must not be empty.".to_string(),
        ));
    }
    let request_payload_json = object
        .get("request_payload_json")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    if !request_payload_json.is_object() {
        return Err(GatewayError::Validation(
            "request_payload_json must be a JSON object.".to_string(),
        ));
    }
    Ok(ApprovalCreatePayload {
        request_type,
        requested_by_actor_id,
        request_payload_json,
    })
}

fn parse_feedback_create(body: &str) -> Result<FeedbackCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Feedback request body")?;
    let target_type = required_string_field(&object, "target_type", 64)?;
    if !is_feedback_target_type(&target_type) {
        return Err(GatewayError::Validation(format!(
            "Unknown feedback target type: {target_type}"
        )));
    }
    let target_id = required_string_field(&object, "target_id", 36)?;
    let label = required_string_field(&object, "label", 64)?;
    if !is_feedback_label(&label) {
        return Err(GatewayError::Validation(format!(
            "Unknown feedback label: {label}"
        )));
    }
    let actor_id = optional_string_field(&object, "actor_id", "local-owner")?;
    let note = optional_nullable_string_field(&object, "note")?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    Ok(FeedbackCreatePayload {
        target_type,
        target_id,
        label,
        actor_id,
        note,
        metadata_json,
    })
}

fn parse_outcome_create(body: &str) -> Result<OutcomeCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Outcome request body")?;
    let target_type = required_string_field(&object, "target_type", 64)?;
    outcome_target_table(&target_type)?;
    let target_id = required_string_field(&object, "target_id", 36)?;
    let outcome_status = required_string_field(&object, "outcome_status", 64)?;
    if !is_outcome_status(&outcome_status) {
        return Err(GatewayError::Validation(format!(
            "Unknown outcome status: {outcome_status}"
        )));
    }
    let summary = optional_nullable_string_field(&object, "summary")?;
    let occurred_at = optional_nullable_string_field(&object, "occurred_at")?;
    let evidence_ids = optional_string_array_field(&object, "evidence_ids")?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    Ok(OutcomeCreatePayload {
        target_type,
        target_id,
        outcome_status,
        summary,
        occurred_at,
        evidence_ids,
        metadata_json,
    })
}

fn parse_json_object(
    body: &str,
    description: &str,
) -> Result<serde_json::Map<String, Value>, GatewayError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| GatewayError::Validation("Request body must be valid JSON.".to_string()))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| GatewayError::Validation(format!("{description} must be a JSON object.")))
}

fn required_string_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    max_len: usize,
) -> Result<String, GatewayError> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(GatewayError::Validation(format!("{key} is required.")));
    }
    if value.len() > max_len {
        return Err(GatewayError::Validation(format!(
            "{key} must be {max_len} characters or fewer."
        )));
    }
    Ok(value)
}

fn optional_string_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    default: &str,
) -> Result<String, GatewayError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(default.to_string()),
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        Some(Value::String(_)) => Err(GatewayError::Validation(format!(
            "{key} must not be empty."
        ))),
        Some(_) => Err(GatewayError::Validation(format!("{key} must be a string."))),
    }
}

fn optional_nullable_string_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, GatewayError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.to_string())),
        Some(_) => Err(GatewayError::Validation(format!(
            "{key} must be a string or null."
        ))),
    }
}

fn optional_object_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Value, GatewayError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(serde_json::json!({})),
        Some(value) if value.is_object() => Ok(value.clone()),
        Some(_) => Err(GatewayError::Validation(format!(
            "{key} must be a JSON object."
        ))),
    }
}

fn optional_string_array_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>, GatewayError> {
    let Some(value) = object.get(key) else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(GatewayError::Validation(format!(
            "{key} must be an array of strings."
        )));
    };
    let mut result = Vec::new();
    for item in items {
        let Some(item) = item.as_str() else {
            return Err(GatewayError::Validation(format!(
                "{key} must be an array of strings."
            )));
        };
        let item = item.trim();
        if item.is_empty() {
            return Err(GatewayError::Validation(format!(
                "{key} must not contain empty IDs."
            )));
        }
        if !result.iter().any(|existing| existing == item) {
            result.push(item.to_string());
        }
    }
    Ok(result)
}

fn is_feedback_target_type(value: &str) -> bool {
    matches!(
        value,
        "source"
            | "document"
            | "evidence_item"
            | "claim"
            | "pattern"
            | "hypothesis"
            | "prediction"
            | "recommendation"
            | "report"
            | "work_item"
    )
}

fn is_feedback_label(value: &str) -> bool {
    matches!(
        value,
        "useful"
            | "not_useful"
            | "wrong"
            | "verified"
            | "incomplete"
            | "noisy"
            | "trusted"
            | "rejected"
    )
}

fn source_trust_update(label: &str) -> Option<(&'static str, bool)> {
    match label {
        "trusted" => Some(("trusted", true)),
        "noisy" => Some(("noisy", true)),
        "rejected" => Some(("rejected", false)),
        _ => None,
    }
}

fn is_weak_feedback_label(value: &str) -> bool {
    matches!(value, "not_useful" | "wrong" | "incomplete" | "rejected")
}

fn improvement_target_area(target_type: &str) -> &'static str {
    match target_type {
        "document" => "parsing",
        "evidence_item" => "retrieval",
        "prediction" => "prediction",
        "report" => "reporting",
        "work_item" => "safety",
        _ => "reasoning",
    }
}

fn outcome_target_table(target_type: &str) -> Result<&'static str, GatewayError> {
    match target_type {
        "prediction" => Ok("predictions"),
        "recommendation" => Ok("recommendations"),
        "work_item" => Ok("work_items"),
        "hypothesis" => Ok("hypotheses"),
        "pattern" => Ok("patterns"),
        "report" => Ok("reports"),
        _ => Err(GatewayError::Validation(format!(
            "Unknown outcome target type: {target_type}"
        ))),
    }
}

fn is_outcome_status(value: &str) -> bool {
    matches!(
        value,
        "correct"
            | "wrong"
            | "useful"
            | "not_useful"
            | "partial"
            | "inconclusive"
            | "confirmed"
            | "disconfirmed"
    )
}

fn outcome_target_status(outcome_status: &str) -> &'static str {
    match outcome_status {
        "correct" => "correct",
        "useful" => "useful",
        "confirmed" => "confirmed",
        "wrong" => "wrong",
        "not_useful" => "not_useful",
        "disconfirmed" => "disconfirmed",
        "partial" => "partial",
        "inconclusive" => "inconclusive",
        _ => "reviewed",
    }
}

fn generated_record_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos:x}")
}

fn postgres_client_url(database_url: &str) -> String {
    for prefix in ["postgresql+", "postgres+"] {
        if let Some(driver_url) = database_url.strip_prefix(prefix) {
            if let Some((_, rest)) = driver_url.split_once("://") {
                return format!("{}://{}", prefix.trim_end_matches('+'), rest);
            }
        }
    }
    database_url.to_string()
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
    "igy6-gateway\n\nUsage:\n  igy6-gateway [--bind 0.0.0.0:8000] [--fallback http://legacy-api:8000]\n  igy6-gateway --help\n\nRoutes:\n  GET /health/live\n  GET /health/ready\n  GET /rust-migration/status\n  GET /agent/capabilities\n  POST /agent/intent\n  POST /chat/retrieval-preview\n  POST /chat/evidence-answer\n  POST /approvals\n  POST /feedback\n  POST /outcomes\n  GET /settings/env\n  GET /memory/vector/chunks\n  GET /memory/graph/schema\n  GET /sources and selected source detail/permission reads\n  GET /approvals and approval detail reads\n  GET /work-items and work item detail reads\n  GET /reports and report detail reads\n  GET /evidence documents/items/chunks/claims and detail reads\n  GET /analysis patterns/hypotheses/predictions/recommendations and detail reads\n  GET /artifacts, /audit-events, /collection-runs, /feedback, /outcomes and detail reads\n\nUnsupported routes are proxied to the configured FastAPI fallback at runtime.\n"
}

fn settings_env_status_json() -> String {
    const GROUPS: &[(&str, &str)] = &[
        ("app", "App"),
        ("postgres", "PostgreSQL"),
        ("redis", "Redis"),
        ("qdrant", "Qdrant"),
        ("neo4j", "Neo4j"),
        ("storage", "Storage"),
        ("policy", "Policy"),
    ];
    const SETTINGS: &[(&str, &str, &str, bool, bool)] = &[
        (
            "APP_ENV",
            "app",
            "Application environment label.",
            false,
            true,
        ),
        ("APP_HOST", "app", "API host binding.", false, true),
        ("APP_PORT", "app", "API port.", false, true),
        (
            "DATABASE_URL",
            "postgres",
            "PostgreSQL connection URL.",
            true,
            true,
        ),
        ("REDIS_URL", "redis", "Redis connection URL.", true, true),
        ("QDRANT_URL", "qdrant", "Qdrant service URL.", false, true),
        (
            "QDRANT_CHUNK_COLLECTION",
            "qdrant",
            "Qdrant collection for chunk vectors.",
            false,
            true,
        ),
        (
            "QDRANT_CHUNK_VECTOR_SIZE",
            "qdrant",
            "Deterministic chunk vector size.",
            false,
            true,
        ),
        (
            "NEO4J_URI",
            "neo4j",
            "Neo4j Bolt URI used by the API.",
            false,
            true,
        ),
        ("NEO4J_USER", "neo4j", "Neo4j username.", false, true),
        ("NEO4J_PASSWORD", "neo4j", "Neo4j password.", true, true),
        (
            "ARTIFACT_STORE_PATH",
            "storage",
            "Container path for content-addressed artifacts.",
            false,
            true,
        ),
        (
            "EXPORT_STORE_PATH",
            "storage",
            "Container path for report/export output.",
            false,
            true,
        ),
        (
            "ENV_FILE_PATH",
            "storage",
            "Controlled container path to the mounted local .env file.",
            false,
            true,
        ),
        (
            "ENV_BACKUP_DIR",
            "storage",
            "Controlled backup directory for .env backups.",
            false,
            true,
        ),
        (
            "IGY6_DATA_ROOT",
            "storage",
            "Host-side folder for local IGY6 runtime data.",
            false,
            true,
        ),
        (
            "EXTERNAL_MODEL_POLICY_DEFAULT",
            "policy",
            "Default external model policy.",
            false,
            true,
        ),
        (
            "SINGLE_USER_MODE",
            "policy",
            "Local single-user mode toggle.",
            false,
            true,
        ),
        (
            "APPROVAL_REQUIRED_DEFAULT",
            "policy",
            "Default approval-required toggle.",
            false,
            true,
        ),
    ];

    let groups = GROUPS
        .iter()
        .map(|(key, label)| format!("{{\"key\":\"{}\",\"label\":\"{}\"}}", key, label))
        .collect::<Vec<_>>()
        .join(",");
    let settings = SETTINGS
        .iter()
        .map(|(key, group, description, secret, restart_required)| {
            let value = env::var(key).ok();
            let has_value = value.as_ref().is_some_and(|value| !value.is_empty());
            let value_json = if *secret {
                "null".to_string()
            } else {
                option_string_json(value.as_deref())
            };
            let masked_value = if *secret && has_value {
                "\"********\"".to_string()
            } else {
                "\"\"".to_string()
            };
            format!(
                "{{\"key\":\"{}\",\"group\":\"{}\",\"group_label\":\"{}\",\"description\":\"{}\",\"value\":{},\"masked_value\":{},\"has_value\":{},\"secret\":{},\"read_only\":true,\"restart_required\":{},\"source\":\"process_env\"}}",
                escape_json(key),
                escape_json(group),
                escape_json(group_label(group)),
                escape_json(description),
                value_json,
                masked_value,
                has_value,
                secret,
                restart_required
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let env_path =
        env::var("ENV_FILE_PATH").unwrap_or_else(|_| "/workspace/project/.env".to_string());
    let backup_dir =
        env::var("ENV_BACKUP_DIR").unwrap_or_else(|_| "/workspace/storage/env_backups".to_string());
    format!(
        "{{\"file_status\":{{\"path\":\"{}\",\"backup_dir\":\"{}\",\"exists\":false,\"writable\":false,\"unknown_key_count\":0,\"output_format\":\"process_env_redacted\"}},\"groups\":[{}],\"settings\":[{}],\"unmanaged\":[],\"warnings\":[\"Rust gateway settings status uses process environment metadata only and does not read .env contents.\"]}}",
        escape_json(&env_path),
        escape_json(&backup_dir),
        groups,
        settings
    )
}

fn group_label(group: &str) -> &str {
    match group {
        "app" => "App",
        "postgres" => "PostgreSQL",
        "redis" => "Redis",
        "qdrant" => "Qdrant",
        "neo4j" => "Neo4j",
        "storage" => "Storage",
        "policy" => "Policy",
        _ => group,
    }
}

fn vector_collection_status_json() -> String {
    let collection_name =
        env::var("QDRANT_CHUNK_COLLECTION").unwrap_or_else(|_| "igy6_chunks".to_string());
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://qdrant:6333".to_string());
    let reachability = tcp_reachability_from_url(&qdrant_url);
    format!(
        "{{\"collection_name\":\"{}\",\"exists\":false,\"detail\":{{\"rust_gateway_status\":\"read_only_status\",\"configured_url\":\"{}\",\"tcp_reachable\":{},\"collection_existence_verified\":false,\"note\":\"Rust gateway does not create, mutate, or inspect Qdrant collections in DIFF-108.\"}}}}",
        escape_json(&collection_name),
        escape_json(&redact_url(&qdrant_url)),
        reachability
    )
}

fn graph_schema_status_json() -> String {
    let neo4j_uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let reachability = tcp_reachability_from_url(&neo4j_uri);
    format!(
        "{{\"constraints\":[{{\"rust_gateway_status\":\"read_only_status\",\"configured_uri\":\"{}\",\"tcp_reachable\":{},\"constraints_verified\":false,\"note\":\"Rust gateway does not query, create, or mutate Neo4j constraints in DIFF-108.\"}}]}}",
        escape_json(&redact_url(&neo4j_uri)),
        reachability
    )
}

fn tcp_reachability_from_url(url: &str) -> bool {
    let Some((host, port)) = host_port_from_url(url) else {
        return false;
    };
    let Ok(addrs) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    for addr in addrs {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
            return true;
        }
    }
    false
}

fn host_port_from_url(url: &str) -> Option<(String, u16)> {
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = without_scheme.split('/').next()?.split('@').next_back()?;
    let (host, port) = authority.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    if host.trim().is_empty() {
        return None;
    }
    Some((host.to_string(), port))
}

fn redact_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    if let Some((_, after_auth)) = rest.rsplit_once('@') {
        return format!("{scheme}://***@{after_auth}");
    }
    url.to_string()
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
    fn db_read_routes_are_rust_native_and_require_database_url() {
        for (method, path) in [
            ("GET", "/sources"),
            ("GET", "/sources/source-1"),
            ("GET", "/sources/source-1/permissions"),
            ("GET", "/analysis/patterns"),
            ("GET", "/analysis/patterns/pattern-1"),
            ("GET", "/analysis/hypotheses"),
            ("GET", "/analysis/hypotheses/hypothesis-1"),
            ("GET", "/analysis/predictions"),
            ("GET", "/analysis/predictions/prediction-1"),
            ("GET", "/analysis/recommendations"),
            ("GET", "/analysis/recommendations/recommendation-1"),
            ("GET", "/approvals"),
            ("GET", "/approvals/approval-1"),
            ("GET", "/artifacts"),
            ("GET", "/artifacts/artifact-1"),
            ("GET", "/audit-events"),
            ("GET", "/audit-events/1"),
            ("GET", "/collection-runs"),
            ("GET", "/collection-runs/run-1"),
            ("GET", "/work-items"),
            ("GET", "/work-items/work-1"),
            ("GET", "/reports"),
            ("GET", "/reports/report-1"),
            ("GET", "/feedback"),
            ("GET", "/feedback/feedback-1"),
            ("GET", "/outcomes"),
            ("GET", "/outcomes/outcome-1"),
            ("GET", "/evidence/documents"),
            ("GET", "/evidence/documents/document-1"),
            ("GET", "/evidence/items"),
            ("GET", "/evidence/items/evidence-1"),
            ("GET", "/evidence/chunks"),
            ("GET", "/evidence/chunks/chunk-1"),
            ("GET", "/evidence/claims"),
            ("GET", "/evidence/claims/claim-1"),
        ] {
            let response = handle_gateway_request_with_db(
                &request(method, path, ""),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains("DATABASE_URL"), "{path}");
        }
    }

    #[test]
    fn db_read_routes_report_database_errors_without_fallback() {
        let response = handle_gateway_request_with_db(
            &request("GET", "/sources", ""),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            Some("not-a-postgres-url"),
        );
        assert_eq!(response.status_code, 502);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("database error"));
    }

    #[test]
    fn status_config_routes_are_rust_native_without_fallback() {
        for (path, expected) in [
            ("/settings/env", "\"file_status\""),
            ("/memory/vector/chunks", "\"collection_name\""),
            ("/memory/graph/schema", "\"constraints\""),
        ] {
            let response = handle_gateway_request_with_db(
                &request("GET", path, ""),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 200, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains(expected), "{path}");
        }
    }

    #[test]
    fn settings_env_status_redacts_secrets_and_does_not_read_env_file() {
        let response = handle_gateway_request_with_db(
            &request("GET", "/settings/env", ""),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 200);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("\"key\":\"DATABASE_URL\""));
        assert!(response.body.contains("\"value\":null"));
        assert!(response.body.contains("does not read .env contents"));
        assert!(!response.body.contains("change-me-local-only"));
    }

    #[test]
    fn approval_create_is_rust_native_and_requires_database_url() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/approvals",
                r#"{"request_type":"agent_action","request_payload_json":{"action_name":"start_stack"}}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));
    }

    #[test]
    fn approval_create_validation_rejects_malformed_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"request_type":""}"#,
            r#"{"request_type":"agent_action","requested_by_actor_id":""}"#,
            r#"{"request_type":"agent_action","request_payload_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/approvals", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn feedback_and_outcome_writes_are_rust_native_and_require_database_url() {
        for (path, body) in [
            (
                "/feedback",
                r#"{"target_type":"prediction","target_id":"prediction-1","label":"wrong","note":"bad answer"}"#,
            ),
            (
                "/outcomes",
                r#"{"target_type":"prediction","target_id":"prediction-1","outcome_status":"wrong","summary":"missed"}"#,
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains("DATABASE_URL"), "{path}");
        }
    }

    #[test]
    fn feedback_write_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"target_type":"bogus","target_id":"x","label":"wrong"}"#,
            r#"{"target_type":"prediction","target_id":"","label":"wrong"}"#,
            r#"{"target_type":"prediction","target_id":"x","label":"bogus"}"#,
            r#"{"target_type":"prediction","target_id":"x","label":"wrong","metadata_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/feedback", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn outcome_write_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"target_type":"source","target_id":"x","outcome_status":"wrong"}"#,
            r#"{"target_type":"prediction","target_id":"","outcome_status":"wrong"}"#,
            r#"{"target_type":"prediction","target_id":"x","outcome_status":"bogus"}"#,
            r#"{"target_type":"prediction","target_id":"x","outcome_status":"wrong","evidence_ids":[""]}"#,
            r#"{"target_type":"prediction","target_id":"x","outcome_status":"wrong","metadata_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/outcomes", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn rust_native_route_registry_covers_db_read_batch() {
        for expected in [
            ("GET", "/sources"),
            ("GET", "/sources/{source_id}"),
            ("GET", "/sources/{source_id}/permissions"),
            ("GET", "/analysis/patterns"),
            ("GET", "/analysis/patterns/{pattern_id}"),
            ("GET", "/analysis/hypotheses"),
            ("GET", "/analysis/hypotheses/{hypothesis_id}"),
            ("GET", "/analysis/predictions"),
            ("GET", "/analysis/predictions/{prediction_id}"),
            ("GET", "/analysis/recommendations"),
            ("GET", "/analysis/recommendations/{recommendation_id}"),
            ("GET", "/approvals"),
            ("GET", "/approvals/{approval_id}"),
            ("POST", "/approvals"),
            ("GET", "/artifacts"),
            ("GET", "/artifacts/{artifact_id}"),
            ("GET", "/audit-events"),
            ("GET", "/audit-events/{audit_event_id}"),
            ("GET", "/collection-runs"),
            ("GET", "/collection-runs/{collection_run_id}"),
            ("GET", "/work-items"),
            ("GET", "/work-items/{work_item_id}"),
            ("GET", "/reports"),
            ("GET", "/reports/{report_id}"),
            ("GET", "/feedback"),
            ("GET", "/feedback/{feedback_id}"),
            ("POST", "/feedback"),
            ("GET", "/memory/graph/schema"),
            ("GET", "/memory/vector/chunks"),
            ("GET", "/outcomes"),
            ("GET", "/outcomes/{outcome_id}"),
            ("POST", "/outcomes"),
            ("GET", "/settings/env"),
            ("GET", "/evidence/documents"),
            ("GET", "/evidence/documents/{document_id}"),
            ("GET", "/evidence/items"),
            ("GET", "/evidence/items/{evidence_item_id}"),
            ("GET", "/evidence/chunks"),
            ("GET", "/evidence/chunks/{chunk_id}"),
            ("GET", "/evidence/claims"),
            ("GET", "/evidence/claims/{claim_id}"),
        ] {
            assert!(RUST_NATIVE_ROUTES.contains(&expected), "{expected:?}");
        }
    }

    #[test]
    fn postgres_client_url_accepts_sqlalchemy_driver_urls() {
        assert_eq!(
            postgres_client_url("postgresql+psycopg://user:pass@postgres:5432/db"),
            "postgresql://user:pass@postgres:5432/db"
        );
        assert_eq!(
            postgres_client_url("postgres://user:pass@postgres:5432/db"),
            "postgres://user:pass@postgres:5432/db"
        );
    }

    #[test]
    fn status_url_helpers_redact_credentials_and_parse_ports() {
        assert_eq!(
            redact_url("postgresql://user:secret@postgres:5432/db"),
            "postgresql://***@postgres:5432/db"
        );
        assert_eq!(
            host_port_from_url("bolt://neo4j:7687"),
            Some(("neo4j".to_string(), 7687))
        );
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
