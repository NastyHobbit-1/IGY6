use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use igy6_agent_api::{
    action_definition, classify_agent_intent, AgentActionDefinition, AgentIntentRequest,
    ACTION_REGISTRY,
};
use igy6_artifacts::ArtifactStore;
use igy6_evidence_answer::build_evidence_answer_packet;
use igy6_host_bridge::{allowed_action as host_bridge_allowed_action, redact_output};
use igy6_read_only_api::summarize_manifest;
use igy6_retrieval_preview::{build_hydrated_chunk_search_result, build_retrieval_preview};
use postgres::{Client, NoTls};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8000";
pub const DEFAULT_FALLBACK_ORIGIN: &str = "http://legacy-api:8000";
pub const RUST_NATIVE_ROUTES: &[(&str, &str)] = &[
    ("GET", "/health/live"),
    ("GET", "/health/ready"),
    ("GET", "/rust-migration/status"),
    ("GET", "/agent/capabilities"),
    ("POST", "/agent/actions/"),
    ("POST", "/agent/actions/{action_name}/execute"),
    ("GET", "/analysis/hypotheses"),
    ("GET", "/analysis/hypotheses/{hypothesis_id}"),
    ("GET", "/analysis/patterns"),
    ("GET", "/analysis/patterns/{pattern_id}"),
    ("POST", "/analysis/patterns"),
    ("POST", "/analysis/patterns/{pattern_id}/review"),
    ("POST", "/analysis/patterns/detect-baseline"),
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
    ("POST", "/approvals/{approval_id}/decision"),
    ("GET", "/artifacts"),
    ("GET", "/artifacts/{artifact_id}"),
    ("GET", "/audit-events"),
    ("GET", "/audit-events/{audit_event_id}"),
    ("GET", "/collection-runs"),
    ("GET", "/collection-runs/{collection_run_id}"),
    ("POST", "/collection-runs/dry-run"),
    ("POST", "/collection-runs/manual-upload"),
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
    ("POST", "/reports"),
    ("POST", "/reports/{report_id}/render"),
    ("GET", "/settings/env"),
    ("POST", "/settings/env/apply"),
    ("POST", "/settings/env/verify"),
    ("GET", "/sources"),
    ("GET", "/sources/{source_id}"),
    ("GET", "/sources/{source_id}/permissions"),
    ("POST", "/sources"),
    ("GET", "/work-items"),
    ("GET", "/work-items/{work_item_id}"),
    ("POST", "/work-items"),
    ("POST", "/work-items/"),
    ("POST", "/work-items/{work_item_id}/dispatch"),
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
    Conflict(String),
    Forbidden(String),
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
            Self::Conflict(error) => write!(formatter, "{error}"),
            Self::Forbidden(error) => write!(formatter, "{error}"),
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
        ("POST", "/agent/actions/") => agent_action_request_response(&request.body, database_url),
        ("POST", "/agent/intent") => json_response(200, "OK", agent_intent_json(&request.body), false),
        ("POST", "/chat/retrieval-preview") => {
            json_response(200, "OK", retrieval_preview_json(&request.body), false)
        }
        ("POST", "/chat/evidence-answer") => {
            json_response(200, "OK", evidence_answer_json(&request.body), false)
        }
        ("POST", "/approvals") => approval_create_response(&request.body, database_url),
        ("POST", "/analysis/patterns") => pattern_create_response(&request.body, database_url),
        ("POST", "/analysis/patterns/detect-baseline") => {
            baseline_patterns_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/dry-run") => {
            collection_dry_run_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/manual-upload") => {
            manual_upload_response(&request.body, database_url)
        }
        ("POST", "/feedback") => feedback_create_response(&request.body, database_url),
        ("POST", "/outcomes") => outcome_create_response(&request.body, database_url),
        ("POST", "/reports") => report_create_response(&request.body, database_url),
        ("POST", "/sources") => source_create_response(&request.body, database_url),
        ("POST", "/work-items") | ("POST", "/work-items/") => {
            work_item_create_response(&request.body, database_url)
        }
        ("GET", "/settings/env") => {
            json_response(200, "OK", settings_env_status_json(), false)
        }
        ("POST", "/settings/env/verify") => settings_env_verify_response(&request.body),
        ("POST", "/settings/env/apply") => settings_env_apply_response(&request.body, database_url),
        ("POST", _) => {
            if let Some(action_name) = agent_action_execute_path(&request.path) {
                agent_action_execute_response(&action_name, &request.body, database_url)
            } else if let Some(pattern_id) = pattern_review_path(&request.path) {
                action_route_response(review_pattern(&pattern_id, &request.body, database_url))
            } else if let Some(approval_id) = approval_decision_path(&request.path) {
                action_route_response(decide_approval(&approval_id, &request.body, database_url))
            } else if let Some(report_id) = report_render_path(&request.path) {
                action_route_response(render_report(&report_id, &request.body, database_url))
            } else if let Some(work_item_id) = work_item_dispatch_path(&request.path) {
                action_route_response(dispatch_work_item(
                    &work_item_id,
                    &request.body,
                    database_url,
                ))
            } else {
                fallback_or_error(request, fallback_origin)
            }
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
        Err(GatewayError::Conflict(message)) => json_response(
            409,
            "Conflict",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
            false,
        ),
        Err(GatewayError::Forbidden(message)) => json_response(
            403,
            "Forbidden",
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

fn source_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_source(body, database_url))
}

fn report_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_report(body, database_url))
}

fn work_item_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_work_item(body, database_url))
}

fn pattern_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_pattern(body, database_url))
}

fn baseline_patterns_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(detect_baseline_patterns(body, database_url))
}

fn collection_dry_run_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_collection_dry_run(body, database_url))
}

fn manual_upload_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_manual_upload_collection(body, database_url))
}

fn agent_action_request_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    action_route_response(record_agent_action_request(body, database_url))
}

fn agent_action_execute_response(
    action_name: &str,
    body: &str,
    database_url: Option<&str>,
) -> GatewayResponse {
    action_route_response(execute_agent_action_route(action_name, body, database_url))
}

fn action_route_response(result: Result<String, GatewayError>) -> GatewayResponse {
    match result {
        Ok(response_body) => json_response(200, "OK", response_body, false),
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
        Err(GatewayError::Forbidden(message)) => json_response(
            403,
            "Forbidden",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
            false,
        ),
        Err(GatewayError::Conflict(message)) => json_response(
            409,
            "Conflict",
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

fn settings_env_verify_response(body: &str) -> GatewayResponse {
    match verify_settings_env(body) {
        Ok(response_body) => json_response(200, "OK", response_body, false),
        Err(GatewayError::Validation(message)) => json_response(
            422,
            "Unprocessable Entity",
            validation_body_json(&message),
            false,
        ),
        Err(error) => json_response(
            409,
            "Conflict",
            format!("{{\"detail\":\"{}\"}}", escape_json(&error.to_string())),
            false,
        ),
    }
}

fn settings_env_apply_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    match apply_settings_env(body, database_url) {
        Ok(response_body) => json_response(200, "OK", response_body, false),
        Err(GatewayError::Validation(message)) => json_response(
            422,
            "Unprocessable Entity",
            validation_body_json(&message),
            false,
        ),
        Err(GatewayError::Conflict(message)) => json_response(409, "Conflict", message, false),
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

fn create_pattern(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_pattern_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    validate_evidence_ids(&mut transaction, &payload.evidence_ids)?;
    let response_body = insert_pattern_with_audit(&mut transaction, &payload)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn detect_baseline_patterns(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_baseline_pattern_detect(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let evidence_items = load_evidence_items_for_baseline(&mut transaction)?;
    let mut existing_keys = load_existing_detector_keys(&mut transaction)?;
    let candidates = baseline_pattern_candidates(&evidence_items, payload.recurrence_threshold);
    let mut responses = Vec::new();
    for candidate in candidates {
        if existing_keys.contains(&candidate.detector_key) {
            continue;
        }
        let pattern_payload = PatternCreatePayload {
            pattern_type: candidate.pattern_type,
            summary: candidate.summary,
            evidence_ids: candidate.evidence_ids,
            confidence: Some(candidate.confidence),
            status: "candidate".to_string(),
            actor_id: payload.actor_id.clone(),
            metadata_json: serde_json::json!({
                "generated_by": "DIFF-069",
                "detector": "baseline_local_v1",
                "detector_key": candidate.detector_key
            }),
        };
        let response = insert_pattern_with_audit(&mut transaction, &pattern_payload)?;
        if let Some(detector_key) = pattern_payload
            .metadata_json
            .get("detector_key")
            .and_then(Value::as_str)
        {
            existing_keys.insert(detector_key.to_string());
        }
        responses.push(response);
    }
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(format!("[{}]", responses.join(",")))
}

fn create_collection_dry_run(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_collection_dry_run(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let source = load_source_for_collection(&mut transaction, &payload.source_id)?;
    if !source.enabled {
        return Err(GatewayError::Conflict("Source is disabled".to_string()));
    }
    let permission =
        load_permission_for_collection(&mut transaction, &payload.source_permission_id)?;
    if permission.source_id != source.id {
        return Err(GatewayError::Conflict(
            "Source permission does not belong to the source".to_string(),
        ));
    }
    if !permission_allows(&permission.allowed_operations, &["dry_run", "read"]) {
        return Err(GatewayError::Forbidden(
            "Source permission does not allow dry-run preview".to_string(),
        ));
    }

    let dry_run_result = connector_dry_run_result(&source, &permission);
    let error_message = dry_run_result
        .as_ref()
        .err()
        .map(std::string::ToString::to_string);
    let status_value = if error_message.is_some() {
        "dry_run_failed"
    } else {
        "dry_run_previewed"
    };
    let summary_json = collection_dry_run_summary(
        &source,
        &permission,
        dry_run_result.as_ref().ok(),
        &payload.notes,
    );
    let collection_run_id = generated_record_id("collection");
    transaction
        .execute(
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, $3, true, $4, $5::jsonb, $6)",
            &[
                &collection_run_id,
                &source.id,
                &status_value,
                &payload.requested_by_actor_id,
                &summary_json,
                &error_message,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let created_details = serde_json::json!({
        "source_id": source.id,
        "dry_run": true,
        "status": status_value
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'collection_run.created', 'recorded', 'collection_run', $2, NULL, $3::jsonb)",
            &[
                &payload.requested_by_actor_id,
                &collection_run_id,
                &created_details,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let dry_run_details = serde_json::json!({
        "source_id": source.id,
        "source_permission_id": permission.id,
        "status": status_value,
        "error_message": error_message
    });
    let decision = if error_message.is_some() {
        "rejected"
    } else {
        "recorded"
    };
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'collection_run.dry_run_preview', $2, 'collection_run', $3, NULL, $4::jsonb)",
            &[
                &payload.requested_by_actor_id,
                &decision,
                &collection_run_id,
                &dry_run_details,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs WHERE id = $1) t",
            &[&collection_run_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_manual_upload_collection(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_manual_upload_collection(body)?;
    let content = decode_base64(&payload.content_base64)?;
    require_supported_text_mime_type(payload.mime_type.as_deref())?;
    require_utf8_text_content(&content)?;
    if content.len() > i32::MAX as usize {
        return Err(GatewayError::Validation(
            "Manual upload content is too large".to_string(),
        ));
    }

    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let source = load_source_for_collection(&mut transaction, &payload.source_id)?;
    if !source.enabled {
        return Err(GatewayError::Conflict("Source is disabled".to_string()));
    }
    if source.source_type != "manual_upload" {
        return Err(GatewayError::Conflict(
            "Source is not a manual_upload source".to_string(),
        ));
    }
    let permission =
        load_permission_for_collection(&mut transaction, &payload.source_permission_id)?;
    if permission.source_id != source.id {
        return Err(GatewayError::Conflict(
            "Source permission does not belong to the source".to_string(),
        ));
    }
    if !permission_allows(&permission.allowed_operations, &["collect", "read"]) {
        return Err(GatewayError::Forbidden(
            "Source permission does not allow manual upload collection".to_string(),
        ));
    }
    let approval_id = require_collection_approval(
        &mut transaction,
        payload.approval_id.as_deref(),
        &source,
        &permission,
        "manual_upload_collection",
    )?;

    let store = ArtifactStore::new(artifact_data_root())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let stored = store
        .write_bytes(&content)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let collection_run_id = generated_record_id("collection");
    let raw_artifact_id = generated_record_id("artifact");
    let work_item_id = generated_record_id("work");
    let summary_json = serde_json::json!({
        "mode": "manual_upload_collection",
        "source_permission_id": permission.id,
        "filename": payload.filename,
        "content_hash": stored.content_hash,
        "storage_path": stored.storage_path,
        "size_bytes": stored.size_bytes,
        "content_already_existed": stored.existed,
        "would_normalize": true,
        "normalization_input_type": "utf_8_text",
        "approval_id": approval_id,
        "normalization_work_item_created": true,
        "normalization_work_item_id": work_item_id,
        "raw_artifact_ids": [raw_artifact_id]
    });
    let artifact_metadata = manual_upload_artifact_metadata(
        &payload.metadata_json,
        payload.filename.as_deref(),
        &permission.id,
        approval_id.as_deref(),
    );
    let work_payload = manual_upload_normalization_work_payload(
        &collection_run_id,
        &source,
        &permission.id,
        &raw_artifact_id,
    );

    transaction
        .execute(
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, 'completed', false, $3, $4::jsonb, NULL)",
            &[
                &collection_run_id,
                &source.id,
                &payload.requested_by_actor_id,
                &summary_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let collection_details = serde_json::json!({
        "source_id": source.id,
        "dry_run": false,
        "status": "completed"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'collection_run.created', 'recorded', 'collection_run', $2, NULL, $3::jsonb)",
            &[&payload.requested_by_actor_id, &collection_run_id, &collection_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .execute(
            "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::integer, $8::jsonb)",
            &[
                &raw_artifact_id,
                &source.id,
                &collection_run_id,
                &stored.content_hash,
                &stored.storage_path,
                &payload.mime_type,
                &(stored.size_bytes as i32),
                &artifact_metadata,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let artifact_details = serde_json::json!({
        "source_id": source.id,
        "collection_run_id": collection_run_id,
        "content_hash": stored.content_hash,
        "storage_path": stored.storage_path,
        "size_bytes": stored.size_bytes,
        "content_already_existed": stored.existed
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'raw_artifact.created', 'recorded', 'raw_artifact', $2, NULL, $3::jsonb)",
            &[&payload.requested_by_actor_id, &raw_artifact_id, &artifact_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, 'collection_normalization', 'queued', $2, $3::jsonb, NULL)",
            &[&work_item_id, &payload.requested_by_actor_id, &work_payload],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let work_details = serde_json::json!({
        "work_type": "collection_normalization",
        "collection_run_id": collection_run_id,
        "raw_artifact_ids": [raw_artifact_id],
        "scaffold_only": false,
        "executes_normalization": true
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'queued', 'work_item', $2, $3, $4::jsonb)",
            &[
                &payload.requested_by_actor_id,
                &work_item_id,
                &collection_run_id,
                &work_details,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs WHERE id = $1) t",
            &[&collection_run_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn record_agent_action_request(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_agent_action_request(body)?;
    let definition = if let Some(action_name) = payload.action_name.as_deref() {
        validate_action_name(action_name)?;
        action_definition(action_name)
    } else {
        None
    };
    let intent_body = if let Some(definition) = definition {
        agent_action_definition_json(definition, &payload.parameters)
    } else {
        let message = payload.message.clone().unwrap_or_default();
        agent_intent_json_from_parts(&message, &payload.parameters)
    };
    let action_name = definition
        .map(|definition| definition.name.to_string())
        .or_else(|| {
            payload
                .message
                .as_deref()
                .and_then(|message| classify_agent_message(message, &payload.parameters))
                .map(str::to_string)
        });
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "action_name": action_name,
        "parameters": safe_parameter_summary(&payload.parameters),
        "source": "rust_gateway"
    });
    client
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'agent.action.requested', 'recorded', 'agent_action', $2, NULL, $3::jsonb)",
            &[
                &payload.actor_id,
                &action_name.clone().unwrap_or_else(|| "unknown".to_string()),
                &details_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(intent_body)
}

fn execute_agent_action_route(
    raw_action_name: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let action_name = validate_action_name(raw_action_name)?;
    let payload = parse_agent_action_execute(body)?;
    let definition = action_definition(&action_name)
        .ok_or_else(|| GatewayError::NotFound("Unknown agent action".to_string()))?;
    validate_required_action_parameters(definition, &payload.parameters)?;
    if definition.approval_required && payload.approval_id.is_none() {
        return Err(GatewayError::Forbidden(
            "Agent action requires approval".to_string(),
        ));
    }
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    if definition.approval_required {
        match require_agent_action_approval(&mut client, definition, &payload) {
            Ok(()) => {}
            Err(error) => {
                let _ = insert_agent_action_audit(
                    &mut client,
                    &payload.actor_id,
                    "agent.action.rejected",
                    "blocked",
                    definition.name,
                    None,
                    serde_json::json!({
                        "reason": error.to_string(),
                        "parameters": safe_parameter_summary(&payload.parameters),
                        "approval_id": payload.approval_id
                    }),
                );
                return Err(error);
            }
        }
    }

    let started_at = now_epoch_string();
    let started_audit_event_id = insert_agent_action_audit(
        &mut client,
        &payload.actor_id,
        "agent.action.started",
        "started",
        definition.name,
        None,
        serde_json::json!({
            "parameters": safe_parameter_summary(&payload.parameters),
            "approval_id": payload.approval_id
        }),
    )?;
    let execution = execute_known_agent_action(definition, &payload.parameters, &mut client);
    let finished_at = now_epoch_string();
    let (status_value, result, stdout_summary, stderr_summary, exit_code) = match execution {
        Ok(result) => (
            result.status,
            result.result,
            result.stdout_summary,
            result.stderr_summary,
            result.exit_code,
        ),
        Err(error) => (
            "failed".to_string(),
            serde_json::json!({"error": error.to_string()}),
            None,
            None,
            None,
        ),
    };
    let finished_audit_event_id = insert_agent_action_audit(
        &mut client,
        &payload.actor_id,
        "agent.action.finished",
        &status_value,
        definition.name,
        Some(started_audit_event_id.to_string()),
        serde_json::json!({
            "status": status_value,
            "started_audit_event_id": started_audit_event_id,
            "exit_code": exit_code
        }),
    )?;
    Ok(serde_json::json!({
        "action_name": definition.name,
        "status": status_value,
        "result": result,
        "stdout_summary": stdout_summary,
        "stderr_summary": stderr_summary,
        "exit_code": exit_code,
        "started_at": started_at,
        "finished_at": finished_at,
        "audit_event_id": finished_audit_event_id
    })
    .to_string())
}

fn verify_settings_env(body: &str) -> Result<String, GatewayError> {
    let payload = parse_settings_candidate(body, false)?;
    let config = settings_env_config();
    let parsed = read_current_settings_env(&config)?;
    let (candidate, unmanaged, changed_keys) =
        build_settings_candidate(&config, &parsed, &payload.values)?;
    let validation = validate_settings_candidate(&candidate, &unmanaged, &changed_keys);
    Ok(settings_verify_response_json(&candidate, &validation))
}

fn apply_settings_env(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_settings_candidate(body, true)?;
    let config = settings_env_config();
    let parsed = read_current_settings_env(&config)?;
    let (candidate, unmanaged, changed_keys) =
        build_settings_candidate(&config, &parsed, &payload.values)?;
    let validation = validate_settings_candidate(&candidate, &unmanaged, &changed_keys);
    if !validation.errors.is_empty() {
        return Err(GatewayError::Conflict(format!(
            "{{\"detail\":{{\"message\":\"Verified candidate no longer passes validation.\",\"errors\":{}}}}}",
            validation_issues_json(&validation.errors)
        )));
    }
    if payload.verification_token.as_deref() != Some(validation.candidate_hash.as_str()) {
        return Err(GatewayError::Conflict(
            "{\"detail\":\"Submitted settings do not match the passing dry-run verification token.\"}"
                .to_string(),
        ));
    }
    if !settings_env_paths_are_safe(&config.env_file_path, &config.backup_dir) {
        return Err(GatewayError::Conflict(
            "{\"detail\":\"Configured .env path is not safe.\"}".to_string(),
        ));
    }
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let backup_path = create_settings_env_backup(&config.env_file_path, &config.backup_dir)?;
    atomic_write_settings_env(
        &config.env_file_path,
        &render_settings_env_content(&candidate, &unmanaged),
    )?;
    let audit_details = serde_json::json!({
        "changed_keys": changed_keys,
        "backup_path": backup_path.to_string_lossy(),
        "restart_required": validation.restart_required,
        "warning_count": validation.warnings.len(),
        "error_count": validation.errors.len(),
        "candidate_hash": validation.candidate_hash,
        "secret_values_recorded": false
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'settings.env.updated', 'saved', 'settings_env', 'local-env', $2, $3::jsonb)",
            &[&payload.actor_id, &validation.candidate_hash, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    let current = settings_env_response_from_values(&candidate, &unmanaged, &config, &parsed);
    Ok(format!(
        "{{\"saved\":true,\"backup_path\":\"{}\",\"changed_keys\":{},\"restart_required\":{},\"restart_notes\":{},\"warnings\":{},\"current\":{}}}",
        escape_json(&backup_path.to_string_lossy()),
        json_owned_string_array(&validation.changed_keys),
        validation.restart_required,
        json_owned_string_array(&validation.restart_notes),
        validation_issues_json(&validation.warnings),
        current
    ))
}

fn create_report(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_report_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let report_id = generated_record_id("report");

    transaction
        .execute(
            "INSERT INTO reports (id, title, report_type, status, requested_by_actor_id, artifact_path, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb)",
            &[
                &report_id,
                &payload.title,
                &payload.report_type,
                &payload.status,
                &payload.requested_by_actor_id,
                &payload.artifact_path,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "report_type": payload.report_type,
        "status": payload.status
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'report.created', 'recorded', 'report', $2, NULL, $3::jsonb)",
            &[&payload.requested_by_actor_id, &report_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, title, report_type, status, requested_by_actor_id, artifact_path, metadata_json, created_at, updated_at FROM reports WHERE id = $1) t",
            &[&report_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_work_item(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_work_item_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let work_item_id = generated_record_id("work");
    let mut payload_json = payload.payload_json;
    payload_json["intent_verification"] = payload.intent.clone();

    transaction
        .execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, $2, 'pending_intent_verification', $3, $4::jsonb, NULL)",
            &[
                &work_item_id,
                &payload.work_type,
                &payload.requested_by_actor_id,
                &payload_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "work_type": payload.work_type,
        "status": "pending_intent_verification"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'intent_verification_required', 'work_item', $2, NULL, $3::jsonb)",
            &[&payload.requested_by_actor_id, &work_item_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, work_type, status, requested_by_actor_id, payload_json, error_message, created_at, updated_at FROM work_items WHERE id = $1) t",
            &[&work_item_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn review_pattern(
    pattern_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let pattern_id = validate_route_id(pattern_id, "pattern_id")?;
    let payload = parse_pattern_review(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let Some(row) = transaction
        .query_opt("SELECT status FROM patterns WHERE id = $1", &[&pattern_id])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Pattern not found".to_string()));
    };
    let previous_status: String = row.get(0);
    if previous_status != "candidate" {
        return Err(GatewayError::Conflict(
            "Only candidate patterns can be reviewed".to_string(),
        ));
    }
    transaction
        .execute(
            "UPDATE patterns SET status = $2, updated_at = now() WHERE id = $1",
            &[&pattern_id, &payload.status],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "previous_status": previous_status,
        "new_status": payload.status,
        "review_note": payload.review_note
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'analysis.pattern.reviewed', $2, 'pattern', $3, NULL, $4::jsonb)",
            &[&payload.reviewed_by_actor_id, &payload.status, &pattern_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, pattern_type, status, summary, evidence_ids, confidence, metadata_json, created_at, updated_at FROM patterns WHERE id = $1) t",
            &[&pattern_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn decide_approval(
    approval_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let approval_id = validate_route_id(approval_id, "approval_id")?;
    let payload = parse_approval_decision(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let Some(row) = transaction
        .query_opt(
            "SELECT request_type, status FROM approvals WHERE id = $1",
            &[&approval_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Approval not found".to_string()));
    };
    let request_type: String = row.get(0);
    let previous_status: String = row.get(1);
    if previous_status != "pending" {
        return Err(GatewayError::Conflict(
            "Approval already decided".to_string(),
        ));
    }
    transaction
        .execute(
            "UPDATE approvals SET status = $2, decided_by_actor_id = $3, decision_reason = $4, decided_at = now(), updated_at = now() WHERE id = $1",
            &[
                &approval_id,
                &payload.status,
                &payload.decided_by_actor_id,
                &payload.decision_reason,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "request_type": request_type,
        "status": payload.status
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'approval.decided', $2, 'approval', $3, NULL, $4::jsonb)",
            &[&payload.decided_by_actor_id, &payload.status, &approval_id, &details_json],
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

fn render_report(
    report_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let report_id = validate_route_id(report_id, "report_id")?;
    let payload = parse_report_render(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let Some(row) = transaction
        .query_opt(
            "SELECT id, title, report_type, status, requested_by_actor_id, metadata_json FROM reports WHERE id = $1",
            &[&report_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Report not found".to_string()));
    };
    let report = ReportRenderRecord {
        id: row.get(0),
        title: row.get(1),
        report_type: row.get(2),
        status: row.get(3),
        requested_by_actor_id: row.get(4),
        metadata_json: row.get(5),
    };
    let content = build_report_markdown(&mut transaction, &report, payload.notes.as_deref())?;
    let store = ArtifactStore::new(artifact_data_root())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let stored = store
        .write_bytes(content.as_bytes())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    if stored.size_bytes > i32::MAX as u64 {
        return Err(GatewayError::Conflict(
            "Rendered report artifact is too large".to_string(),
        ));
    }
    let artifact_id = generated_record_id("artifact");
    let mut metadata_json = report.metadata_json;
    if let Value::Object(ref mut object) = metadata_json {
        object.insert(
            "rendered_artifact_id".to_string(),
            Value::String(artifact_id.clone()),
        );
        object.insert(
            "rendered_mime_type".to_string(),
            Value::String("text/markdown".to_string()),
        );
    } else {
        metadata_json = serde_json::json!({
            "rendered_artifact_id": artifact_id,
            "rendered_mime_type": "text/markdown"
        });
    }
    let artifact_metadata = serde_json::json!({
        "generated_by": "DIFF-120",
        "artifact_kind": "report",
        "report_id": report.id,
        "report_type": report.report_type,
        "filename": format!("{}.md", report.id)
    });
    transaction
        .execute(
            "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, NULL, NULL, $2, $3, 'text/markdown', $4::integer, $5::jsonb)",
            &[
                &artifact_id,
                &stored.content_hash,
                &stored.storage_path,
                &(stored.size_bytes as i32),
                &artifact_metadata,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .execute(
            "UPDATE reports SET status = 'ready', artifact_path = $2, metadata_json = $3::jsonb, updated_at = now() WHERE id = $1",
            &[&report.id, &stored.storage_path, &metadata_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "artifact_id": artifact_id,
        "artifact_path": stored.storage_path,
        "content_hash": stored.content_hash,
        "content_already_existed": stored.existed
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'report.rendered', 'ready', 'report', $2, $3, $4::jsonb)",
            &[&payload.actor_id, &report.id, &artifact_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, title, report_type, status, requested_by_actor_id, artifact_path, metadata_json, created_at, updated_at FROM reports WHERE id = $1) t",
            &[&report.id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn dispatch_work_item(
    work_item_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let work_item_id = validate_route_id(work_item_id, "work_item_id")?;
    let payload = parse_work_item_dispatch(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let Some(row) = transaction
        .query_opt(
            "SELECT id, work_type, status, payload_json FROM work_items WHERE id = $1",
            &[&work_item_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Work item not found".to_string()));
    };
    let work_item = WorkItemDispatchRecord {
        id: row.get(0),
        work_type: row.get(1),
        status: row.get(2),
        payload_json: row.get(3),
    };
    if work_item.status != "queued" {
        return Err(GatewayError::Conflict(
            "Only queued work items can be dispatched".to_string(),
        ));
    }
    if !has_intent_verification(&work_item.payload_json) {
        return Err(GatewayError::Conflict(
            "Work item requires recorded intent verification before dispatch".to_string(),
        ));
    }
    let task_name = dispatch_task_name(&work_item)?;
    let task_id = generated_record_id("dispatch");
    let mut payload_json = work_item.payload_json;
    match payload_json {
        Value::Object(ref mut object) => {
            object.insert(
                "dispatch".to_string(),
                serde_json::json!({
                    "task_name": task_name,
                    "task_id": task_id,
                    "dispatched_by_actor_id": payload.actor_id,
                    "rust_gateway_execution": "not_executed",
                    "safe_dispatch_only": true,
                    "parity_limit": "DIFF-120 does not invoke Celery from the Rust gateway"
                }),
            );
        }
        _ => {
            payload_json = serde_json::json!({
                "dispatch": {
                    "task_name": task_name,
                    "task_id": task_id,
                    "dispatched_by_actor_id": payload.actor_id,
                    "rust_gateway_execution": "not_executed",
                    "safe_dispatch_only": true,
                    "parity_limit": "DIFF-120 does not invoke Celery from the Rust gateway"
                }
            });
        }
    }
    transaction
        .execute(
            "UPDATE work_items SET payload_json = $2::jsonb, updated_at = now() WHERE id = $1",
            &[&work_item.id, &payload_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "work_type": work_item.work_type,
        "task_name": task_name,
        "task_id": task_id,
        "rust_gateway_execution": "not_executed",
        "safe_dispatch_only": true
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.dispatched', 'queued_without_execution', 'work_item', $2, $3, $4::jsonb)",
            &[&payload.actor_id, &work_item.id, &task_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(format!(
        "{{\"work_item_id\":\"{}\",\"work_type\":\"{}\",\"task_name\":\"{}\",\"task_id\":\"{}\",\"status\":\"{}\",\"rust_gateway_execution\":\"not_executed\"}}",
        escape_json(&work_item.id),
        escape_json(&work_item.work_type),
        escape_json(&task_name),
        escape_json(&task_id),
        escape_json(&work_item.status)
    ))
}

fn insert_pattern_with_audit(
    transaction: &mut postgres::Transaction<'_>,
    payload: &PatternCreatePayload,
) -> Result<String, GatewayError> {
    let pattern_id = generated_record_id("pattern");
    let evidence_ids_json = Value::Array(
        payload
            .evidence_ids
            .iter()
            .map(|id| Value::String(id.clone()))
            .collect(),
    );
    transaction
        .execute(
            "INSERT INTO patterns (id, pattern_type, status, summary, evidence_ids, confidence, metadata_json) VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7::jsonb)",
            &[
                &pattern_id,
                &payload.pattern_type,
                &payload.status,
                &payload.summary,
                &evidence_ids_json,
                &payload.confidence,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "evidence_ids": payload.evidence_ids
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'analysis.pattern.created', 'recorded', 'pattern', $2, NULL, $3::jsonb)",
            &[&payload.actor_id, &pattern_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, pattern_type, status, summary, evidence_ids, confidence, metadata_json, created_at, updated_at FROM patterns WHERE id = $1) t",
            &[&pattern_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn validate_evidence_ids(
    transaction: &mut postgres::Transaction<'_>,
    evidence_ids: &[String],
) -> Result<(), GatewayError> {
    for evidence_id in evidence_ids {
        let exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM evidence_items WHERE id = $1)",
                &[evidence_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !exists {
            return Err(GatewayError::Validation(
                "Analysis records must reference existing evidence items".to_string(),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct BaselineEvidenceItem {
    id: String,
    evidence_type: String,
    statement: String,
    source_id: Option<String>,
}

#[derive(Debug, Clone)]
struct BaselinePatternCandidate {
    pattern_type: String,
    summary: String,
    evidence_ids: Vec<String>,
    confidence: i32,
    detector_key: String,
}

fn load_evidence_items_for_baseline(
    transaction: &mut postgres::Transaction<'_>,
) -> Result<Vec<BaselineEvidenceItem>, GatewayError> {
    transaction
        .query(
            "SELECT id, evidence_type, statement, source_id FROM evidence_items ORDER BY created_at DESC",
            &[],
        )
        .map(|rows| {
            rows.into_iter()
                .map(|row| BaselineEvidenceItem {
                    id: row.get::<_, String>(0),
                    evidence_type: row.get::<_, String>(1),
                    statement: row.get::<_, String>(2),
                    source_id: row.get::<_, Option<String>>(3),
                })
                .collect()
        })
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn load_existing_detector_keys(
    transaction: &mut postgres::Transaction<'_>,
) -> Result<HashSet<String>, GatewayError> {
    transaction
        .query(
            "SELECT metadata_json->>'detector_key' FROM patterns WHERE metadata_json ? 'detector_key'",
            &[],
        )
        .map(|rows| {
            rows.into_iter()
                .filter_map(|row| row.get::<_, Option<String>>(0))
                .collect()
        })
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn build_report_markdown(
    transaction: &mut postgres::Transaction<'_>,
    report: &ReportRenderRecord,
    notes: Option<&str>,
) -> Result<String, GatewayError> {
    let counts = [
        ("approvals", count_table(transaction, "approvals")?),
        ("artifacts", count_table(transaction, "raw_artifacts")?),
        (
            "collection_runs",
            count_table(transaction, "collection_runs")?,
        ),
        (
            "evidence_items",
            count_table(transaction, "evidence_items")?,
        ),
        ("experiments", count_table(transaction, "experiment_runs")?),
        (
            "feedback_events",
            count_table(transaction, "feedback_events")?,
        ),
        (
            "improvement_items",
            count_table(transaction, "improvement_items")?,
        ),
        ("outcomes", count_table(transaction, "outcomes")?),
        ("patterns", count_table(transaction, "patterns")?),
        (
            "recommendations",
            count_table(transaction, "recommendations")?,
        ),
        ("sources", count_table(transaction, "sources")?),
        ("work_items", count_table(transaction, "work_items")?),
    ];
    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        format!("- Report ID: `{}`", report.id),
        format!("- Report type: `{}`", report.report_type),
        format!("- Requested by: `{}`", report.requested_by_actor_id),
        format!("- Status before render: `{}`", report.status),
        String::new(),
        "## Inventory Counts".to_string(),
        String::new(),
    ];
    for (label, count) in counts {
        lines.push(format!("- {label}: {count}"));
    }
    lines.extend([
        String::new(),
        "## Boundaries".to_string(),
        String::new(),
        "- This report is generated from local metadata records only.".to_string(),
        "- It does not read raw artifact contents.".to_string(),
        "- It does not call external models or execute actions.".to_string(),
    ]);
    if let Some(notes) = notes.filter(|value| !value.trim().is_empty()) {
        lines.extend([
            String::new(),
            "## Notes".to_string(),
            String::new(),
            notes.to_string(),
        ]);
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn count_table(
    transaction: &mut postgres::Transaction<'_>,
    table_name: &str,
) -> Result<i64, GatewayError> {
    let sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
    transaction
        .query_one(&sql, &[])
        .map(|row| row.get::<_, i64>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn has_intent_verification(payload_json: &Value) -> bool {
    payload_json
        .get("intent_verification")
        .is_some_and(Value::is_object)
        || payload_json
            .get("intent_verification_recorded")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn dispatch_task_name(work_item: &WorkItemDispatchRecord) -> Result<String, GatewayError> {
    match work_item.work_type.as_str() {
        "collection_normalization" => {
            if !work_item
                .payload_json
                .get("collection_run_id")
                .is_some_and(Value::is_string)
                || !work_item
                    .payload_json
                    .get("raw_artifact_ids")
                    .is_some_and(Value::is_array)
            {
                return Err(GatewayError::Validation(
                    "Invalid normalization payload".to_string(),
                ));
            }
            Ok("collection.normalize_collection_run".to_string())
        }
        "document_chunking" => {
            let has_document_ids = work_item
                .payload_json
                .get("document_ids")
                .is_some_and(Value::is_array);
            let has_document_id = work_item
                .payload_json
                .get("document_id")
                .is_some_and(Value::is_string);
            if !has_document_ids && !has_document_id {
                return Err(GatewayError::Validation(
                    "Invalid document chunking payload".to_string(),
                ));
            }
            Ok("evidence.generate_document_chunks".to_string())
        }
        "chunk_vector_upsert" => Ok("memory.vector.upsert_chunks".to_string()),
        _ => Err(GatewayError::Validation(
            "Unsupported work item dispatch type".to_string(),
        )),
    }
}

fn baseline_pattern_candidates(
    evidence_items: &[BaselineEvidenceItem],
    recurrence_threshold: i32,
) -> Vec<BaselinePatternCandidate> {
    if evidence_items.is_empty() {
        return vec![BaselinePatternCandidate {
            pattern_type: "missing_information_gap".to_string(),
            summary: "No evidence items exist yet, so the system cannot detect grounded patterns."
                .to_string(),
            evidence_ids: Vec::new(),
            confidence: 100,
            detector_key: "missing_information_gap:no_evidence".to_string(),
        }];
    }

    let mut candidates = Vec::new();
    let mut by_type: HashMap<String, Vec<&BaselineEvidenceItem>> = HashMap::new();
    let mut by_statement: HashMap<String, Vec<&BaselineEvidenceItem>> = HashMap::new();
    for item in evidence_items {
        by_type
            .entry(item.evidence_type.clone())
            .or_default()
            .push(item);
        by_statement
            .entry(normalize_statement(&item.statement))
            .or_default()
            .push(item);
    }

    let mut type_keys = by_type.keys().cloned().collect::<Vec<_>>();
    type_keys.sort();
    for evidence_type in type_keys {
        let items = &by_type[&evidence_type];
        if items.len() >= recurrence_threshold as usize {
            candidates.push(BaselinePatternCandidate {
                pattern_type: "recurrence".to_string(),
                summary: format!(
                    "{} evidence items share evidence type `{}`.",
                    items.len(),
                    evidence_type
                ),
                evidence_ids: items.iter().take(10).map(|item| item.id.clone()).collect(),
                confidence: std::cmp::min(90, 50 + items.len() as i32 * 5),
                detector_key: format!("recurrence:evidence_type:{evidence_type}"),
            });
        }
    }

    let mut statement_keys = by_statement.keys().cloned().collect::<Vec<_>>();
    statement_keys.sort();
    for normalized_statement in statement_keys {
        let items = &by_statement[&normalized_statement];
        let source_ids = items
            .iter()
            .filter_map(|item| item.source_id.as_deref())
            .collect::<HashSet<_>>();
        if source_ids.len() >= 2 {
            candidates.push(BaselinePatternCandidate {
                pattern_type: "cross_source_conflict".to_string(),
                summary: "Multiple sources contain the same normalized evidence statement; review whether they agree, duplicate, or conflict.".to_string(),
                evidence_ids: items.iter().take(10).map(|item| item.id.clone()).collect(),
                confidence: 60,
                detector_key: format!("cross_source_statement:{normalized_statement}"),
            });
        }
    }

    candidates
}

fn normalize_statement(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn create_source(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_source_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let source_id = generated_record_id("source");

    transaction
        .execute(
            "INSERT INTO sources (id, name, source_type, location, owner_actor_id, sensitivity, trust_level, enabled, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb)",
            &[
                &source_id,
                &payload.name,
                &payload.source_type,
                &payload.location,
                &payload.owner_actor_id,
                &payload.sensitivity,
                &payload.trust_level,
                &payload.enabled,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    if let Some(permission) = &payload.permission {
        let permission_id = generated_record_id("permission");
        let allowed_operations_json = Value::Array(
            permission
                .allowed_operations
                .iter()
                .map(|operation| Value::String(operation.clone()))
                .collect(),
        );
        transaction
            .execute(
                "INSERT INTO source_permissions (id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id) VALUES ($1, $2, $3::jsonb, $4::jsonb, $5, $6, $7)",
                &[
                    &permission_id,
                    &source_id,
                    &permission.scope_json,
                    &allowed_operations_json,
                    &permission.external_model_policy,
                    &permission.approval_required,
                    &permission.created_by_actor_id,
                ],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    }

    let details_json = serde_json::json!({
        "source_type": payload.source_type,
        "sensitivity": payload.sensitivity,
        "permission_included": payload.permission.is_some()
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'source.created', 'recorded', 'source', $2, NULL, $3::jsonb)",
            &[&payload.owner_actor_id, &source_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT s.id, s.name, s.source_type, s.location, s.owner_actor_id, s.sensitivity, s.trust_level, s.enabled, s.metadata_json, s.created_at, s.updated_at, COALESCE((SELECT json_agg(row_to_json(p)) FROM (SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id, created_at, updated_at FROM source_permissions WHERE source_id = s.id ORDER BY created_at ASC) p), '[]'::json) AS permissions FROM sources s WHERE s.id = $1) t",
            &[&source_id],
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

struct SourceCreatePayload {
    name: String,
    source_type: String,
    location: Option<String>,
    owner_actor_id: String,
    sensitivity: String,
    trust_level: String,
    enabled: bool,
    metadata_json: Value,
    permission: Option<SourcePermissionCreatePayload>,
}

struct SourcePermissionCreatePayload {
    scope_json: Value,
    allowed_operations: Vec<String>,
    external_model_policy: String,
    approval_required: bool,
    created_by_actor_id: String,
}

struct ReportCreatePayload {
    title: String,
    report_type: String,
    status: String,
    requested_by_actor_id: String,
    artifact_path: Option<String>,
    metadata_json: Value,
}

struct ReportRenderPayload {
    actor_id: String,
    notes: Option<String>,
}

struct WorkItemCreatePayload {
    work_type: String,
    requested_by_actor_id: String,
    intent: Value,
    payload_json: Value,
}

struct WorkItemDispatchPayload {
    actor_id: String,
}

struct PatternCreatePayload {
    pattern_type: String,
    summary: String,
    evidence_ids: Vec<String>,
    confidence: Option<i32>,
    status: String,
    actor_id: String,
    metadata_json: Value,
}

struct PatternReviewPayload {
    status: String,
    reviewed_by_actor_id: String,
    review_note: Option<String>,
}

struct ApprovalDecisionPayload {
    status: String,
    decided_by_actor_id: String,
    decision_reason: Option<String>,
}

struct BaselinePatternDetectPayload {
    actor_id: String,
    recurrence_threshold: i32,
}

struct CollectionDryRunPayload {
    source_id: String,
    source_permission_id: String,
    requested_by_actor_id: String,
    notes: Value,
}

struct ManualUploadCollectionPayload {
    source_id: String,
    source_permission_id: String,
    approval_id: Option<String>,
    content_base64: String,
    filename: Option<String>,
    mime_type: Option<String>,
    metadata_json: Value,
    requested_by_actor_id: String,
}

struct AgentActionRequestPayload {
    message: Option<String>,
    action_name: Option<String>,
    parameters: Value,
    actor_id: String,
}

struct AgentActionExecutePayload {
    parameters: Value,
    approval_id: Option<String>,
    actor_id: String,
}

struct AgentActionExecutionResult {
    status: String,
    result: Value,
    stdout_summary: Option<String>,
    stderr_summary: Option<String>,
    exit_code: Option<i32>,
}

struct CollectionApproval {
    id: String,
    status: String,
    request_type: String,
    request_payload_json: Value,
}

struct CollectionSource {
    id: String,
    name: String,
    source_type: String,
    location: Option<String>,
    sensitivity: String,
    enabled: bool,
    metadata_json: Value,
}

struct CollectionPermission {
    id: String,
    source_id: String,
    scope_json: Value,
    allowed_operations: Vec<String>,
    external_model_policy: String,
    approval_required: bool,
}

struct ReportRenderRecord {
    id: String,
    title: String,
    report_type: String,
    status: String,
    requested_by_actor_id: String,
    metadata_json: Value,
}

struct WorkItemDispatchRecord {
    id: String,
    work_type: String,
    status: String,
    payload_json: Value,
}

#[derive(Debug)]
struct CollectionDryRunConnectorResult {
    connector_name: String,
    allowed: bool,
    summary: String,
    estimated_items: Option<i32>,
    warnings: Vec<String>,
    metadata: Value,
}

struct SettingsCandidatePayload {
    values: HashMap<String, String>,
    actor_id: String,
    verification_token: Option<String>,
}

struct SettingsEnvConfig {
    env_file_path: PathBuf,
    backup_dir: PathBuf,
    igy6_data_root: String,
}

struct ParsedSettingsEnv {
    values: HashMap<String, String>,
    unmanaged_order: Vec<String>,
}

#[derive(Clone)]
struct SettingsValidationIssue {
    key: Option<String>,
    message: String,
}

struct SettingsValidation {
    errors: Vec<SettingsValidationIssue>,
    warnings: Vec<SettingsValidationIssue>,
    changed_keys: Vec<String>,
    restart_required: bool,
    restart_notes: Vec<String>,
    candidate_hash: String,
    compose_validation: Value,
}

#[derive(Clone, Copy)]
struct SettingDefinition {
    key: &'static str,
    group: &'static str,
    description: &'static str,
}

type SettingsCandidateBuild = (
    HashMap<String, String>,
    HashMap<String, String>,
    Vec<String>,
);

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

fn parse_source_create(body: &str) -> Result<SourceCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Source request body")?;
    let name = required_string_field(&object, "name", 255)?;
    let source_type = required_string_field(&object, "source_type", 64)?;
    if !is_source_type(&source_type) {
        return Err(GatewayError::Validation(format!(
            "Unknown source type: {source_type}"
        )));
    }
    let location = optional_nullable_string_field(&object, "location")?;
    let owner_actor_id =
        optional_string_field_with_max(&object, "owner_actor_id", "local-owner", 128)?;
    let sensitivity = optional_string_field_with_max(&object, "sensitivity", "internal", 64)?;
    if !is_sensitivity_label(&sensitivity) {
        return Err(GatewayError::Validation(format!(
            "Unknown sensitivity label: {sensitivity}"
        )));
    }
    let trust_level = optional_string_field_with_max(&object, "trust_level", "unreviewed", 64)?;
    let enabled = optional_bool_field(&object, "enabled", true)?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    let permission = match object.get("permission") {
        None | Some(Value::Null) => None,
        Some(Value::Object(permission)) => Some(parse_source_permission_create(permission)?),
        Some(_) => {
            return Err(GatewayError::Validation(
                "permission must be a JSON object or null.".to_string(),
            ))
        }
    };

    Ok(SourceCreatePayload {
        name,
        source_type,
        location,
        owner_actor_id,
        sensitivity,
        trust_level,
        enabled,
        metadata_json,
        permission,
    })
}

fn parse_source_permission_create(
    object: &serde_json::Map<String, Value>,
) -> Result<SourcePermissionCreatePayload, GatewayError> {
    let scope_json = optional_object_field(object, "scope_json")?;
    let allowed_operations = optional_string_array_field(object, "allowed_operations")?;
    for operation in &allowed_operations {
        if !is_allowed_source_operation(operation) {
            return Err(GatewayError::Validation(format!(
                "Unknown allowed operation: {operation}"
            )));
        }
    }
    let external_model_policy =
        optional_string_field_with_max(object, "external_model_policy", "blocked", 64)?;
    if !is_external_model_policy(&external_model_policy) {
        return Err(GatewayError::Validation(format!(
            "Unknown external model policy: {external_model_policy}"
        )));
    }
    let approval_required = optional_bool_field(object, "approval_required", true)?;
    let created_by_actor_id =
        optional_string_field_with_max(object, "created_by_actor_id", "local-owner", 128)?;

    Ok(SourcePermissionCreatePayload {
        scope_json,
        allowed_operations,
        external_model_policy,
        approval_required,
        created_by_actor_id,
    })
}

fn parse_report_create(body: &str) -> Result<ReportCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Report request body")?;
    let title = required_string_field(&object, "title", 255)?;
    let report_type = required_string_field(&object, "report_type", 64)?;
    if !is_report_type(&report_type) {
        return Err(GatewayError::Validation(format!(
            "Unknown report type: {report_type}"
        )));
    }
    let status = optional_string_field_with_max(&object, "status", "requested", 64)?;
    if !is_report_status(&status) {
        return Err(GatewayError::Validation(format!(
            "Unknown report status: {status}"
        )));
    }
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let artifact_path = optional_nullable_string_field(&object, "artifact_path")?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;

    Ok(ReportCreatePayload {
        title,
        report_type,
        status,
        requested_by_actor_id,
        artifact_path,
        metadata_json,
    })
}

fn parse_report_render(body: &str) -> Result<ReportRenderPayload, GatewayError> {
    let object = parse_json_object(body, "Report render request body")?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    let notes = optional_nullable_string_field(&object, "notes")?;
    Ok(ReportRenderPayload { actor_id, notes })
}

fn parse_work_item_create(body: &str) -> Result<WorkItemCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Work item request body")?;
    let work_type = required_string_field(&object, "work_type", 64)?;
    if !is_supported_work_item_type(&work_type) {
        return Err(GatewayError::Validation(format!(
            "Unsupported work item type: {work_type}"
        )));
    }
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let intent = match object.get("intent") {
        Some(Value::Object(intent)) => validate_work_item_intent(intent)?,
        Some(_) => {
            return Err(GatewayError::Validation(
                "intent must be a JSON object.".to_string(),
            ))
        }
        None => return Err(GatewayError::Validation("intent is required.".to_string())),
    };
    let payload_json = optional_object_field(&object, "payload_json")?;

    Ok(WorkItemCreatePayload {
        work_type,
        requested_by_actor_id,
        intent,
        payload_json,
    })
}

fn parse_work_item_dispatch(body: &str) -> Result<WorkItemDispatchPayload, GatewayError> {
    let object = parse_json_object(body, "Work item dispatch request body")?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    Ok(WorkItemDispatchPayload { actor_id })
}

fn validate_work_item_intent(
    object: &serde_json::Map<String, Value>,
) -> Result<Value, GatewayError> {
    let original_request = required_text_field(object, "original_request")?;
    let interpretation = required_text_field(object, "interpretation")?;
    let proposed_work_type = required_text_field(object, "proposed_work_type")?;
    let expected_output = required_text_field(object, "expected_output")?;
    let safety_requirements = optional_string_array_field(object, "safety_requirements")?;
    let assumptions = optional_string_array_field(object, "assumptions")?;
    let missing_information = optional_string_array_field(object, "missing_information")?;
    let sources_likely_used = optional_string_array_field(object, "sources_likely_used")?;

    Ok(serde_json::json!({
        "original_request": original_request,
        "interpretation": interpretation,
        "proposed_work_type": proposed_work_type,
        "expected_output": expected_output,
        "safety_requirements": safety_requirements,
        "assumptions": assumptions,
        "missing_information": missing_information,
        "sources_likely_used": sources_likely_used
    }))
}

fn parse_pattern_create(body: &str) -> Result<PatternCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Pattern request body")?;
    let pattern_type = required_string_field(&object, "pattern_type", 64)?;
    let summary = required_text_field(&object, "summary")?;
    let evidence_ids = required_string_array_field(&object, "evidence_ids")?;
    let confidence = optional_i32_field(&object, "confidence", 0, 100)?;
    let status = optional_string_field_with_max(&object, "status", "candidate", 64)?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    Ok(PatternCreatePayload {
        pattern_type,
        summary,
        evidence_ids,
        confidence,
        status,
        actor_id,
        metadata_json,
    })
}

fn parse_pattern_review(body: &str) -> Result<PatternReviewPayload, GatewayError> {
    let object = parse_json_object(body, "Pattern review request body")?;
    let status = required_string_field(&object, "status", 64)?;
    if !matches!(status.as_str(), "verified" | "rejected") {
        return Err(GatewayError::Validation(
            "Pattern review status must be verified or rejected".to_string(),
        ));
    }
    let reviewed_by_actor_id =
        optional_string_field_with_max(&object, "reviewed_by_actor_id", "local-owner", 128)?;
    let review_note = optional_nullable_string_field(&object, "review_note")?;
    Ok(PatternReviewPayload {
        status,
        reviewed_by_actor_id,
        review_note,
    })
}

fn parse_approval_decision(body: &str) -> Result<ApprovalDecisionPayload, GatewayError> {
    let object = parse_json_object(body, "Approval decision request body")?;
    let status = required_string_field(&object, "status", 64)?;
    if !matches!(status.as_str(), "approved" | "denied") {
        return Err(GatewayError::Validation(
            "Approval decision must be approved or denied".to_string(),
        ));
    }
    let decided_by_actor_id =
        optional_string_field_with_max(&object, "decided_by_actor_id", "local-owner", 128)?;
    let decision_reason = optional_nullable_string_field(&object, "decision_reason")?;
    Ok(ApprovalDecisionPayload {
        status,
        decided_by_actor_id,
        decision_reason,
    })
}

fn parse_baseline_pattern_detect(body: &str) -> Result<BaselinePatternDetectPayload, GatewayError> {
    let object = parse_json_object(body, "Baseline pattern detect request body")?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    let recurrence_threshold =
        optional_i32_field(&object, "recurrence_threshold", 2, 20)?.unwrap_or(3);
    Ok(BaselinePatternDetectPayload {
        actor_id,
        recurrence_threshold,
    })
}

fn parse_collection_dry_run(body: &str) -> Result<CollectionDryRunPayload, GatewayError> {
    let object = parse_json_object(body, "Collection dry-run request body")?;
    let source_id = required_string_field(&object, "source_id", 36)?;
    let source_permission_id = required_string_field(&object, "source_permission_id", 36)?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let notes = optional_object_field(&object, "notes")?;
    Ok(CollectionDryRunPayload {
        source_id,
        source_permission_id,
        requested_by_actor_id,
        notes,
    })
}

fn parse_manual_upload_collection(
    body: &str,
) -> Result<ManualUploadCollectionPayload, GatewayError> {
    let object = parse_json_object(body, "Manual upload request body")?;
    let source_id = required_string_field(&object, "source_id", 36)?;
    let source_permission_id = required_string_field(&object, "source_permission_id", 36)?;
    let approval_id = optional_nullable_string_field_with_max(&object, "approval_id", 36)?;
    let content_base64 = required_text_field(&object, "content_base64")?;
    let filename = optional_nullable_string_field_with_max(&object, "filename", 255)?;
    if let Some(filename) = &filename {
        validate_safe_filename(filename)?;
    }
    let mime_type = optional_nullable_string_field_with_max(&object, "mime_type", 255)?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    Ok(ManualUploadCollectionPayload {
        source_id,
        source_permission_id,
        approval_id,
        content_base64,
        filename,
        mime_type,
        metadata_json,
        requested_by_actor_id,
    })
}

fn parse_agent_action_request(body: &str) -> Result<AgentActionRequestPayload, GatewayError> {
    let object = parse_json_object(body, "Agent action request body")?;
    let message = optional_nullable_string_field_with_max(&object, "message", 4096)?;
    let action_name = optional_nullable_string_field_with_max(&object, "action_name", 128)?;
    if message
        .as_ref()
        .is_none_or(|message| message.trim().is_empty())
        && action_name
            .as_ref()
            .is_none_or(|action_name| action_name.trim().is_empty())
    {
        return Err(GatewayError::Validation(
            "message or action_name is required.".to_string(),
        ));
    }
    let parameters = optional_object_field(&object, "parameters")?;
    reject_user_provided_argv(&parameters)?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    Ok(AgentActionRequestPayload {
        message: message.map(|value| value.trim().to_string()),
        action_name: action_name.map(|value| value.trim().to_string()),
        parameters,
        actor_id,
    })
}

fn parse_agent_action_execute(body: &str) -> Result<AgentActionExecutePayload, GatewayError> {
    let object = parse_json_object(body, "Agent action execute body")?;
    let parameters = optional_object_field(&object, "parameters")?;
    reject_user_provided_argv(&parameters)?;
    let approval_id = optional_nullable_string_field_with_max(&object, "approval_id", 36)?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    Ok(AgentActionExecutePayload {
        parameters,
        approval_id: approval_id.map(|value| value.trim().to_string()),
        actor_id,
    })
}

const MASKED_VALUE: &str = "********";
const DEFAULT_IGY6_DATA_ROOT: &str = "../IGY6_Data";
const SAFE_ENV_FILE_PATH: &str = "/workspace/project/.env";
const SAFE_BACKUP_ROOT: &str = "/workspace/storage";
const SETTING_GROUPS: &[(&str, &str)] = &[
    ("app", "App / Web"),
    ("postgres", "PostgreSQL"),
    ("redis", "Redis / Celery"),
    ("qdrant", "Qdrant"),
    ("neo4j", "Neo4j"),
    ("mlflow", "MLflow"),
    ("phoenix", "Phoenix"),
    ("storage", "Storage"),
    ("policy", "Policy / Safety"),
];
const SETTING_DEFINITIONS: &[SettingDefinition] = &[
    SettingDefinition { key: "APP_ENV", group: "app", description: "Local application environment label." },
    SettingDefinition { key: "APP_HOST", group: "app", description: "API bind host used by local configuration." },
    SettingDefinition { key: "APP_PORT", group: "app", description: "Published local API port." },
    SettingDefinition { key: "API_BASE_URL", group: "app", description: "Browser-facing API base URL." },
    SettingDefinition { key: "WEB_BASE_URL", group: "app", description: "Browser-facing web UI base URL." },
    SettingDefinition { key: "POSTGRES_HOST", group: "postgres", description: "PostgreSQL service hostname." },
    SettingDefinition { key: "POSTGRES_PORT", group: "postgres", description: "Published local PostgreSQL port." },
    SettingDefinition { key: "POSTGRES_DB", group: "postgres", description: "PostgreSQL database name." },
    SettingDefinition { key: "POSTGRES_USER", group: "postgres", description: "PostgreSQL username." },
    SettingDefinition { key: "POSTGRES_PASSWORD", group: "postgres", description: "PostgreSQL password." },
    SettingDefinition { key: "DATABASE_URL", group: "postgres", description: "SQLAlchemy PostgreSQL connection URL." },
    SettingDefinition { key: "REDIS_HOST", group: "redis", description: "Redis service hostname." },
    SettingDefinition { key: "REDIS_PORT", group: "redis", description: "Published local Redis port." },
    SettingDefinition { key: "REDIS_URL", group: "redis", description: "Redis URL used by API health checks." },
    SettingDefinition { key: "CELERY_BROKER_URL", group: "redis", description: "Celery broker Redis URL." },
    SettingDefinition { key: "CELERY_RESULT_BACKEND", group: "redis", description: "Celery result backend Redis URL." },
    SettingDefinition { key: "QDRANT_HOST", group: "qdrant", description: "Qdrant service hostname." },
    SettingDefinition { key: "QDRANT_PORT", group: "qdrant", description: "Published local Qdrant port." },
    SettingDefinition { key: "QDRANT_URL", group: "qdrant", description: "Qdrant API URL used by API and worker." },
    SettingDefinition { key: "QDRANT_CHUNK_COLLECTION", group: "qdrant", description: "Qdrant collection for chunk vectors." },
    SettingDefinition { key: "QDRANT_CHUNK_VECTOR_SIZE", group: "qdrant", description: "Deterministic chunk vector size." },
    SettingDefinition { key: "NEO4J_HOST", group: "neo4j", description: "Neo4j service hostname." },
    SettingDefinition { key: "NEO4J_HTTP_PORT", group: "neo4j", description: "Published local Neo4j browser port." },
    SettingDefinition { key: "NEO4J_BOLT_PORT", group: "neo4j", description: "Published local Neo4j Bolt port." },
    SettingDefinition { key: "NEO4J_USER", group: "neo4j", description: "Neo4j username." },
    SettingDefinition { key: "NEO4J_PASSWORD", group: "neo4j", description: "Neo4j password." },
    SettingDefinition { key: "NEO4J_URI", group: "neo4j", description: "Neo4j Bolt URI used by the API." },
    SettingDefinition { key: "MLFLOW_TRACKING_URI", group: "mlflow", description: "Reserved local MLflow tracking URI." },
    SettingDefinition { key: "MLFLOW_ARTIFACT_ROOT", group: "mlflow", description: "Reserved MLflow artifact root inside the service." },
    SettingDefinition { key: "PHOENIX_HOST", group: "phoenix", description: "Phoenix service hostname." },
    SettingDefinition { key: "PHOENIX_PORT", group: "phoenix", description: "Published local Phoenix port." },
    SettingDefinition { key: "PHOENIX_COLLECTOR_ENDPOINT", group: "phoenix", description: "Reserved local Phoenix endpoint." },
    SettingDefinition { key: "ARTIFACT_STORE_PATH", group: "storage", description: "Container path for content-addressed artifacts." },
    SettingDefinition { key: "EXPORT_STORE_PATH", group: "storage", description: "Container path for report/export output." },
    SettingDefinition { key: "ENV_FILE_PATH", group: "storage", description: "Controlled container path to the mounted local .env file." },
    SettingDefinition { key: "ENV_BACKUP_DIR", group: "storage", description: "Controlled backup directory for .env backups." },
    SettingDefinition { key: "IGY6_DATA_ROOT", group: "storage", description: "Host-side folder where IGY6 stores database, vector, graph, artifact, report, backup, MLflow, and Phoenix runtime data." },
    SettingDefinition { key: "EXTERNAL_MODEL_POLICY_DEFAULT", group: "policy", description: "Default external model policy." },
    SettingDefinition { key: "SINGLE_USER_MODE", group: "policy", description: "Local single-user mode toggle." },
    SettingDefinition { key: "AUDIT_LOG_LEVEL", group: "policy", description: "Audit logging verbosity label." },
    SettingDefinition { key: "APPROVAL_REQUIRED_DEFAULT", group: "policy", description: "Default approval-required toggle." },
];

fn parse_settings_candidate(
    body: &str,
    require_token: bool,
) -> Result<SettingsCandidatePayload, GatewayError> {
    let object = parse_json_object(body, "Settings request body")?;
    let values = match object.get("values") {
        None | Some(Value::Null) => HashMap::new(),
        Some(Value::Object(values)) => {
            let mut result = HashMap::new();
            for (key, value) in values {
                let Some(value) = value.as_str() else {
                    return Err(GatewayError::Validation(format!(
                        "{{\"detail\":\"values.{key} must be a string.\"}}"
                    )));
                };
                result.insert(key.to_string(), value.to_string());
            }
            result
        }
        Some(_) => {
            return Err(GatewayError::Validation(
                "{\"detail\":\"values must be a JSON object.\"}".to_string(),
            ))
        }
    };
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    let verification_token = match object.get("verification_token") {
        None | Some(Value::Null) if require_token => {
            return Err(GatewayError::Validation(
                "{\"detail\":\"verification_token is required.\"}".to_string(),
            ))
        }
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Some(Value::String(_)) => {
            return Err(GatewayError::Validation(
                "{\"detail\":\"verification_token must not be empty.\"}".to_string(),
            ))
        }
        Some(_) => {
            return Err(GatewayError::Validation(
                "{\"detail\":\"verification_token must be a string.\"}".to_string(),
            ))
        }
    };
    Ok(SettingsCandidatePayload {
        values,
        actor_id,
        verification_token,
    })
}

fn settings_env_config() -> SettingsEnvConfig {
    SettingsEnvConfig {
        env_file_path: PathBuf::from(
            env::var("ENV_FILE_PATH").unwrap_or_else(|_| SAFE_ENV_FILE_PATH.to_string()),
        ),
        backup_dir: PathBuf::from(
            env::var("ENV_BACKUP_DIR")
                .unwrap_or_else(|_| "/workspace/storage/env_backups".to_string()),
        ),
        igy6_data_root: env::var("IGY6_DATA_ROOT")
            .unwrap_or_else(|_| DEFAULT_IGY6_DATA_ROOT.to_string()),
    }
}

fn read_current_settings_env(
    config: &SettingsEnvConfig,
) -> Result<ParsedSettingsEnv, GatewayError> {
    if !config.env_file_path.exists() {
        return Ok(ParsedSettingsEnv {
            values: HashMap::new(),
            unmanaged_order: Vec::new(),
        });
    }
    let content = fs::read_to_string(&config.env_file_path)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(parse_settings_env_content(&content))
}

fn parse_settings_env_content(content: &str) -> ParsedSettingsEnv {
    let mut values = HashMap::new();
    let mut unmanaged_order = Vec::new();
    for line in content.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') || !line.contains('=') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut parsed_value = value.trim().to_string();
        if parsed_value.len() >= 2 {
            let first = parsed_value.as_bytes()[0] as char;
            let last = parsed_value.as_bytes()[parsed_value.len() - 1] as char;
            if first == last && matches!(first, '\'' | '"') {
                parsed_value = parsed_value[1..parsed_value.len() - 1].to_string();
            }
        }
        values.insert(key.to_string(), parsed_value);
        if !setting_key_allowed(key) && !unmanaged_order.iter().any(|existing| existing == key) {
            unmanaged_order.push(key.to_string());
        }
    }
    ParsedSettingsEnv {
        values,
        unmanaged_order,
    }
}

fn build_settings_candidate(
    config: &SettingsEnvConfig,
    parsed: &ParsedSettingsEnv,
    requested_values: &HashMap<String, String>,
) -> Result<SettingsCandidateBuild, GatewayError> {
    let mut unknown_changes = requested_values
        .keys()
        .filter(|key| !setting_key_allowed(key))
        .cloned()
        .collect::<Vec<_>>();
    unknown_changes.sort();
    if !unknown_changes.is_empty() {
        return Err(GatewayError::Validation(format!(
            "{{\"detail\":{{\"message\":\"Unknown settings keys are read-only unmanaged keys.\",\"keys\":{}}}}}",
            json_owned_string_array(&unknown_changes)
        )));
    }
    let mut read_only_changes = requested_values
        .keys()
        .filter(|key| setting_read_only(key))
        .cloned()
        .collect::<Vec<_>>();
    read_only_changes.sort();
    if !read_only_changes.is_empty() {
        return Err(GatewayError::Validation(format!(
            "{{\"detail\":{{\"message\":\"Read-only settings cannot be changed.\",\"keys\":{}}}}}",
            json_owned_string_array(&read_only_changes)
        )));
    }

    let base = settings_base_values(config, parsed);
    let unmanaged = settings_unmanaged_values(parsed);
    let mut candidate = HashMap::new();
    for definition in SETTING_DEFINITIONS {
        candidate.insert(
            definition.key.to_string(),
            base.get(definition.key).cloned().unwrap_or_default(),
        );
    }
    for (key, value) in requested_values {
        candidate.insert(key.clone(), value.clone());
    }
    let mut changed_keys = candidate
        .iter()
        .filter_map(|(key, value)| {
            if value != base.get(key).unwrap_or(&String::new()) {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    changed_keys.sort();
    Ok((candidate, unmanaged, changed_keys))
}

fn settings_base_values(
    config: &SettingsEnvConfig,
    parsed: &ParsedSettingsEnv,
) -> HashMap<String, String> {
    let mut values = parsed
        .values
        .iter()
        .filter(|(key, _)| setting_key_allowed(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<HashMap<_, _>>();
    values
        .entry("ENV_FILE_PATH".to_string())
        .or_insert_with(|| config.env_file_path.to_string_lossy().to_string());
    values
        .entry("ENV_BACKUP_DIR".to_string())
        .or_insert_with(|| config.backup_dir.to_string_lossy().to_string());
    values
        .entry("IGY6_DATA_ROOT".to_string())
        .or_insert_with(|| config.igy6_data_root.clone());
    values
}

fn settings_unmanaged_values(parsed: &ParsedSettingsEnv) -> HashMap<String, String> {
    parsed
        .unmanaged_order
        .iter()
        .filter_map(|key| {
            parsed
                .values
                .get(key)
                .map(|value| (key.clone(), value.clone()))
        })
        .collect()
}

fn validate_settings_candidate(
    candidate: &HashMap<String, String>,
    unmanaged: &HashMap<String, String>,
    changed_keys: &[String],
) -> SettingsValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for definition in SETTING_DEFINITIONS {
        if !setting_read_only(definition.key)
            && candidate.get(definition.key).is_none_or(String::is_empty)
        {
            errors.push(settings_issue(
                Some(definition.key),
                "Required setting is missing.",
            ));
        }
    }
    for key in SETTINGS_PORT_KEYS {
        match candidate
            .get(*key)
            .and_then(|value| value.parse::<u16>().ok())
        {
            Some(port) if port > 0 => {}
            _ => errors.push(settings_issue(
                Some(key),
                "Port must be between 1 and 65535.",
            )),
        }
    }
    for key in SETTINGS_BOOLEAN_KEYS {
        if !is_settings_bool(candidate.get(*key).map_or("", String::as_str)) {
            errors.push(settings_issue(Some(key), "Boolean must be true or false."));
        }
    }
    for key in SETTINGS_URL_KEYS {
        let value = candidate.get(*key).map_or("", String::as_str);
        if !value.is_empty() && !settings_url_plausible(key, value) {
            errors.push(settings_issue(
                Some(key),
                "URL or URI is not syntactically plausible.",
            ));
        }
    }
    validate_settings_url_agreement(
        candidate,
        "DATABASE_URL",
        "POSTGRES_HOST",
        "POSTGRES_PORT",
        Some("POSTGRES_USER"),
        Some("POSTGRES_PASSWORD"),
        Some("POSTGRES_DB"),
        &mut errors,
    );
    validate_settings_url_agreement(
        candidate,
        "NEO4J_URI",
        "NEO4J_HOST",
        "NEO4J_BOLT_PORT",
        None,
        None,
        None,
        &mut errors,
    );
    validate_settings_url_agreement(
        candidate,
        "QDRANT_URL",
        "QDRANT_HOST",
        "QDRANT_PORT",
        None,
        None,
        None,
        &mut errors,
    );

    for key in SETTINGS_STORAGE_KEYS {
        let value = candidate.get(*key).map_or("", String::as_str);
        if !value.is_empty() && !settings_storage_path_safe(value) {
            errors.push(settings_issue(
                Some(key),
                "Storage path must be absolute and must not contain traversal.",
            ));
        }
    }
    if let Some(issue) = settings_host_data_root_issue(
        candidate
            .get("IGY6_DATA_ROOT")
            .map_or("", std::string::String::as_str),
    ) {
        errors.push(settings_issue(Some("IGY6_DATA_ROOT"), issue));
    }
    let env_path = Path::new(candidate.get("ENV_FILE_PATH").map_or("", String::as_str));
    let backup_dir = Path::new(candidate.get("ENV_BACKUP_DIR").map_or("", String::as_str));
    if !settings_env_paths_are_safe(env_path, backup_dir) {
        errors.push(settings_issue(
            Some("ENV_FILE_PATH"),
            "Settings editor can only target /workspace/project/.env with backups under /workspace/storage.",
        ));
    }
    if !matches!(
        candidate
            .get("EXTERNAL_MODEL_POLICY_DEFAULT")
            .map_or("", String::as_str),
        "blocked" | "metadata_only" | "allowed_with_approval"
    ) {
        errors.push(settings_issue(
            Some("EXTERNAL_MODEL_POLICY_DEFAULT"),
            "External model policy must be blocked, metadata_only, or allowed_with_approval.",
        ));
    }
    if !matches!(
        candidate
            .get("AUDIT_LOG_LEVEL")
            .map_or("", String::as_str)
            .to_ascii_lowercase()
            .as_str(),
        "debug" | "info" | "warning" | "error"
    ) {
        errors.push(settings_issue(
            Some("AUDIT_LOG_LEVEL"),
            "Audit log level must be debug, info, warning, or error.",
        ));
    }
    match candidate
        .get("QDRANT_CHUNK_VECTOR_SIZE")
        .and_then(|value| value.parse::<i64>().ok())
    {
        Some(value) if value > 0 => {
            if changed_keys
                .iter()
                .any(|key| key == "QDRANT_CHUNK_VECTOR_SIZE")
            {
                warnings.push(settings_issue(
                    Some("QDRANT_CHUNK_VECTOR_SIZE"),
                    "Changing vector size can require rebuilding vector storage.",
                ));
            }
        }
        _ => errors.push(settings_issue(
            Some("QDRANT_CHUNK_VECTOR_SIZE"),
            "Vector size must be a positive integer.",
        )),
    }

    let restart_changed = changed_keys
        .iter()
        .filter(|key| {
            SETTINGS_RESTART_KEYS
                .iter()
                .any(|restart| restart == &key.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    let restart_required = !restart_changed.is_empty();
    let mut restart_notes = vec![
        "Saved settings are written to .env only; running containers do not receive them until restart or recreate.".to_string(),
    ];
    if restart_required {
        restart_notes.push(format!(
            "Changed keys likely requiring Docker stack restart/recreate: {}",
            restart_changed.join(", ")
        ));
        add_restart_warnings(&restart_changed, &mut warnings);
    }
    if changed_keys.iter().any(|key| key == "IGY6_DATA_ROOT") {
        warnings.push(settings_issue(
            Some("IGY6_DATA_ROOT"),
            "Changing IGY6_DATA_ROOT requires Docker stack restart/recreate and does not migrate existing data.",
        ));
        warnings.push(settings_issue(
            Some("IGY6_DATA_ROOT"),
            "The target data folder must already exist or be creatable by Docker.",
        ));
    }
    warnings.push(settings_issue(
        None,
        "Rust gateway does not execute Docker Compose from HTTP request handlers; run docker compose config during verification.",
    ));
    let compose_validation = serde_json::json!({
        "available": false,
        "passed": Value::Null,
        "message": "Rust gateway does not execute Docker Compose from HTTP request handlers; run docker compose config during verification."
    });
    let candidate_hash = settings_candidate_hash(candidate, unmanaged);
    SettingsValidation {
        errors,
        warnings,
        changed_keys: changed_keys.to_vec(),
        restart_required,
        restart_notes,
        candidate_hash,
        compose_validation,
    }
}

const SETTINGS_READ_ONLY_KEYS: &[&str] = &["ENV_FILE_PATH", "ENV_BACKUP_DIR"];
const SETTINGS_SECRET_KEYS: &[&str] = &["POSTGRES_PASSWORD", "DATABASE_URL", "NEO4J_PASSWORD"];
const SETTINGS_BOOLEAN_KEYS: &[&str] = &["SINGLE_USER_MODE", "APPROVAL_REQUIRED_DEFAULT"];
const SETTINGS_PORT_KEYS: &[&str] = &[
    "APP_PORT",
    "POSTGRES_PORT",
    "REDIS_PORT",
    "QDRANT_PORT",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "PHOENIX_PORT",
];
const SETTINGS_URL_KEYS: &[&str] = &[
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
const SETTINGS_STORAGE_KEYS: &[&str] = &[
    "ARTIFACT_STORE_PATH",
    "EXPORT_STORE_PATH",
    "ENV_FILE_PATH",
    "ENV_BACKUP_DIR",
];
const SETTINGS_RESTART_KEYS: &[&str] = &[
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
    "EXTERNAL_MODEL_POLICY_DEFAULT",
    "SINGLE_USER_MODE",
    "AUDIT_LOG_LEVEL",
    "APPROVAL_REQUIRED_DEFAULT",
    "ENV_FILE_PATH",
    "ENV_BACKUP_DIR",
    "IGY6_DATA_ROOT",
];

fn setting_key_allowed(key: &str) -> bool {
    SETTING_DEFINITIONS
        .iter()
        .any(|definition| definition.key == key)
}

fn setting_read_only(key: &str) -> bool {
    SETTINGS_READ_ONLY_KEYS
        .iter()
        .any(|candidate| candidate == &key)
}

fn settings_secret_key(key: &str) -> bool {
    if SETTINGS_SECRET_KEYS
        .iter()
        .any(|candidate| candidate == &key)
    {
        return true;
    }
    let upper = key.to_ascii_uppercase();
    if upper.contains("PASSWORD") || upper.contains("TOKEN") || upper.contains("SECRET") {
        return true;
    }
    upper.contains("KEY") && upper != "QDRANT_CHUNK_COLLECTION"
}

fn settings_issue(key: Option<&str>, message: &str) -> SettingsValidationIssue {
    SettingsValidationIssue {
        key: key.map(std::string::ToString::to_string),
        message: message.to_string(),
    }
}

fn is_settings_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on" | "false" | "0" | "no" | "off"
    )
}

fn settings_storage_path_safe(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn settings_env_paths_are_safe(env_path: &Path, backup_dir: &Path) -> bool {
    env_path == Path::new(SAFE_ENV_FILE_PATH) && backup_dir.starts_with(SAFE_BACKUP_ROOT)
}

fn settings_host_data_root_issue(value: &str) -> Option<&'static str> {
    let stripped = value.trim();
    if stripped.is_empty() {
        return Some("IGY6_DATA_ROOT must not be empty.");
    }
    let normalized = stripped.replace('\\', "/");
    if normalized == DEFAULT_IGY6_DATA_ROOT {
        return None;
    }
    if matches!(normalized.as_str(), "/" | "~") {
        return Some("IGY6_DATA_ROOT must point to a dedicated folder, not a filesystem root.");
    }
    let windows_drive_root =
        normalized.len() == 3 && normalized.as_bytes()[1] == b':' && normalized.ends_with('/');
    if windows_drive_root {
        return Some("IGY6_DATA_ROOT must not be a drive root such as C:/ or D:/.");
    }
    if stripped.contains('\\') {
        return Some("Use forward slashes in IGY6_DATA_ROOT, for example D:/Projects/IGY6_Data.");
    }
    let windows_absolute =
        normalized.len() > 3 && normalized.as_bytes()[1] == b':' && &normalized[2..3] == "/";
    let linux_absolute = normalized.starts_with('/');
    if !windows_absolute && !linux_absolute && normalized != DEFAULT_IGY6_DATA_ROOT {
        return Some("Use ../IGY6_Data or an absolute path such as D:/Projects/IGY6_Data or /home/user/IGY6_Data.");
    }
    let parts = normalized
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.iter().any(|part| part == &"..") {
        return Some(
            "IGY6_DATA_ROOT must not contain path traversal, except for the default ../IGY6_Data.",
        );
    }
    None
}

#[derive(Default)]
struct ParsedSettingsUrl {
    scheme: String,
    username: Option<String>,
    password: Option<String>,
    hostname: Option<String>,
    port: Option<u16>,
    path: String,
}

fn parse_settings_url(value: &str) -> ParsedSettingsUrl {
    let Some((scheme, rest)) = value.split_once("://") else {
        return ParsedSettingsUrl::default();
    };
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (userinfo, hostport) = authority
        .rsplit_once('@')
        .map_or((None, authority), |(user, host)| (Some(user), host));
    let (username, password) = userinfo.map_or((None, None), |user| {
        let (name, pass) = user.split_once(':').unwrap_or((user, ""));
        (
            Some(name.to_string()),
            if pass.is_empty() {
                None
            } else {
                Some(pass.to_string())
            },
        )
    });
    let (host, port) = hostport
        .rsplit_once(':')
        .map_or((hostport, None), |(host, port)| {
            (host, port.parse::<u16>().ok())
        });
    ParsedSettingsUrl {
        scheme: scheme.to_string(),
        username,
        password,
        hostname: if host.is_empty() {
            None
        } else {
            Some(host.to_string())
        },
        port,
        path: path.to_string(),
    }
}

fn settings_url_plausible(key: &str, value: &str) -> bool {
    let parsed = parse_settings_url(value);
    match key {
        "NEO4J_URI" => {
            matches!(parsed.scheme.as_str(), "bolt" | "neo4j") && parsed.hostname.is_some()
        }
        "DATABASE_URL" => {
            parsed.scheme.starts_with("postgresql")
                && parsed.hostname.is_some()
                && !parsed.path.trim_matches('/').is_empty()
        }
        "REDIS_URL" | "CELERY_BROKER_URL" | "CELERY_RESULT_BACKEND" => {
            parsed.scheme == "redis" && parsed.hostname.is_some()
        }
        _ => matches!(parsed.scheme.as_str(), "http" | "https") && parsed.hostname.is_some(),
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_settings_url_agreement(
    values: &HashMap<String, String>,
    url_key: &str,
    host_key: &str,
    port_key: &str,
    user_key: Option<&str>,
    password_key: Option<&str>,
    database_key: Option<&str>,
    errors: &mut Vec<SettingsValidationIssue>,
) {
    let raw_url = values.get(url_key).map_or("", String::as_str);
    if raw_url.is_empty() {
        return;
    }
    let parsed = parse_settings_url(raw_url);
    if let Some(hostname) = parsed.hostname {
        if hostname != values.get(host_key).map_or("", String::as_str) {
            errors.push(settings_issue(
                Some(url_key),
                &format!("{url_key} host must match {host_key}."),
            ));
        }
    }
    if let Some(port) = parsed.port {
        if port.to_string() != values.get(port_key).map_or("", String::as_str) {
            errors.push(settings_issue(
                Some(url_key),
                &format!("{url_key} port must match {port_key}."),
            ));
        }
    }
    if let Some(user_key) = user_key {
        if let Some(username) = parsed.username {
            if username != values.get(user_key).map_or("", String::as_str) {
                errors.push(settings_issue(
                    Some(url_key),
                    &format!("{url_key} username must match {user_key}."),
                ));
            }
        }
    }
    if let Some(password_key) = password_key {
        if let Some(password) = parsed.password {
            if password != values.get(password_key).map_or("", String::as_str) {
                errors.push(settings_issue(
                    Some(url_key),
                    &format!("{url_key} password must match {password_key}."),
                ));
            }
        }
    }
    if let Some(database_key) = database_key {
        if parsed.path.trim_matches('/') != values.get(database_key).map_or("", String::as_str) {
            errors.push(settings_issue(
                Some(url_key),
                &format!("{url_key} database name must match {database_key}."),
            ));
        }
    }
}

fn add_restart_warnings(restart_changed: &[String], warnings: &mut Vec<SettingsValidationIssue>) {
    for key in restart_changed {
        let message = if matches!(
            key.as_str(),
            "DATABASE_URL"
                | "POSTGRES_HOST"
                | "POSTGRES_PORT"
                | "POSTGRES_DB"
                | "POSTGRES_USER"
                | "POSTGRES_PASSWORD"
        ) {
            "Database changes may require stack recreate and migration checks."
        } else if key.starts_with("REDIS") || key.starts_with("CELERY") {
            "Redis/Celery changes may require API, worker, and beat restart."
        } else if key.starts_with("QDRANT") {
            "Qdrant changes may require vector collection review."
        } else if key.starts_with("NEO4J") {
            "Neo4j changes may require graph connectivity review."
        } else if key.starts_with("MLFLOW") || key.starts_with("PHOENIX") {
            "Reserved service changes may require stack restart."
        } else if SETTINGS_STORAGE_KEYS
            .iter()
            .any(|storage_key| storage_key == &key.as_str())
        {
            "Storage path changes may require mounted volume review."
        } else if key == "IGY6_DATA_ROOT" {
            "Host data-root changes may require moving data manually while the stack is stopped."
        } else {
            continue;
        };
        warnings.push(settings_issue(Some(key), message));
    }
}

fn settings_verify_response_json(
    candidate: &HashMap<String, String>,
    validation: &SettingsValidation,
) -> String {
    let token = if validation.errors.is_empty() {
        Some(validation.candidate_hash.as_str())
    } else {
        None
    };
    format!(
        "{{\"passed\":{},\"errors\":{},\"warnings\":{},\"normalized_candidate\":{},\"changed_keys\":{},\"restart_required\":{},\"restart_notes\":{},\"verification_token\":{},\"candidate_hash\":{},\"expires_at\":null,\"compose_validation\":{}}}",
        validation.errors.is_empty(),
        validation_issues_json(&validation.errors),
        validation_issues_json(&validation.warnings),
        sanitize_settings_json(candidate, "candidate"),
        json_owned_string_array(&validation.changed_keys),
        validation.restart_required,
        json_owned_string_array(&validation.restart_notes),
        option_string_json(token),
        option_string_json(token),
        validation.compose_validation
    )
}

fn settings_env_response_from_values(
    values: &HashMap<String, String>,
    unmanaged: &HashMap<String, String>,
    config: &SettingsEnvConfig,
    parsed: &ParsedSettingsEnv,
) -> String {
    let groups = SETTING_GROUPS
        .iter()
        .map(|(key, label)| {
            format!(
                "{{\"key\":\"{}\",\"label\":\"{}\"}}",
                escape_json(key),
                escape_json(label)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let unmanaged_json = parsed
        .unmanaged_order
        .iter()
        .filter_map(|key| unmanaged.get(key).map(|value| (key, value)))
        .map(|(key, value)| {
            let secret = settings_secret_key(key);
            let masked = if secret && !value.is_empty() {
                MASKED_VALUE
            } else {
                value
            };
            format!(
                "{{\"key\":\"{}\",\"masked_value\":\"{}\",\"has_value\":{},\"secret\":{},\"read_only\":true}}",
                escape_json(key),
                escape_json(masked),
                !value.is_empty(),
                secret
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let warnings = if parsed.unmanaged_order.is_empty() {
        "[]".to_string()
    } else {
        "[\"Unknown .env keys are preserved as read-only unmanaged settings.\"]".to_string()
    };
    format!(
        "{{\"file_status\":{{\"path\":\"{}\",\"backup_dir\":\"{}\",\"exists\":{},\"writable\":{},\"unknown_key_count\":{},\"output_format\":\"normalized_env\"}},\"groups\":[{}],\"settings\":{},\"unmanaged\":[{}],\"warnings\":{}}}",
        escape_json(&config.env_file_path.to_string_lossy()),
        escape_json(&config.backup_dir.to_string_lossy()),
        config.env_file_path.exists(),
        settings_path_writable(&config.env_file_path),
        parsed.unmanaged_order.len(),
        groups,
        sanitize_settings_json(values, "env"),
        unmanaged_json,
        warnings
    )
}

fn sanitize_settings_json(values: &HashMap<String, String>, source: &str) -> String {
    let items = SETTING_DEFINITIONS
        .iter()
        .map(|definition| {
            let value = values.get(definition.key).map(String::as_str);
            let secret = settings_secret_key(definition.key);
            let has_value = value.is_some_and(|value| !value.is_empty());
            let value_json = if secret {
                "null".to_string()
            } else {
                option_string_json(value)
            };
            let masked_value = if secret && has_value {
                MASKED_VALUE
            } else {
                ""
            };
            format!(
                "{{\"key\":\"{}\",\"group\":\"{}\",\"group_label\":\"{}\",\"description\":\"{}\",\"value\":{},\"masked_value\":\"{}\",\"has_value\":{},\"secret\":{},\"read_only\":{},\"restart_required\":{},\"source\":\"{}\"}}",
                escape_json(definition.key),
                escape_json(definition.group),
                escape_json(settings_group_label(definition.group)),
                escape_json(definition.description),
                value_json,
                escape_json(masked_value),
                has_value,
                secret,
                setting_read_only(definition.key),
                SETTINGS_RESTART_KEYS.iter().any(|key| key == &definition.key),
                escape_json(source)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn validation_issues_json(issues: &[SettingsValidationIssue]) -> String {
    let items = issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"key\":{},\"message\":\"{}\"}}",
                option_string_json(issue.key.as_deref()),
                escape_json(&issue.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn render_settings_env_content(
    values: &HashMap<String, String>,
    unmanaged: &HashMap<String, String>,
) -> String {
    let mut lines = vec![
        "# IGY6 local environment".to_string(),
        "# Generated by the local Settings dry-run/apply workflow.".to_string(),
        "# Comments from previous .env content are normalized by this writer.".to_string(),
        String::new(),
    ];
    let mut current_group = "";
    for definition in SETTING_DEFINITIONS {
        if current_group != definition.group {
            if !current_group.is_empty() {
                lines.push(String::new());
            }
            lines.push(format!("# {}", settings_group_label(definition.group)));
            current_group = definition.group;
        }
        lines.push(format!(
            "{}={}",
            definition.key,
            values.get(definition.key).map_or("", String::as_str)
        ));
    }
    if !unmanaged.is_empty() {
        lines.push(String::new());
        lines.push("# Unmanaged keys preserved read-only".to_string());
        let mut keys = unmanaged.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            lines.push(format!(
                "{}={}",
                key,
                unmanaged.get(&key).map_or("", String::as_str)
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn settings_candidate_hash(
    values: &HashMap<String, String>,
    unmanaged: &HashMap<String, String>,
) -> String {
    sha256_hex(render_settings_env_content(values, unmanaged).as_bytes())
}

fn sha256_hex(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn create_settings_env_backup(env_path: &Path, backup_dir: &Path) -> Result<PathBuf, GatewayError> {
    if !env_path.exists() {
        return Err(GatewayError::Conflict(
            "{\"detail\":\"Cannot back up missing .env file\"}".to_string(),
        ));
    }
    fs::create_dir_all(backup_dir).map_err(|error| GatewayError::Database(error.to_string()))?;
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| GatewayError::Database(error.to_string()))?
        .as_secs();
    let mut backup_path = backup_dir.join(format!(".env.{seconds}.bak"));
    let mut counter = 1;
    while backup_path.exists() {
        backup_path = backup_dir.join(format!(".env.{seconds}.{counter}.bak"));
        counter += 1;
    }
    fs::copy(env_path, &backup_path).map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(backup_path)
}

fn atomic_write_settings_env(env_path: &Path, content: &str) -> Result<(), GatewayError> {
    if let Some(parent) = env_path.parent() {
        fs::create_dir_all(parent).map_err(|error| GatewayError::Database(error.to_string()))?;
    }
    let temp_path = env_path.with_extension("tmp");
    fs::write(&temp_path, content).map_err(|error| GatewayError::Database(error.to_string()))?;
    fs::rename(&temp_path, env_path).map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(())
}

fn settings_path_writable(path: &Path) -> bool {
    if path.exists() {
        fs::OpenOptions::new().append(true).open(path).is_ok()
    } else {
        path.parent().is_some_and(Path::exists)
    }
}

fn settings_group_label(group: &str) -> &str {
    SETTING_GROUPS
        .iter()
        .find_map(|(key, label)| if *key == group { Some(*label) } else { None })
        .unwrap_or(group)
}

fn load_source_for_collection(
    transaction: &mut postgres::Transaction<'_>,
    source_id: &str,
) -> Result<CollectionSource, GatewayError> {
    let Some(row) = transaction
        .query_opt(
            "SELECT id, name, source_type, location, sensitivity, enabled, metadata_json FROM sources WHERE id = $1",
            &[&source_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Source not found".to_string()));
    };
    Ok(CollectionSource {
        id: row.get(0),
        name: row.get(1),
        source_type: row.get(2),
        location: row.get(3),
        sensitivity: row.get(4),
        enabled: row.get(5),
        metadata_json: row.get(6),
    })
}

fn load_permission_for_collection(
    transaction: &mut postgres::Transaction<'_>,
    permission_id: &str,
) -> Result<CollectionPermission, GatewayError> {
    let Some(row) = transaction
        .query_opt(
            "SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required FROM source_permissions WHERE id = $1",
            &[&permission_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound(
            "Source permission not found".to_string(),
        ));
    };
    let allowed_operations = json_value_string_array(row.get(3), "allowed_operations")?;
    Ok(CollectionPermission {
        id: row.get(0),
        source_id: row.get(1),
        scope_json: row.get(2),
        allowed_operations,
        external_model_policy: row.get(4),
        approval_required: row.get(5),
    })
}

fn load_collection_approval(
    transaction: &mut postgres::Transaction<'_>,
    approval_id: &str,
) -> Result<CollectionApproval, GatewayError> {
    let Some(row) = transaction
        .query_opt(
            "SELECT id, status, request_type, request_payload_json FROM approvals WHERE id = $1",
            &[&approval_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Approval not found".to_string()));
    };
    Ok(CollectionApproval {
        id: row.get(0),
        status: row.get(1),
        request_type: row.get(2),
        request_payload_json: row.get(3),
    })
}

fn json_value_string_array(value: Value, field_name: &str) -> Result<Vec<String>, GatewayError> {
    let Some(items) = value.as_array() else {
        return Err(GatewayError::Database(format!(
            "{field_name} must be an array of strings"
        )));
    };
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        let Some(item) = item.as_str() else {
            return Err(GatewayError::Database(format!(
                "{field_name} must be an array of strings"
            )));
        };
        strings.push(item.to_string());
    }
    Ok(strings)
}

fn permission_allows(allowed_operations: &[String], allowed: &[&str]) -> bool {
    allowed_operations
        .iter()
        .any(|operation| allowed.iter().any(|candidate| operation == candidate))
}

fn require_collection_approval(
    transaction: &mut postgres::Transaction<'_>,
    approval_id: Option<&str>,
    source: &CollectionSource,
    permission: &CollectionPermission,
    operation: &str,
) -> Result<Option<String>, GatewayError> {
    if !permission.approval_required {
        return Ok(None);
    }
    let approval_id = approval_id.ok_or_else(|| {
        GatewayError::Forbidden("Collection requires an approved approval record".to_string())
    })?;
    let approval = load_collection_approval(transaction, approval_id)?;
    if approval.status != "approved" {
        return Err(GatewayError::Forbidden(
            "Approval is not approved".to_string(),
        ));
    }
    if !matches!(
        approval.request_type.as_str(),
        "source_collection" | "manual_upload_collection" | "local_project_collection"
    ) {
        return Err(GatewayError::Conflict(
            "Approval is not for source collection".to_string(),
        ));
    }
    approval_payload_matches(&approval.request_payload_json, "source_id", &source.id)?;
    approval_payload_matches(
        &approval.request_payload_json,
        "source_permission_id",
        &permission.id,
    )?;
    approval_payload_matches(&approval.request_payload_json, "operation", operation)?;
    Ok(Some(approval.id))
}

fn approval_payload_matches(
    payload: &Value,
    key: &str,
    expected: &str,
) -> Result<(), GatewayError> {
    let Some(value) = payload.get(key) else {
        return Err(GatewayError::Conflict(format!(
            "Approval is missing required {key} for requested collection"
        )));
    };
    if value.as_str() == Some(expected) {
        Ok(())
    } else {
        Err(GatewayError::Conflict(format!(
            "Approval {key} does not match requested collection"
        )))
    }
}

fn connector_dry_run_result(
    source: &CollectionSource,
    permission: &CollectionPermission,
) -> Result<CollectionDryRunConnectorResult, String> {
    if permission.source_id != source.id {
        return Err("Source permission does not belong to the source".to_string());
    }
    let connector_name = match source.source_type.as_str() {
        "local_project" => "local_project",
        "manual_upload" => "manual_upload",
        _ => {
            return Err(format!(
                "No connector registered for source type: {}",
                source.source_type
            ))
        }
    };
    if !permission.allowed_operations.is_empty()
        && !permission_allows(&permission.allowed_operations, &["dry_run", "read"])
    {
        return Err(format!(
            "{} sources must allow dry_run or read operations",
            source.source_type
        ));
    }

    Ok(CollectionDryRunConnectorResult {
        connector_name: connector_name.to_string(),
        allowed: true,
        summary: format!(
            "{} dry-run validated source and permission metadata only.",
            source.name
        ),
        estimated_items: None,
        warnings: Vec::new(),
        metadata: serde_json::json!({
            "source_type": source.source_type.clone(),
            "source_location": source.location.clone(),
            "source_metadata": source.metadata_json.clone(),
            "permission_id": permission.id.clone(),
            "permission_scope": permission.scope_json.clone(),
            "allowed_operations": permission.allowed_operations.clone(),
            "external_model_policy": permission.external_model_policy.clone(),
            "approval_required": permission.approval_required,
            "preview_only": true
        }),
    })
}

fn collection_dry_run_summary(
    source: &CollectionSource,
    permission: &CollectionPermission,
    result: Option<&CollectionDryRunConnectorResult>,
    notes: &Value,
) -> Value {
    let connector_result = result.map(|result| {
        serde_json::json!({
            "connector_name": result.connector_name.clone(),
            "allowed": result.allowed,
            "summary": result.summary.clone(),
            "estimated_items": result.estimated_items,
            "warnings": result.warnings.clone(),
            "metadata": result.metadata.clone()
        })
    });
    serde_json::json!({
        "source": {
            "id": source.id.clone(),
            "name": source.name.clone(),
            "source_type": source.source_type.clone(),
            "sensitivity": source.sensitivity.clone(),
            "enabled": source.enabled
        },
        "permission": {
            "id": permission.id.clone(),
            "allowed_operations": permission.allowed_operations.clone(),
            "scope": permission.scope_json.clone(),
            "external_model_policy": permission.external_model_policy.clone(),
            "approval_required": permission.approval_required
        },
        "preview": {
            "mode": "connector_dry_run_preview",
            "would_collect": false,
            "would_create_artifacts": false,
            "would_normalize": false,
            "would_enqueue_worker": false
        },
        "connector_result": connector_result,
        "notes": notes.clone()
    })
}

fn artifact_data_root() -> PathBuf {
    if let Ok(artifact_store_path) = env::var("ARTIFACT_STORE_PATH") {
        let path = PathBuf::from(artifact_store_path);
        if path.file_name().is_some_and(|name| name == "artifacts") {
            return path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("/workspace/storage"));
        }
        return path;
    }
    if let Ok(path) = env::var("IGY6_ARTIFACT_DATA_ROOT") {
        return PathBuf::from(path);
    }
    if let Ok(path) = env::var("IGY6_DATA_ROOT") {
        let candidate = PathBuf::from(path);
        if !candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return candidate;
        }
    }
    PathBuf::from("/workspace/storage")
}

fn decode_base64(value: &str) -> Result<Vec<u8>, GatewayError> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 4];
    let mut buffer_len = 0;
    let mut saw_padding = false;
    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        let decoded = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                saw_padding = true;
                64
            }
            _ => {
                return Err(GatewayError::Validation(
                    "Invalid base64 content".to_string(),
                ))
            }
        };
        if saw_padding && decoded != 64 {
            return Err(GatewayError::Validation(
                "Invalid base64 content".to_string(),
            ));
        }
        buffer[buffer_len] = decoded;
        buffer_len += 1;
        if buffer_len == 4 {
            decode_base64_quad(&buffer, &mut output)?;
            buffer_len = 0;
        }
    }
    if buffer_len != 0 {
        return Err(GatewayError::Validation(
            "Invalid base64 content".to_string(),
        ));
    }
    Ok(output)
}

fn decode_base64_quad(quad: &[u8; 4], output: &mut Vec<u8>) -> Result<(), GatewayError> {
    if quad[0] == 64 || quad[1] == 64 {
        return Err(GatewayError::Validation(
            "Invalid base64 content".to_string(),
        ));
    }
    output.push((quad[0] << 2) | (quad[1] >> 4));
    match (quad[2], quad[3]) {
        (64, 64) => Ok(()),
        (64, _) => Err(GatewayError::Validation(
            "Invalid base64 content".to_string(),
        )),
        (third, 64) => {
            output.push((quad[1] << 4) | (third >> 2));
            Ok(())
        }
        (third, fourth) => {
            output.push((quad[1] << 4) | (third >> 2));
            output.push((third << 6) | fourth);
            Ok(())
        }
    }
}

fn require_utf8_text_content(content: &[u8]) -> Result<(), GatewayError> {
    let text = std::str::from_utf8(content).map_err(|_| {
        GatewayError::Validation(
            "Manual upload normalization currently supports UTF-8 text artifacts only".to_string(),
        )
    })?;
    if text.trim().is_empty() {
        return Err(GatewayError::Validation(
            "Manual upload content is empty".to_string(),
        ));
    }
    Ok(())
}

fn require_supported_text_mime_type(mime_type: Option<&str>) -> Result<(), GatewayError> {
    let Some(mime_type) = mime_type else {
        return Ok(());
    };
    if mime_type.trim().is_empty() {
        return Ok(());
    }
    let normalized = mime_type
        .split_once(';')
        .map(|(value, _)| value)
        .unwrap_or(mime_type)
        .trim()
        .to_ascii_lowercase();
    if normalized.starts_with("text/") || normalized == "application/json" {
        Ok(())
    } else {
        Err(GatewayError::Validation(
            "Unsupported manual upload file type; this ingestion path supports UTF-8 text only"
                .to_string(),
        ))
    }
}

fn validate_safe_filename(filename: &str) -> Result<(), GatewayError> {
    if filename.contains('/')
        || filename.contains('\\')
        || filename.contains('\0')
        || filename == "."
        || filename == ".."
    {
        return Err(GatewayError::Validation(
            "filename must not contain path traversal.".to_string(),
        ));
    }
    Ok(())
}

fn manual_upload_artifact_metadata(
    request_metadata: &Value,
    filename: Option<&str>,
    permission_id: &str,
    approval_id: Option<&str>,
) -> Value {
    let mut metadata = request_metadata.as_object().cloned().unwrap_or_default();
    metadata.insert(
        "filename".to_string(),
        filename.map_or(Value::Null, |value| Value::String(value.to_string())),
    );
    metadata.insert(
        "source_permission_id".to_string(),
        Value::String(permission_id.to_string()),
    );
    metadata.insert(
        "approval_id".to_string(),
        approval_id.map_or(Value::Null, |value| Value::String(value.to_string())),
    );
    Value::Object(metadata)
}

fn manual_upload_normalization_work_payload(
    collection_run_id: &str,
    source: &CollectionSource,
    permission_id: &str,
    raw_artifact_id: &str,
) -> Value {
    serde_json::json!({
        "collection_run_id": collection_run_id,
        "source_id": source.id,
        "source_permission_id": permission_id,
        "raw_artifact_ids": [raw_artifact_id],
        "artifact_count": 1,
        "collection_mode": "manual_upload_collection",
        "scaffold_only": false,
        "executes_normalization": true,
        "worker_task_name": "collection.normalize_collection_run",
        "normalization_input_type": "utf_8_text",
        "intent_verification_recorded": true,
        "intent_verification": {
            "original_request": "Queue normalization for manual_upload_collection",
            "interpretation": "Create a queued worker item to normalize collected UTF-8 text artifacts.",
            "proposed_work_type": "collection_normalization",
            "sources_likely_used": [source.id.clone()],
            "expected_output": "Normalized document records for the collected raw artifacts.",
            "safety_requirements": [
                "Use only stored local artifacts linked to this collection run.",
                "Do not perform external model calls or system-changing actions."
            ],
            "assumptions": ["Collected artifacts are UTF-8 text artifacts supported by the current worker."],
            "missing_information": [],
            "recorded_by": "DIFF-074 collection enqueue governance"
        }
    })
}

fn agent_action_execute_path(path: &str) -> Option<String> {
    let stripped = path.strip_prefix("/agent/actions/")?;
    let action_name = stripped.strip_suffix("/execute")?;
    if action_name.is_empty() || action_name.contains('/') {
        return None;
    }
    Some(percent_decode_path_segment(action_name))
}

fn pattern_review_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/analysis/patterns/", "/review")
}

fn approval_decision_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/approvals/", "/decision")
}

fn report_render_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/reports/", "/render")
}

fn work_item_dispatch_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/work-items/", "/dispatch")
}

fn dynamic_post_id_path(path: &str, prefix: &str, suffix: &str) -> Option<String> {
    let id = path.strip_prefix(prefix)?.strip_suffix(suffix)?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    Some(percent_decode_path_segment(id))
}

fn validate_route_id(value: &str, field_name: &str) -> Result<String, GatewayError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GatewayError::Validation(format!(
            "{field_name} must not be empty."
        )));
    }
    if trimmed.len() > 128 {
        return Err(GatewayError::Validation(format!(
            "{field_name} must be 128 characters or fewer."
        )));
    }
    if trimmed.contains('/') || trimmed.contains("..") {
        return Err(GatewayError::Validation(format!(
            "{field_name} contains invalid path characters."
        )));
    }
    Ok(trimmed.to_string())
}

fn percent_decode_path_segment(value: &str) -> String {
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).to_string()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn validate_action_name(value: &str) -> Result<String, GatewayError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GatewayError::Validation(
            "action_name is required.".to_string(),
        ));
    }
    if trimmed.len() > 128
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(GatewayError::Validation(
            "action_name must be a fixed registry identifier.".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn reject_user_provided_argv(parameters: &Value) -> Result<(), GatewayError> {
    let Some(object) = parameters.as_object() else {
        return Err(GatewayError::Validation(
            "parameters must be a JSON object.".to_string(),
        ));
    };
    for key in object.keys() {
        let lowered = key.to_ascii_lowercase();
        if matches!(
            lowered.as_str(),
            "argv" | "args" | "command" | "cmd" | "shell" | "script"
        ) {
            return Err(GatewayError::Validation(
                "Agent actions do not accept user-provided argv or command fields.".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_required_action_parameters(
    definition: &AgentActionDefinition,
    parameters: &Value,
) -> Result<(), GatewayError> {
    let object = parameters
        .as_object()
        .ok_or_else(|| GatewayError::Validation("parameters must be a JSON object.".to_string()))?;
    let missing = definition
        .required_parameters
        .iter()
        .filter(|parameter| {
            object
                .get(**parameter)
                .is_none_or(|value| value.as_str().is_none_or(|text| text.trim().is_empty()))
        })
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(GatewayError::Validation(format!(
            "Missing required action parameters: {}",
            missing.join(", ")
        )))
    }
}

fn classify_agent_message(message: &str, parameters: &Value) -> Option<&'static str> {
    let response = classify_agent_intent(&AgentIntentRequest {
        message: message.to_string(),
        parameters: value_object_to_string_pairs(parameters),
    });
    response.proposed_action
}

fn agent_intent_json_from_parts(message: &str, parameters: &Value) -> String {
    let response = classify_agent_intent(&AgentIntentRequest {
        message: message.to_string(),
        parameters: value_object_to_string_pairs(parameters),
    });
    agent_intent_response_json(&response)
}

fn agent_action_definition_json(definition: &AgentActionDefinition, parameters: &Value) -> String {
    let missing_parameters = definition
        .required_parameters
        .iter()
        .copied()
        .filter(|parameter| {
            parameters
                .get(*parameter)
                .is_none_or(|value| value.as_str().is_none_or(|text| text.trim().is_empty()))
        })
        .collect::<Vec<_>>();
    let response = igy6_agent_api::AgentIntentResponse {
        original_message: definition.name.to_string(),
        interpreted_intent: definition.interpreted_intent.to_string(),
        proposed_action: Some(definition.name),
        action_type: definition.action_type.clone(),
        approval_required: definition.approval_required,
        risk_level: definition.risk_level.clone(),
        required_parameters: definition.required_parameters.to_vec(),
        missing_parameters: missing_parameters.clone(),
        safety_notes: definition.safety_notes.to_vec(),
        executable_now: missing_parameters.is_empty() && !definition.approval_required,
        reason: if definition.approval_required {
            Some("Approval required before execution.".to_string())
        } else {
            None
        },
    };
    agent_intent_response_json(&response)
}

fn value_object_to_string_pairs(value: &Value) -> Vec<(String, String)> {
    value
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter())
        .filter_map(|(key, value)| {
            let text = value.as_str().map(str::to_string).or_else(|| {
                if value.is_null() {
                    None
                } else {
                    Some(value.to_string())
                }
            })?;
            Some((key.clone(), text))
        })
        .collect()
}

fn execute_known_agent_action(
    definition: &AgentActionDefinition,
    parameters: &Value,
    client: &mut Client,
) -> Result<AgentActionExecutionResult, GatewayError> {
    match definition.name {
        "show_project_health" => Ok(AgentActionExecutionResult {
            status: "completed".to_string(),
            result: serde_json::json!({
                "health": {
                    "status": "ok",
                    "primary_gateway": "rust",
                    "service": "igy6-gateway"
                }
            }),
            stdout_summary: None,
            stderr_summary: None,
            exit_code: None,
        }),
        "show_git_status" => Ok(AgentActionExecutionResult {
            status: "completed".to_string(),
            result: serde_json::json!({"git": git_status_from_files(repo_root_path())}),
            stdout_summary: None,
            stderr_summary: None,
            exit_code: None,
        }),
        "show_latest_diff" => Ok(AgentActionExecutionResult {
            status: "completed".to_string(),
            result: serde_json::json!({"latest_diff": latest_diff_metadata(repo_root_path())}),
            stdout_summary: None,
            stderr_summary: None,
            exit_code: None,
        }),
        "show_work_items" => Ok(AgentActionExecutionResult {
            status: "completed".to_string(),
            result: serde_json::json!({"work_items": recent_work_items(client)?}),
            stdout_summary: None,
            stderr_summary: None,
            exit_code: None,
        }),
        "run_retrieval_preview" => {
            let message = parameters
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Ok(AgentActionExecutionResult {
                status: "completed".to_string(),
                result: serde_json::json!({
                    "retrieval_preview": serde_json::from_str::<Value>(&retrieval_preview_json(
                        &serde_json::json!({"message": message}).to_string(),
                    ))
                    .unwrap_or_else(|_| serde_json::json!({}))
                }),
                stdout_summary: None,
                stderr_summary: None,
                exit_code: None,
            })
        }
        "start_stack" | "stop_stack" | "run_last_healthy_stack" => {
            execute_host_bridge_action(definition.name)
        }
        _ => Err(GatewayError::NotFound("Unknown agent action".to_string())),
    }
}

fn require_agent_action_approval(
    client: &mut Client,
    definition: &AgentActionDefinition,
    payload: &AgentActionExecutePayload,
) -> Result<(), GatewayError> {
    let approval_id = payload
        .approval_id
        .as_deref()
        .ok_or_else(|| GatewayError::Forbidden("Agent action requires approval".to_string()))?;
    let Some(row) = client
        .query_opt(
            "SELECT id, status, request_type, request_payload_json FROM approvals WHERE id = $1",
            &[&approval_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Approval not found".to_string()));
    };
    let status: String = row.get(1);
    let request_type: String = row.get(2);
    let request_payload: Value = row.get(3);
    if status != "approved" {
        return Err(GatewayError::Forbidden(
            "Approval is not approved".to_string(),
        ));
    }
    if request_type != "agent_action" {
        return Err(GatewayError::Conflict(
            "Approval is not for an agent action".to_string(),
        ));
    }
    approval_payload_matches(&request_payload, "action_name", definition.name)?;
    if let Some(approved_parameters) = request_payload.get("parameters") {
        if approved_parameters != &payload.parameters {
            return Err(GatewayError::Conflict(
                "Approval parameters do not match".to_string(),
            ));
        }
    }
    Ok(())
}

fn insert_agent_action_audit(
    client: &mut Client,
    actor_id: &str,
    event_type: &str,
    decision: &str,
    action_name: &str,
    correlation_id: Option<String>,
    details_json: Value,
) -> Result<i32, GatewayError> {
    client
        .query_one(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, $3, 'agent_action', $4, $5, $6::jsonb) RETURNING id",
            &[
                &actor_id,
                &event_type,
                &decision,
                &action_name,
                &correlation_id,
                &details_json,
            ],
        )
        .map(|row| row.get::<_, i32>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn safe_parameter_summary(parameters: &Value) -> Value {
    let Some(object) = parameters.as_object() else {
        return serde_json::json!({});
    };
    let mut safe = serde_json::Map::new();
    for (key, value) in object {
        let lowered = key.to_ascii_lowercase();
        if ["password", "token", "secret", "key"]
            .iter()
            .any(|needle| lowered.contains(needle))
        {
            safe.insert(key.clone(), Value::String("[redacted]".to_string()));
        } else {
            safe.insert(key.clone(), value.clone());
        }
    }
    Value::Object(safe)
}

fn repo_root_path() -> PathBuf {
    env::var("ENV_FILE_PATH")
        .ok()
        .and_then(|value| PathBuf::from(value).parent().map(Path::to_path_buf))
        .filter(|candidate| candidate.join("AGENTS.md").is_file())
        .or_else(|| {
            let candidate = PathBuf::from("/workspace/project");
            candidate.join("AGENTS.md").is_file().then_some(candidate)
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

fn git_status_from_files(root: PathBuf) -> Value {
    let git_dir = root.join(".git");
    let head_path = git_dir.join("HEAD");
    if !head_path.is_file() {
        return serde_json::json!({
            "branch": "unknown",
            "commit": "unknown",
            "dirty": null,
            "changed_path_count": null,
            "status_source": "unavailable",
            "note": "Git metadata is unavailable in this runtime."
        });
    }
    let head_value = fs::read_to_string(&head_path)
        .unwrap_or_default()
        .trim()
        .to_string();
    let (branch, commit) = if let Some(ref_name) = head_value.strip_prefix("ref: ") {
        let ref_name = ref_name.trim();
        let branch = ref_name.trim_start_matches("refs/heads/").to_string();
        let commit = fs::read_to_string(git_dir.join(ref_name))
            .ok()
            .map(|value| value.trim().to_string())
            .or_else(|| read_packed_ref(&git_dir.join("packed-refs"), ref_name))
            .unwrap_or_else(|| "unknown".to_string());
        (branch, commit)
    } else {
        ("detached".to_string(), head_value)
    };
    serde_json::json!({
        "branch": branch,
        "commit": commit,
        "dirty": null,
        "changed_path_count": null,
        "status_source": "git_files",
        "note": "Rust gateway does not execute git; dirty state is unavailable from raw metadata."
    })
}

fn read_packed_ref(path: &Path, ref_name: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        let (commit, packed_ref_name) = line.split_once(' ')?;
        if packed_ref_name == ref_name {
            return Some(commit.to_string());
        }
    }
    None
}

fn latest_diff_metadata(root: PathBuf) -> Value {
    let diff_dir = root.join("docs").join("diffs");
    let Ok(entries) = fs::read_dir(&diff_dir) else {
        return serde_json::json!({"path": null, "status": null});
    };
    let latest = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DIFF-") && name.ends_with(".md"))
        })
        .max_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    let Some(latest) = latest else {
        return serde_json::json!({"path": null, "status": null});
    };
    let status = fs::read_to_string(&latest).ok().and_then(|content| {
        content.lines().find_map(|line| {
            line.to_ascii_lowercase()
                .starts_with("status:")
                .then(|| {
                    line.split_once(':')
                        .map(|(_, value)| value.trim().to_string())
                })
                .flatten()
        })
    });
    let relative = latest
        .strip_prefix(&root)
        .ok()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| latest.clone());
    serde_json::json!({"path": relative.to_string_lossy(), "status": status})
}

fn recent_work_items(client: &mut Client) -> Result<Value, GatewayError> {
    client
        .query_one(
            "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, work_type, status, requested_by_actor_id, error_message FROM work_items ORDER BY created_at DESC LIMIT 20) t), '[]')",
            &[],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))
        .and_then(|row| {
            let text = row.get::<_, String>(0);
            serde_json::from_str(&text)
                .map_err(|error| GatewayError::Database(error.to_string()))
        })
}

fn execute_host_bridge_action(
    action_name: &str,
) -> Result<AgentActionExecutionResult, GatewayError> {
    if host_bridge_allowed_action(action_name).is_none() {
        return Err(GatewayError::NotFound("Unknown agent action".to_string()));
    }
    let token = host_bridge_token()?;
    let host = env::var("IGY6_HOST_BRIDGE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    if host != "127.0.0.1" {
        return Err(GatewayError::Conflict(
            "Host bridge must be configured for 127.0.0.1 only".to_string(),
        ));
    }
    let port = env::var("IGY6_HOST_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8765);
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|error| GatewayError::Conflict(format!("Host bridge is unavailable: {error}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(310)))
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let request = format!(
        "POST /actions/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        action_name,
        port,
        token
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let (head, body) = response.split_once("\r\n\r\n").unwrap_or(("", ""));
    let status_code = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(502);
    let body_json: Value = serde_json::from_str(body).unwrap_or_else(|_| {
        serde_json::json!({"error": "invalid_host_bridge_response", "detail": "Host bridge returned non-JSON output"})
    });
    if status_code != 200 {
        return Err(GatewayError::Conflict(format!(
            "Host bridge rejected action: {}",
            body_json
                .get("detail")
                .and_then(Value::as_str)
                .unwrap_or("unknown host bridge error")
        )));
    }
    let stdout_summary = body_json
        .get("stdout_summary")
        .and_then(Value::as_str)
        .map(redact_output);
    let stderr_summary = body_json
        .get("stderr_summary")
        .and_then(Value::as_str)
        .map(redact_output);
    let exit_code = body_json
        .get("exit_code")
        .and_then(Value::as_i64)
        .map(|value| value as i32);
    let status = match body_json.get("status").and_then(Value::as_str) {
        Some("completed") => "completed",
        Some("timed_out") => "failed",
        Some("failed") => "failed",
        _ => "failed",
    }
    .to_string();
    Ok(AgentActionExecutionResult {
        status,
        result: serde_json::json!({
            "host_bridge": {
                "status": body_json.get("status").cloned().unwrap_or(Value::Null),
                "bridge_version": body_json.get("bridge_version").cloned().unwrap_or(Value::Null)
            }
        }),
        stdout_summary,
        stderr_summary,
        exit_code,
    })
}

fn host_bridge_token() -> Result<String, GatewayError> {
    if let Ok(token) = env::var("IGY6_HOST_BRIDGE_TOKEN") {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    if let Ok(path) = env::var("IGY6_HOST_BRIDGE_TOKEN_FILE") {
        let token = fs::read_to_string(&path)
            .map_err(|_| GatewayError::Conflict("Host bridge token is unavailable".to_string()))?;
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Err(GatewayError::Conflict(
        "Host bridge token is unavailable".to_string(),
    ))
}

fn now_epoch_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
        .to_string()
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

fn required_text_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
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

fn optional_string_field_with_max(
    object: &serde_json::Map<String, Value>,
    key: &str,
    default: &str,
    max_len: usize,
) -> Result<String, GatewayError> {
    let value = optional_string_field(object, key, default)?;
    if value.len() > max_len {
        return Err(GatewayError::Validation(format!(
            "{key} must be {max_len} characters or fewer."
        )));
    }
    Ok(value)
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

fn optional_nullable_string_field_with_max(
    object: &serde_json::Map<String, Value>,
    key: &str,
    max_len: usize,
) -> Result<Option<String>, GatewayError> {
    let value = optional_nullable_string_field(object, key)?;
    if value.as_ref().is_some_and(|value| value.len() > max_len) {
        return Err(GatewayError::Validation(format!(
            "{key} must be {max_len} characters or fewer."
        )));
    }
    Ok(value)
}

fn optional_bool_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, GatewayError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(GatewayError::Validation(format!(
            "{key} must be a boolean."
        ))),
    }
}

fn optional_i32_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    min: i32,
    max: i32,
) -> Result<Option<i32>, GatewayError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => {
            let Some(value) = number.as_i64() else {
                return Err(GatewayError::Validation(format!(
                    "{key} must be an integer."
                )));
            };
            if value < min as i64 || value > max as i64 {
                return Err(GatewayError::Validation(format!(
                    "{key} must be between {min} and {max}."
                )));
            }
            Ok(Some(value as i32))
        }
        Some(_) => Err(GatewayError::Validation(format!(
            "{key} must be an integer."
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

fn required_string_array_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>, GatewayError> {
    let result = optional_string_array_field(object, key)?;
    if result.is_empty() {
        return Err(GatewayError::Validation(format!(
            "{key} must contain at least one ID."
        )));
    }
    Ok(result)
}

fn is_source_type(value: &str) -> bool {
    matches!(
        value,
        "manual_upload"
            | "local_project"
            | "local_pc_diagnostics"
            | "web_public"
            | "web_authorized_account"
            | "router_network"
            | "user_observation"
            | "conversation_history"
    )
}

fn is_allowed_source_operation(value: &str) -> bool {
    matches!(
        value,
        "dry_run" | "read" | "collect" | "normalize" | "classify_sensitivity" | "extract_metadata"
    )
}

fn is_sensitivity_label(value: &str) -> bool {
    matches!(value, "public" | "internal" | "sensitive" | "secret")
}

fn is_external_model_policy(value: &str) -> bool {
    matches!(value, "blocked" | "metadata_only" | "allowed_with_approval")
}

fn is_report_type(value: &str) -> bool {
    matches!(
        value,
        "summary" | "evidence_review" | "decision_note" | "handoff" | "experiment_summary"
    )
}

fn is_report_status(value: &str) -> bool {
    matches!(
        value,
        "placeholder" | "requested" | "draft" | "ready" | "archived"
    )
}

fn is_supported_work_item_type(value: &str) -> bool {
    matches!(
        value,
        "collection_normalization"
            | "document_chunking"
            | "chunk_vector_upsert"
            | "report_generation"
    )
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
    "igy6-gateway\n\nUsage:\n  igy6-gateway [--bind 0.0.0.0:8000] [--fallback http://legacy-api:8000]\n  igy6-gateway --help\n\nRoutes:\n  GET /health/live\n  GET /health/ready\n  GET /rust-migration/status\n  GET /agent/capabilities\n  POST /agent/intent\n  POST /chat/retrieval-preview\n  POST /chat/evidence-answer\n  POST /approvals\n  POST /feedback\n  POST /outcomes\n  GET/POST /analysis patterns and GET analysis hypotheses/predictions/recommendations\n  GET/POST /reports and report detail reads\n  GET/POST /sources and selected source detail/permission reads\n  POST /work-items creation only\n  GET /settings/env\n  GET /memory/vector/chunks\n  GET /memory/graph/schema\n  GET /approvals and approval detail reads\n  GET /work-items and work item detail reads\n  GET /evidence documents/items/chunks/claims and detail reads\n  GET /artifacts, /audit-events, /collection-runs, /feedback, /outcomes and detail reads\n\nUnsupported routes are proxied to the configured FastAPI fallback at runtime.\n"
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
    let parameters = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| value.get("parameters").cloned())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    agent_intent_json_from_parts(&message, &parameters)
}

fn agent_intent_response_json(response: &igy6_agent_api::AgentIntentResponse) -> String {
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

fn validation_body_json(message: &str) -> String {
    if message.trim_start().starts_with('{') {
        message.to_string()
    } else {
        format!("{{\"detail\":\"{}\"}}", escape_json(message))
    }
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
        let request = request("POST", "/collection-runs/manual-upload/ingest", "{}");
        let plan = build_fallback_proxy_plan(&request, "http://legacy-api:8000").expect("plan");
        assert_eq!(plan.host, "legacy-api");
        assert_eq!(plan.port, 8000);
        assert_eq!(plan.request_target, "/collection-runs/manual-upload/ingest");

        let response = handle_gateway_request(&request, None, "http://legacy-api:8000");
        assert_eq!(response.status_code, 502);
        assert!(response.proxied_to_fallback);
    }

    #[test]
    fn agent_action_request_is_rust_native_and_requires_database_url_after_validation() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/actions/",
                r#"{"message":"show project health","parameters":{},"actor_id":"local-owner"}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));

        let parsed = parse_agent_action_request(
            r#"{"action_name":"show_project_health","parameters":{},"actor_id":"local-owner"}"#,
        )
        .expect("agent action request");
        assert_eq!(parsed.action_name.as_deref(), Some("show_project_health"));
    }

    #[test]
    fn agent_action_execute_validates_allowlist_parameters_and_approval_without_fallback() {
        let unknown = handle_gateway_request_with_db(
            &request("POST", "/agent/actions/not_allowed/execute", "{}"),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(unknown.status_code, 404);
        assert!(!unknown.proxied_to_fallback);

        let malformed = handle_gateway_request_with_db(
            &request("POST", "/agent/actions/rm%20-rf/execute", "{}"),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(malformed.status_code, 422);
        assert!(!malformed.proxied_to_fallback);

        let missing_parameter = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/actions/run_retrieval_preview/execute",
                r#"{"parameters":{},"actor_id":"local-owner"}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(missing_parameter.status_code, 422);
        assert!(!missing_parameter.proxied_to_fallback);

        let user_argv = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/actions/show_project_health/execute",
                r#"{"parameters":{"argv":["/bin/sh"]},"actor_id":"local-owner"}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(user_argv.status_code, 422);
        assert!(!user_argv.proxied_to_fallback);

        let missing_approval = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/actions/start_stack/execute",
                r#"{"parameters":{},"actor_id":"local-owner"}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(missing_approval.status_code, 403);
        assert!(!missing_approval.proxied_to_fallback);
    }

    #[test]
    fn agent_action_execute_is_rust_native_and_requires_database_for_audit() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/actions/show_project_health/execute",
                r#"{"parameters":{},"actor_id":"local-owner"}"#,
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
    fn agent_action_helpers_redact_and_reject_command_surfaces() {
        let rejected = reject_user_provided_argv(&serde_json::json!({
            "command": "scripts/run.sh"
        }))
        .expect_err("command field rejected");
        assert!(rejected.to_string().contains("argv"));

        let safe = safe_parameter_summary(&serde_json::json!({
            "message": "hello",
            "api_token": "secret-value"
        }));
        assert_eq!(safe["message"], "hello");
        assert_eq!(safe["api_token"], "[redacted]");

        assert!(host_bridge_allowed_action("start_stack").is_some());
        assert!(host_bridge_allowed_action("bash -c docker compose down").is_none());
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
    fn settings_env_verify_and_apply_are_rust_native_without_fallback() {
        let verify_response = handle_gateway_request_with_db(
            &request("POST", "/settings/env/verify", r#"{"values":{}}"#),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(verify_response.status_code, 200);
        assert!(!verify_response.proxied_to_fallback);
        assert!(verify_response.body.contains("\"passed\":false"));

        let apply_response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/settings/env/apply",
                r#"{"values":{},"verification_token":"bad"}"#,
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(apply_response.status_code, 409);
        assert!(!apply_response.proxied_to_fallback);
    }

    #[test]
    fn settings_env_verify_rejects_unknown_and_read_only_keys_without_fallback() {
        for body in [
            r#"{"values":{"UNMANAGED_SECRET":"x"}}"#,
            r#"{"values":{"ENV_FILE_PATH":"/tmp/.env"}}"#,
            r#"{"values":[]}"#,
            r#"{"values":{"APP_PORT":8080}}"#,
            r#"{"values":{},"actor_id":""}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/settings/env/verify", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn settings_env_verify_redacts_secrets_and_preserves_token_shape() {
        let config = test_settings_config();
        let parsed = ParsedSettingsEnv {
            values: HashMap::new(),
            unmanaged_order: Vec::new(),
        };
        let requested = valid_settings_values();
        let (candidate, unmanaged, changed_keys) =
            build_settings_candidate(&config, &parsed, &requested).expect("candidate");
        let validation = validate_settings_candidate(&candidate, &unmanaged, &changed_keys);
        assert!(
            validation.errors.is_empty(),
            "{:?}",
            validation.errors.len()
        );
        let response = settings_verify_response_json(&candidate, &validation);

        assert!(response.contains("\"passed\":true"));
        assert!(response.contains("\"verification_token\":\""));
        assert!(response.contains("\"key\":\"DATABASE_URL\""));
        assert!(response.contains("\"value\":null"));
        assert!(response.contains("\"masked_value\":\"********\""));
        assert!(!response.contains("change-me-local-only"));
    }

    #[test]
    fn settings_env_apply_requires_matching_token_and_database_before_write() {
        let config = test_settings_config();
        let parsed = ParsedSettingsEnv {
            values: HashMap::new(),
            unmanaged_order: Vec::new(),
        };
        let requested = valid_settings_values();
        let (candidate, unmanaged, changed_keys) =
            build_settings_candidate(&config, &parsed, &requested).expect("candidate");
        let validation = validate_settings_candidate(&candidate, &unmanaged, &changed_keys);
        let body = serde_json::json!({
            "values": requested,
            "verification_token": validation.candidate_hash
        })
        .to_string();
        let response = handle_gateway_request_with_db(
            &request("POST", "/settings/env/apply", &body),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));
    }

    #[test]
    fn settings_env_apply_rejects_mismatched_token_without_database_or_write() {
        let body = serde_json::json!({
            "values": valid_settings_values(),
            "verification_token": "not-the-candidate-hash"
        })
        .to_string();
        let response = handle_gateway_request_with_db(
            &request("POST", "/settings/env/apply", &body),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 409);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("verification token"));
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
    fn source_create_is_rust_native_and_requires_database_url() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/sources",
                r#"{"name":"Manual notes","source_type":"manual_upload","sensitivity":"internal","permission":{"scope_json":{},"allowed_operations":["dry_run","read"],"external_model_policy":"blocked","approval_required":true}}"#,
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
    fn report_create_is_rust_native_and_requires_database_url() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/reports",
                r#"{"title":"MVP report","report_type":"summary","status":"requested","metadata_json":{"scope":"local"}}"#,
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
    fn work_item_create_is_rust_native_creation_only_and_requires_database_url() {
        let response = handle_gateway_request_with_db(
            &request("POST", "/work-items/", &valid_work_item_create_body()),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));

        let parsed = parse_work_item_create(&valid_work_item_create_body()).expect("work item");
        assert_eq!(parsed.work_type, "document_chunking");
        assert_eq!(parsed.requested_by_actor_id, "local-owner");
        assert_eq!(parsed.intent["proposed_work_type"], "document_chunking");
        assert_eq!(parsed.payload_json["document_ids"][0], "doc-1");
    }

    #[test]
    fn work_item_create_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"work_type":"","intent":{}}"#,
            r#"{"work_type":"shell_command","intent":{"original_request":"x","interpretation":"x","proposed_work_type":"shell_command","expected_output":"x"}}"#,
            r#"{"work_type":"document_chunking","requested_by_actor_id":"","intent":{"original_request":"x","interpretation":"x","proposed_work_type":"document_chunking","expected_output":"x"}}"#,
            r#"{"work_type":"document_chunking","payload_json":[],"intent":{"original_request":"x","interpretation":"x","proposed_work_type":"document_chunking","expected_output":"x"}}"#,
            r#"{"work_type":"document_chunking","intent":[]}"#,
            r#"{"work_type":"document_chunking","intent":{"original_request":"","interpretation":"x","proposed_work_type":"document_chunking","expected_output":"x"}}"#,
            r#"{"work_type":"document_chunking","intent":{"original_request":"x","interpretation":"x","proposed_work_type":"","expected_output":"x"}}"#,
            r#"{"work_type":"document_chunking","intent":{"original_request":"x","interpretation":"x","proposed_work_type":"document_chunking","expected_output":"x","safety_requirements":[""]}}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/work-items/", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn work_item_created_audit_shape_is_deterministic() {
        let payload = parse_work_item_create(&valid_work_item_create_body()).expect("payload");
        let details = serde_json::json!({
            "work_type": payload.work_type,
            "status": "pending_intent_verification"
        });
        assert_eq!(details["work_type"], "document_chunking");
        assert_eq!(details["status"], "pending_intent_verification");
    }

    #[test]
    fn report_create_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"title":"","report_type":"summary"}"#,
            r#"{"title":"x","report_type":"daily_brief"}"#,
            r#"{"title":"x","report_type":"summary","status":"published"}"#,
            r#"{"title":"x","report_type":"summary","requested_by_actor_id":""}"#,
            r#"{"title":"x","report_type":"summary","artifact_path":[]}"#,
            r#"{"title":"x","report_type":"summary","metadata_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/reports", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn analysis_pattern_writes_are_rust_native_and_require_database_url() {
        for (path, body) in [
            (
                "/analysis/patterns",
                r#"{"pattern_type":"recurrence","summary":"Repeated signal","evidence_ids":["evidence-1"],"confidence":80}"#,
            ),
            (
                "/analysis/patterns/detect-baseline",
                r#"{"recurrence_threshold":3}"#,
            ),
            (
                "/analysis/patterns/pattern-1/review",
                r#"{"status":"verified","review_note":"grounded"}"#,
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
    fn analysis_pattern_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"pattern_type":"","summary":"x","evidence_ids":["e1"]}"#,
            r#"{"pattern_type":"recurrence","summary":"","evidence_ids":["e1"]}"#,
            r#"{"pattern_type":"recurrence","summary":"x","evidence_ids":[]}"#,
            r#"{"pattern_type":"recurrence","summary":"x","evidence_ids":[""]}"#,
            r#"{"pattern_type":"recurrence","summary":"x","evidence_ids":["e1"],"confidence":101}"#,
            r#"{"pattern_type":"recurrence","summary":"x","evidence_ids":["e1"],"confidence":1.5}"#,
            r#"{"pattern_type":"recurrence","summary":"x","evidence_ids":["e1"],"metadata_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/analysis/patterns", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        for body in [
            "{}",
            r#"{"status":"candidate"}"#,
            r#"{"status":"verified","reviewed_by_actor_id":""}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/analysis/patterns/pattern-1/review", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn dynamic_web_control_routes_are_rust_native_without_fallback() {
        for (path, body) in [
            (
                "/approvals/approval-1/decision",
                r#"{"status":"approved","decision_reason":"ok"}"#,
            ),
            ("/reports/report-1/render", r#"{"notes":"handoff"}"#),
            ("/work-items/work-1/dispatch", "{}"),
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
    fn dynamic_web_control_validation_rejects_invalid_requests_without_fallback() {
        for (path, body) in [
            ("/approvals/approval-1/decision", "{}"),
            ("/approvals/approval-1/decision", r#"{"status":"pending"}"#),
            (
                "/approvals/approval-1/decision",
                r#"{"status":"approved","decided_by_actor_id":""}"#,
            ),
            ("/reports/report-1/render", "[]"),
            ("/reports/report-1/render", r#"{"actor_id":""}"#),
            ("/work-items/work-1/dispatch", "[]"),
            ("/work-items/work-1/dispatch", r#"{"actor_id":""}"#),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{path} {body}");
            assert!(!response.proxied_to_fallback, "{path} {body}");
        }
    }

    #[test]
    fn dynamic_web_control_path_helpers_reject_unsafe_ids() {
        assert_eq!(
            pattern_review_path("/analysis/patterns/pattern-1/review").as_deref(),
            Some("pattern-1")
        );
        assert!(pattern_review_path("/analysis/patterns/../x/review").is_none());
        assert!(validate_route_id("../x", "pattern_id").is_err());
        assert!(validate_route_id("x/y", "pattern_id").is_err());
    }

    #[test]
    fn work_item_dispatch_plan_is_allowlisted_and_non_executing() {
        let work_item = WorkItemDispatchRecord {
            id: "work-1".to_string(),
            work_type: "document_chunking".to_string(),
            status: "queued".to_string(),
            payload_json: serde_json::json!({
                "document_id": "doc-1",
                "intent_verification": {"recorded_by": "test"}
            }),
        };
        assert!(has_intent_verification(&work_item.payload_json));
        assert_eq!(
            dispatch_task_name(&work_item).expect("task"),
            "evidence.generate_document_chunks"
        );

        let unsupported = WorkItemDispatchRecord {
            work_type: "report_generation".to_string(),
            ..work_item
        };
        assert!(dispatch_task_name(&unsupported).is_err());
    }

    #[test]
    fn collection_dry_run_is_rust_native_and_requires_database_url() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/collection-runs/dry-run",
                r#"{"source_id":"source-1","source_permission_id":"permission-1","notes":{"reason":"preview"}}"#,
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
    fn manual_upload_is_rust_native_and_requires_database_url_after_validation() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/collection-runs/manual-upload",
                &valid_manual_upload_body(),
            ),
            None,
            DEFAULT_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));

        let parsed =
            parse_manual_upload_collection(&valid_manual_upload_body()).expect("manual upload");
        assert_eq!(parsed.source_id, "source-1");
        assert_eq!(parsed.source_permission_id, "permission-1");
        assert_eq!(parsed.filename.as_deref(), Some("manual-note.txt"));
        assert_eq!(
            decode_base64(&parsed.content_base64).expect("base64"),
            b"hello\n"
        );
    }

    #[test]
    fn manual_upload_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"source_id":"","source_permission_id":"permission-1","content_base64":"aGVsbG8K"}"#,
            r#"{"source_id":"source-1","source_permission_id":"","content_base64":"aGVsbG8K"}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"not base64"}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"ICAg"}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"//8="}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","mime_type":"image/png"}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","filename":"../secret.txt"}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","metadata_json":[]}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","requested_by_actor_id":""}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/collection-runs/manual-upload", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn manual_upload_summary_and_work_payload_are_deterministic() {
        let source = CollectionSource {
            id: "source-1".to_string(),
            name: "Manual".to_string(),
            source_type: "manual_upload".to_string(),
            location: None,
            sensitivity: "internal".to_string(),
            enabled: true,
            metadata_json: serde_json::json!({}),
        };
        let work_payload = manual_upload_normalization_work_payload(
            "collection-1",
            &source,
            "permission-1",
            "artifact-1",
        );
        assert_eq!(work_payload["work_item_id"], Value::Null);
        assert_eq!(work_payload["work_type"], Value::Null);
        assert_eq!(work_payload["collection_run_id"], "collection-1");
        assert_eq!(work_payload["raw_artifact_ids"][0], "artifact-1");
        assert_eq!(work_payload["executes_normalization"], true);
        assert_eq!(
            work_payload["intent_verification"]["recorded_by"],
            "DIFF-074 collection enqueue governance"
        );

        let metadata = manual_upload_artifact_metadata(
            &serde_json::json!({"operator_note": "ok"}),
            Some("manual-note.txt"),
            "permission-1",
            Some("approval-1"),
        );
        assert_eq!(metadata["filename"], "manual-note.txt");
        assert_eq!(metadata["source_permission_id"], "permission-1");
        assert_eq!(metadata["approval_id"], "approval-1");
        assert_eq!(metadata["operator_note"], "ok");
    }

    #[test]
    fn collection_dry_run_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"source_id":"","source_permission_id":"permission-1"}"#,
            r#"{"source_id":"source-1","source_permission_id":""}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","requested_by_actor_id":""}"#,
            r#"{"source_id":"source-1","source_permission_id":"permission-1","notes":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/collection-runs/dry-run", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn collection_dry_run_preview_shape_is_deterministic() {
        let source = CollectionSource {
            id: "source-1".to_string(),
            name: "Project".to_string(),
            source_type: "local_project".to_string(),
            location: Some("/workspace".to_string()),
            sensitivity: "internal".to_string(),
            enabled: true,
            metadata_json: serde_json::json!({"root": "repo"}),
        };
        let permission = CollectionPermission {
            id: "permission-1".to_string(),
            source_id: "source-1".to_string(),
            scope_json: serde_json::json!({"paths": ["src"]}),
            allowed_operations: vec!["dry_run".to_string()],
            external_model_policy: "blocked".to_string(),
            approval_required: true,
        };
        let result = connector_dry_run_result(&source, &permission).expect("dry run");
        let summary = collection_dry_run_summary(
            &source,
            &permission,
            Some(&result),
            &serde_json::json!({"reason": "preview"}),
        );

        assert_eq!(summary["source"]["id"], "source-1");
        assert_eq!(summary["preview"]["mode"], "connector_dry_run_preview");
        assert_eq!(summary["preview"]["would_collect"], false);
        assert_eq!(summary["preview"]["would_create_artifacts"], false);
        assert_eq!(summary["preview"]["would_normalize"], false);
        assert_eq!(summary["preview"]["would_enqueue_worker"], false);
        assert_eq!(
            summary["connector_result"]["connector_name"],
            "local_project"
        );
        assert_eq!(
            summary["connector_result"]["summary"],
            "Project dry-run validated source and permission metadata only."
        );
        assert_eq!(
            summary["connector_result"]["metadata"]["preview_only"],
            true
        );
        assert_eq!(summary["notes"]["reason"], "preview");
    }

    #[test]
    fn collection_dry_run_rejects_unsupported_connector_without_collection() {
        let source = CollectionSource {
            id: "source-1".to_string(),
            name: "Web".to_string(),
            source_type: "web_public".to_string(),
            location: Some("https://example.test".to_string()),
            sensitivity: "public".to_string(),
            enabled: true,
            metadata_json: serde_json::json!({}),
        };
        let permission = CollectionPermission {
            id: "permission-1".to_string(),
            source_id: "source-1".to_string(),
            scope_json: serde_json::json!({}),
            allowed_operations: vec!["read".to_string()],
            external_model_policy: "blocked".to_string(),
            approval_required: true,
        };
        let error = connector_dry_run_result(&source, &permission).expect_err("unsupported");
        assert_eq!(error, "No connector registered for source type: web_public");
    }

    #[test]
    fn collection_dry_run_permission_checks_are_deterministic() {
        assert!(permission_allows(
            &["read".to_string()],
            &["dry_run", "read"]
        ));
        assert!(permission_allows(
            &["dry_run".to_string()],
            &["dry_run", "read"]
        ));
        assert!(!permission_allows(
            &["collect".to_string()],
            &["dry_run", "read"]
        ));
        assert!(!permission_allows(
            &Vec::<String>::new(),
            &["dry_run", "read"]
        ));

        let forbidden = write_route_response(Err(GatewayError::Forbidden(
            "Source permission does not allow dry-run preview".to_string(),
        )));
        assert_eq!(forbidden.status_code, 403);
        let conflict = write_route_response(Err(GatewayError::Conflict(
            "Source is disabled".to_string(),
        )));
        assert_eq!(conflict.status_code, 409);
    }

    #[test]
    fn baseline_pattern_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "[]",
            r#"{"actor_id":""}"#,
            r#"{"recurrence_threshold":1}"#,
            r#"{"recurrence_threshold":21}"#,
            r#"{"recurrence_threshold":2.5}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/analysis/patterns/detect-baseline", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn baseline_pattern_candidates_are_deterministic() {
        let evidence = vec![
            BaselineEvidenceItem {
                id: "e1".to_string(),
                evidence_type: "log".to_string(),
                statement: "Same Signal".to_string(),
                source_id: Some("s1".to_string()),
            },
            BaselineEvidenceItem {
                id: "e2".to_string(),
                evidence_type: "log".to_string(),
                statement: "same   signal".to_string(),
                source_id: Some("s2".to_string()),
            },
        ];
        let candidates = baseline_pattern_candidates(&evidence, 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].pattern_type, "recurrence");
        assert_eq!(candidates[0].evidence_ids, vec!["e1", "e2"]);
        assert_eq!(candidates[1].pattern_type, "cross_source_conflict");
        assert_eq!(
            candidates[1].detector_key,
            "cross_source_statement:same signal"
        );
    }

    #[test]
    fn source_create_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"name":"","source_type":"manual_upload"}"#,
            r#"{"name":"x","source_type":"generic_chatbot"}"#,
            r#"{"name":"x","source_type":"manual_upload","sensitivity":"classified"}"#,
            r#"{"name":"x","source_type":"manual_upload","enabled":"yes"}"#,
            r#"{"name":"x","source_type":"manual_upload","metadata_json":[]}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":[]}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":{"scope_json":[]}}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":{"allowed_operations":["write"]}}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":{"external_model_policy":"allow_all"}}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":{"approval_required":"yes"}}"#,
            r#"{"name":"x","source_type":"manual_upload","permission":{"created_by_actor_id":""}}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/sources", body),
                None,
                DEFAULT_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
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
            ("POST", "/sources"),
            ("GET", "/analysis/patterns"),
            ("GET", "/analysis/patterns/{pattern_id}"),
            ("POST", "/analysis/patterns"),
            ("POST", "/analysis/patterns/detect-baseline"),
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
            ("POST", "/collection-runs/dry-run"),
            ("GET", "/work-items"),
            ("GET", "/work-items/{work_item_id}"),
            ("GET", "/reports"),
            ("GET", "/reports/{report_id}"),
            ("POST", "/reports"),
            ("GET", "/feedback"),
            ("GET", "/feedback/{feedback_id}"),
            ("POST", "/feedback"),
            ("GET", "/memory/graph/schema"),
            ("GET", "/memory/vector/chunks"),
            ("GET", "/outcomes"),
            ("GET", "/outcomes/{outcome_id}"),
            ("POST", "/outcomes"),
            ("GET", "/settings/env"),
            ("POST", "/settings/env/apply"),
            ("POST", "/settings/env/verify"),
            ("GET", "/evidence/documents"),
            ("GET", "/evidence/documents/{document_id}"),
            ("GET", "/evidence/items"),
            ("GET", "/evidence/items/{evidence_item_id}"),
            ("GET", "/evidence/chunks"),
            ("GET", "/evidence/chunks/{chunk_id}"),
            ("GET", "/evidence/claims"),
            ("GET", "/evidence/claims/{claim_id}"),
            ("POST", "/collection-runs/manual-upload"),
            ("POST", "/work-items"),
            ("POST", "/work-items/"),
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

    fn test_settings_config() -> SettingsEnvConfig {
        SettingsEnvConfig {
            env_file_path: PathBuf::from("/workspace/project/.env"),
            backup_dir: PathBuf::from("/workspace/storage/env_backups"),
            igy6_data_root: "../IGY6_Data".to_string(),
        }
    }

    fn valid_settings_values() -> HashMap<String, String> {
        HashMap::from([
            ("APP_ENV".to_string(), "local".to_string()),
            ("APP_HOST".to_string(), "127.0.0.1".to_string()),
            ("APP_PORT".to_string(), "8000".to_string()),
            ("API_BASE_URL".to_string(), "http://127.0.0.1:8000".to_string()),
            ("WEB_BASE_URL".to_string(), "http://127.0.0.1:3000".to_string()),
            ("POSTGRES_HOST".to_string(), "postgres".to_string()),
            ("POSTGRES_PORT".to_string(), "5432".to_string()),
            ("POSTGRES_DB".to_string(), "adaptive_intelligence".to_string()),
            ("POSTGRES_USER".to_string(), "adaptive".to_string()),
            ("POSTGRES_PASSWORD".to_string(), "change-me-local-only".to_string()),
            (
                "DATABASE_URL".to_string(),
                "postgresql+psycopg://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence"
                    .to_string(),
            ),
            ("REDIS_HOST".to_string(), "redis".to_string()),
            ("REDIS_PORT".to_string(), "6379".to_string()),
            ("REDIS_URL".to_string(), "redis://redis:6379/0".to_string()),
            (
                "CELERY_BROKER_URL".to_string(),
                "redis://redis:6379/0".to_string(),
            ),
            (
                "CELERY_RESULT_BACKEND".to_string(),
                "redis://redis:6379/1".to_string(),
            ),
            ("QDRANT_HOST".to_string(), "qdrant".to_string()),
            ("QDRANT_PORT".to_string(), "6333".to_string()),
            ("QDRANT_URL".to_string(), "http://qdrant:6333".to_string()),
            (
                "QDRANT_CHUNK_COLLECTION".to_string(),
                "igy6_chunks".to_string(),
            ),
            ("QDRANT_CHUNK_VECTOR_SIZE".to_string(), "384".to_string()),
            ("NEO4J_HOST".to_string(), "neo4j".to_string()),
            ("NEO4J_HTTP_PORT".to_string(), "7474".to_string()),
            ("NEO4J_BOLT_PORT".to_string(), "7687".to_string()),
            ("NEO4J_USER".to_string(), "neo4j".to_string()),
            ("NEO4J_PASSWORD".to_string(), "change-me-local-only".to_string()),
            ("NEO4J_URI".to_string(), "bolt://neo4j:7687".to_string()),
            (
                "MLFLOW_TRACKING_URI".to_string(),
                "http://mlflow:5000".to_string(),
            ),
            (
                "MLFLOW_ARTIFACT_ROOT".to_string(),
                "/mlflow/artifacts".to_string(),
            ),
            ("PHOENIX_HOST".to_string(), "phoenix".to_string()),
            ("PHOENIX_PORT".to_string(), "6006".to_string()),
            (
                "PHOENIX_COLLECTOR_ENDPOINT".to_string(),
                "http://phoenix:6006".to_string(),
            ),
            (
                "ARTIFACT_STORE_PATH".to_string(),
                "/workspace/storage/artifacts".to_string(),
            ),
            (
                "EXPORT_STORE_PATH".to_string(),
                "/workspace/storage/exports".to_string(),
            ),
            ("IGY6_DATA_ROOT".to_string(), "../IGY6_Data".to_string()),
            (
                "EXTERNAL_MODEL_POLICY_DEFAULT".to_string(),
                "blocked".to_string(),
            ),
            ("SINGLE_USER_MODE".to_string(), "true".to_string()),
            ("AUDIT_LOG_LEVEL".to_string(), "info".to_string()),
            (
                "APPROVAL_REQUIRED_DEFAULT".to_string(),
                "true".to_string(),
            ),
        ])
    }

    fn valid_work_item_create_body() -> String {
        serde_json::json!({
            "work_type": "document_chunking",
            "requested_by_actor_id": "local-owner",
            "intent": {
                "original_request": "Chunk selected documents.",
                "interpretation": "Create a local document chunking work item without dispatching it.",
                "proposed_work_type": "document_chunking",
                "expected_output": "Queued chunking work item after explicit intent verification.",
                "safety_requirements": ["No dispatch in DIFF-116"],
                "assumptions": ["Documents already exist"],
                "missing_information": [],
                "sources_likely_used": ["local_project"]
            },
            "payload_json": {
                "document_ids": ["doc-1"],
                "chunk_size": 1000
            }
        })
        .to_string()
    }

    fn valid_manual_upload_body() -> String {
        serde_json::json!({
            "source_id": "source-1",
            "source_permission_id": "permission-1",
            "approval_id": null,
            "filename": "manual-note.txt",
            "mime_type": "text/plain",
            "content_base64": "aGVsbG8K",
            "metadata_json": {
                "operator_note": "test"
            },
            "requested_by_actor_id": "local-owner"
        })
        .to_string()
    }
}
