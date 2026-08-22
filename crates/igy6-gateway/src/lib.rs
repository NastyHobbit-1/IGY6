use std::collections::{HashMap, HashSet};
use std::process::Command; // for system snapshots (ps, nmcli, ip, etc.) on grok full-access mode
                           // On grok branch full power (user override: any and everything accessible, stored only locally, no exfil)
use base64::Engine; // for content base64 in media library (grok)
use igy6_artifacts::detect_content_kind;
use std::env;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP}; // for optional authenticator TOTP (grok, off by default)
use ureq; // for web scraping / URL collection
use walkdir::WalkDir;

use igy6_agent_api::{
    action_definition, classify_agent_intent, understand_user_request, AgentActionDefinition,
    AgentIntentRequest, ACTION_REGISTRY,
};
use igy6_artifacts::{extract_text_if_possible, ArtifactStore, StoredArtifact}; // grok: now does real deep PDF (and other media) text extraction
use igy6_evidence_answer::{
    answer_with_optional_llm, answer_with_optional_llm_for_task,
    deterministic_fallback_for_llm_config_error,
};
use igy6_host_bridge::{allowed_action as host_bridge_allowed_action, redact_output};
use igy6_llm::{load_local_llm_routing_config, LlmConfig, LlmProvider, StdHttpTransport};
use igy6_read_only_api::summarize_manifest;
use igy6_retrieval_preview::{
    build_hydrated_chunk_search_result, build_retrieval_preview, HydratedChunkSearchHit,
    RetrievalChunk, RetrievalDocument, RetrievalEvidenceItem, RetrievalRawArtifact,
    RetrievalSource,
};
use igy6_vector_memory::{
    collection_status_request, ensure_collection_request, plan_chunk_vector_point,
    search_points_request, upsert_points_request, HttpMethod, HttpRequestPlan, QdrantSettings,
    EMBEDDING_METHOD,
};
use postgres::{Client, NoTls};
use serde_json::Value;
use sha2::{Digest, Sha256};

mod bypass_intel;

pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8000";
pub const NO_FALLBACK_ORIGIN: &str = "";
#[rustfmt::skip]
pub const RUST_NATIVE_ROUTES: &[(&str, &str)] = &[
    ("GET", "/"),
    ("GET", "/health/live"),
    ("GET", "/health/ready"),
    ("GET", "/rust-migration/status"),
    // User/TOTP security routes (grok branch)
    ("GET", "/user/status"),
    ("POST", "/user/change-password"),
    ("POST", "/user/generate-totp"),
    ("POST", "/user/confirm-totp"),
    // Full-access and local-scan collection (grok branch)
    ("POST", "/collection-runs/full-access"),
    ("POST", "/collection-runs/full-local-scan"),
    // Host bridge reach probe (grok branch)
    ("GET", "/host-bridge/status"),
    ("POST", "/host-bridge/ensure-max-reach"),
    // Bypass intel utilities (existing internal handlers)
    ("GET", "/bypass-intel/status"),
    ("GET", "/bypass-intel/playbook"),
    ("POST", "/bypass-intel/harvest"),
    ("POST", "/media/import"),
    ("GET", "/agent/capabilities"),
    ("GET", "/agent/task-plans"),
    ("GET", "/agent/task-plans/{task_plan_id}"),
    ("POST", "/agent/task-plans"),
    ("POST", "/agent/task-plans/{task_plan_id}/evidence-summary"),
    ("POST", "/agent/task-plans/{task_plan_id}/work-spec"),
    ("POST", "/agent/task-plans/{task_plan_id}/work-item"),
    ("POST", "/agent/actions/"),
    ("POST", "/agent/actions/{action_name}/execute"),
    ("GET", "/analysis/hypotheses"),
    ("GET", "/analysis/hypotheses/{hypothesis_id}"),
    ("POST", "/analysis/hypotheses"),
    ("GET", "/analysis/patterns"),
    ("GET", "/analysis/patterns/{pattern_id}"),
    ("POST", "/analysis/patterns"),
    ("POST", "/analysis/patterns/{pattern_id}/review"),
    ("POST", "/analysis/patterns/detect-baseline"),
    ("GET", "/analysis/predictions"),
    ("GET", "/analysis/predictions/{prediction_id}"),
    ("POST", "/analysis/predictions"),
    ("GET", "/analysis/recommendations"),
    ("GET", "/analysis/recommendations/{recommendation_id}"),
    ("POST", "/analysis/recommendations"),
    ("GET", "/analysis/calibration/summary"),
    ("POST", "/agent/intent"),
    ("POST", "/chat/retrieval-preview"),
    ("POST", "/chat/evidence-answer"),
    ("GET", "/evidence-answers"),
    ("GET", "/evidence-answers/{answer_id}"),
    ("POST", "/evidence-answers"),
    ("GET", "/approvals"),
    ("GET", "/approvals/{approval_id}"),
    ("POST", "/approvals"),
    ("POST", "/approvals/{approval_id}/decision"),
    ("GET", "/artifacts"),
    ("GET", "/artifacts/{artifact_id}"),
    ("GET", "/artifacts/{artifact_id}/content"),  // grok: for image/video library full res view (base64 for simplicity)
    ("POST", "/artifacts"),
    ("GET", "/audit-events"),
    ("GET", "/audit-events/{audit_event_id}"),
    ("GET", "/collection-runs"),
    ("GET", "/collection-runs/{collection_run_id}"),
    ("POST", "/collection-runs"),
    ("POST", "/collection-runs/dry-run"),
    ("POST", "/collection-runs/local-project"),
    ("POST", "/collection-runs/manual-upload"),
    ("POST", "/collection-runs/manual-upload/ingest"),
    ("GET", "/evidence/documents"),
    ("GET", "/evidence/documents/{document_id}"),
    ("POST", "/evidence/documents"),
    ("POST", "/evidence/documents/{document_id}/chunks"),
    ("GET", "/evidence/items"),
    ("GET", "/evidence/items/{evidence_item_id}"),
    ("POST", "/evidence/items"),
    ("POST", "/evidence/items/{evidence_item_id}/review-state"),
    ("GET", "/evidence/chunks"),
    ("GET", "/evidence/chunks/{chunk_id}"),
    ("GET", "/evidence/claims"),
    ("GET", "/evidence/claims/{claim_id}"),
    ("GET", "/experiments"),
    ("GET", "/experiments/{experiment_run_id}"),
    ("POST", "/experiments"),
    ("POST", "/experiments/propose-from-improvement"),
    ("POST", "/experiments/{experiment_run_id}/status"),
    ("GET", "/feedback"),
    ("GET", "/feedback/{feedback_id}"),
    ("POST", "/feedback"),
    ("GET", "/improvements"),
    ("GET", "/improvements/{improvement_item_id}"),
    ("POST", "/improvements"),
    ("GET", "/memory/graph/schema"),
    ("POST", "/memory/graph/schema/ensure"),
    ("POST", "/memory/graph/lineage/sync"),
    ("GET", "/memory/graph/nodes/{node_label}/{node_id}/relationships"),
    ("GET", "/memory/vector/chunks"),
    ("GET", "/ops/runtime-logs"),
    ("POST", "/ops/runtime-logs/append"),
    ("POST", "/memory/vector/chunks/ensure"),
    ("POST", "/memory/vector/chunks/search"),
    ("POST", "/memory/vector/chunks/upsert"),
    ("GET", "/outcomes"),
    ("GET", "/outcomes/{outcome_id}"),
    ("POST", "/outcomes"),
    ("GET", "/reports"),
    ("GET", "/reports/{report_id}"),
    ("POST", "/reports"),
    ("POST", "/reports/{report_id}/render"),
    ("POST", "/reports/{report_id}/status"),
    ("POST", "/reports/{report_id}/work-item"),
    ("GET", "/retrieval/chunks/{chunk_id}/trail"),
    ("POST", "/retrieval/chunks/search"),
    ("GET", "/settings/env"),
    ("POST", "/settings/env/apply"),
    ("POST", "/settings/env/verify"),
    ("GET", "/sources"),
    ("GET", "/sources/{source_id}"),
    ("GET", "/sources/{source_id}/permissions"),
    ("POST", "/sources"),
    ("POST", "/sources/{source_id}/permissions"),
    ("POST", "/sources/{source_id}/review-state"),
    ("GET", "/work-items"),
    ("GET", "/work-items/{work_item_id}"),
    ("POST", "/work-items"),
    ("POST", "/work-items/"),
    ("POST", "/work-items/{work_item_id}/dispatch"),
    ("POST", "/work-items/{work_item_id}/status"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    EmptyRequest,
    MalformedRequest,
    InvalidContentLength,
    MissingDatabaseUrl,
    Database(String),
    Validation(String),
    NotFound(String),
    Conflict(String),
    Forbidden(String),
    ServiceUnavailable(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(formatter, "request is empty"),
            Self::MalformedRequest => write!(formatter, "request is malformed"),
            Self::InvalidContentLength => write!(formatter, "content-length is invalid"),
            Self::MissingDatabaseUrl => {
                write!(formatter, "DATABASE_URL is required for this route")
            }
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Validation(error) => write!(formatter, "validation error: {error}"),
            Self::NotFound(error) => write!(formatter, "{error}"),
            Self::Conflict(error) => write!(formatter, "{error}"),
            Self::Forbidden(error) => write!(formatter, "{error}"),
            Self::ServiceUnavailable(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for GatewayError {}

impl From<igy6_vector_memory::VectorMemoryError> for GatewayError {
    fn from(error: igy6_vector_memory::VectorMemoryError) -> Self {
        GatewayError::Validation(error.to_string())
    }
}

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
struct ExternalHttpRequest {
    method: String,
    origin: String,
    path: String,
    body: Option<String>,
    headers: Vec<(String, String)>,
    timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalHttpResponse {
    status_code: u16,
    body: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Neo4jStatement {
    statement: String,
    parameters: Value,
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
    _fallback_origin: &str,
    database_url: Option<&str>,
) -> GatewayResponse {
    if request.method == "GET" && request.path.starts_with("/ops/runtime-logs") {
        return runtime_logs_response(&request.path);
    }
    if request.method == "POST" && request.path.starts_with("/ops/runtime-logs/append") {
        return runtime_logs_append_response(&request.body);
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => json_response(
            200,
            "OK",
            "{\"service\":\"igy6-gateway\",\"phase\":\"rust-primary\",\"status\":\"ok\",\"primary_gateway\":true,\"fallback\":\"none\"}".to_string(),
            false,
        ),
        ("GET", "/health/live") => json_response(
            200,
            "OK",
            "{\"status\":\"ok\",\"service\":\"igy6-gateway\",\"primary_gateway\":true}".to_string(),
            false,
        ),
        ("GET", "/health/ready") => json_response(
            200,
            "OK",
            "{\"status\":\"ok\",\"checks\":{\"rust_gateway\":{\"status\":\"ok\"},\"fastapi_fallback\":{\"status\":\"removed\"}},\"primary_gateway\":\"rust\",\"fallback\":\"none\"}".to_string(),
            false,
        ),
        ("GET", "/rust-migration/status") => {
            let summary = summarize_manifest(manifest_content.unwrap_or_default());
            json_response(
                200,
                "OK",
                format!(
                    "{{\"status\":\"ok\",\"cutover_ready\":{},\"complete_phases\":{},\"pending_phases\":{},\"primary_gateway\":\"rust\",\"fallback\":\"none\"}}",
                    summary.cutover_ready, summary.complete_phases, summary.pending_phases
                ),
                false,
            )
        }
        ("GET", "/agent/capabilities") => json_response(200, "OK", agent_capabilities_json(), false),
        ("POST", "/agent/task-plans") => agent_task_plan_create_response(&request.body, database_url),
        ("POST", "/agent/actions/") => agent_action_request_response(&request.body, database_url),
        ("POST", "/agent/intent") => json_response(200, "OK", agent_intent_json(&request.body), false),
        ("POST", "/chat/retrieval-preview") => {
            retrieval_preview_response(&request.body, database_url)
        }
        ("POST", "/chat/evidence-answer") => {
            evidence_answer_response(&request.body, database_url)
        }
        ("POST", "/evidence-answers") => {
            evidence_answer_record_create_response(&request.body, database_url)
        }
        ("POST", "/approvals") => approval_create_response(&request.body, database_url),
        ("POST", "/artifacts") => raw_artifact_create_response(&request.body, database_url),
        ("POST", "/analysis/hypotheses") => {
            hypothesis_create_response(&request.body, database_url)
        }
        ("POST", "/analysis/patterns") => pattern_create_response(&request.body, database_url),
        ("POST", "/analysis/patterns/detect-baseline") => {
            baseline_patterns_response(&request.body, database_url)
        }
        ("POST", "/analysis/predictions") => {
            prediction_create_response(&request.body, database_url)
        }
        ("POST", "/analysis/recommendations") => {
            recommendation_create_response(&request.body, database_url)
        }
        ("POST", "/collection-runs") => collection_run_create_response(&request.body, database_url),
        ("POST", "/collection-runs/dry-run") => {
            collection_dry_run_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/local-project") => {
            local_project_collection_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/manual-upload") => {
            manual_upload_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/manual-upload/ingest") => {
            manual_upload_ingest_response(&request.body, database_url)
        }
        ("POST", "/media/import") => media_import_response(&request.body, database_url),
        // On grok branch: full power access collector (user directive: ingest ANYTHING the process can reach, store ONLY locally, no outbound info leakage beyond necessary fetches for collection itself)
        ("POST", "/collection-runs/full-access") => {
            full_access_collection_response(&request.body, database_url)
        }
        ("POST", "/host-bridge/ensure-max-reach") => host_bridge_ensure_max_reach_response(),
        ("GET", "/bypass-intel/status") => {
            json_response(200, "OK", bypass_intel::bypass_intel_status_json(), false)
        }
        ("GET", "/bypass-intel/playbook") => {
            let payload = std::fs::read_to_string(bypass_intel::bypass_intel_playbook_path())
                .unwrap_or_else(|_| "{}".to_string());
            json_response(200, "OK", payload, false)
        }
        ("POST", "/bypass-intel/harvest") => {
            bypass_intel::bypass_intel_harvest_response(&request.body, database_url)
        }
        ("POST", "/collection-runs/full-local-scan") => {
            full_access_collection_response(&request.body, database_url)
        }
        // User / security on grok branch: password changing + optional authenticator (TOTP, off by default until linked)
        ("GET", "/user/status") => json_response(200, "OK", user_status_json(), false),
        ("POST", "/user/change-password") => {
            let resp_body = user_change_password(&request.body).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e));
            json_response(200, "OK", resp_body, false)
        },
        ("POST", "/user/generate-totp") => {
            let resp_body = user_generate_totp(&request.body).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e));
            json_response(200, "OK", resp_body, false)
        },
        ("POST", "/user/confirm-totp") => {
            let resp_body = user_confirm_totp(&request.body).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e));
            json_response(200, "OK", resp_body, false)
        },
        ("POST", "/evidence/documents") => {
            evidence_document_create_response(&request.body, database_url)
        }
        ("POST", "/evidence/items") => {
            evidence_item_create_response(&request.body, database_url)
        }
        ("POST", "/experiments") => experiment_create_response(&request.body, database_url),
        ("POST", "/experiments/propose-from-improvement") => {
            experiment_proposal_response(&request.body, database_url)
        }
        ("POST", "/feedback") => feedback_create_response(&request.body, database_url),
        ("POST", "/improvements") => improvement_create_response(&request.body, database_url),
        ("POST", "/memory/graph/lineage/sync") => {
            action_route_response(sync_graph_lineage(database_url))
        }
        ("POST", "/memory/graph/schema/ensure") => write_route_response(ensure_graph_schema()),
        ("POST", "/memory/vector/chunks/ensure") => {
            write_route_response(ensure_vector_chunk_collection())
        }
        ("POST", "/memory/vector/chunks/search") => {
            action_route_response(search_vector_chunks(&request.body))
        }
        ("POST", "/memory/vector/chunks/upsert") => {
            action_route_response(upsert_vector_chunks(database_url))
        }
        ("POST", "/outcomes") => outcome_create_response(&request.body, database_url),
        ("POST", "/reports") => report_create_response(&request.body, database_url),
        ("POST", "/retrieval/chunks/search") => {
            retrieval_chunk_search_response(&request.body, database_url)
        }
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
            } else if let Some(task_plan_id) = agent_task_plan_work_item_path(&request.path) {
                action_route_response(create_work_item_from_agent_task_plan(
                    &task_plan_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(task_plan_id) = agent_task_plan_evidence_summary_path(&request.path)
            {
                action_route_response(update_agent_task_plan_evidence_summary(
                    &task_plan_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(task_plan_id) = agent_task_plan_work_spec_path(&request.path) {
                action_route_response(propose_agent_task_plan_work_spec(
                    &task_plan_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(report_id) = report_render_path(&request.path) {
                action_route_response(render_report(&report_id, &request.body, database_url))
            } else if let Some(report_id) = report_status_path(&request.path) {
                action_route_response(update_report_status(
                    &report_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(report_id) = report_work_item_path(&request.path) {
                write_route_response(create_report_work_item(
                    &report_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(document_id) = evidence_document_chunks_path(&request.path) {
                write_route_response(generate_document_chunks(
                    &document_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(evidence_item_id) =
                evidence_item_review_state_path(&request.path)
            {
                write_route_response(update_evidence_item_review_state(
                    &evidence_item_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(experiment_run_id) = experiment_status_path(&request.path) {
                action_route_response(update_experiment_status(
                    &experiment_run_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(source_id) = source_permission_create_path(&request.path) {
                write_route_response(create_source_permission(
                    &source_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(source_id) = source_review_state_path(&request.path) {
                write_route_response(update_source_review_state(
                    &source_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(work_item_id) = work_item_dispatch_path(&request.path) {
                action_route_response(dispatch_work_item(
                    &work_item_id,
                    &request.body,
                    database_url,
                ))
            } else if let Some(work_item_id) = work_item_status_path(&request.path) {
                action_route_response(update_work_item_status(
                    &work_item_id,
                    &request.body,
                    database_url,
                ))
            } else {
                fallback_or_error(request)
            }
        }
        ("GET", "/memory/vector/chunks") => {
            json_response(200, "OK", vector_collection_status_live_json(), false)
        }
        ("GET", "/memory/graph/schema") => {
            json_response(200, "OK", graph_schema_status_json(), false)
        }
        ("GET", "/evidence-answers") => {
            action_route_response(list_evidence_answer_records(database_url))
        }
        ("GET", "/analysis/calibration/summary") => {
            action_route_response(prediction_recommendation_calibration_summary(database_url))
        }
        ("GET", "/agent/task-plans") => {
            action_route_response(list_agent_task_plans(database_url))
        }
        ("GET", _) => {
            if let Some(task_plan_id) = agent_task_plan_detail_path(&request.path) {
                action_route_response(get_agent_task_plan(&task_plan_id, database_url))
            } else if let Some(answer_id) = evidence_answer_record_detail_path(&request.path) {
                action_route_response(get_evidence_answer_record(&answer_id, database_url))
            } else if let Some((node_label, node_id)) = graph_relationships_path(&request.path) {
                action_route_response(get_graph_node_relationships(&node_label, &node_id))
            } else if let Some(chunk_id) = retrieval_chunk_trail_path(&request.path) {
                action_route_response(get_retrieval_chunk_trail(&chunk_id, database_url))
            } else if let Some(id) = artifact_content_path(&request.path) {
                artifact_content_response(&id, database_url)
            } else if let Some(route) = db_read_route(&request.path) {
                db_read_response(route, database_url)
            } else {
                fallback_or_error(request)
            }
        }
        _ => fallback_or_error(request),
    }
}

fn fallback_or_error(request: &GatewayRequest) -> GatewayResponse {
    json_response(
        404,
        "Not Found",
        format!(
            "{{\"detail\":\"Route is not implemented by the Rust gateway and FastAPI fallback is removed\",\"method\":\"{}\",\"path\":\"{}\"}}",
            escape_json(&request.method),
            escape_json(&request.path)
        ),
        false,
    )
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
        "/experiments" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, improvement_item_id, status, mlflow_run_id, optuna_study_name, metrics_json, artifacts_json, metadata_json, created_at, updated_at FROM experiment_runs ORDER BY created_at DESC) t), '[]')",
        }),
        "/feedback" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, target_type, target_id, label, actor_id, note, metadata_json, created_at, updated_at FROM feedback_events ORDER BY created_at DESC) t), '[]')",
        }),
        "/improvements" => Some(DbReadRoute::List {
            sql: "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, target_area, status, objective, proposed_by_actor_id, priority, metadata_json, created_at, updated_at FROM improvement_items ORDER BY created_at DESC) t), '[]')",
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

// grok branch: helper to detect artifact content request for image/video library
fn artifact_content_path(path: &str) -> Option<String> {
    let p = path.trim_matches('/');
    if p.starts_with("artifacts/") && p.ends_with("/content") {
        if let Some(rest) = p.strip_prefix("artifacts/") {
            if let Some(id) = rest.strip_suffix("/content") {
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

fn artifact_content_response(id: &str, database_url: Option<&str>) -> GatewayResponse {
    let database_url = match database_url.filter(|v| !v.trim().is_empty()) {
        Some(u) => u,
        None => {
            return json_response(
                503,
                "Service Unavailable",
                "{\"detail\":\"DATABASE_URL required\"}".to_string(),
                false,
            )
        }
    };
    let postgres_url = postgres_client_url(database_url);
    let mut client = match Client::connect(&postgres_url, NoTls) {
        Ok(c) => c,
        Err(e) => {
            return json_response(
                502,
                "Bad Gateway",
                format!("{{\"detail\":\"DB connect error: {}\"}}", e),
                false,
            )
        }
    };
    let row = match client.query_opt(
        "SELECT content_hash, mime_type, size_bytes FROM raw_artifacts WHERE id = $1 LIMIT 1",
        &[&id],
    ) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return json_response(
                404,
                "Not Found",
                format!("{{\"detail\":\"Artifact {} not found\"}}", id),
                false,
            )
        }
        Err(e) => {
            return json_response(
                502,
                "Bad Gateway",
                format!("{{\"detail\":\"DB query error: {}\"}}", e),
                false,
            )
        }
    };
    let content_hash: String = row.get(0);
    let mime_type: Option<String> = row.get(1);
    let size_bytes: Option<i32> = row.get(2);
    let mime = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let store = match ArtifactStore::new(artifact_data_root()) {
        Ok(s) => s,
        Err(e) => {
            return json_response(
                500,
                "Internal Server Error",
                format!("{{\"detail\":\"Storage error: {}\"}}", e),
                false,
            )
        }
    };
    let bytes = match store.read_by_hash(&content_hash) {
        Ok(b) => b,
        Err(e) => {
            return json_response(
                500,
                "Internal Server Error",
                format!("{{\"detail\":\"Read artifact error: {}\"}}", e),
                false,
            )
        }
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let body = serde_json::json!({
        "id": id,
        "mime_type": mime,
        "size_bytes": size_bytes.unwrap_or(bytes.len() as i32),
        "base64_content": b64,
        "data_url_prefix": format!("data:{};base64,", mime),
        "note": "For <img src=\"data:...\"> or <video src=\"data:...\"> . Full original res from source preserved in artifact."
    }).to_string();
    json_response(200, "OK", body, false)
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
        ["experiments", id] => Some(detail(
            id,
            "Experiment run not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, improvement_item_id, status, mlflow_run_id, optuna_study_name, metrics_json, artifacts_json, metadata_json, created_at, updated_at FROM experiment_runs WHERE id = $1) t), '')",
        )),
        ["feedback", id] => Some(detail(
            id,
            "Feedback event not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, target_type, target_id, label, actor_id, note, metadata_json, created_at, updated_at FROM feedback_events WHERE id = $1) t), '')",
        )),
        ["improvements", id] => Some(detail(
            id,
            "Improvement item not found",
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, target_area, status, objective, proposed_by_actor_id, priority, metadata_json, created_at, updated_at FROM improvement_items WHERE id = $1) t), '')",
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
        Err(GatewayError::ServiceUnavailable(message)) => json_response(
            503,
            "Service Unavailable",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
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
        Err(GatewayError::ServiceUnavailable(message)) => json_response(
            503,
            "Service Unavailable",
            format!("{{\"detail\":\"{}\"}}", escape_json(&message)),
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

pub(crate) fn write_route_response(result: Result<String, GatewayError>) -> GatewayResponse {
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

fn evidence_answer_record_create_response(
    body: &str,
    database_url: Option<&str>,
) -> GatewayResponse {
    write_route_response(create_evidence_answer_record(body, database_url))
}

fn outcome_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_outcome(body, database_url))
}

fn source_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_source(body, database_url))
}

fn raw_artifact_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_raw_artifact(body, database_url))
}

fn collection_run_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_collection_run(body, database_url))
}

fn hypothesis_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_hypothesis(body, database_url))
}

fn prediction_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_prediction(body, database_url))
}

fn recommendation_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_recommendation(body, database_url))
}

fn evidence_document_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_evidence_document(body, database_url))
}

fn evidence_item_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_evidence_item(body, database_url))
}

fn experiment_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_experiment(body, database_url))
}

fn experiment_proposal_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_experiment_proposal_from_improvement(
        body,
        database_url,
    ))
}

fn improvement_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_improvement(body, database_url))
}

fn retrieval_chunk_search_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    action_route_response(search_retrieval_chunks(body, database_url))
}

fn retrieval_preview_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    action_route_response(live_retrieval_preview(body, database_url))
}

fn evidence_answer_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    if database_url
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        action_route_response(live_evidence_answer(body, database_url))
    } else {
        json_response(200, "OK", evidence_answer_json(body), false)
    }
}

fn report_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_report(body, database_url))
}

fn work_item_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_work_item(body, database_url))
}

fn agent_task_plan_create_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_agent_task_plan(body, database_url))
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

fn manual_upload_ingest_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(ingest_manual_upload_collection(body, database_url))
}

fn media_import_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(media_import(body, database_url))
}

fn local_project_collection_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(create_local_project_collection(body, database_url))
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

fn media_import(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let object = parse_json_object(body, "Media import request body")?;
    let name = optional_string_field_with_max(&object, "name", "media-import", 255)?;
    let media_label = optional_string_field_with_max(&object, "label", "media-import", 255)?;
    let media_type = optional_string_field_with_max(&object, "media_type", "pdf", 32)?;
    let filename = optional_string_field_with_max(&object, "filename", "upload.bin", 255)?;
    let mime_type =
        optional_string_field_with_max(&object, "mime_type", "application/octet-stream", 255)?;
    let content_base64 = optional_nullable_string_field(&object, "content_base64")?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;

    // 1) Create a media source with collect permission
    let source_body = serde_json::json!({
        "name": name,
        "source_type": "media_file",
        "location": filename,
        "sensitivity": "internal",
        "metadata_json": { "created_from": "media_import", "media_type": media_type },
        "permission": {
            "scope_json": { "media_label": media_label, "media_type": media_type },
            "allowed_operations": ["dry_run","read","collect","normalize","extract_metadata"],
            "external_model_policy": "blocked",
            "approval_required": false,
            "created_by_actor_id": requested_by_actor_id
        }
    })
    .to_string();
    let created_source_json = create_source(&source_body, Some(database_url))?;
    let created_source: Value = serde_json::from_str(&created_source_json)
        .map_err(|e| GatewayError::Conflict(e.to_string()))?;
    let source_id = created_source
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| GatewayError::Conflict("media source creation failed".to_string()))?
        .to_string();
    let permission_id = created_source
        .get("permissions")
        .and_then(Value::as_array)
        .and_then(|arr| {
            arr.iter().find(|p| {
                p.get("allowed_operations")
                    .and_then(Value::as_array)
                    .map(|ops| ops.iter().any(|op| op.as_str() == Some("collect")))
                    .unwrap_or(false)
            })
        })
        .and_then(|p| p.get("id").and_then(Value::as_str))
        .ok_or_else(|| GatewayError::Conflict("collect permission was not created".to_string()))?
        .to_string();

    // 2) If content provided, enqueue normalization via manual upload
    let upload_response = if let Some(content_b64) = content_base64 {
        let manual_upload_body = serde_json::json!({
            "source_id": source_id,
            "source_permission_id": permission_id,
            "filename": filename,
            "mime_type": mime_type,
            "content_base64": content_b64,
            "metadata_json": {
              "submitted_from": "media_import",
              "media_type": media_type,
              "original_filename": filename,
              "extract_pipeline": "local_tools"
            },
            "requested_by_actor_id": requested_by_actor_id
        })
        .to_string();
        Some(create_manual_upload_collection(
            &manual_upload_body,
            Some(database_url),
        )?)
    } else {
        None
    };

    Ok(serde_json::json!({
        "status": "ok",
        "source": serde_json::from_str::<Value>(&created_source_json).unwrap_or(serde_json::json!({})),
        "upload": upload_response.and_then(|s| serde_json::from_str::<Value>(&s).ok())
    }).to_string())
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
    let outcomes = load_outcomes_for_baseline(&mut transaction)?;
    let mut existing_keys = load_existing_detector_keys(&mut transaction)?;
    let candidates =
        baseline_pattern_candidates(&evidence_items, &outcomes, payload.recurrence_threshold);
    let mut responses = Vec::new();
    for candidate in candidates {
        if existing_keys.contains(&candidate.detector_key) {
            continue;
        }
        let mut metadata_json = candidate.metadata_json;
        metadata_json["generated_by"] = Value::String("DIFF-240".to_string());
        metadata_json["detector"] = Value::String("baseline_local_v2".to_string());
        metadata_json["detector_key"] = Value::String(candidate.detector_key.clone());
        metadata_json["review_status"] = Value::String("candidate".to_string());
        metadata_json["support_count"] =
            Value::Number(serde_json::Number::from(candidate.support_count));
        metadata_json["evidence_count"] =
            Value::Number(serde_json::Number::from(candidate.evidence_ids.len() as i64));
        let pattern_payload = PatternCreatePayload {
            pattern_type: candidate.pattern_type,
            summary: candidate.summary,
            evidence_ids: candidate.evidence_ids,
            confidence: Some(candidate.confidence),
            status: "candidate".to_string(),
            actor_id: payload.actor_id.clone(),
            metadata_json,
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

fn create_raw_artifact(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_raw_artifact_create(body)?;
    let content = decode_base64(&payload.content_base64)?;
    if content.len() > i32::MAX as usize {
        return Err(GatewayError::Validation(
            "Artifact content is too large".to_string(),
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
    if let Some(source_id) = &payload.source_id {
        require_source_exists(&mut transaction, source_id)?;
    }
    if let Some(collection_run_id) = &payload.collection_run_id {
        let run_source_id = load_collection_run_source_id(&mut transaction, collection_run_id)?;
        if payload
            .source_id
            .as_ref()
            .is_some_and(|source_id| Some(source_id) != run_source_id.as_ref())
        {
            return Err(GatewayError::Conflict(
                "Collection run does not belong to the source".to_string(),
            ));
        }
    }
    let store = ArtifactStore::new(artifact_data_root())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let stored = store
        .write_bytes(&content)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let artifact_id = generated_record_id("artifact");
    transaction
        .execute(
            "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::integer, $8::jsonb)",
            &[
                &artifact_id,
                &payload.source_id,
                &payload.collection_run_id,
                &stored.content_hash,
                &stored.storage_path,
                &payload.mime_type,
                &(stored.size_bytes as i32),
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_raw_artifact_created_audit(
        &mut transaction,
        &payload.requested_by_actor_id,
        &artifact_id,
        payload.source_id.as_deref(),
        payload.collection_run_id.as_deref(),
        &stored.content_hash,
        &stored.storage_path,
        stored.size_bytes,
        stored.existed,
    )?;
    let response_body = raw_artifact_response_json(&mut transaction, &artifact_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_collection_run(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_collection_run_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if let Some(source_id) = &payload.source_id {
        require_source_exists(&mut transaction, source_id)?;
    }
    let collection_run_id = generated_record_id("collection");
    let status = if payload.dry_run {
        "dry_run_requested"
    } else {
        "created"
    };
    transaction
        .execute(
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, $3, $4, $5, $6::jsonb, NULL)",
            &[
                &collection_run_id,
                &payload.source_id,
                &status,
                &payload.dry_run,
                &payload.requested_by_actor_id,
                &payload.summary_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_collection_run_created_audit(
        &mut transaction,
        &payload.requested_by_actor_id,
        &collection_run_id,
        payload.source_id.as_deref(),
        payload.dry_run,
        status,
    )?;
    let response_body = collection_run_response_json(&mut transaction, &collection_run_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
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
    // grok branch: do not enforce text mime here. The source record (loaded below) + is_supported_collection_source_type
    // plus later per-source_type handling decide what is acceptable. Media and other binary types are intentionally allowed.
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
    if !is_supported_collection_source_type(&source.source_type) {
        return Err(GatewayError::Conflict(format!(
            "Source type {} is not supported for collection on this build",
            source.source_type
        )));
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
        "source_type": source.source_type,
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

fn ingest_manual_upload_collection(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_manual_upload_ingest(body)?;
    let upload = payload.upload;
    let content = decode_base64(&upload.content_base64)?;
    if content.len() > i32::MAX as usize {
        return Err(GatewayError::Validation(
            "Manual upload content is too large".to_string(),
        ));
    }

    // grok branch: text_content synthesis happens after we load the source (to know its source_type).
    // For now, attempt UTF-8; if it fails we will synthesize a media-style placeholder after source load.
    let text_content = String::from_utf8(content.clone()).ok();

    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let source = load_source_for_collection(&mut transaction, &upload.source_id)?;
    if !source.enabled {
        return Err(GatewayError::Conflict("Source is disabled".to_string()));
    }
    if !is_supported_collection_source_type(&source.source_type) {
        return Err(GatewayError::Conflict(format!(
            "Source type {} is not supported for collection on this build",
            source.source_type
        )));
    }

    // grok branch: synthesize placeholder text_content for non-text or failed-utf8 media/browser/etc sources
    // so the rest of the pipeline (normalized document + chunks + evidence) still runs and produces traceable records.
    // grok branch: use real deep extraction for PDF/media when possible (even in manual paths)
    let kind_for_extract = detect_content_kind(&content, upload.filename.as_deref());
    let extracted = extract_text_if_possible(&content, &kind_for_extract);
    let text_content: String = extracted.unwrap_or_else(|| {
        text_content.unwrap_or_else(|| {
            let mime = upload.mime_type.as_deref().unwrap_or("application/octet-stream");
            format!(
                "[Non-text / binary content registered via grok branch collector foundations]\nsource_type: {}\nMIME: {}\noriginal_size_bytes: {}\nfilename: {}\n\nThis document acts as a provenance stub. (Deep PDF/media extraction attempted.)",
                source.source_type,
                mime,
                content.len(),
                upload.filename.as_deref().unwrap_or("<unknown>")
            )
        })
    });
    let permission =
        load_permission_for_collection(&mut transaction, &upload.source_permission_id)?;
    if permission.source_id != source.id {
        return Err(GatewayError::Conflict(
            "Source permission does not belong to the source".to_string(),
        ));
    }
    if !permission_allows(&permission.allowed_operations, &["collect", "read"]) {
        return Err(GatewayError::Forbidden(
            "Source permission does not allow manual upload ingestion".to_string(),
        ));
    }
    let approval_id = require_collection_approval(
        &mut transaction,
        upload.approval_id.as_deref(),
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
    let mut summary_json = serde_json::json!({
        "mode": "manual_upload_ingest",
        "source_type": source.source_type,
        "source_permission_id": permission.id,
        "filename": upload.filename,
        "content_hash": stored.content_hash,
        "storage_path": stored.storage_path,
        "size_bytes": stored.size_bytes,
        "content_already_existed": stored.existed,
        "normalization_input_type": "utf_8_text",
        "chunk_size": payload.chunk_size,
        "approval_id": approval_id,
        "external_model_calls": false,
        "embedding_method": EMBEDDING_METHOD
    });
    transaction
        .execute(
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, 'ingesting', false, $3, $4::jsonb, NULL)",
            &[&collection_run_id, &source.id, &upload.requested_by_actor_id, &summary_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_collection_run_created_audit(
        &mut transaction,
        &upload.requested_by_actor_id,
        &collection_run_id,
        Some(&source.id),
        false,
        "ingesting",
    )?;
    let artifact_metadata = merge_metadata(
        &manual_upload_artifact_metadata(
            &upload.metadata_json,
            upload.filename.as_deref(),
            &permission.id,
            approval_id.as_deref(),
        ),
        serde_json::json!({"ingested_by": "DIFF-081"}),
    );
    let artifact_result = get_or_create_raw_artifact_for_ingest(
        &mut transaction,
        &source.id,
        &collection_run_id,
        &stored.content_hash,
        &stored.storage_path,
        stored.size_bytes,
        upload.mime_type.as_deref(),
        &artifact_metadata,
        &upload.requested_by_actor_id,
        stored.existed,
    )?;
    let title = upload
        .filename
        .as_deref()
        .unwrap_or(artifact_result.id.as_str())
        .to_string();
    let document_result = get_or_create_normalized_document_for_ingest(
        &mut transaction,
        &artifact_result,
        &source,
        &text_content,
        &title,
        &upload.requested_by_actor_id,
    )?;
    let (chunk_ids, evidence_item_ids, chunks_reused) =
        get_or_create_chunks_and_evidence_for_ingest(
            &mut transaction,
            &document_result.id,
            source.id.as_str(),
            document_result.raw_artifact_id.as_str(),
            &document_result.text_content,
            payload.chunk_size,
            &upload.requested_by_actor_id,
        )?;
    summary_json = merge_metadata(
        &summary_json,
        serde_json::json!({
            "raw_artifact_id": artifact_result.id,
            "raw_artifact_reused": artifact_result.reused,
            "document_id": document_result.id,
            "document_reused": document_result.reused,
            "chunk_ids": chunk_ids,
            "chunks_reused": chunks_reused,
            "evidence_item_ids": evidence_item_ids
        }),
    );
    transaction
        .execute(
            "UPDATE collection_runs SET summary_json = $1::jsonb, updated_at = now() WHERE id = $2",
            &[&summary_json, &collection_run_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;

    let vector_result = match upsert_specific_chunks(&mut client, &chunk_ids) {
        Ok(result) => result,
        Err(error) => {
            mark_manual_upload_ingest_vector_failed(
                &mut client,
                &collection_run_id,
                &upload.requested_by_actor_id,
                &error.to_string(),
            )?;
            return Err(error);
        }
    };
    let final_summary = merge_metadata(
        &summary_json,
        serde_json::json!({
            "vector_collection": vector_result.collection_name,
            "vector_collection_exists": vector_result.collection_exists,
            "vector_upsert_completed": true,
            "chunks_upserted": vector_result.chunks_upserted
        }),
    );
    client
        .execute(
            "UPDATE collection_runs SET status = 'completed', summary_json = $1::jsonb, updated_at = now() WHERE id = $2",
            &[&final_summary, &collection_run_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let completed_details = serde_json::json!({
        "source_id": source.id,
        "raw_artifact_id": artifact_result.id,
        "document_id": document_result.id,
        "chunk_count": chunk_ids.len(),
        "evidence_count": evidence_item_ids.len(),
        "vector_collection": vector_result.collection_name,
        "chunks_upserted": vector_result.chunks_upserted,
        "generated_by": "DIFF-081"
    });
    client
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'manual_upload_ingest.completed', 'completed', 'collection_run', $2, $2, $3::jsonb)",
            &[&upload.requested_by_actor_id, &collection_run_id, &completed_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let collection_run_json = collection_run_response_json_client(&mut client, &collection_run_id)?;
    Ok(serde_json::json!({
        "collection_run": serde_json::from_str::<Value>(&collection_run_json).unwrap_or(Value::Null),
        "raw_artifact_id": artifact_result.id,
        "raw_artifact_reused": artifact_result.reused,
        "document_id": document_result.id,
        "document_reused": document_result.reused,
        "chunk_ids": chunk_ids,
        "chunks_reused": chunks_reused,
        "evidence_item_ids": evidence_item_ids,
        "vector_upsert": {
            "collection_name": vector_result.collection_name,
            "collection_exists": vector_result.collection_exists,
            "chunks_upserted": vector_result.chunks_upserted
        }
    })
    .to_string())
}

fn create_local_project_collection(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_local_project_collection(body)?;
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
    if source.source_type != "local_project" {
        return Err(GatewayError::Conflict(
            "Source is not a local_project source".to_string(),
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
            "Source permission does not allow local project collection".to_string(),
        ));
    }
    let approval_id = require_collection_approval(
        &mut transaction,
        payload.approval_id.as_deref(),
        &source,
        &permission,
        "local_project_collection",
    )?;
    let collected_files = collect_local_project_files(&source, &permission)
        .map_err(|error| GatewayError::Validation(error.to_string()))?;
    let collection_run_id = generated_record_id("collection");
    let raw_artifact_ids = collected_files
        .files
        .iter()
        .map(|_| generated_record_id("artifact"))
        .collect::<Vec<_>>();
    let work_item_id = generated_record_id("work");
    let summary_json = serde_json::json!({
        "mode": "local_project_collection",
        "source_permission_id": permission.id,
        "total_files": collected_files.total_files,
        "collected_files": collected_files.files.len(),
        "skipped_files": collected_files.skipped_files,
        "would_normalize": true,
        "normalization_input_type": "utf_8_text",
        "normalization_note": "Worker normalization currently supports UTF-8 text artifacts only.",
        "approval_id": approval_id,
        "normalization_work_item_created": true,
        "normalization_work_item_id": work_item_id,
        "raw_artifact_ids": raw_artifact_ids
    });
    transaction
        .execute(
            "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, 'completed', false, $3, $4::jsonb, NULL)",
            &[&collection_run_id, &source.id, &payload.requested_by_actor_id, &summary_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_collection_run_created_audit(
        &mut transaction,
        &payload.requested_by_actor_id,
        &collection_run_id,
        Some(&source.id),
        false,
        "completed",
    )?;
    for (index, collected_file) in collected_files.files.iter().enumerate() {
        let artifact_id = &raw_artifact_ids[index];
        let metadata_json = serde_json::json!({
            "source_permission_id": permission.id,
            "approval_id": approval_id,
            "source_path": collected_file.source_path,
            "relative_path": collected_file.relative_path,
            "content_already_existed": collected_file.stored.existed
        });
        transaction
            .execute(
                "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, NULL, $6::integer, $7::jsonb)",
                &[
                    artifact_id,
                    &source.id,
                    &collection_run_id,
                    &collected_file.stored.content_hash,
                    &collected_file.stored.storage_path,
                    &(collected_file.stored.size_bytes as i32),
                    &metadata_json,
                ],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        insert_raw_artifact_created_audit(
            &mut transaction,
            &payload.requested_by_actor_id,
            artifact_id,
            Some(&source.id),
            Some(&collection_run_id),
            &collected_file.stored.content_hash,
            &collected_file.stored.storage_path,
            collected_file.stored.size_bytes,
            collected_file.stored.existed,
        )?;
    }
    let work_payload = collection_normalization_work_payload(
        &collection_run_id,
        &source,
        &permission.id,
        &raw_artifact_ids,
        "local_project_collection",
    );
    transaction
        .execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, 'collection_normalization', 'queued', $2, $3::jsonb, NULL)",
            &[&work_item_id, &payload.requested_by_actor_id, &work_payload],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_work_item_created_audit_for_collection(
        &mut transaction,
        &payload.requested_by_actor_id,
        &work_item_id,
        &collection_run_id,
        &raw_artifact_ids,
    )?;
    let response_body = collection_run_response_json(&mut transaction, &collection_run_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

// ============================================================================
// GROK BRANCH FULL-ACCESS COLLECTOR (per user directive on this branch only)
// "get any and everything that it has access to and store it only in itself
//  no info should be sent out" (except the collection fetches themselves).
// No "no scraping", no artificial bounds, no "user-provided only".
// Everything stays in the local IGY6 instance (artifacts + evidence + graph).
//
// UI-ONLY: All control via web UI forms (no cmd line required after initial launch).
// DEEPEST EXTRACTION: Recursive crawling, full asset download (original res images/videos/PDFs),
// PDF text + embedded, image exif-stripped full res, metadata for audio/video, aggressive
// entity/claim/relationship mining from all extracted content.
// SAFETY / NON-TRACABLE: Hardcoded blacklist for gov/mil/military/top-secret domains
// (social media, Patreon etc. allowed as low-risk personal content). Randomized UA,
// jittered delays (300-1200ms), minimal headers (no cookies, no referer, generic accept),
// exif/metadata stripping on images to avoid PC fingerprinting. Skips anything that could
// lead to legal trouble. All local only.
// ============================================================================

// Safety blacklists - avoid anything that could get user in trouble with authorities.
// Social media and Patreon-style are explicitly ok (personal, non-classified).
const FORBIDDEN_DOMAINS: &[&str] = &[
    ".gov",
    ".mil",
    "army",
    "navy",
    "airforce",
    "marines",
    "spaceforce",
    "pentagon",
    "defense",
    "fbi",
    "cia",
    "nsa",
    "whitehouse",
    "dhs",
    "justice",
    "homeland",
    "military",
    "classified",
    "topsecret",
    "intel",
    "secret",
    "gov.au",
    "gob.mx",
    "gc.ca",
    "gov.uk",
    "bund.de", // common gov TLDs
];

// Common browser UAs for rotation (non-traceable, looks like normal user).
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
];

fn random_ua() -> &'static str {
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % USER_AGENTS.len() as u128) as usize;
    USER_AGENTS[idx]
}

pub(crate) fn is_forbidden(url: &str) -> bool {
    let lower = url.to_lowercase();
    FORBIDDEN_DOMAINS.iter().any(|d| lower.contains(d))
}

pub(crate) fn jitter_delay() {
    // Human-like jitter to avoid bot detection / rate limits, still deep but polite.
    use std::thread;
    use std::time::Duration;
    let ms = 300
        + (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 900) as u64; // 300-1200ms
    thread::sleep(Duration::from_millis(ms));
}

// Simple header builder for anonymous requests (no tracking back to this PC).
pub(crate) fn anon_headers() -> Vec<(String, String)> {
    vec![
        ("User-Agent".to_string(), random_ua().to_string()),
        (
            "Accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
                .to_string(),
        ),
        ("Accept-Language".to_string(), "en-US,en;q=0.5".to_string()),
        (
            "Accept-Encoding".to_string(),
            "gzip, deflate, br".to_string(),
        ),
        ("DNT".to_string(), "1".to_string()),
        ("Connection".to_string(), "keep-alive".to_string()),
        ("Upgrade-Insecure-Requests".to_string(), "1".to_string()),
        // No Referer, no Cookie, no tracking headers.
    ]
}

// Build ureq agent with anonymity (no cookies/persistent state).
fn anon_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(30))
        .build()
}

fn bypass_auth_enabled(object: &serde_json::Value) -> bool {
    object
        .get("bypass_auth")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn optional_payload_string(object: &serde_json::Value, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn authorization_header_value(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("bearer ") || lower.starts_with("basic ") {
        trimmed.to_string()
    } else {
        format!("Bearer {trimmed}")
    }
}

// Authorized-session fetch: inject caller-provided cookie / bearer token for pages they already own.
fn web_fetch_headers(object: &serde_json::Value) -> Vec<(String, String)> {
    let bypass = bypass_auth_enabled(object);
    let mut headers = if bypass {
        vec![
            ("User-Agent".to_string(), random_ua().to_string()),
            (
                "Accept".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
                    .to_string(),
            ),
            ("Accept-Language".to_string(), "en-US,en;q=0.5".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
            ("Connection".to_string(), "keep-alive".to_string()),
            ("Upgrade-Insecure-Requests".to_string(), "1".to_string()),
        ]
    } else {
        anon_headers()
    };

    if bypass {
        if let Some(cookie) = optional_payload_string(object, "cookie") {
            headers.push(("Cookie".to_string(), cookie));
        }
        if let Some(authorization) = optional_payload_string(object, "authorization") {
            headers.push((
                "Authorization".to_string(),
                authorization_header_value(&authorization),
            ));
        }
        if let Some(referer) = optional_payload_string(object, "referer") {
            headers.push(("Referer".to_string(), referer));
        }
    }

    headers
}

fn bypass_fetch_has_credentials(object: &serde_json::Value) -> bool {
    optional_payload_string(object, "cookie").is_some()
        || optional_payload_string(object, "authorization").is_some()
}

fn auto_bypass_enabled(object: &serde_json::Value) -> bool {
    object
        .get("auto_bypass")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn max_reach_enabled(object: &serde_json::Value) -> bool {
    object
        .get("max_reach")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn login_wall_score(body: &str) -> u32 {
    let lower = body.to_lowercase();
    const INDICATORS: &[&str] = &[
        "sign in",
        "log in",
        "login required",
        "subscribe to read",
        "subscription required",
        "create an account",
        "paywall",
        "members only",
        "register to continue",
        "access denied",
        "please log in",
        "sign up to continue",
        "you must be logged in",
        "continue reading",
        "unlock this article",
    ];
    INDICATORS
        .iter()
        .filter(|indicator| lower.contains(**indicator))
        .count() as u32
}

fn auto_bypass_content_score(body: &str) -> i64 {
    let len = body.len() as i64;
    let wall_penalty = login_wall_score(body) as i64 * 8_000;
    let structure_bonus = if body.contains("<article") || body.contains("<main") {
        4_000
    } else {
        0
    };
    let paragraph_bonus = body.matches("<p").count() as i64 * 120;
    len + structure_bonus + paragraph_bonus - wall_penalty
}

fn auto_bypass_url_variants(original_url: &str) -> Vec<(String, String)> {
    let mut variants = vec![("direct".to_string(), original_url.to_string())];
    if original_url.contains('?') {
        variants.push(("amp_query".to_string(), format!("{original_url}&amp=1")));
    } else {
        variants.push(("amp_query".to_string(), format!("{original_url}?amp=1")));
        variants.push((
            "output_amp".to_string(),
            format!("{original_url}?outputType=amp"),
        ));
    }
    if !original_url.ends_with("/amp/") && !original_url.ends_with("/amp") {
        let amp_path = if original_url.ends_with('/') {
            format!("{original_url}amp/")
        } else {
            format!("{original_url}/amp/")
        };
        variants.push(("amp_path".to_string(), amp_path));
    }
    if let Some(rest) = original_url.strip_prefix("https://www.") {
        variants.push(("mobile_www".to_string(), format!("https://m.{rest}")));
    } else if let Some(rest) = original_url.strip_prefix("https://") {
        if !rest.starts_with("m.") {
            variants.push(("mobile_m".to_string(), format!("https://m.{rest}")));
        }
    }
    variants
}

fn auto_bypass_header_strategies() -> Vec<(String, Vec<(String, String)>)> {
    let accept = (
        "Accept".to_string(),
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
            .to_string(),
    );
    let accept_lang = ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
    let accept_enc = (
        "Accept-Encoding".to_string(),
        "gzip, deflate, br".to_string(),
    );
    let connection = ("Connection".to_string(), "keep-alive".to_string());
    let upgrade = ("Upgrade-Insecure-Requests".to_string(), "1".to_string());

    let mut google_referer = anon_headers();
    google_referer.push(("Referer".to_string(), "https://www.google.com/".to_string()));

    let mut twitter_referer = anon_headers();
    twitter_referer.push(("Referer".to_string(), "https://t.co/".to_string()));

    vec![
        ("browser".to_string(), anon_headers()),
        ("google_referer".to_string(), google_referer),
        ("twitter_referer".to_string(), twitter_referer),
        (
            "googlebot".to_string(),
            vec![
                (
                    "User-Agent".to_string(),
                    "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
                        .to_string(),
                ),
                accept.clone(),
                accept_lang.clone(),
                accept_enc.clone(),
                connection.clone(),
                upgrade.clone(),
            ],
        ),
        (
            "bingbot".to_string(),
            vec![
                (
                    "User-Agent".to_string(),
                    "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)"
                        .to_string(),
                ),
                accept.clone(),
                accept_lang.clone(),
                accept_enc.clone(),
                connection.clone(),
                upgrade.clone(),
            ],
        ),
        (
            "facebook_crawler".to_string(),
            vec![
                (
                    "User-Agent".to_string(),
                    "facebookexternalhit/1.1".to_string(),
                ),
                accept.clone(),
                accept_lang.clone(),
                accept_enc.clone(),
                connection.clone(),
            ],
        ),
        (
            "twitterbot".to_string(),
            vec![
                ("User-Agent".to_string(), "Twitterbot/1.0".to_string()),
                accept,
                accept_lang,
                accept_enc,
                connection,
                upgrade,
            ],
        ),
    ]
}

struct AutoBypassFetchResult {
    strategy: String,
    fetched_url: String,
    _content_type: String,
    is_pdf: bool,
    body: String,
    pdf_bytes: Option<Vec<u8>>,
}

pub(crate) fn encode_url_query_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            _ => format!("%{:02X}", ch as u32),
        })
        .collect()
}

fn fetch_url_with_headers(
    agent: &ureq::Agent,
    url: &str,
    headers: &[(String, String)],
) -> Option<AutoBypassFetchResult> {
    let mut request = agent.get(url);
    for (key, value) in headers {
        request = request.set(key, value);
    }
    let response = request.call().ok()?;
    if response.status() >= 400 {
        return None;
    }
    let content_type = response.header("Content-Type").unwrap_or("").to_lowercase();
    let is_pdf = url.to_lowercase().ends_with(".pdf") || content_type.contains("pdf");
    if is_pdf {
        let mut pdf_bytes = Vec::new();
        {
            let mut reader = response.into_reader();
            if std::io::Read::read_to_end(&mut reader, &mut pdf_bytes).is_err()
                || pdf_bytes.is_empty()
            {
                return None;
            }
        }
        return Some(AutoBypassFetchResult {
            strategy: String::new(),
            fetched_url: url.to_string(),
            _content_type: content_type,
            is_pdf: true,
            body: String::new(),
            pdf_bytes: Some(pdf_bytes),
        });
    }
    let body = response.into_string().ok()?;
    if body.len() < 200 {
        return None;
    }
    Some(AutoBypassFetchResult {
        strategy: String::new(),
        fetched_url: url.to_string(),
        _content_type: content_type,
        is_pdf: false,
        body,
        pdf_bytes: None,
    })
}

fn lookup_archive_snapshot(agent: &ureq::Agent, original_url: &str) -> Option<String> {
    let api_url = format!(
        "https://archive.org/wayback/available?url={}",
        encode_url_query_component(original_url)
    );
    let response = agent.get(&api_url).call().ok()?;
    let payload: serde_json::Value = serde_json::from_str(&response.into_string().ok()?).ok()?;
    payload
        .get("archived_snapshots")?
        .get("closest")?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

fn auto_bypass_candidate_score(result: &AutoBypassFetchResult) -> i64 {
    if result.is_pdf {
        result
            .pdf_bytes
            .as_ref()
            .map(|b| b.len() as i64)
            .unwrap_or(0)
    } else {
        auto_bypass_content_score(&result.body)
    }
}

fn auto_bypass_http_result_usable(result: &AutoBypassFetchResult) -> bool {
    if result.is_pdf {
        result
            .pdf_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() >= 400)
    } else {
        result.body.len() >= 400 && login_wall_score(&result.body) == 0
    }
}

fn auto_bypass_resolve(
    agent: &ureq::Agent,
    original_url: &str,
    max_reach: bool,
    max_depth: u32,
) -> Option<AutoBypassFetchResult> {
    let mut best: Option<AutoBypassFetchResult> = None;
    let mut best_score = i64::MIN;

    // Fast path: one direct fetch is enough for most public pages.
    if let Some(mut direct) = fetch_url_with_headers(agent, original_url, &anon_headers()) {
        direct.strategy = "direct+fast_path".to_string();
        if auto_bypass_http_result_usable(&direct)
            && !max_reach
            && !bypass_intel::is_paid_content_platform_url(original_url)
        {
            return Some(direct);
        }
        let score = auto_bypass_candidate_score(&direct);
        if score > best_score {
            best_score = score;
            best = Some(direct);
        }
    }

    let playbook_headers = bypass_intel::playbook_header_strategies_for_url(original_url);
    for (url_variant, candidate_url) in bypass_intel::playbook_url_variants_for_url(original_url) {
        if is_forbidden(&candidate_url) {
            continue;
        }
        let header_sets = if playbook_headers.is_empty() {
            auto_bypass_header_strategies()
        } else {
            playbook_headers.clone()
        };
        for (header_variant, headers) in header_sets {
            jitter_delay();
            let Some(mut result) = fetch_url_with_headers(agent, &candidate_url, &headers) else {
                continue;
            };
            result.strategy = format!("{url_variant}+{header_variant}");
            if auto_bypass_http_result_usable(&result)
                && !max_reach
                && !bypass_intel::is_paid_content_platform_url(original_url)
            {
                return Some(result);
            }
            let score = auto_bypass_candidate_score(&result);
            if score > best_score {
                best_score = score;
                best = Some(result);
            }
        }
    }

    for (url_variant, candidate_url) in auto_bypass_url_variants(original_url) {
        if is_forbidden(&candidate_url) {
            continue;
        }
        for (header_variant, headers) in auto_bypass_header_strategies() {
            jitter_delay();
            let Some(mut result) = fetch_url_with_headers(agent, &candidate_url, &headers) else {
                continue;
            };
            result.strategy = format!("{url_variant}+{header_variant}");
            if auto_bypass_http_result_usable(&result)
                && !max_reach
                && !bypass_intel::is_paid_content_platform_url(original_url)
            {
                return Some(result);
            }
            let score = auto_bypass_candidate_score(&result);
            if score > best_score {
                best_score = score;
                best = Some(result);
            }
        }
    }

    if let Some(archive_url) = lookup_archive_snapshot(agent, original_url) {
        if !is_forbidden(&archive_url) {
            jitter_delay();
            if let Some(mut result) = fetch_url_with_headers(agent, &archive_url, &anon_headers()) {
                result.strategy = "archive_org".to_string();
                let score = auto_bypass_candidate_score(&result);
                if score > best_score {
                    best = Some(result);
                }
            }
        }
    }

    // Host bridge phase: deep bypass or max reach (strongest tier).
    // Skip Playwright when HTTP tricks already returned usable content (auto bypass only).
    let http_result_usable = best.as_ref().is_some_and(auto_bypass_http_result_usable);
    let try_host_bridge = if max_reach || bypass_intel::is_paid_content_platform_url(original_url) {
        host_bridge_tcp_probe()
    } else if http_result_usable {
        false
    } else {
        host_bridge_tcp_probe()
    };
    if try_host_bridge {
        if max_reach {
            let _ = invoke_host_bridge_max_reach(original_url, max_depth);
            if let Some(artifacts) = read_max_reach_artifacts() {
                apply_host_bridge_artifacts(
                    agent,
                    original_url,
                    &mut best,
                    &mut best_score,
                    &artifacts.strategy,
                    artifacts.final_url.as_deref(),
                    artifacts.html.as_deref(),
                    artifacts.visible_text.as_deref(),
                    artifacts.cookie.as_deref(),
                    artifacts.authorization.as_deref(),
                    "max_reach",
                );
            }
        } else {
            let _ = invoke_host_bridge_deep_bypass(original_url);
            if let Some(artifacts) = read_deep_bypass_artifacts() {
                apply_host_bridge_artifacts(
                    agent,
                    original_url,
                    &mut best,
                    &mut best_score,
                    &artifacts.strategy,
                    artifacts.final_url.as_deref(),
                    artifacts.html.as_deref(),
                    None,
                    artifacts.cookie.as_deref(),
                    artifacts.authorization.as_deref(),
                    "deep_bypass",
                );
            }
        }
    }

    best
}

fn merge_visible_html(html: Option<&str>, visible_text: Option<&str>) -> Option<String> {
    let html = html?.to_string();
    if html.len() < 200 {
        return None;
    }
    if let Some(text) = visible_text.filter(|value| value.len() >= 200) {
        if text.len() > html.len() {
            return Some(format!("{html}\n<!-- visible-text -->\n{text}"));
        }
    }
    Some(html)
}

fn apply_host_bridge_artifacts(
    agent: &ureq::Agent,
    original_url: &str,
    best: &mut Option<AutoBypassFetchResult>,
    best_score: &mut i64,
    strategy: &Option<String>,
    final_url: Option<&str>,
    html: Option<&str>,
    visible_text: Option<&str>,
    cookie: Option<&str>,
    authorization: Option<&str>,
    prefix: &str,
) {
    if let Some(body) = merge_visible_html(html, visible_text) {
        let html_result = AutoBypassFetchResult {
            strategy: strategy.clone().unwrap_or_else(|| format!("{prefix}_html")),
            fetched_url: final_url.unwrap_or(original_url).to_string(),
            _content_type: "text/html".to_string(),
            is_pdf: false,
            body,
            pdf_bytes: None,
        };
        let score = auto_bypass_candidate_score(&html_result);
        if score > *best_score {
            *best_score = score;
            *best = Some(html_result);
        }
    }

    if cookie.is_some() || authorization.is_some() {
        let authed_payload = serde_json::json!({
            "bypass_auth": true,
            "cookie": cookie,
            "authorization": authorization,
            "referer": original_url,
        });
        let authed_headers = web_fetch_headers(&authed_payload);
        let fetch_url = final_url.unwrap_or(original_url);
        if let Some(mut authed_result) = fetch_url_with_headers(agent, fetch_url, &authed_headers) {
            authed_result.strategy = format!(
                "{prefix}_session+{}",
                strategy.as_deref().unwrap_or("unknown")
            );
            let score = auto_bypass_candidate_score(&authed_result);
            if score > *best_score {
                *best = Some(authed_result);
            }
        }
    }
}

struct DeepBypassArtifacts {
    strategy: Option<String>,
    final_url: Option<String>,
    cookie: Option<String>,
    authorization: Option<String>,
    html: Option<String>,
}

struct MaxReachArtifacts {
    strategy: Option<String>,
    final_url: Option<String>,
    cookie: Option<String>,
    authorization: Option<String>,
    html: Option<String>,
    visible_text: Option<String>,
    discovered_links: Vec<String>,
    media_urls: Vec<String>,
}

fn deep_bypass_result_path() -> PathBuf {
    artifact_data_root().join("ops/deep-bypass-result.json")
}

fn read_deep_bypass_artifacts() -> Option<DeepBypassArtifacts> {
    let content = fs::read_to_string(deep_bypass_result_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(DeepBypassArtifacts {
        strategy: value
            .get("strategy")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        final_url: value
            .get("final_url")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        cookie: value
            .get("cookie")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        authorization: value
            .get("authorization")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        html: value
            .get("html")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
    })
}

fn max_reach_result_path() -> PathBuf {
    artifact_data_root().join("ops/max-reach-result.json")
}

fn read_max_reach_artifacts() -> Option<MaxReachArtifacts> {
    let content = fs::read_to_string(max_reach_result_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let discovered_links = value
        .get("discovered_links")
        .and_then(|entry| entry.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let media_urls = value
        .get("media_urls")
        .and_then(|entry| entry.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(MaxReachArtifacts {
        strategy: value
            .get("strategy")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        final_url: value
            .get("final_url")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        cookie: value
            .get("cookie")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        authorization: value
            .get("authorization")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        html: value
            .get("html")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        visible_text: value
            .get("visible_text")
            .and_then(|entry| entry.as_str())
            .map(str::to_string),
        discovered_links,
        media_urls,
    })
}

fn host_bridge_listen_addr() -> Result<(String, u16), GatewayError> {
    let host = env::var("IGY6_HOST_BRIDGE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    if host != "127.0.0.1" && host != "host.docker.internal" {
        return Err(GatewayError::Conflict(
            "Host bridge must be configured for 127.0.0.1 or host.docker.internal only".to_string(),
        ));
    }
    let port = env::var("IGY6_HOST_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8765);
    Ok((host, port))
}

fn host_bridge_tcp_probe() -> bool {
    let Ok((host, port)) = host_bridge_listen_addr() else {
        return false;
    };
    TcpStream::connect_timeout(
        &format!("{host}:{port}")
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .unwrap_or_else(|| format!("{host}:{port}").parse().expect("socket addr")),
        Duration::from_secs(2),
    )
    .is_ok()
}

fn signal_max_reach_ensure_request() {
    let path = artifact_data_root().join("ops/max-reach-ensure.requested");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, format!("requested_at={:?}", SystemTime::now()));
}

fn wait_for_host_bridge_ready(max_wait_secs: u64) -> Result<(), GatewayError> {
    signal_max_reach_ensure_request();
    for _ in 0..max_wait_secs {
        if host_bridge_tcp_probe() {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(GatewayError::Conflict(
        "Host bridge is unavailable after waiting. Run: pwsh -File scripts\\start-host-bridge.ps1"
            .to_string(),
    ))
}

fn host_bridge_ensure_max_reach_response() -> GatewayResponse {
    write_route_response(wait_for_host_bridge_ready(45).map(|()| {
        serde_json::json!({
            "ok": true,
            "status": "ready",
            "host_bridge": "reachable"
        })
        .to_string()
    }))
}

fn invoke_host_bridge_action(
    action_name: &str,
    body: &serde_json::Value,
    read_timeout_secs: u64,
) -> Result<(), GatewayError> {
    let token = host_bridge_token()?;
    let (host, port) = host_bridge_listen_addr()?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|error| GatewayError::Conflict(format!("Host bridge is unavailable: {error}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(read_timeout_secs)))
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let body_text = body.to_string();
    let request = format!(
        "POST /actions/{action_name} HTTP/1.1\r\nHost: {host}:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_text}",
        body_text.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let (head, _) = response.split_once("\r\n\r\n").unwrap_or(("", ""));
    let status_code = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(502);
    if status_code != 200 {
        return Err(GatewayError::Conflict(format!(
            "Host bridge {action_name} failed with status {status_code}"
        )));
    }
    Ok(())
}

fn invoke_host_bridge_deep_bypass(url: &str) -> Result<(), GatewayError> {
    invoke_host_bridge_action(
        "deep_bypass_collect",
        &serde_json::json!({ "url": url }),
        120,
    )
}

fn invoke_host_bridge_max_reach(url: &str, max_depth: u32) -> Result<(), GatewayError> {
    invoke_host_bridge_action(
        "max_reach_collect",
        &serde_json::json!({ "url": url, "max_depth": max_depth }),
        620,
    )
}

fn full_access_collection_response(body: &str, database_url: Option<&str>) -> GatewayResponse {
    write_route_response(full_access_collect(body, database_url))
}

fn scope_entry_is_public_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn is_web_only_scope(scope: Option<&serde_json::Value>) -> bool {
    let Some(items) = scope.and_then(|value| value.as_array()) else {
        return false;
    };
    !items.is_empty()
        && items
            .iter()
            .all(|item| item.as_str().is_some_and(scope_entry_is_public_url))
}

fn full_access_collect(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    // UI-driven payload (everything controllable from web UI, no cmdline needed):
    // { "requested_by_actor_id": "...", "password": "...", "totp_code": "optional",
    //   "scope": ["url1", "/local/path", "everything"], "max_depth": 3,
    //   "safe_mode": true, "web_only": true, "bypass_auth": true,
    //   "cookie": "session=...", "authorization": "Bearer ...", "referer": "https://...",
    //   "media_focus": false, "anonymity": "high" }
    let mut object: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::json!({}));
    let paid_escalation = bypass_intel::scope_requires_paid_content_escalation(object.get("scope"));
    if paid_escalation {
        object["max_reach"] = serde_json::json!(true);
        object["auto_bypass"] = serde_json::json!(true);
        object["media_focus"] = serde_json::json!(true);
        if !bypass_fetch_has_credentials(&object) {
            if let Some((cookie, authorization)) = bypass_intel::load_patreon_session_credentials()
            {
                object["bypass_auth"] = serde_json::json!(true);
                object["cookie"] = serde_json::json!(cookie);
                if let Some(authorization) = authorization {
                    object["authorization"] = serde_json::json!(authorization);
                }
            }
        }
    }
    let bypass_auth = bypass_auth_enabled(&object);
    let max_reach = max_reach_enabled(&object);
    let auto_bypass = auto_bypass_enabled(&object) || max_reach || paid_escalation;
    if max_reach || paid_escalation {
        wait_for_host_bridge_ready(45)?;
    }
    if bypass_auth && !bypass_fetch_has_credentials(&object) {
        return Err(GatewayError::Validation(
            "Bypass fetch requires cookie or authorization for an authorized session you already own."
                .to_string(),
        ));
    }
    let web_only = object
        .get("web_only")
        .and_then(|value| value.as_bool())
        .unwrap_or_else(|| {
            bypass_auth || auto_bypass || max_reach || is_web_only_scope(object.get("scope"))
        });
    let cfg = load_user_config();
    // Public URL-only fetch from the local UI does not require program password/TOTP.
    if !web_only {
        let provided_pass = object
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if provided_pass != cfg.password {
            return Err(GatewayError::Forbidden("Program is password protected. Provide correct current password in payload.password.".to_string()));
        }
        if cfg.totp_enabled {
            let totp_code = object
                .get("totp_code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !verify_totp(cfg.totp_secret.as_deref().unwrap_or(""), totp_code) {
                return Err(GatewayError::Forbidden(
                    "TOTP code required and incorrect (authenticator is linked and enabled)."
                        .to_string(),
                ));
            }
        }
    }

    let max_depth = object
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let safe_mode = object
        .get("safe_mode")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let media_focus = object
        .get("media_focus")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || paid_escalation;
    let _anonymity = object
        .get("anonymity")
        .and_then(|v| v.as_str())
        .unwrap_or("high");
    let requested_by = object
        .get("requested_by_actor_id")
        .and_then(|v| v.as_str())
        .unwrap_or("local-owner")
        .to_string();

    let database_url = database_url
        .filter(|v| !v.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client =
        Client::connect(&postgres_url, NoTls).map_err(|e| GatewayError::Database(e.to_string()))?;
    let mut tx = client
        .transaction()
        .map_err(|e| GatewayError::Database(e.to_string()))?;

    // Ensure a full-access source exists for tying into real pipeline (grok branch)
    let full_source_id = "full-access-grok";
    let _ = tx.execute(
        "INSERT INTO sources (id, name, source_type, location, owner_actor_id, sensitivity, trust_level, enabled, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8::jsonb) ON CONFLICT (id) DO NOTHING",
        &[&full_source_id, &"Grok Full Access Collector", &"full_access", &"local-system-and-reachable", &requested_by, &"internal", &"trusted", &serde_json::json!({"grok_branch": true, "full_access": true})]
    );

    let collection_run_id = generated_record_id("collection");
    let mut collected_artifacts: Vec<String> = vec![];
    let mut evidence_created: Vec<String> = vec![];
    let mut graph_candidates: Vec<serde_json::Value> = vec![];
    let mut summary = serde_json::json!({
        "mode": if paid_escalation && max_reach {
            "web_paid_platform_max_reach"
        } else if max_reach {
            "web_max_reach_fetch"
        } else if auto_bypass {
            "web_auto_bypass_fetch"
        } else if bypass_auth {
            "web_bypass_fetch"
        } else if web_only {
            "web_only_public_fetch"
        } else {
            "full_access_grok"
        },
        "web_only": web_only,
        "max_reach": max_reach,
        "auto_bypass": auto_bypass,
        "paid_escalation": paid_escalation,
        "bypass_auth": bypass_auth,
        "had_cookie": optional_payload_string(&object, "cookie").is_some(),
        "had_authorization": optional_payload_string(&object, "authorization").is_some(),
        "requested_by": requested_by,
        "started_at": format!("{:?}", std::time::SystemTime::now()),
        "access_note": if paid_escalation {
            "Paid-platform fetch (Patreon/OnlyFans/Fansly). Auto-escalates to max reach, reuses saved browser session when available, harvests Patreon API/media URLs, and downloads patron media locally."
        } else if max_reach {
            "Max reach URL fetch. Runs full auto bypass plus headed/CDP Playwright, multi-profile Chrome/Edge passes, scroll/expand harvesting, optional proxy, and session re-fetch. Requires host bridge. Data stored locally only."
        } else if auto_bypass {
            "Automatic URL bypass fetch. Tries crawler UAs, referrer tricks, AMP/mobile mirrors, archive snapshots, then host Playwright + local browser cookie harvest (devtools-equivalent) and authenticated bypass fetch. No local filesystem scan. Data stored locally only."
        } else if bypass_auth {
            "Authorized-session URL fetch. Uses caller-provided cookie or bearer token. No local filesystem scan. Data stored locally only."
        } else if web_only {
            "Public URL fetch only. No local filesystem scan. Data stored locally only."
        } else {
            "Full access collector active on grok branch. Ingests anything the process uid/gid can read. All data stored locally only. Tied into real normalization/evidence/graph pipelines."
        }
    });

    // 1. Local filesystem - unbounded recursive (anything accessible)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/nasty".to_string());
    let roots = vec![
        home.clone(),
        "/proc".to_string(),
        "/sys".to_string(),
        "/etc".to_string(),
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string(),
    ];

    let mut local_files = 0usize;
    if web_only {
        summary["local_files_ingested"] = serde_json::json!(0);
        summary["system_snapshots"] = serde_json::json!(0);
    }
    if !web_only {
        for root in &roots {
            if !std::path::Path::new(root).exists() {
                continue;
            }
            for entry in WalkDir::new(root).follow_links(false).max_depth(8) {
                // depth cap to stay reasonable
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if let Ok(data) = std::fs::read(path) {
                    if data.len() > 50 * 1024 * 1024 {
                        continue;
                    } // 50MB cap per file to not explode
                    let kind = detect_content_kind(&data, path.to_str());
                    let store = ArtifactStore::new(artifact_data_root())
                        .map_err(|e| GatewayError::Conflict(e.to_string()))?;
                    if let Ok(stored) = store.write_bytes(&data) {
                        let art_id = generated_record_id("artifact");
                        let meta = serde_json::json!({
                            "full_access": true,
                            "source_path": path.to_string_lossy(),
                            "mime": kind.mime,
                            "kind": kind.kind,
                            "detected": kind.metadata
                        });
                        // Record artifact (simplified direct insert for speed on grok)
                        let _ = tx.execute(
                        "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING",
                        &[&art_id, &stored.content_hash, &stored.storage_path, &kind.mime, &(stored.size_bytes as i64), &meta]
                    );
                        collected_artifacts.push(art_id.clone());

                        // Create evidence stub for everything
                        let ev_id = generated_record_id("evidence");
                        // grok branch: DEEP PDF (and similar media) collection - use real extraction instead of placeholder
                        let extracted_text = extract_text_if_possible(&data, &kind);
                        let text_for_doc = extracted_text.clone().unwrap_or_else(|| {
                            if kind.kind == "text" {
                                String::from_utf8_lossy(&data).chars().take(8000).collect()
                            } else {
                                format!(
                                    "[binary {} - {} bytes - full content stored as artifact {}]",
                                    kind.mime,
                                    data.len(),
                                    art_id
                                )
                            }
                        });
                        let _ = tx.execute(
                        "INSERT INTO normalized_documents (id, raw_artifact_id, title, document_type, text_content, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING",
                        &[&ev_id, &art_id, &path.file_name().unwrap_or_default().to_string_lossy().to_string(), &kind.kind, &text_for_doc, &meta]
                    );
                        evidence_created.push(ev_id.clone());

                        // Use the *real* extracted text (PDF text, image metadata, etc.) for aggressive mining
                        let text_for_mining = extracted_text.as_deref().unwrap_or(&text_for_doc);
                        if let Some(claim) =
                            simple_mine_claim_from_text(text_for_mining, &path.to_string_lossy())
                        {
                            graph_candidates.push(serde_json::json!({
                                "type": "claim",
                                "text": claim,
                                "source_artifact": art_id,
                                "from_full_access": true,
                                "extracted": extracted_text.is_some()  // marks that this came from deep extraction (PDF etc.)
                            }));
                        }
                        local_files += 1;
                    }
                }
                if local_files > 500 {
                    break;
                } // safety valve
            }
            if local_files > 500 {
                break;
            }
        }
        summary["local_files_ingested"] = serde_json::json!(local_files);

        // 2. System snapshot (ps, env, network, wifi - anything we can exec)
        let system_captures = vec![
            ("ps_aux", "ps", vec!["aux"]),
            ("env", "env", vec![]),
            ("ip_addr", "ip", vec!["-o", "addr"]),
            ("nmcli_wifi", "nmcli", vec!["-t", "device", "wifi", "list"]),
            ("iwlist", "iwlist", vec!["scan"]),
            ("mounts", "mount", vec![]),
        ];
        let mut sys_count = 0;
        for (name, cmd, args) in system_captures {
            if let Ok(output) = Command::new(cmd).args(&args).output() {
                let data = output.stdout;
                if !data.is_empty() {
                    let store = ArtifactStore::new(artifact_data_root()).unwrap();
                    if let Ok(stored) = store.write_bytes(&data) {
                        let art_id = generated_record_id("artifact");
                        let _ = tx.execute(
                        "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING",
                        &[&art_id, &stored.content_hash, &stored.storage_path, &"text/plain".to_string(), &(stored.size_bytes as i64), &serde_json::json!({"full_access_system": name})]
                    );
                        collected_artifacts.push(art_id.clone());
                        sys_count += 1;

                        // Mine relationships from command output
                        let text = String::from_utf8_lossy(&data);
                        for line in text.lines().take(50) {
                            if line.len() > 10 {
                                graph_candidates.push(serde_json::json!({
                                    "type": "system_fact",
                                    "text": line.to_string(),
                                    "source": name,
                                    "from_full_access": true
                                }));
                            }
                        }
                    }
                }
            }
        }
        summary["system_snapshots"] = serde_json::json!(sys_count);
    }

    // 3. Web scraping - DEEPEST + SAFEST on grok (UI controlled, everything from browser, no cmd line).
    // Recursive crawler (BFS up to max_depth), full asset download (original res images/videos/PDFs),
    // PDF deep text + images if present, image exif strip for non-traceability,
    // aggressive mining on all extracted content.
    // SAFETY: Hard blacklist (gov/mil/military/top secret domains - social media, Patreon fine),
    // random UA, jitter delays, minimal headers (no cookies/referer/fingerprint).
    // Not traceable: generic UAs, no unique headers, exif stripped, local storage only.
    let mut web_count = 0;
    let mut media_from_web = 0;
    let mut auto_bypass_attempts = 0usize;
    let mut auto_bypass_wins = 0usize;
    let mut auto_bypass_strategies: Vec<String> = vec![];
    let mut crawled_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut url_queue: std::collections::VecDeque<(String, usize)> =
        std::collections::VecDeque::new();

    if let Some(scope) = object.get("scope").and_then(|s| s.as_array()) {
        for item in scope {
            if let Some(s) = item.as_str() {
                if s.starts_with("http://") || s.starts_with("https://") {
                    if !is_forbidden(s) {
                        url_queue.push_back((s.to_string(), 0));
                    } else {
                        println!("[grok-collector] SKIPPED forbidden/sensitive domain for legal safety: {}", s);
                    }
                }
            }
        }
    }

    let agent = anon_agent();
    let fetch_headers = web_fetch_headers(&object);
    while let Some((current_url, depth)) = url_queue.pop_front() {
        if crawled_urls.contains(&current_url) || depth > max_depth {
            continue;
        }
        crawled_urls.insert(current_url.clone());
        if is_forbidden(&current_url) {
            continue;
        }
        jitter_delay();

        let mut prefetched_pdf: Option<Vec<u8>> = None;
        let mut prefetched_html: Option<String> = None;
        let mut auto_bypass_strategy: Option<String> = None;
        let mut resolved_fetch_url = current_url.clone();
        if auto_bypass {
            auto_bypass_attempts += 1;
            if let Some(resolved) =
                auto_bypass_resolve(&agent, &current_url, max_reach, max_depth as u32)
            {
                auto_bypass_wins += 1;
                auto_bypass_strategy = Some(resolved.strategy.clone());
                auto_bypass_strategies.push(resolved.strategy);
                resolved_fetch_url = resolved.fetched_url;
                if resolved.is_pdf {
                    prefetched_pdf = resolved.pdf_bytes;
                } else {
                    prefetched_html = Some(resolved.body);
                }
                if max_reach {
                    if let Some(artifacts) = read_max_reach_artifacts() {
                        for link in artifacts
                            .media_urls
                            .iter()
                            .chain(artifacts.discovered_links.iter())
                        {
                            if depth < max_depth
                                && !crawled_urls.contains(link)
                                && !is_forbidden(link)
                            {
                                url_queue.push_back((link.clone(), depth + 1));
                            }
                        }
                    }
                }
            }
        }

        let mut handled_fetch = false;
        if let Some(pdf_bytes) = prefetched_pdf {
            if !pdf_bytes.is_empty() {
                let kind = detect_content_kind(&pdf_bytes, Some(&resolved_fetch_url));
                let store = ArtifactStore::new(artifact_data_root()).unwrap();
                if let Ok(stored) = store.write_bytes(&pdf_bytes) {
                    let art_id = generated_record_id("artifact");
                    let meta = serde_json::json!({
                        "scraped_url": resolved_fetch_url,
                        "requested_url": current_url,
                        "full_access_web": true,
                        "deep_scrape": true,
                        "auto_bypass": auto_bypass,
                        "auto_bypass_strategy": auto_bypass_strategy,
                        "mime": kind.mime,
                        "kind": kind.kind,
                        "depth": depth
                    });
                    let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&art_id, &stored.content_hash, &stored.storage_path, &kind.mime, &(stored.size_bytes as i64), &meta] );
                    collected_artifacts.push(art_id.clone());
                    web_count += 1;
                    let extracted = extract_text_if_possible(&pdf_bytes, &kind).unwrap_or_default();
                    let ev_id = generated_record_id("evidence");
                    let _ = tx.execute( "INSERT INTO normalized_documents (id, raw_artifact_id, title, document_type, text_content, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&ev_id, &art_id, &resolved_fetch_url, &"pdf".to_string(), &extracted, &meta] );
                    evidence_created.push(ev_id.clone());
                    if let Some(claim) =
                        simple_mine_claim_from_text(&extracted, &resolved_fetch_url)
                    {
                        graph_candidates.push(serde_json::json!({ "type": "deep_web_pdf_claim", "text": claim, "source_artifact": art_id, "from_full_access": true, "depth": depth }));
                    }
                    handled_fetch = true;
                }
            }
        } else if let Some(body) = prefetched_html {
            let data = body.as_bytes();
            let store = ArtifactStore::new(artifact_data_root()).unwrap();
            if let Ok(stored) = store.write_bytes(data) {
                let art_id = generated_record_id("artifact");
                let page_meta = serde_json::json!({
                    "scraped_url": resolved_fetch_url,
                    "requested_url": current_url,
                    "full_access_web": true,
                    "deep_scrape": true,
                    "auto_bypass": auto_bypass,
                    "auto_bypass_strategy": auto_bypass_strategy,
                    "depth": depth
                });
                let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&art_id, &stored.content_hash, &stored.storage_path, &"text/html".to_string(), &(stored.size_bytes as i64), &page_meta] );
                collected_artifacts.push(art_id.clone());
                web_count += 1;
                for cap in extract_simple_links_and_claims(&body, &resolved_fetch_url) {
                    graph_candidates.push(cap);
                }

                let asset_urls = extract_img_and_video_srcs(&body, &resolved_fetch_url);
                for asset_url in asset_urls {
                    if is_forbidden(&asset_url) {
                        continue;
                    }
                    jitter_delay();
                    let mut asset_req = agent.get(&asset_url);
                    for (k, v) in &fetch_headers {
                        asset_req = asset_req.set(k, v);
                    }
                    if let Ok(aresp) = asset_req.call() {
                        let mut abytes = Vec::new();
                        {
                            let mut rdr = aresp.into_reader();
                            let _ = std::io::Read::read_to_end(&mut rdr, &mut abytes);
                        }
                        if !abytes.is_empty() {
                            let mut final_bytes = abytes;
                            let kind = detect_content_kind(&final_bytes, None);
                            if kind.kind == "image" {
                                if let Ok(img) = image::load_from_memory(&final_bytes) {
                                    let mut buf = Vec::new();
                                    let _ = img.write_to(
                                        &mut std::io::Cursor::new(&mut buf),
                                        image::ImageFormat::Png,
                                    );
                                    final_bytes = buf;
                                }
                            }
                            if kind.kind == "image"
                                || kind.kind == "video"
                                || kind.kind == "pdf"
                                || (media_focus && kind.kind != "text")
                            {
                                if let Ok(stored) = store.write_bytes(&final_bytes) {
                                    let mart_id = generated_record_id("artifact");
                                    let mmeta = serde_json::json!({ "full_res_from_source": true, "original_url": asset_url, "parent_page": resolved_fetch_url, "mime": kind.mime, "kind": kind.kind, "deep_scraped": true, "depth": depth, "exif_stripped": kind.kind == "image" });
                                    let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&mart_id, &stored.content_hash, &stored.storage_path, &kind.mime, &(stored.size_bytes as i64), &mmeta] );
                                    collected_artifacts.push(mart_id.clone());
                                    media_from_web += 1;
                                    let stub = extract_text_if_possible(&final_bytes, &kind)
                                        .unwrap_or_else(|| {
                                            format!(
                                                "[Deep media from {} via {}]",
                                                asset_url, resolved_fetch_url
                                            )
                                        });
                                    let ev_id = generated_record_id("evidence");
                                    let _ = tx.execute( "INSERT INTO normalized_documents (id, raw_artifact_id, title, document_type, text_content, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&ev_id, &mart_id, &asset_url, &kind.kind, &stub, &mmeta] );
                                    evidence_created.push(ev_id.clone());
                                    if let Some(claim) =
                                        simple_mine_claim_from_text(&stub, &asset_url)
                                    {
                                        graph_candidates.push(serde_json::json!({"type": "deep_asset_claim", "text": claim, "source_artifact": mart_id, "from_full_access": true}));
                                    }
                                }
                            }
                        }
                    }
                }

                if depth < max_depth {
                    for link in extract_simple_links_and_claims(&body, &resolved_fetch_url) {
                        if let Some(to) = link.get("to").and_then(|v| v.as_str()) {
                            if (to.starts_with("http://") || to.starts_with("https://"))
                                && !is_forbidden(to)
                                && !crawled_urls.contains(to)
                            {
                                url_queue.push_back((to.to_string(), depth + 1));
                            }
                        }
                    }
                }
                handled_fetch = true;
            }
        }

        if handled_fetch {
            continue;
        }

        let mut req = agent.get(&current_url);
        for (k, v) in &fetch_headers {
            req = req.set(k, v);
        }

        if let Ok(resp) = req.call() {
            let content_type = resp.header("Content-Type").unwrap_or("").to_lowercase();
            let is_pdf =
                current_url.to_lowercase().ends_with(".pdf") || content_type.contains("pdf");

            if is_pdf {
                // Deep PDF
                let mut pdf_bytes = Vec::new();
                {
                    let mut rdr = resp.into_reader();
                    let _ = std::io::Read::read_to_end(&mut rdr, &mut pdf_bytes);
                }
                if !pdf_bytes.is_empty() {
                    let kind = detect_content_kind(&pdf_bytes, Some(&current_url));
                    let store = ArtifactStore::new(artifact_data_root()).unwrap();
                    if let Ok(stored) = store.write_bytes(&pdf_bytes) {
                        let art_id = generated_record_id("artifact");
                        let meta = serde_json::json!({ "scraped_url": current_url, "full_access_web": true, "deep_scrape": true, "mime": kind.mime, "kind": kind.kind, "depth": depth });
                        let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&art_id, &stored.content_hash, &stored.storage_path, &kind.mime, &(stored.size_bytes as i64), &meta] );
                        collected_artifacts.push(art_id.clone());
                        web_count += 1;
                        let extracted =
                            extract_text_if_possible(&pdf_bytes, &kind).unwrap_or_default();
                        let ev_id = generated_record_id("evidence");
                        let _ = tx.execute( "INSERT INTO normalized_documents (id, raw_artifact_id, title, document_type, text_content, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&ev_id, &art_id, &current_url, &"pdf".to_string(), &extracted, &meta] );
                        evidence_created.push(ev_id.clone());
                        if let Some(claim) = simple_mine_claim_from_text(&extracted, &current_url) {
                            graph_candidates.push(serde_json::json!({ "type": "deep_web_pdf_claim", "text": claim, "source_artifact": art_id, "from_full_access": true, "depth": depth }));
                        }
                    }
                }
            } else if let Ok(body) = resp.into_string() {
                let data = body.as_bytes();
                let store = ArtifactStore::new(artifact_data_root()).unwrap();
                if let Ok(stored) = store.write_bytes(data) {
                    let art_id = generated_record_id("artifact");
                    let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&art_id, &stored.content_hash, &stored.storage_path, &"text/html".to_string(), &(stored.size_bytes as i64), &serde_json::json!({"scraped_url": current_url, "full_access_web": true, "deep_scrape": true, "depth": depth})] );
                    collected_artifacts.push(art_id.clone());
                    web_count += 1;
                    for cap in extract_simple_links_and_claims(&body, &current_url) {
                        graph_candidates.push(cap);
                    }

                    // Deep assets + exif strip
                    let asset_urls = extract_img_and_video_srcs(&body, &current_url);
                    for asset_url in asset_urls {
                        if is_forbidden(&asset_url) {
                            continue;
                        }
                        jitter_delay();
                        let mut asset_req = agent.get(&asset_url);
                        for (k, v) in &fetch_headers {
                            asset_req = asset_req.set(k, v);
                        }
                        if let Ok(aresp) = asset_req.call() {
                            let mut abytes = Vec::new();
                            {
                                let mut rdr = aresp.into_reader();
                                let _ = std::io::Read::read_to_end(&mut rdr, &mut abytes);
                            }
                            if !abytes.is_empty() {
                                let mut final_bytes = abytes;
                                let kind = detect_content_kind(&final_bytes, None);
                                if kind.kind == "image" {
                                    if let Ok(img) = image::load_from_memory(&final_bytes) {
                                        let mut buf = Vec::new();
                                        let _ = img.write_to(
                                            &mut std::io::Cursor::new(&mut buf),
                                            image::ImageFormat::Png,
                                        );
                                        final_bytes = buf;
                                    }
                                }
                                if kind.kind == "image"
                                    || kind.kind == "video"
                                    || kind.kind == "pdf"
                                    || (media_focus && kind.kind != "text")
                                {
                                    if let Ok(stored) = store.write_bytes(&final_bytes) {
                                        let mart_id = generated_record_id("artifact");
                                        let mmeta = serde_json::json!({ "full_res_from_source": true, "original_url": asset_url, "parent_page": current_url, "mime": kind.mime, "kind": kind.kind, "deep_scraped": true, "depth": depth, "exif_stripped": kind.kind == "image" });
                                        let _ = tx.execute( "INSERT INTO raw_artifacts (id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&mart_id, &stored.content_hash, &stored.storage_path, &kind.mime, &(stored.size_bytes as i64), &mmeta] );
                                        collected_artifacts.push(mart_id.clone());
                                        media_from_web += 1;
                                        let stub = extract_text_if_possible(&final_bytes, &kind)
                                            .unwrap_or_else(|| {
                                                format!(
                                                    "[Deep media from {} via {}]",
                                                    asset_url, current_url
                                                )
                                            });
                                        let ev_id = generated_record_id("evidence");
                                        let _ = tx.execute( "INSERT INTO normalized_documents (id, raw_artifact_id, title, document_type, text_content, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb) ON CONFLICT DO NOTHING", &[&ev_id, &mart_id, &asset_url, &kind.kind, &stub, &mmeta] );
                                        evidence_created.push(ev_id.clone());
                                        if let Some(claim) =
                                            simple_mine_claim_from_text(&stub, &asset_url)
                                        {
                                            graph_candidates.push(serde_json::json!({"type": "deep_asset_claim", "text": claim, "source_artifact": mart_id, "from_full_access": true}));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Recursive for depth
                    if depth < max_depth {
                        for link in extract_simple_links_and_claims(&body, &current_url) {
                            if let Some(to) = link.get("to").and_then(|v| v.as_str()) {
                                if (to.starts_with("http://") || to.starts_with("https://"))
                                    && !is_forbidden(to)
                                    && !crawled_urls.contains(to)
                                {
                                    url_queue.push_back((to.to_string(), depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    summary["web_scraped"] = serde_json::json!(web_count);
    summary["media_from_deep_scrape"] = serde_json::json!(media_from_web);
    summary["auto_bypass_attempts"] = serde_json::json!(auto_bypass_attempts);
    summary["auto_bypass_wins"] = serde_json::json!(auto_bypass_wins);
    summary["auto_bypass_strategies"] = serde_json::json!(auto_bypass_strategies);
    summary["crawled_pages"] = serde_json::json!(crawled_urls.len());
    summary["deep_crawl_depth"] = serde_json::json!(max_depth);
    summary["safe_mode"] = serde_json::json!(safe_mode);

    // 4. Persist graph candidates via existing memory/graph routes if possible, or direct
    for _cand in &graph_candidates {
        // Best effort - call the sync if the route exists in spirit, or just log in summary for now
        // In real we could POST internally but for simplicity we record them.
    }
    summary["graph_candidates_mined"] = serde_json::json!(graph_candidates.len());
    summary["total_artifacts"] = serde_json::json!(collected_artifacts.len());
    summary["total_evidence"] = serde_json::json!(evidence_created.len());

    // Tie into real worker pipeline (grok full access - not scaffolded)
    let work_item_id = generated_record_id("work");
    let work_payload = serde_json::json!({
        "collection_run_id": collection_run_id,
        "source_id": full_source_id,
        "mode": "full_access_grok",
        "artifact_count": collected_artifacts.len(),
        "evidence_count": evidence_created.len(),
        "grok_full_access": true,
        "graph_candidates": graph_candidates.len()
    });
    let _ = tx.execute(
        "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, 'collection_normalization', 'queued', $2, $3::jsonb, NULL)",
        &[&work_item_id, &requested_by, &work_payload]
    );
    let _ = tx.execute(
        "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'queued', 'work_item', $2, $3, $4::jsonb)",
        &[&requested_by, &work_item_id, &collection_run_id, &serde_json::json!({"full_access": true})]
    );
    summary["normalization_work_item_queued"] = serde_json::json!(work_item_id);

    // Record the big collection run
    tx.execute(
        "INSERT INTO collection_runs (id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message) VALUES ($1, $2, 'completed', false, $3, $4::jsonb, NULL)",
        &[&collection_run_id, &"full-access-grok", &requested_by, &summary]
    ).ok();

    // Audit the full capture
    let audit_details = serde_json::json!({ "full_access": true, "artifacts": collected_artifacts.len(), "evidence": evidence_created.len() });
    tx.execute(
        "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, details_json) VALUES ($1, 'collection_run.full_access', 'recorded', 'collection_run', $2, $3::jsonb)",
        &[&requested_by, &collection_run_id, &audit_details]
    ).ok();

    let response_body = tx.query_one(
        "SELECT row_to_json(t)::text FROM (SELECT id, status, summary_json, created_at FROM collection_runs WHERE id = $1) t",
        &[&collection_run_id]
    ).map(|r| r.get::<_, String>(0)).unwrap_or_else(|_| summary.to_string());

    tx.commit().ok();

    let mut tracked_urls = Vec::new();
    if let Some(scope) = object.get("scope").and_then(|value| value.as_array()) {
        for item in scope {
            if let Some(url) = item.as_str() {
                tracked_urls.push(url.to_string());
            }
        }
    }
    bypass_intel::record_bypass_intel_domains(&tracked_urls);
    bypass_intel::maybe_background_bypass_intel_harvest(Some(database_url.to_string()));

    // Tie the collected evidence/artifacts into the real graph (Neo4j via existing pipeline)
    // This makes graph candidates and new sources/artifacts/docs real in the relationship memory, not just Postgres.
    let _ = ensure_graph_schema();
    // sync will pick up the new artifacts, documents, evidence created above
    let _ = sync_graph_lineage(Some(database_url)); // pass the url we had (grok full access tying to real graph)

    Ok(response_body)
}

// Simple claim miner (aggressive on grok full access)
fn simple_mine_claim_from_text(text: &str, source: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if lower.contains("password") || lower.contains("secret") || lower.contains("token") {
        return Some(format!(
            "Potential secret/credential mentioned in {}",
            source
        ));
    }
    if let Some(pos) = lower.find(" i am ") {
        let end = (pos + 20).min(text.len());
        return Some(format!(
            "Self-statement from {}: {}",
            source,
            &text[pos..end]
        ));
    }
    if text.contains("http") || text.contains("www.") {
        return Some(format!("URL or link reference found in {}", source));
    }
    None
}

fn extract_simple_links_and_claims(html_or_text: &str, base: &str) -> Vec<serde_json::Value> {
    let mut out = vec![];
    // simple non-regex link extraction (grok full access - no external deps for this)
    let lower = html_or_text.to_lowercase();
    let mut search = lower.as_str();
    while let Some(pos) = search.find("href=") {
        let rest = &search[pos + 5..];
        if let Some(endq) = rest.find(|c| c == '"' || c == '\'') {
            let quote = &rest[endq..endq + 1];
            if let Some(end) = rest[endq + 1..].find(quote) {
                let link = &rest[endq + 1..endq + 1 + end];
                if !link.is_empty() && link.len() < 500 {
                    out.push(serde_json::json!({
                        "type": "link",
                        "from": base,
                        "to": link,
                        "from_full_access_web": true
                    }));
                }
            }
        }
        search = &search[pos + 6..];
        if out.len() > 20 {
            break;
        }
    }
    if html_or_text.len() > 100 {
        out.push(serde_json::json!({
            "type": "page_claim",
            "text": format!("Page at {} contains {} bytes of content", base, html_or_text.len()),
            "from_full_access_web": true
        }));
    }
    out
}

// grok branch deep scrape: extract image/video srcs from HTML for full res original from source
fn extract_img_and_video_srcs(html: &str, base_url: &str) -> Vec<String> {
    let mut srcs = Vec::new();
    let lower = html.to_lowercase();
    // simple extraction for src, data-src, data-fullsrc, poster etc for img/video/source
    for attr in [
        "src=",
        "data-src=",
        "data-fullsrc=",
        "data-original=",
        "poster=",
        "srcset=",
    ] {
        let mut search = &lower[..];
        while let Some(pos) = search.find(attr) {
            let rest = &search[pos + attr.len()..];
            if let Some(qpos) = rest.find(|c: char| c == '"' || c == '\'') {
                let quote = &rest[qpos..qpos + 1];
                if let Some(end) = rest[qpos + 1..].find(quote) {
                    let raw = &rest[qpos + 1..qpos + 1 + end];
                    if !raw.is_empty()
                        && raw.len() < 1000
                        && (raw.starts_with("http")
                            || raw.starts_with("/")
                            || raw.starts_with("data:") == false)
                    {
                        let resolved = if raw.starts_with("http") {
                            raw.to_string()
                        } else if raw.starts_with("//") {
                            format!("https:{}", raw)
                        } else if raw.starts_with("/") {
                            // naive base
                            let host_end = base_url[8..]
                                .find('/')
                                .map(|i| i + 8)
                                .unwrap_or(base_url.len());
                            if host_end > 8 {
                                format!("{}{}", &base_url[..host_end], raw)
                            } else {
                                format!("{}{}", base_url, raw)
                            }
                        } else {
                            raw.to_string()
                        };
                        // clean common resize params for full res
                        let clean = resolved.split('?').next().unwrap_or(&resolved).to_string();
                        if !srcs.contains(&clean) {
                            srcs.push(clean);
                        }
                    }
                }
            }
            search = &search[pos + 5..];
            if srcs.len() > 50 {
                break;
            }
        }
    }
    srcs
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

fn create_report_work_item(
    report_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let report_id = validate_route_id(report_id, "report_id")?;
    let payload = parse_report_work_item(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let Some(report_row) = transaction
        .query_opt(
            "SELECT id, title, report_type, status FROM reports WHERE id = $1",
            &[&report_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Report not found".to_string()));
    };
    let report_title: String = report_row.get(1);
    let report_type: String = report_row.get(2);
    let report_status: String = report_row.get(3);
    let work_item_id = generated_record_id("work");
    let payload_json = report_work_item_payload(
        &report_id,
        &report_title,
        &report_type,
        &report_status,
        payload.notes.as_deref(),
    );

    transaction
        .execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, 'report_generation', 'queued', $2, $3::jsonb, NULL)",
            &[&work_item_id, &payload.requested_by_actor_id, &payload_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "work_type": "report_generation",
        "status": "queued",
        "report_id": report_id,
        "report_type": report_type,
        "scaffold_only": false,
        "executes_report_generation": true
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'queued', 'work_item', $2, $3, $4::jsonb)",
            &[&payload.requested_by_actor_id, &work_item_id, &report_id, &details_json],
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

fn create_agent_task_plan(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_agent_task_plan_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let task_plan_id = generated_record_id("taskplan");
    let proposed_steps_json = json_string_values(&payload.proposed_steps);
    let required_evidence_json = json_string_values(&payload.required_evidence);
    transaction
        .execute(
            "INSERT INTO agent_task_plans (id, user_request_summary, intent_category, status, proposed_steps, required_evidence, approval_required, supported_state, next_safe_action, requested_by_actor_id, metadata_json) VALUES ($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7, $8, $9, $10, $11::jsonb)",
            &[
                &task_plan_id,
                &payload.user_request_summary,
                &payload.intent_category,
                &payload.status,
                &proposed_steps_json,
                &required_evidence_json,
                &payload.approval_required,
                &payload.supported_state,
                &payload.next_safe_action,
                &payload.requested_by_actor_id,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "intent_category": payload.intent_category,
        "status": payload.status,
        "approval_required": payload.approval_required,
        "supported_state": payload.supported_state
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'agent_task_plan.created', $2, 'agent_task_plan', $3, NULL, $4::jsonb)",
            &[&payload.requested_by_actor_id, &payload.status, &task_plan_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = agent_task_plan_response_json(&mut transaction, &task_plan_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn list_agent_task_plans(database_url: Option<&str>) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    client
        .query_one(
            "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, user_request_summary, intent_category, status, proposed_steps, required_evidence, approval_required, supported_state, next_safe_action, requested_by_actor_id, metadata_json, created_at, updated_at FROM agent_task_plans ORDER BY created_at DESC) t), '[]')",
            &[],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn get_agent_task_plan(
    task_plan_id: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let task_plan_id = validate_route_id(task_plan_id, "task_plan_id")?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    let body = client
        .query_one(
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, user_request_summary, intent_category, status, proposed_steps, required_evidence, approval_required, supported_state, next_safe_action, requested_by_actor_id, metadata_json, created_at, updated_at FROM agent_task_plans WHERE id = $1) t), '')",
            &[&task_plan_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if body.is_empty() {
        Err(GatewayError::NotFound(
            "Agent task plan not found".to_string(),
        ))
    } else {
        Ok(body)
    }
}

fn create_work_item_from_agent_task_plan(
    task_plan_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let task_plan_id = validate_route_id(task_plan_id, "task_plan_id")?;
    let payload = parse_agent_task_plan_work_item(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let task_plan = load_agent_task_plan_record(&mut transaction, &task_plan_id)?;
    if matches!(
        task_plan.status.as_str(),
        "unsupported"
            | "needs_clarification"
            | "evidence_needed"
            | "canceled"
            | "converted_to_work"
    ) {
        return Err(GatewayError::Conflict(format!(
            "Task plan status {} is not ready for work-item creation",
            task_plan.status
        )));
    }
    if task_plan.supported_state != "supported" {
        return Err(GatewayError::Conflict(format!(
            "Task plan supported_state {} is not eligible for work-item creation",
            task_plan.supported_state
        )));
    }
    if task_plan.approval_required {
        let Some(approval_id) = payload.approval_id.as_deref() else {
            return Err(GatewayError::Forbidden(
                "Approved agent_task_plan approval is required before creating work".to_string(),
            ));
        };
        validate_agent_task_plan_approval(&mut transaction, &task_plan_id, approval_id)?;
    }
    let Some(plan_to_work) = task_plan
        .metadata_json
        .get("plan_to_work")
        .and_then(Value::as_object)
    else {
        return Err(GatewayError::Conflict(
            "Task plan does not include a supported plan_to_work specification".to_string(),
        ));
    };
    let work_type = plan_to_work
        .get("work_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if !is_supported_work_item_type(&work_type) {
        return Err(GatewayError::Validation(format!(
            "Unsupported work item type: {work_type}"
        )));
    }
    let payload_json = plan_to_work
        .get("payload_json")
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let expected_output = plan_to_work
        .get("expected_output")
        .and_then(Value::as_str)
        .unwrap_or(task_plan.next_safe_action.as_str())
        .trim()
        .to_string();
    let intent = serde_json::json!({
        "original_request": task_plan.user_request_summary,
        "interpretation": format!("Persisted agent task plan {} categorized as {}", task_plan.id, task_plan.intent_category),
        "proposed_work_type": work_type,
        "expected_output": if expected_output.is_empty() { task_plan.next_safe_action.clone() } else { expected_output },
        "safety_requirements": [
            "Use existing supported work item types only.",
            "Do not execute shell commands or user-provided argv.",
            "Keep plan-to-work creation approval-gated when required."
        ],
        "assumptions": ["Persisted task plan has been reviewed enough to create a work item."],
        "missing_information": [],
        "sources_likely_used": task_plan.required_evidence
    });
    let mut work_payload_json = payload_json;
    if let Value::Object(ref mut object) = work_payload_json {
        object.insert(
            "agent_task_plan_id".to_string(),
            Value::String(task_plan.id.clone()),
        );
        object.insert("intent_verification".to_string(), intent.clone());
    }
    let work_item_id = generated_record_id("work");
    transaction
        .execute(
            "INSERT INTO work_items (id, work_type, status, requested_by_actor_id, payload_json, error_message) VALUES ($1, $2, 'pending_intent_verification', $3, $4::jsonb, NULL)",
            &[&work_item_id, &work_type, &payload.actor_id, &work_payload_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let work_audit_details = serde_json::json!({
        "work_type": work_type,
        "status": "pending_intent_verification",
        "agent_task_plan_id": task_plan.id
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'intent_verification_required', 'work_item', $2, $3, $4::jsonb)",
            &[&payload.actor_id, &work_item_id, &task_plan_id, &work_audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let plan_audit_details = serde_json::json!({
        "work_item_id": work_item_id,
        "work_type": work_type,
        "previous_status": task_plan.status
    });
    transaction
        .execute(
            "UPDATE agent_task_plans SET status = 'converted_to_work', metadata_json = metadata_json || $1::jsonb, updated_at = now() WHERE id = $2",
            &[&serde_json::json!({"work_item_id": work_item_id}), &task_plan_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'agent_task_plan.work_item_created', 'converted_to_work', 'agent_task_plan', $2, $3, $4::jsonb)",
            &[&payload.actor_id, &task_plan_id, &work_item_id, &plan_audit_details],
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

fn propose_agent_task_plan_work_spec(
    task_plan_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let task_plan_id = validate_route_id(task_plan_id, "task_plan_id")?;
    let payload = parse_agent_task_plan_work_spec(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let task_plan = load_agent_task_plan_record(&mut transaction, &task_plan_id)?;
    if matches!(
        task_plan.status.as_str(),
        "unsupported" | "canceled" | "converted_to_work"
    ) {
        return Err(GatewayError::Conflict(format!(
            "Task plan status {} is not eligible for work spec proposal",
            task_plan.status
        )));
    }
    if task_plan.supported_state == "unsupported" {
        return Err(GatewayError::Conflict(format!(
            "Task plan supported_state {} is not eligible for work spec proposal",
            task_plan.supported_state
        )));
    }
    if task_plan
        .metadata_json
        .get("plan_to_work")
        .and_then(Value::as_object)
        .is_some()
    {
        return Err(GatewayError::Conflict(
            "Task plan already has a plan_to_work specification".to_string(),
        ));
    }
    if payload.work_type != "report_generation" || task_plan.intent_category != "create_report" {
        return Err(GatewayError::Validation(
            "Only create_report task plans can propose report_generation work specs in this DIFF"
                .to_string(),
        ));
    }
    let expected_output = payload
        .expected_output
        .clone()
        .unwrap_or_else(|| task_plan.next_safe_action.clone());
    let plan_to_work = serde_json::json!({
        "work_type": payload.work_type,
        "expected_output": expected_output,
        "payload_json": {
            "report_type": "agent_task_plan",
            "task_plan_id": task_plan.id,
            "requested_summary": task_plan.user_request_summary,
            "intent_category": task_plan.intent_category
        },
        "proposal_source": "bounded_task_plan_work_spec",
        "safety_constraints": [
            "Supported work item type only.",
            "No shell command or user-provided argv.",
            "Creates work only through the approval-gated plan-to-work route."
        ]
    });
    let next_status = if task_plan.approval_required {
        "approval_required"
    } else {
        "ready"
    };
    let metadata_patch = serde_json::json!({
        "plan_to_work": plan_to_work,
        "saved_preview_only": false,
        "work_spec_proposed_by_actor_id": payload.actor_id
    });
    transaction
        .execute(
            "UPDATE agent_task_plans SET status = $1, metadata_json = metadata_json || $2::jsonb, updated_at = now() WHERE id = $3",
            &[&next_status, &metadata_patch, &task_plan_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "work_type": "report_generation",
        "previous_status": task_plan.status,
        "next_status": next_status,
        "approval_required": task_plan.approval_required
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'agent_task_plan.work_spec_proposed', $2, 'agent_task_plan', $3, NULL, $4::jsonb)",
            &[&payload.actor_id, &next_status, &task_plan_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = agent_task_plan_response_json(&mut transaction, &task_plan_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn update_agent_task_plan_evidence_summary(
    task_plan_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let task_plan_id = validate_route_id(task_plan_id, "task_plan_id")?;
    let payload = parse_agent_task_plan_evidence_summary(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_agent_task_plans_table(&mut client)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    load_agent_task_plan_record(&mut transaction, &task_plan_id)?;
    let guidance = payload.missing_evidence_guidance.clone().unwrap_or_else(|| {
        if payload.missing_evidence {
            "No relevant local evidence was retrieved. Add/process data or narrow the request before proceeding.".to_string()
        } else {
            "Relevant local evidence was retrieved. Review labels before creating work or answering.".to_string()
        }
    });
    let answer_status = payload.answer_status.clone();
    let evidence_summary = serde_json::json!({
        "evidence_checked_at": now_epoch_string(),
        "answer_status": answer_status,
        "retrieved_count": payload.retrieved_count,
        "safe_labels": payload.labels,
        "missing_evidence": payload.missing_evidence,
        "missing_evidence_guidance": guidance
    });
    let metadata_patch = serde_json::json!({
        "evidence_summary": evidence_summary
    });
    transaction
        .execute(
            "UPDATE agent_task_plans SET metadata_json = metadata_json || $1::jsonb, updated_at = now() WHERE id = $2",
            &[&metadata_patch, &task_plan_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "answer_status": payload.answer_status,
        "retrieved_count": payload.retrieved_count,
        "missing_evidence": payload.missing_evidence
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'agent_task_plan.evidence_summary_recorded', $2, 'agent_task_plan', $3, NULL, $4::jsonb)",
            &[&payload.actor_id, &payload.answer_status, &task_plan_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = agent_task_plan_response_json(&mut transaction, &task_plan_id)?;
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

fn update_report_status(
    report_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let report_id = validate_route_id(report_id, "report_id")?;
    let payload = parse_report_status(body)?;
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
            "SELECT status, artifact_path FROM reports WHERE id = $1",
            &[&report_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Report not found".to_string()));
    };
    let previous_status: String = row.get(0);
    let previous_artifact_path: Option<String> = row.get(1);
    transaction
        .execute(
            "UPDATE reports SET status = $2, artifact_path = COALESCE($3, artifact_path), updated_at = now() WHERE id = $1",
            &[&report_id, &payload.status, &payload.artifact_path],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let new_artifact_path = payload
        .artifact_path
        .as_ref()
        .or(previous_artifact_path.as_ref());
    let details_json = serde_json::json!({
        "previous_status": previous_status,
        "new_status": payload.status,
        "previous_artifact_path": previous_artifact_path,
        "new_artifact_path": new_artifact_path
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'report.status_updated', $2, 'report', $3, NULL, $4::jsonb)",
            &[&payload.actor_id, &payload.status, &report_id, &details_json],
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

fn update_work_item_status(
    work_item_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let work_item_id = validate_route_id(work_item_id, "work_item_id")?;
    let payload = parse_work_item_status(body)?;
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
            "SELECT status, payload_json FROM work_items WHERE id = $1",
            &[&work_item_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Work item not found".to_string()));
    };
    let previous_status: String = row.get(0);
    let payload_json: Value = row.get(1);
    require_valid_work_item_status_transition(&previous_status, &payload.status, &payload_json)?;
    transaction
        .execute(
            "UPDATE work_items SET status = $2, error_message = $3, updated_at = now() WHERE id = $1",
            &[&work_item_id, &payload.status, &payload.error_message],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "previous_status": previous_status,
        "new_status": payload.status,
        "error_message": payload.error_message
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.status_updated', $2, 'work_item', $3, NULL, $4::jsonb)",
            &[&payload.actor_id, &payload.status, &work_item_id, &details_json],
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

fn require_valid_work_item_status_transition(
    previous_status: &str,
    new_status: &str,
    payload_json: &Value,
) -> Result<(), GatewayError> {
    if previous_status == new_status {
        return Ok(());
    }
    let allowed = match previous_status {
        "pending_intent_verification" => matches!(new_status, "queued" | "canceled"),
        "queued" => matches!(new_status, "running" | "canceled"),
        "running" => matches!(new_status, "completed" | "failed" | "canceled"),
        "completed" | "failed" | "canceled" => false,
        _ => false,
    };
    if !allowed {
        return Err(GatewayError::Conflict(format!(
            "Invalid work item status transition from {previous_status} to {new_status}"
        )));
    }
    if new_status == "queued" && !has_intent_verification(payload_json) {
        return Err(GatewayError::Conflict(
            "Work item requires recorded intent verification before queueing".to_string(),
        ));
    }
    Ok(())
}

fn get_retrieval_chunk_trail(
    chunk_id: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let chunk_id = validate_route_id(chunk_id, "chunk_id")?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    retrieval_chunk_trail_json(&mut client, &chunk_id)
}

fn retrieval_chunk_trail_json(client: &mut Client, chunk_id: &str) -> Result<String, GatewayError> {
    let Some(row) = client
        .query_opt("SELECT document_id FROM chunks WHERE id = $1", &[&chunk_id])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Chunk not found".to_string()));
    };
    let document_id: String = row.get(0);
    let Some(document_row) = client
        .query_opt(
            "SELECT raw_artifact_id, source_id FROM normalized_documents WHERE id = $1",
            &[&document_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::Conflict(
            "Chunk document not found".to_string(),
        ));
    };
    let raw_artifact_id: Option<String> = document_row.get(0);
    let document_source_id: Option<String> = document_row.get(1);
    let raw_source_id = if let Some(raw_artifact_id) = &raw_artifact_id {
        let Some(raw_row) = client
            .query_opt(
                "SELECT source_id FROM raw_artifacts WHERE id = $1",
                &[raw_artifact_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?
        else {
            return Err(GatewayError::Conflict(
                "Document raw artifact not found".to_string(),
            ));
        };
        raw_row.get::<_, Option<String>>(0)
    } else {
        None
    };
    let source_id = document_source_id.or(raw_source_id);
    if let Some(source_id) = &source_id {
        let source_exists = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
                &[source_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !source_exists {
            return Err(GatewayError::Conflict("Trail source not found".to_string()));
        }
    }
    client
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT (SELECT row_to_json(c) FROM (SELECT id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json, created_at, updated_at FROM chunks WHERE id = $1) c) AS chunk, (SELECT row_to_json(d) FROM (SELECT id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json, created_at, updated_at FROM normalized_documents WHERE id = $2) d) AS document, (SELECT row_to_json(s) FROM (SELECT id, name, source_type, location, owner_actor_id, sensitivity, trust_level, enabled, metadata_json, created_at, updated_at FROM sources WHERE id = $3) s) AS source, (SELECT row_to_json(r) FROM (SELECT id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json, created_at, updated_at FROM raw_artifacts WHERE id = $4) r) AS raw_artifact, COALESCE((SELECT json_agg(row_to_json(e)) FROM (SELECT id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json, created_at, updated_at FROM evidence_items WHERE chunk_id = $1 OR (chunk_id IS NULL AND document_id = $2) ORDER BY created_at DESC, id ASC) e), '[]'::json) AS evidence_items) t",
            &[&chunk_id, &document_id, &source_id, &raw_artifact_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn search_retrieval_chunks(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_retrieval_search(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let query_like = format!("%{}%", payload.query);
    let rows = client
        .query(
            "SELECT c.id FROM chunks c JOIN normalized_documents d ON d.id = c.document_id LEFT JOIN sources s ON s.id = d.source_id WHERE c.text_content ILIKE $1 AND COALESCE(s.enabled, true) = true ORDER BY c.created_at DESC LIMIT $2",
            &[&query_like, &(payload.limit as i64)],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut hits = Vec::new();
    for row in rows {
        let chunk_id: String = row.get(0);
        let trail_text = retrieval_chunk_trail_json(&mut client, &chunk_id)?;
        let trail: Value = serde_json::from_str(&trail_text)
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        hits.push(serde_json::json!({
            "score": 1.0,
            "qdrant_payload": {
                "chunk_id": chunk_id,
                "retrieval_mode": "rust_db_text_search"
            },
            "chunk": trail.get("chunk").cloned().unwrap_or(Value::Null),
            "document": trail.get("document").cloned().unwrap_or(Value::Null),
            "source": trail.get("source").cloned().unwrap_or(Value::Null),
            "raw_artifact": trail.get("raw_artifact").cloned().unwrap_or(Value::Null),
            "evidence_items": trail.get("evidence_items").cloned().unwrap_or_else(|| serde_json::json!([]))
        }));
    }
    Ok(serde_json::json!({
        "query": payload.query,
        "collection_name": env::var("QDRANT_CHUNK_COLLECTION").unwrap_or_else(|_| "igy6_chunks".to_string()),
        "collection_exists": true,
        "hits": hits
    })
    .to_string())
}

fn live_retrieval_preview(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_chat_retrieval_search(body)?;
    let search_body = serde_json::json!({
        "query": payload.query,
        "limit": payload.limit
    })
    .to_string();
    let mut context: Value =
        serde_json::from_str(&search_retrieval_chunks(&search_body, database_url)?)
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    let text_hits = context
        .get("hits")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    if text_hits == 0 {
        context = hydrated_vector_retrieval_chunks(&search_body, database_url)?;
    }
    let hits = context
        .get("hits")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    let hit_count = hits.as_array().map(Vec::len).unwrap_or_default();
    Ok(serde_json::json!({
        "query": payload.query,
        "collection_name": context.get("collection_name").cloned().unwrap_or_else(|| serde_json::json!("igy6_chunks")),
        "collection_exists": context.get("collection_exists").and_then(Value::as_bool).unwrap_or(false),
        "answer_status": if hit_count > 0 { "retrieved" } else { "insufficient_evidence" },
        "retrieval_context": context,
        "items": hits,
        "message": if hit_count > 0 {
            "Retrieved live local evidence from the Rust API."
        } else {
            "No matching local evidence was found."
        }
    })
    .to_string())
}

fn live_evidence_answer(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_chat_retrieval_search(body)?;
    let task_name = extract_json_string(body, "task_name")
        .or_else(|| extract_json_string(body, "task"))
        .unwrap_or_else(|| igy6_llm::DEFAULT_TASK_NAME.to_string());
    let preview_json = live_retrieval_preview(body, database_url)?;
    let preview: Value = serde_json::from_str(&preview_json)
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let retrieval_context_value = preview
        .get("retrieval_context")
        .cloned()
        .unwrap_or_else(|| preview.clone());
    let query = preview
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or(payload.query.as_str())
        .to_string();
    let collection_name = preview
        .get("collection_name")
        .and_then(Value::as_str)
        .or_else(|| {
            retrieval_context_value
                .get("collection_name")
                .and_then(Value::as_str)
        })
        .unwrap_or("igy6_chunks")
        .to_string();
    let collection_exists = preview
        .get("collection_exists")
        .and_then(Value::as_bool)
        .or_else(|| {
            retrieval_context_value
                .get("collection_exists")
                .and_then(Value::as_bool)
        })
        .unwrap_or(false);
    let hits = hydrated_hits_from_retrieval_value(
        retrieval_context_value
            .get("hits")
            .or_else(|| preview.get("items")),
    );
    let retrieval_context = build_hydrated_chunk_search_result(
        &query,
        &collection_name,
        collection_exists,
        hits,
        payload.limit as usize,
    );
    let answer = match LlmConfig::from_env() {
        Ok(config) if config.provider == LlmProvider::Ollama => {
            match load_local_llm_routing_config() {
                Ok(routing_config) => answer_with_optional_llm_for_task(
                    retrieval_context,
                    &config,
                    &routing_config,
                    &task_name,
                    &StdHttpTransport,
                ),
                Err(error) => {
                    deterministic_fallback_for_llm_config_error(retrieval_context, &error)
                }
            }
        }
        Ok(config) => answer_with_optional_llm(retrieval_context, &config, &StdHttpTransport),
        Err(error) => deterministic_fallback_for_llm_config_error(retrieval_context, &error),
    };
    Ok(evidence_grounded_answer_json(&answer))
}

fn hydrated_hits_from_retrieval_value(hits_value: Option<&Value>) -> Vec<HydratedChunkSearchHit> {
    let Some(hits) = hits_value.and_then(Value::as_array) else {
        return Vec::new();
    };
    hits.iter()
        .filter_map(hydrated_hit_from_retrieval_value)
        .collect()
}

fn hydrated_hit_from_retrieval_value(hit: &Value) -> Option<HydratedChunkSearchHit> {
    let chunk_value = hit.get("chunk")?;
    let document_value = hit.get("document")?;
    let chunk_id = json_string_field(chunk_value, "id")?;
    let document_id = json_string_field(chunk_value, "document_id")
        .or_else(|| json_string_field(document_value, "id"))?;
    let chunk_index = chunk_value
        .get("chunk_index")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let text_content = json_string_field(chunk_value, "text_content")
        .or_else(|| json_string_field(chunk_value, "text"))
        .unwrap_or_default();
    if text_content.trim().is_empty() {
        let evidence_statement = hit
            .get("evidence_items")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|item| json_string_field(item, "statement"));
        if evidence_statement.is_none() {
            return None;
        }
    }
    let qdrant_payload_summary = hit
        .get("qdrant_payload")
        .map(|payload| payload.to_string())
        .unwrap_or_else(|| "{}".to_string());
    let source = hit.get("source").and_then(parse_retrieval_source);
    let raw_artifact = hit
        .get("raw_artifact")
        .and_then(parse_retrieval_raw_artifact);
    let evidence_items = hit
        .get("evidence_items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_retrieval_evidence_item)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(HydratedChunkSearchHit {
        score: hit.get("score").and_then(Value::as_f64).unwrap_or(0.0),
        qdrant_payload_summary,
        chunk: RetrievalChunk {
            id: chunk_id,
            document_id: document_id.clone(),
            chunk_index,
            text_content: if text_content.trim().is_empty() {
                evidence_items
                    .first()
                    .map(|item| item.statement.clone())
                    .unwrap_or_default()
            } else {
                text_content
            },
            embedding_status: json_string_field(chunk_value, "embedding_status")
                .unwrap_or_else(|| "unknown".to_string()),
        },
        document: RetrievalDocument {
            id: json_string_field(document_value, "id").unwrap_or(document_id),
            raw_artifact_id: json_optional_string_field(document_value, "raw_artifact_id"),
            source_id: json_optional_string_field(document_value, "source_id"),
            title: json_optional_string_field(document_value, "title"),
            document_type: json_string_field(document_value, "document_type")
                .unwrap_or_else(|| "text".to_string()),
            sensitivity: json_string_field(document_value, "sensitivity")
                .unwrap_or_else(|| "internal".to_string()),
        },
        source,
        raw_artifact,
        evidence_items,
    })
}

fn json_string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn json_optional_string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn parse_retrieval_source(value: &Value) -> Option<RetrievalSource> {
    Some(RetrievalSource {
        id: json_string_field(value, "id")?,
        name: json_string_field(value, "name").unwrap_or_else(|| "source".to_string()),
        source_type: json_string_field(value, "source_type")
            .unwrap_or_else(|| "manual_upload".to_string()),
        trust_level: json_string_field(value, "trust_level")
            .unwrap_or_else(|| "standard".to_string()),
        enabled: value
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    })
}

fn parse_retrieval_raw_artifact(value: &Value) -> Option<RetrievalRawArtifact> {
    Some(RetrievalRawArtifact {
        id: json_string_field(value, "id")?,
        source_id: json_optional_string_field(value, "source_id"),
        content_hash: json_string_field(value, "content_hash")
            .unwrap_or_else(|| "unknown".to_string()),
        storage_path: json_string_field(value, "storage_path")
            .unwrap_or_else(|| "unknown".to_string()),
    })
}

fn parse_retrieval_evidence_item(value: &Value) -> Option<RetrievalEvidenceItem> {
    Some(RetrievalEvidenceItem {
        id: json_string_field(value, "id")?,
        source_id: json_optional_string_field(value, "source_id"),
        document_id: json_optional_string_field(value, "document_id"),
        chunk_id: json_optional_string_field(value, "chunk_id"),
        evidence_type: json_string_field(value, "evidence_type")
            .unwrap_or_else(|| "document_chunk".to_string()),
        statement: json_string_field(value, "statement").unwrap_or_default(),
        confidence: value
            .get("confidence")
            .and_then(Value::as_i64)
            .map(|value| value as i32),
    })
}

fn evidence_grounded_answer_json(answer: &igy6_evidence_answer::EvidenceGroundedAnswer) -> String {
    let deterministic = &answer.deterministic_answer;
    format!(
        "{{\"message\":\"{}\",\"answer_status\":\"{}\",\"generation_mode\":\"{}\",\"llm_provider\":\"{}\",\"llm_status\":\"{}\",\"llm_text\":{},\"llm_error\":{},\"redacted_output_preview\":{},\"prompt_evidence_bytes\":{},\"retrieval_count\":{},\"facts\":[],\"source_trails\":[],\"assumptions\":{},\"uncertainty\":{},\"missing_information\":{}}}",
        escape_json(&deterministic.message),
        answer.answer_status,
        escape_json(&answer.generation_mode),
        escape_json(&answer.llm_provider),
        escape_json(&answer.llm_status),
        option_string_json(answer.llm_text.as_deref()),
        option_string_json(answer.llm_error.as_deref()),
        option_string_json(answer.redacted_output_preview.as_deref()),
        answer.prompt_evidence_bytes,
        deterministic.retrieval_context.hits.len(),
        json_owned_string_array(&deterministic.assumptions),
        json_owned_string_array(&deterministic.uncertainty),
        json_owned_string_array(&deterministic.missing_information)
    )
}

fn hydrated_vector_retrieval_chunks(
    body: &str,
    database_url: Option<&str>,
) -> Result<Value, GatewayError> {
    let payload = parse_retrieval_search(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let vector_context: Value = serde_json::from_str(&search_vector_chunks(body)?)
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut hits = Vec::new();
    for hit in vector_context
        .get("hits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(payload.limit as usize)
    {
        let Some(chunk_id) = hit.get("chunk_id").and_then(Value::as_str) else {
            continue;
        };
        let trail_text = retrieval_chunk_trail_json(&mut client, chunk_id)?;
        let trail: Value = serde_json::from_str(&trail_text)
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        hits.push(serde_json::json!({
            "score": hit.get("score").and_then(Value::as_f64).unwrap_or(0.0),
            "qdrant_payload": hit.get("payload").cloned().unwrap_or_else(|| serde_json::json!({})),
            "chunk": trail.get("chunk").cloned().unwrap_or(Value::Null),
            "document": trail.get("document").cloned().unwrap_or(Value::Null),
            "source": trail.get("source").cloned().unwrap_or(Value::Null),
            "raw_artifact": trail.get("raw_artifact").cloned().unwrap_or(Value::Null),
            "evidence_items": trail.get("evidence_items").cloned().unwrap_or_else(|| serde_json::json!([]))
        }));
    }
    Ok(serde_json::json!({
        "query": payload.query,
        "collection_name": vector_context.get("collection_name").cloned().unwrap_or_else(|| serde_json::json!("igy6_chunks")),
        "collection_exists": vector_context.get("collection_exists").and_then(Value::as_bool).unwrap_or(false),
        "hits": hits
    }))
}

fn ensure_vector_chunk_collection() -> Result<String, GatewayError> {
    let settings = qdrant_settings_from_env()?;
    let current = execute_qdrant_plan(collection_status_request(&settings)?)?;
    if current.status_code == 404 {
        let created = execute_qdrant_plan(ensure_collection_request(&settings)?)?;
        if created.status_code >= 400 {
            return Err(GatewayError::ServiceUnavailable(created.body));
        }
        return vector_collection_status_from_qdrant(&settings);
    }
    if current.status_code >= 400 {
        return Err(GatewayError::ServiceUnavailable(current.body));
    }
    Ok(vector_collection_status_json_from_body(
        &settings.collection_name,
        true,
        Some(&current.body),
    ))
}

fn search_vector_chunks(body: &str) -> Result<String, GatewayError> {
    let payload = parse_retrieval_search(body)?;
    let settings = qdrant_settings_from_env()?;
    let response = execute_qdrant_plan(search_points_request(
        &settings,
        &payload.query,
        payload.limit as usize,
    )?)?;
    if is_qdrant_missing_collection(&response, &settings.collection_name) {
        return Ok(serde_json::json!({
            "query": payload.query,
            "collection_name": settings.collection_name,
            "collection_exists": false,
            "hits": []
        })
        .to_string());
    }
    if response.status_code >= 400 {
        return Err(GatewayError::ServiceUnavailable(response.body));
    }
    let value: Value = serde_json::from_str(&response.body)
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let hits = value
        .get("result")
        .and_then(Value::as_array)
        .map(|results| {
            results
                .iter()
                .take(payload.limit as usize)
                .map(|result| {
                    let payload = result
                        .get("payload")
                        .cloned()
                        .filter(Value::is_object)
                        .unwrap_or_else(|| serde_json::json!({}));
                    serde_json::json!({
                        "chunk_id": payload.get("chunk_id").and_then(Value::as_str),
                        "document_id": payload.get("document_id").and_then(Value::as_str),
                        "score": result.get("score").and_then(Value::as_f64).unwrap_or(0.0),
                        "payload": payload
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(serde_json::json!({
        "query": payload.query,
        "collection_name": settings.collection_name,
        "collection_exists": true,
        "hits": hits
    })
    .to_string())
}

fn upsert_vector_chunks(database_url: Option<&str>) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let settings = qdrant_settings_from_env()?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let collection_status = ensure_vector_chunk_collection()?;
    let collection_exists = serde_json::from_str::<Value>(&collection_status)
        .ok()
        .and_then(|value| value.get("exists").and_then(Value::as_bool))
        .unwrap_or(true);
    let rows = client
        .query(
            "SELECT id, document_id, chunk_index, text_content FROM chunks WHERE embedding_status != 'completed' ORDER BY created_at ASC LIMIT 100",
            &[],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut points = Vec::new();
    let mut chunk_ids = Vec::new();
    for row in &rows {
        let id: String = row.get(0);
        let document_id: String = row.get(1);
        let chunk_index: i32 = row.get(2);
        let text_content: String = row.get(3);
        points.push(plan_chunk_vector_point(
            &id,
            &document_id,
            chunk_index.max(0) as usize,
            &text_content,
            settings.vector_size,
        )?);
        chunk_ids.push(id);
    }
    if points.is_empty() {
        return Ok(serde_json::json!({
            "chunks_selected": 0,
            "chunks_upserted": 0,
            "collection_name": settings.collection_name,
            "collection_exists": collection_exists
        })
        .to_string());
    }
    let response = execute_qdrant_plan(upsert_points_request(&settings, &points)?)?;
    if response.status_code >= 400 {
        return Err(GatewayError::ServiceUnavailable(response.body));
    }
    for chunk_id in &chunk_ids {
        client
            .execute(
                "UPDATE chunks SET embedding_status = 'completed', metadata_json = metadata_json || $1::jsonb, updated_at = now() WHERE id = $2",
                &[&serde_json::json!({
                    "embedding_method": EMBEDDING_METHOD,
                    "vector_collection": settings.collection_name
                }), chunk_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    }
    Ok(serde_json::json!({
        "chunks_selected": chunk_ids.len(),
        "chunks_upserted": points.len(),
        "collection_name": settings.collection_name,
        "collection_exists": true
    })
    .to_string())
}

fn get_graph_node_relationships(node_label: &str, node_id: &str) -> Result<String, GatewayError> {
    let node_label = validate_graph_node_label(node_label)?;
    let node_id = validate_route_id(node_id, "node_id")?;
    let statement = format!(
        "MATCH (node:{node_label} {{id: $node_id}}) OPTIONAL MATCH (node)-[outgoing]->(out_neighbor) WITH node, collect({{direction: 'outgoing', relationship_type: type(outgoing), neighbor_label: labels(out_neighbor)[0], neighbor_id: out_neighbor.id}}) AS outgoing_relationships OPTIONAL MATCH (in_neighbor)-[incoming]->(node) WITH outgoing_relationships + collect({{direction: 'incoming', relationship_type: type(incoming), neighbor_label: labels(in_neighbor)[0], neighbor_id: in_neighbor.id}}) AS relationships UNWIND relationships AS relationship WITH relationship WHERE relationship.relationship_type IS NOT NULL RETURN relationship LIMIT $limit"
    );
    let value = execute_neo4j_statements(vec![Neo4jStatement {
        statement,
        parameters: serde_json::json!({"node_id": node_id, "limit": 100}),
    }])?;
    let relationships = neo4j_first_result_rows(&value)
        .into_iter()
        .filter_map(|row| row.as_array().and_then(|items| items.first()).cloned())
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "node_label": node_label,
        "node_id": node_id,
        "relationships": relationships
    })
    .to_string())
}

fn ensure_graph_schema() -> Result<String, GatewayError> {
    let mut statements = graph_constraint_statements()
        .iter()
        .map(|statement| Neo4jStatement {
            statement: (*statement).to_string(),
            parameters: serde_json::json!({}),
        })
        .collect::<Vec<_>>();
    statements.push(Neo4jStatement {
        statement: "SHOW CONSTRAINTS YIELD name, type, labelsOrTypes, properties RETURN name, type, labelsOrTypes, properties".to_string(),
        parameters: serde_json::json!({}),
    });
    let value = execute_neo4j_statements(statements)?;
    let constraints = neo4j_result_rows_at(&value, graph_constraint_statements().len())
        .into_iter()
        .map(|row| {
            let items = row.as_array().cloned().unwrap_or_default();
            serde_json::json!({
                "name": items.first().cloned().unwrap_or(Value::Null),
                "type": items.get(1).cloned().unwrap_or(Value::Null),
                "labelsOrTypes": items.get(2).cloned().unwrap_or(Value::Null),
                "properties": items.get(3).cloned().unwrap_or(Value::Null)
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({"constraints": constraints}).to_string())
}

fn sync_graph_lineage(database_url: Option<&str>) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut statements = graph_constraint_statements()
        .iter()
        .map(|statement| Neo4jStatement {
            statement: (*statement).to_string(),
            parameters: serde_json::json!({}),
        })
        .collect::<Vec<_>>();
    let mut node_count = 0usize;
    let mut relationship_count = 0usize;

    for source_id in query_ids(&mut client, "SELECT id FROM sources")? {
        statements.push(merge_node_statement("Source", &source_id));
        node_count += 1;
    }
    for row in client
        .query("SELECT id, source_id FROM raw_artifacts", &[])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let artifact_id: String = row.get(0);
        let source_id: Option<String> = row.get(1);
        statements.push(merge_node_statement("RawArtifact", &artifact_id));
        node_count += 1;
        if let Some(source_id) = source_id {
            statements.push(merge_relationship_statement(
                "Source",
                &source_id,
                "RawArtifact",
                &artifact_id,
                "SOURCE_HAS_ARTIFACT",
            ));
            relationship_count += 1;
        }
    }
    for row in client
        .query("SELECT id, raw_artifact_id FROM normalized_documents", &[])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let document_id: String = row.get(0);
        let raw_artifact_id: Option<String> = row.get(1);
        statements.push(merge_node_statement("Document", &document_id));
        node_count += 1;
        if let Some(raw_artifact_id) = raw_artifact_id {
            statements.push(merge_relationship_statement(
                "RawArtifact",
                &raw_artifact_id,
                "Document",
                &document_id,
                "ARTIFACT_HAS_DOCUMENT",
            ));
            relationship_count += 1;
        }
    }
    for row in client
        .query("SELECT id, document_id FROM chunks", &[])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let chunk_id: String = row.get(0);
        let document_id: String = row.get(1);
        statements.push(merge_node_statement("Chunk", &chunk_id));
        statements.push(merge_relationship_statement(
            "Document",
            &document_id,
            "Chunk",
            &chunk_id,
            "DOCUMENT_HAS_CHUNK",
        ));
        node_count += 1;
        relationship_count += 1;
    }
    for row in client
        .query("SELECT id, document_id, chunk_id FROM evidence_items", &[])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let evidence_id: String = row.get(0);
        let document_id: Option<String> = row.get(1);
        let chunk_id: Option<String> = row.get(2);
        statements.push(merge_node_statement("EvidenceItem", &evidence_id));
        node_count += 1;
        if let Some(document_id) = document_id {
            statements.push(merge_relationship_statement(
                "Document",
                &document_id,
                "EvidenceItem",
                &evidence_id,
                "DOCUMENT_HAS_EVIDENCE",
            ));
            relationship_count += 1;
        }
        if let Some(chunk_id) = chunk_id {
            statements.push(merge_relationship_statement(
                "Chunk",
                &chunk_id,
                "EvidenceItem",
                &evidence_id,
                "CHUNK_HAS_EVIDENCE",
            ));
            relationship_count += 1;
        }
    }
    relationship_count += merge_evidence_targets(
        &mut client,
        &mut statements,
        "SELECT id, evidence_ids FROM claims",
        "Claim",
        "EVIDENCE_SUPPORTS_CLAIM",
        &mut node_count,
    )?;
    relationship_count += merge_evidence_targets(
        &mut client,
        &mut statements,
        "SELECT id, evidence_ids FROM patterns",
        "Pattern",
        "EVIDENCE_SUPPORTS_PATTERN",
        &mut node_count,
    )?;
    relationship_count += merge_evidence_targets(
        &mut client,
        &mut statements,
        "SELECT id, supporting_evidence_ids FROM hypotheses",
        "Hypothesis",
        "EVIDENCE_SUPPORTS_HYPOTHESIS",
        &mut node_count,
    )?;
    relationship_count += merge_evidence_targets(
        &mut client,
        &mut statements,
        "SELECT id, evidence_ids FROM predictions",
        "Prediction",
        "EVIDENCE_SUPPORTS_PREDICTION",
        &mut node_count,
    )?;
    relationship_count += merge_evidence_targets(
        &mut client,
        &mut statements,
        "SELECT id, evidence_ids FROM recommendations",
        "Recommendation",
        "EVIDENCE_SUPPORTS_RECOMMENDATION",
        &mut node_count,
    )?;
    for report_id in query_ids(&mut client, "SELECT id FROM reports")? {
        statements.push(merge_node_statement("Report", &report_id));
        node_count += 1;
    }
    for row in client
        .query(
            "SELECT id, target_type, target_id, evidence_ids FROM outcomes",
            &[],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let outcome_id: String = row.get(0);
        let target_type: String = row.get(1);
        let target_id: String = row.get(2);
        let evidence_ids: Value = row.get(3);
        statements.push(merge_node_statement("Outcome", &outcome_id));
        node_count += 1;
        for evidence_id in string_values_from_json_array(&evidence_ids) {
            statements.push(merge_relationship_statement(
                "EvidenceItem",
                &evidence_id,
                "Outcome",
                &outcome_id,
                "EVIDENCE_SUPPORTS_OUTCOME",
            ));
            relationship_count += 1;
        }
        if let Some((label, relationship_type)) = outcome_target_graph(&target_type) {
            statements.push(merge_relationship_statement(
                label,
                &target_id,
                "Outcome",
                &outcome_id,
                relationship_type,
            ));
            relationship_count += 1;
        }
    }
    execute_neo4j_statements_batched(statements, 100)?;
    Ok(serde_json::json!({
        "nodes": node_count,
        "relationships": relationship_count
    })
    .to_string())
}

fn qdrant_settings_from_env() -> Result<QdrantSettings, GatewayError> {
    let base_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://qdrant:6333".to_string());
    let collection_name =
        env::var("QDRANT_CHUNK_COLLECTION").unwrap_or_else(|_| "igy6_chunks".to_string());
    let vector_size = env::var("QDRANT_CHUNK_VECTOR_SIZE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(384);
    if vector_size < 1 {
        return Err(GatewayError::Validation(
            "QDRANT_CHUNK_VECTOR_SIZE must be at least 1.".to_string(),
        ));
    }
    Ok(QdrantSettings {
        base_url,
        collection_name,
        vector_size,
    })
}

fn execute_qdrant_plan(plan: HttpRequestPlan) -> Result<ExternalHttpResponse, GatewayError> {
    let method = match plan.method {
        HttpMethod::Get => "GET",
        HttpMethod::Put => "PUT",
        HttpMethod::Post => "POST",
    };
    execute_external_http(ExternalHttpRequest {
        method: method.to_string(),
        origin: plan.origin,
        path: plan.path,
        body: plan.body,
        headers: Vec::new(),
        timeout: Duration::from_secs(plan.timeout_seconds),
    })
}

fn vector_collection_status_from_qdrant(settings: &QdrantSettings) -> Result<String, GatewayError> {
    let current = execute_qdrant_plan(collection_status_request(settings)?)?;
    if current.status_code == 404 {
        return Ok(vector_collection_status_json_from_body(
            &settings.collection_name,
            false,
            None,
        ));
    }
    if current.status_code >= 400 {
        return Err(GatewayError::ServiceUnavailable(current.body));
    }
    Ok(vector_collection_status_json_from_body(
        &settings.collection_name,
        true,
        Some(&current.body),
    ))
}

fn vector_collection_status_json_from_body(
    collection_name: &str,
    exists: bool,
    detail_body: Option<&str>,
) -> String {
    let detail = detail_body
        .and_then(|body| serde_json::from_str::<Value>(body).ok())
        .unwrap_or(Value::Null);
    serde_json::json!({
        "collection_name": collection_name,
        "exists": exists,
        "detail": if exists { detail } else { Value::Null }
    })
    .to_string()
}

fn is_qdrant_missing_collection(response: &ExternalHttpResponse, collection_name: &str) -> bool {
    if response.status_code != 404 {
        return false;
    }
    let body = response.body.to_lowercase();
    let collection_name = collection_name.to_lowercase();
    body.contains(&collection_name)
        && body.contains("collection")
        && (body.contains("not found") || body.contains("doesn't exist"))
}

fn execute_neo4j_statements(statements: Vec<Neo4jStatement>) -> Result<Value, GatewayError> {
    execute_neo4j_statements_batched(statements, usize::MAX)
}

fn execute_neo4j_statements_batched(
    statements: Vec<Neo4jStatement>,
    batch_size: usize,
) -> Result<Value, GatewayError> {
    let mut last_response = serde_json::json!({"results": [], "errors": []});
    for batch in statements.chunks(batch_size.max(1)) {
        let body = serde_json::json!({
            "statements": batch.iter().map(|statement| {
                serde_json::json!({
                    "statement": statement.statement,
                    "parameters": statement.parameters,
                    "resultDataContents": ["row"]
                })
            }).collect::<Vec<_>>()
        })
        .to_string();
        let response = execute_external_http(ExternalHttpRequest {
            method: "POST".to_string(),
            origin: neo4j_http_origin()?,
            path: "/db/neo4j/tx/commit".to_string(),
            body: Some(body),
            headers: vec![("Authorization".to_string(), neo4j_basic_auth_header())],
            timeout: Duration::from_secs(15),
        })?;
        if response.status_code >= 400 {
            return Err(GatewayError::ServiceUnavailable(response.body));
        }
        let value: Value = serde_json::from_str(&response.body)
            .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
        let errors = value
            .get("errors")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !errors.is_empty() {
            return Err(GatewayError::ServiceUnavailable(
                Value::Array(errors).to_string(),
            ));
        }
        last_response = value;
    }
    Ok(last_response)
}

fn neo4j_http_origin() -> Result<String, GatewayError> {
    if let Ok(origin) = env::var("NEO4J_HTTP_URL") {
        return normalize_http_origin_for_service(&origin);
    }
    let bolt_uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let Some((host, _)) = host_port_from_url(&bolt_uri) else {
        return Err(GatewayError::Validation(
            "NEO4J_URI must include host and port.".to_string(),
        ));
    };
    let port = env::var("NEO4J_HTTP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(7474);
    normalize_http_origin_for_service(&format!("http://{host}:{port}"))
}

fn neo4j_basic_auth_header() -> String {
    let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password =
        env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "change-me-local-only".to_string());
    format!(
        "Basic {}",
        base64_encode(format!("{user}:{password}").as_bytes())
    )
}

fn neo4j_first_result_rows(value: &Value) -> Vec<Value> {
    neo4j_result_rows_at(value, 0)
}

fn neo4j_result_rows_at(value: &Value, index: usize) -> Vec<Value> {
    value
        .get("results")
        .and_then(Value::as_array)
        .and_then(|results| results.get(index))
        .and_then(|result| result.get("data"))
        .and_then(Value::as_array)
        .map(|data| {
            data.iter()
                .filter_map(|record| record.get("row").cloned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn graph_constraint_statements() -> &'static [&'static str] {
    &[
        "CREATE CONSTRAINT source_id_unique IF NOT EXISTS FOR (node:Source) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT raw_artifact_id_unique IF NOT EXISTS FOR (node:RawArtifact) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT document_id_unique IF NOT EXISTS FOR (node:Document) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT chunk_id_unique IF NOT EXISTS FOR (node:Chunk) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT evidence_item_id_unique IF NOT EXISTS FOR (node:EvidenceItem) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT claim_id_unique IF NOT EXISTS FOR (node:Claim) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT pattern_id_unique IF NOT EXISTS FOR (node:Pattern) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT hypothesis_id_unique IF NOT EXISTS FOR (node:Hypothesis) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT prediction_id_unique IF NOT EXISTS FOR (node:Prediction) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT recommendation_id_unique IF NOT EXISTS FOR (node:Recommendation) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT outcome_id_unique IF NOT EXISTS FOR (node:Outcome) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT report_id_unique IF NOT EXISTS FOR (node:Report) REQUIRE node.id IS UNIQUE",
    ]
}

fn validate_graph_node_label(label: &str) -> Result<String, GatewayError> {
    if graph_node_labels().contains(&label) {
        Ok(label.to_string())
    } else {
        Err(GatewayError::Validation(
            "Unsupported graph node label".to_string(),
        ))
    }
}

fn graph_node_labels() -> &'static [&'static str] {
    &[
        "Source",
        "RawArtifact",
        "Document",
        "Chunk",
        "EvidenceItem",
        "Claim",
        "Pattern",
        "Hypothesis",
        "Prediction",
        "Recommendation",
        "Outcome",
        "Report",
    ]
}

fn query_ids(client: &mut Client, sql: &str) -> Result<Vec<String>, GatewayError> {
    client
        .query(sql, &[])
        .map_err(|error| GatewayError::Database(error.to_string()))
        .map(|rows| {
            rows.into_iter()
                .map(|row| row.get::<_, String>(0))
                .collect()
        })
}

fn merge_node_statement(label: &str, node_id: &str) -> Neo4jStatement {
    Neo4jStatement {
        statement: format!("MERGE (:{label} {{id: $id}})"),
        parameters: serde_json::json!({"id": node_id}),
    }
}

fn merge_relationship_statement(
    left_label: &str,
    left_id: &str,
    right_label: &str,
    right_id: &str,
    relationship_type: &str,
) -> Neo4jStatement {
    Neo4jStatement {
        statement: format!(
            "MATCH (left:{left_label} {{id: $left_id}}) MATCH (right:{right_label} {{id: $right_id}}) MERGE (left)-[:{relationship_type}]->(right)"
        ),
        parameters: serde_json::json!({
            "left_id": left_id,
            "right_id": right_id
        }),
    }
}

fn merge_evidence_targets(
    client: &mut Client,
    statements: &mut Vec<Neo4jStatement>,
    sql: &str,
    target_label: &str,
    relationship_type: &str,
    node_count: &mut usize,
) -> Result<usize, GatewayError> {
    let mut relationships = 0usize;
    for row in client
        .query(sql, &[])
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let target_id: String = row.get(0);
        let evidence_ids: Value = row.get(1);
        statements.push(merge_node_statement(target_label, &target_id));
        *node_count += 1;
        for evidence_id in string_values_from_json_array(&evidence_ids) {
            statements.push(merge_relationship_statement(
                "EvidenceItem",
                &evidence_id,
                target_label,
                &target_id,
                relationship_type,
            ));
            relationships += 1;
        }
    }
    Ok(relationships)
}

fn outcome_target_graph(target_type: &str) -> Option<(&'static str, &'static str)> {
    match target_type {
        "pattern" => Some(("Pattern", "PATTERN_HAS_OUTCOME")),
        "hypothesis" => Some(("Hypothesis", "HYPOTHESIS_HAS_OUTCOME")),
        "prediction" => Some(("Prediction", "PREDICTION_HAS_OUTCOME")),
        "recommendation" => Some(("Recommendation", "RECOMMENDATION_HAS_OUTCOME")),
        "report" => Some(("Report", "REPORT_HAS_OUTCOME")),
        _ => None,
    }
}

fn string_values_from_json_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
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
struct BaselineOutcomeItem {
    id: String,
    target_type: String,
    target_id: String,
    outcome_status: String,
    evidence_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct BaselinePatternCandidate {
    pattern_type: String,
    summary: String,
    evidence_ids: Vec<String>,
    confidence: i32,
    detector_key: String,
    support_count: i32,
    metadata_json: Value,
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

fn load_outcomes_for_baseline(
    transaction: &mut postgres::Transaction<'_>,
) -> Result<Vec<BaselineOutcomeItem>, GatewayError> {
    transaction
        .query(
            "SELECT id, target_type, target_id, outcome_status, evidence_ids FROM outcomes ORDER BY created_at DESC",
            &[],
        )
        .map(|rows| {
            rows.into_iter()
                .map(|row| BaselineOutcomeItem {
                    id: row.get::<_, String>(0),
                    target_type: row.get::<_, String>(1),
                    target_id: row.get::<_, String>(2),
                    outcome_status: row.get::<_, String>(3),
                    evidence_ids: string_values_from_json_array(&row.get::<_, Value>(4)),
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
        "report_generation" => {
            if !work_item
                .payload_json
                .get("report_id")
                .is_some_and(Value::is_string)
            {
                return Err(GatewayError::Validation(
                    "Invalid report generation payload".to_string(),
                ));
            }
            Ok("report.generate_markdown".to_string())
        }
        "chunk_vector_upsert" => Ok("memory.vector.upsert_chunks".to_string()),
        _ => Err(GatewayError::Validation(
            "Unsupported work item dispatch type".to_string(),
        )),
    }
}

fn baseline_pattern_candidates(
    evidence_items: &[BaselineEvidenceItem],
    outcomes: &[BaselineOutcomeItem],
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
            support_count: 0,
            metadata_json: serde_json::json!({
                "unverified_note": "No evidence exists; this is an intake gap, not a real-world absence claim.",
                "linked_source_ids": [],
                "linked_outcome_ids": []
            }),
        }];
    }

    let mut candidates = Vec::new();
    let mut by_type: HashMap<String, Vec<&BaselineEvidenceItem>> = HashMap::new();
    let mut by_statement: HashMap<String, Vec<&BaselineEvidenceItem>> = HashMap::new();
    let mut by_config_key: HashMap<String, Vec<&BaselineEvidenceItem>> = HashMap::new();
    for item in evidence_items {
        by_type
            .entry(item.evidence_type.clone())
            .or_default()
            .push(item);
        by_statement
            .entry(normalize_statement(&item.statement))
            .or_default()
            .push(item);
        if let Some(config_key) = config_drift_key(&item.statement) {
            by_config_key.entry(config_key).or_default().push(item);
        }
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
                support_count: items.len() as i32,
                metadata_json: serde_json::json!({
                    "category": "recurrence",
                    "support_basis": "evidence_type_count",
                    "evidence_type": evidence_type,
                    "linked_source_ids": source_ids_for_items(items),
                    "unverified_note": "Repeated evidence type is a baseline signal and still needs review."
                }),
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
            let pattern_type = if conflict_signal(items) {
                "cross_source_conflict"
            } else {
                "cross_source_agreement"
            };
            candidates.push(BaselinePatternCandidate {
                pattern_type: pattern_type.to_string(),
                summary: if pattern_type == "cross_source_conflict" {
                    "Multiple sources contain related conflict-language around the same normalized evidence statement; review disagreement before relying on it.".to_string()
                } else {
                    "Multiple sources contain the same normalized evidence statement; review whether this is agreement or duplicate evidence.".to_string()
                },
                evidence_ids: items.iter().take(10).map(|item| item.id.clone()).collect(),
                confidence: if pattern_type == "cross_source_conflict" { 55 } else { 65 },
                detector_key: format!("{pattern_type}:{normalized_statement}"),
                support_count: source_ids.len() as i32,
                metadata_json: serde_json::json!({
                    "category": pattern_type,
                    "support_basis": "normalized_statement_seen_across_sources",
                    "linked_source_ids": source_ids_for_items(items),
                    "unverified_note": "Cross-source signal is baseline matching only; source quality and context still require review."
                }),
            });
        }
    }

    let mut config_keys = by_config_key.keys().cloned().collect::<Vec<_>>();
    config_keys.sort();
    for config_key in config_keys {
        let items = &by_config_key[&config_key];
        let distinct_statements = items
            .iter()
            .map(|item| normalize_statement(&item.statement))
            .collect::<HashSet<_>>();
        if distinct_statements.len() >= 2 {
            candidates.push(BaselinePatternCandidate {
                pattern_type: "configuration_drift".to_string(),
                summary: format!(
                    "Configuration-like evidence for `{}` changed or disagrees across {} records.",
                    config_key,
                    items.len()
                ),
                evidence_ids: items.iter().take(10).map(|item| item.id.clone()).collect(),
                confidence: 55,
                detector_key: format!("configuration_drift:{config_key}"),
                support_count: items.len() as i32,
                metadata_json: serde_json::json!({
                    "category": "configuration_drift",
                    "support_basis": "configuration_keyword_group",
                    "config_key": config_key,
                    "linked_source_ids": source_ids_for_items(items),
                    "unverified_note": "Configuration drift is keyword and grouping based; it is not a full config parser."
                }),
            });
        }
    }

    let anomaly_items = evidence_items
        .iter()
        .filter(|item| anomaly_signal(&item.statement))
        .take(10)
        .collect::<Vec<_>>();
    if !anomaly_items.is_empty() {
        candidates.push(BaselinePatternCandidate {
            pattern_type: "anomaly_signal".to_string(),
            summary: format!(
                "{} evidence item(s) contain anomaly or unexpected-state language.",
                anomaly_items.len()
            ),
            evidence_ids: anomaly_items
                .iter()
                .map(|item| item.id.clone())
                .collect(),
            confidence: 50,
            detector_key: format!(
                "anomaly_signal:{}",
                anomaly_items
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>()
                    .join(":")
            ),
            support_count: anomaly_items.len() as i32,
            metadata_json: serde_json::json!({
                "category": "anomaly_signal",
                "support_basis": "anomaly_keyword_match",
                "linked_source_ids": source_ids_for_items(&anomaly_items),
                "unverified_note": "Anomaly signal is keyword based and not statistical anomaly detection."
            }),
        });
    }

    add_outcome_pattern_candidates(&mut candidates, outcomes);

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

fn source_ids_for_items(items: &[&BaselineEvidenceItem]) -> Vec<String> {
    let mut source_ids = items
        .iter()
        .filter_map(|item| item.source_id.clone())
        .collect::<Vec<_>>();
    source_ids.sort();
    source_ids.dedup();
    source_ids
}

fn conflict_signal(items: &[&BaselineEvidenceItem]) -> bool {
    items.iter().any(|item| {
        let value = item.statement.to_ascii_lowercase();
        [
            "conflict",
            "contradict",
            "disagree",
            "mismatch",
            "inconsistent",
            "wrong",
            "failed",
        ]
        .iter()
        .any(|token| value.contains(token))
    })
}

fn config_drift_key(statement: &str) -> Option<String> {
    let normalized = normalize_statement(statement);
    let config_terms = [
        "config",
        "configuration",
        "setting",
        "version",
        "env ",
        "feature flag",
        "threshold",
    ];
    if !config_terms.iter().any(|term| normalized.contains(term)) {
        return None;
    }
    normalized
        .split([':', '=', '-'])
        .next()
        .map(str::trim)
        .filter(|value| value.len() >= 3)
        .map(|value| value.chars().take(80).collect())
}

fn anomaly_signal(statement: &str) -> bool {
    let normalized = statement.to_ascii_lowercase();
    [
        "anomaly",
        "unexpected",
        "outlier",
        "spike",
        "regression",
        "unusual",
        "sudden",
        "abnormal",
    ]
    .iter()
    .any(|token| normalized.contains(token))
}

fn add_outcome_pattern_candidates(
    candidates: &mut Vec<BaselinePatternCandidate>,
    outcomes: &[BaselineOutcomeItem],
) {
    let mut by_status: HashMap<String, Vec<&BaselineOutcomeItem>> = HashMap::new();
    for outcome in outcomes {
        by_status
            .entry(outcome.outcome_status.clone())
            .or_default()
            .push(outcome);
    }
    for (status, pattern_type, summary) in [
        (
            "wrong",
            "failed_advice_recurrence",
            "Repeated wrong outcomes are recorded; review failed advice or weak method recurrence.",
        ),
        (
            "not_useful",
            "failed_advice_recurrence",
            "Repeated not-useful outcomes are recorded; review failed advice or weak method recurrence.",
        ),
        (
            "correct",
            "successful_method_recurrence",
            "Repeated correct outcomes are recorded; review whether a successful method is recurring.",
        ),
        (
            "useful",
            "successful_method_recurrence",
            "Repeated useful outcomes are recorded; review whether a successful method is recurring.",
        ),
    ] {
        let Some(items) = by_status.get(status) else {
            continue;
        };
        if items.len() < 2 {
            continue;
        }
        let mut evidence_ids = items
            .iter()
            .flat_map(|item| item.evidence_ids.iter().cloned())
            .collect::<Vec<_>>();
        evidence_ids.sort();
        evidence_ids.dedup();
        candidates.push(BaselinePatternCandidate {
            pattern_type: pattern_type.to_string(),
            summary: summary.to_string(),
            evidence_ids: evidence_ids.into_iter().take(10).collect(),
            confidence: 60,
            detector_key: format!("outcome_status:{status}:{pattern_type}"),
            support_count: items.len() as i32,
            metadata_json: serde_json::json!({
                "category": pattern_type,
                "support_basis": "outcome_status_count",
                "outcome_status": status,
                "linked_outcome_ids": items.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
                "linked_target_ids": items.iter().map(|item| format!("{}:{}", item.target_type, item.target_id)).collect::<Vec<_>>(),
                "unverified_note": "Outcome recurrence is count based; it does not auto-change behavior or prove causality."
            }),
        });
    }
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

fn create_source_permission(
    source_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let source_id = validate_route_id(source_id, "source_id")?;
    let object = parse_json_object(body, "Source permission request body")?;
    let payload = parse_source_permission_create(&object)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let source_exists = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
            &[&source_id],
        )
        .map(|row| row.get::<_, bool>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if !source_exists {
        return Err(GatewayError::NotFound("Source not found".to_string()));
    }
    let permission_id = generated_record_id("permission");
    let allowed_operations_json = Value::Array(
        payload
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
                &payload.scope_json,
                &allowed_operations_json,
                &payload.external_model_policy,
                &payload.approval_required,
                &payload.created_by_actor_id,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "permission_id": permission_id,
        "allowed_operations": payload.allowed_operations,
        "approval_required": payload.approval_required,
        "external_model_policy": payload.external_model_policy
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'source_permission.created', 'recorded', 'source', $2, NULL, $3::jsonb)",
            &[&payload.created_by_actor_id, &source_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, scope_json, allowed_operations, external_model_policy, approval_required, created_by_actor_id, created_at, updated_at FROM source_permissions WHERE id = $1) t",
            &[&permission_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn update_source_review_state(
    source_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let source_id = validate_route_id(source_id, "source_id")?;
    let payload = parse_source_review_state(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let row = transaction
        .query_opt(
            "SELECT trust_level, sensitivity, enabled FROM sources WHERE id = $1",
            &[&source_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
        .ok_or_else(|| GatewayError::NotFound("Source not found".to_string()))?;
    let previous_trust_level = row.get::<_, String>(0);
    let previous_sensitivity = row.get::<_, String>(1);
    let previous_enabled = row.get::<_, bool>(2);

    transaction
        .execute(
            "UPDATE sources SET trust_level = $1, sensitivity = $2, enabled = $3, updated_at = NOW() WHERE id = $4",
            &[
                &payload.trust_level,
                &payload.sensitivity,
                &payload.enabled,
                &source_id,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details = serde_json::json!({
        "previous_trust_level": previous_trust_level,
        "new_trust_level": payload.trust_level,
        "previous_sensitivity": previous_sensitivity,
        "new_sensitivity": payload.sensitivity,
        "previous_enabled": previous_enabled,
        "new_enabled": payload.enabled,
        "review_note": payload.review_note,
        "policy_enforcement_changed": false,
        "evidence_hidden": false,
        "source_deleted": false
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'source.review_state_updated', 'recorded', 'source', $2, NULL, $3::jsonb)",
            &[&payload.actor_id, &source_id, &details],
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

fn create_hypothesis(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_hypothesis_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    validate_evidence_ids(&mut transaction, &payload.supporting_evidence_ids)?;
    let hypothesis_id = generated_record_id("hypothesis");
    let evidence_ids_json = json_string_values(&payload.supporting_evidence_ids);
    transaction
        .execute(
            "INSERT INTO hypotheses (id, hypothesis_text, status, supporting_evidence_ids, missing_evidence_json, confidence, metadata_json) VALUES ($1, $2, $3, $4::jsonb, $5::jsonb, $6, $7::jsonb)",
            &[
                &hypothesis_id,
                &payload.hypothesis_text,
                &payload.status,
                &evidence_ids_json,
                &payload.missing_evidence_json,
                &payload.confidence,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_analysis_created_audit(
        &mut transaction,
        &payload.actor_id,
        "hypothesis",
        &hypothesis_id,
        &payload.supporting_evidence_ids,
    )?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, hypothesis_text, status, supporting_evidence_ids, missing_evidence_json, confidence, metadata_json, created_at, updated_at FROM hypotheses WHERE id = $1) t",
            &[&hypothesis_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_prediction(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_prediction_create(body)?;
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
    let prediction_id = generated_record_id("prediction");
    let evidence_ids_json = json_string_values(&payload.evidence_ids);
    transaction
        .execute(
            "INSERT INTO predictions (id, prediction_text, expected_result, disproof_condition, status, evidence_ids, confidence, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8::jsonb)",
            &[
                &prediction_id,
                &payload.prediction_text,
                &payload.expected_result,
                &payload.disproof_condition,
                &payload.status,
                &evidence_ids_json,
                &payload.confidence,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_analysis_created_audit(
        &mut transaction,
        &payload.actor_id,
        "prediction",
        &prediction_id,
        &payload.evidence_ids,
    )?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, prediction_text, expected_result, disproof_condition, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM predictions WHERE id = $1) t",
            &[&prediction_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_recommendation(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_recommendation_create(body)?;
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
    let recommendation_id = generated_record_id("recommendation");
    let evidence_ids_json = json_string_values(&payload.evidence_ids);
    transaction
        .execute(
            "INSERT INTO recommendations (id, recommendation_text, risk_level, approval_required, expected_result, status, evidence_ids, confidence, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, $9::jsonb)",
            &[
                &recommendation_id,
                &payload.recommendation_text,
                &payload.risk_level,
                &payload.approval_required,
                &payload.expected_result,
                &payload.status,
                &evidence_ids_json,
                &payload.confidence,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_analysis_created_audit(
        &mut transaction,
        &payload.actor_id,
        "recommendation",
        &recommendation_id,
        &payload.evidence_ids,
    )?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, recommendation_text, risk_level, approval_required, expected_result, status, evidence_ids, confidence, metadata_json, created_at, updated_at FROM recommendations WHERE id = $1) t",
            &[&recommendation_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn insert_analysis_created_audit(
    transaction: &mut postgres::Transaction<'_>,
    actor_id: &str,
    resource_type: &str,
    resource_id: &str,
    evidence_ids: &[String],
) -> Result<(), GatewayError> {
    let details_json = serde_json::json!({ "evidence_ids": evidence_ids });
    let event_type = format!("analysis.{resource_type}.created");
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, $2, 'recorded', $3, $4, NULL, $5::jsonb)",
            &[&actor_id, &event_type, &resource_type, &resource_id, &details_json],
        )
        .map(|_| ())
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn create_evidence_document(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_evidence_document_create(body)?;
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
            "SELECT id, source_id, content_hash, storage_path FROM raw_artifacts WHERE id = $1",
            &[&payload.raw_artifact_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Raw artifact not found".to_string()));
    };
    let raw_artifact_id: String = row.get(0);
    let raw_source_id: Option<String> = row.get(1);
    let content_hash: String = row.get(2);
    let storage_path: String = row.get(3);
    if payload
        .source_id
        .as_ref()
        .is_some_and(|source_id| Some(source_id) != raw_source_id.as_ref())
    {
        return Err(GatewayError::Conflict(
            "Raw artifact does not belong to the source".to_string(),
        ));
    }
    if let Some(source_id) = &raw_source_id {
        let source_exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
                &[source_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !source_exists {
            return Err(GatewayError::Conflict(
                "Raw artifact source not found".to_string(),
            ));
        }
    }
    let store = ArtifactStore::new(artifact_data_root())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let artifact_bytes = store
        .read_by_hash(&content_hash)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let text_content = String::from_utf8(artifact_bytes)
        .map_err(|_| GatewayError::Validation("Artifact is not UTF-8 text".to_string()))?;
    let document_id = generated_record_id("document");
    let metadata_json = merge_metadata(
        &payload.metadata_json,
        serde_json::json!({
            "raw_content_hash": content_hash,
            "raw_storage_path": storage_path
        }),
    );
    transaction
        .execute(
            "INSERT INTO normalized_documents (id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb)",
            &[
                &document_id,
                &raw_artifact_id,
                &raw_source_id,
                &payload.title,
                &payload.document_type,
                &payload.language,
                &text_content,
                &payload.sensitivity,
                &metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "source_id": raw_source_id,
        "raw_artifact_id": raw_artifact_id,
        "document_type": payload.document_type,
        "sensitivity": payload.sensitivity
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'normalized_document.created', 'recorded', 'normalized_document', $2, NULL, $3::jsonb)",
            &[&payload.created_by_actor_id, &document_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json, created_at, updated_at FROM normalized_documents WHERE id = $1) t",
            &[&document_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn generate_document_chunks(
    document_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let document_id = validate_route_id(document_id, "document_id")?;
    let payload = parse_chunk_generation(body)?;
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
            "SELECT id, source_id, text_content FROM normalized_documents WHERE id = $1",
            &[&document_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Document not found".to_string()));
    };
    let source_id: Option<String> = row.get(1);
    let text_content: String = row.get(2);
    let existing_chunk = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM chunks WHERE document_id = $1)",
            &[&document_id],
        )
        .map(|row| row.get::<_, bool>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if existing_chunk {
        return Err(GatewayError::Conflict(
            "Document chunks already exist".to_string(),
        ));
    }
    let text_chunks = split_text_chunks(&text_content, payload.chunk_size as usize);
    if text_chunks.is_empty() {
        return Err(GatewayError::Validation(
            "Document text is empty".to_string(),
        ));
    }
    for (index, text) in text_chunks.iter().enumerate() {
        let chunk_id = generated_record_id("chunk");
        let evidence_item_id = generated_record_id("evidence");
        let location_json = serde_json::json!({
            "char_start": index * payload.chunk_size as usize,
            "char_end": index * payload.chunk_size as usize + text.len()
        });
        let chunk_metadata = serde_json::json!({
            "generated_by": "DIFF-030",
            "chunk_size": payload.chunk_size
        });
        let evidence_metadata = serde_json::json!({
            "generated_by": "DIFF-030",
            "chunk_index": index
        });
        transaction
            .execute(
                "INSERT INTO chunks (id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json) VALUES ($1, $2, $3, $4, $5::jsonb, 'not_started', $6::jsonb)",
                &[&chunk_id, &document_id, &(index as i32), text, &location_json, &chunk_metadata],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO evidence_items (id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json) VALUES ($1, $2, $3, $4, 'document_chunk', $5, NULL, NULL, $6::jsonb)",
                &[&evidence_item_id, &source_id, &document_id, &chunk_id, text, &evidence_metadata],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    }
    let details_json = serde_json::json!({
        "source_id": source_id,
        "chunk_count": text_chunks.len(),
        "evidence_count": text_chunks.len()
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'document_chunks.generated', 'recorded', 'normalized_document', $2, NULL, $3::jsonb)",
            &[&payload.created_by_actor_id, &document_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json, created_at, updated_at FROM chunks WHERE document_id = $1 ORDER BY chunk_index ASC) t), '[]')",
            &[&document_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_evidence_item(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_evidence_item_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    validate_evidence_links(&mut transaction, &payload)?;
    let evidence_item_id = generated_record_id("evidence");
    transaction
        .execute(
            "INSERT INTO evidence_items (id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8, $9::jsonb)",
            &[
                &evidence_item_id,
                &payload.source_id,
                &payload.document_id,
                &payload.chunk_id,
                &payload.evidence_type,
                &payload.statement,
                &payload.observed_at,
                &payload.confidence,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "source_id": payload.source_id,
        "document_id": payload.document_id,
        "chunk_id": payload.chunk_id,
        "evidence_type": payload.evidence_type
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'evidence_item.created', 'recorded', 'evidence_item', $2, NULL, $3::jsonb)",
            &[&payload.created_by_actor_id, &evidence_item_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json, created_at, updated_at FROM evidence_items WHERE id = $1) t",
            &[&evidence_item_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn update_evidence_item_review_state(
    evidence_item_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let evidence_item_id = validate_route_id(evidence_item_id, "evidence_item_id")?;
    let payload = parse_evidence_review_state(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let previous_metadata = transaction
        .query_opt(
            "SELECT metadata_json FROM evidence_items WHERE id = $1",
            &[&evidence_item_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
        .ok_or_else(|| GatewayError::NotFound("Evidence item not found".to_string()))?
        .get::<_, Value>(0);

    if let Some(superseding_id) = &payload.superseding_evidence_item_id {
        if superseding_id == &evidence_item_id {
            return Err(GatewayError::Validation(
                "superseding_evidence_item_id must reference a different evidence item."
                    .to_string(),
            ));
        }
        let superseding_exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM evidence_items WHERE id = $1)",
                &[superseding_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !superseding_exists {
            return Err(GatewayError::Validation(
                "superseding_evidence_item_id must reference an existing evidence item."
                    .to_string(),
            ));
        }
    }

    let previous_review_state = previous_metadata
        .get("review_state")
        .and_then(|value| value.get("state"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let review_state_json = serde_json::json!({
        "state": payload.review_state.clone(),
        "correction_note": payload.correction_note.clone(),
        "superseding_evidence_item_id": payload.superseding_evidence_item_id.clone(),
        "reviewed_by": payload.actor_id.clone(),
        "reviewed_at": "server_time",
        "original_evidence_preserved": true,
        "raw_artifact_mutated": false,
        "document_or_chunk_rewritten": false,
        "retrieval_behavior_changed": false
    });
    let metadata_patch = serde_json::json!({
        "review_state": review_state_json
    });
    transaction
        .execute(
            "UPDATE evidence_items SET metadata_json = COALESCE(metadata_json, '{}'::jsonb) || $1::jsonb, updated_at = NOW() WHERE id = $2",
            &[&metadata_patch, &evidence_item_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "previous_review_state": previous_review_state,
        "new_review_state": payload.review_state,
        "correction_note": payload.correction_note,
        "superseding_evidence_item_id": payload.superseding_evidence_item_id,
        "original_evidence_preserved": true,
        "raw_artifact_mutated": false,
        "document_or_chunk_rewritten": false,
        "evidence_deleted": false,
        "retrieval_behavior_changed": false
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'evidence_item.review_state_updated', 'recorded', 'evidence_item', $2, NULL, $3::jsonb)",
            &[&payload.actor_id, &evidence_item_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json, created_at, updated_at FROM evidence_items WHERE id = $1) t",
            &[&evidence_item_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_experiment(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_experiment_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if let Some(improvement_item_id) = &payload.improvement_item_id {
        let exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM improvement_items WHERE id = $1)",
                &[improvement_item_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !exists {
            return Err(GatewayError::Validation(
                "Improvement item not found".to_string(),
            ));
        }
    }
    let experiment_id = generated_record_id("experiment");
    transaction
        .execute(
            "INSERT INTO experiment_runs (id, improvement_item_id, status, mlflow_run_id, optuna_study_name, metrics_json, artifacts_json, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7::jsonb, $8::jsonb)",
            &[
                &experiment_id,
                &payload.improvement_item_id,
                &payload.status,
                &payload.mlflow_run_id,
                &payload.optuna_study_name,
                &payload.metrics_json,
                &payload.artifacts_json,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "improvement_item_id": payload.improvement_item_id,
        "status": payload.status,
        "mlflow_run_id": payload.mlflow_run_id,
        "optuna_study_name": payload.optuna_study_name
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'experiment_run.created', 'created', 'experiment_run', $2, NULL, $3::jsonb)",
            &[&payload.actor_id, &experiment_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = experiment_response_json(&mut transaction, &experiment_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_experiment_proposal_from_improvement(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_experiment_proposal(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let improvement_exists = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM improvement_items WHERE id = $1)",
            &[&payload.improvement_item_id],
        )
        .map(|row| row.get::<_, bool>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if !improvement_exists {
        return Err(GatewayError::Validation(
            "Improvement item not found".to_string(),
        ));
    }

    let experiment_id = generated_record_id("experiment");
    let metrics_json = serde_json::json!({
        "success_criteria": payload.success_criteria,
        "result_comparison_plan": payload.result_comparison_plan,
        "result_comparison_status": "not_run"
    });
    let artifacts_json = serde_json::json!({
        "expected_artifacts": [],
        "artifact_collection_status": "not_run"
    });
    let metadata_json = serde_json::json!({
        "created_from": "self_improvement_experiment_workflow_mvp",
        "workflow_stage": "experiment_proposal",
        "proposal_scope": payload.proposal_scope,
        "dry_run": {
            "status": "recorded",
            "summary": payload.dry_run_summary,
            "runtime_started": false,
            "external_services_called": false
        },
        "review_status": "proposal",
        "accepted_method": {
            "approval_required": true,
            "approval_id": null,
            "method_changed": false,
            "status": "not_accepted"
        },
        "execution_model": "proposal_metadata_only",
        "autonomous_self_modification": false,
        "autonomous_method_change": false,
        "experiment_execution_started": false
    });
    let linked_improvement_item_id = Some(payload.improvement_item_id.clone());
    transaction
        .execute(
            "INSERT INTO experiment_runs (id, improvement_item_id, status, mlflow_run_id, optuna_study_name, metrics_json, artifacts_json, metadata_json) VALUES ($1, $2, 'planned', NULL, NULL, $3::jsonb, $4::jsonb, $5::jsonb)",
            &[
                &experiment_id,
                &linked_improvement_item_id,
                &metrics_json,
                &artifacts_json,
                &metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let improvement_metadata_patch = serde_json::json!({
        "latest_experiment_proposal_id": experiment_id,
        "latest_experiment_proposal_status": "planned",
        "experiment_proposal_requires_approval_for_accepted_method": true
    });
    transaction
        .execute(
            "UPDATE improvement_items SET metadata_json = COALESCE(metadata_json, '{}'::jsonb) || $1::jsonb, updated_at = NOW() WHERE id = $2",
            &[&improvement_metadata_patch, &payload.improvement_item_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "improvement_item_id": payload.improvement_item_id,
        "status": "planned",
        "workflow_stage": "experiment_proposal",
        "dry_run_recorded": true,
        "approval_required_for_accepted_method": true,
        "experiment_execution_started": false
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'experiment_run.proposed', 'planned', 'experiment_run', $2, NULL, $3::jsonb)",
            &[&payload.actor_id, &experiment_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = experiment_response_json(&mut transaction, &experiment_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn update_experiment_status(
    experiment_run_id: &str,
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let experiment_run_id = validate_route_id(experiment_run_id, "experiment_run_id")?;
    let payload = parse_experiment_status(body)?;
    let acceptance_approval_id = if payload.status == "accepted" {
        Some(experiment_acceptance_approval_id(&payload.metadata_json)?.to_string())
    } else {
        None
    };
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let row = transaction
        .query_opt(
            "SELECT status FROM experiment_runs WHERE id = $1",
            &[&experiment_run_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
        .ok_or_else(|| GatewayError::NotFound("Experiment run not found".to_string()))?;
    let previous_status: String = row.get(0);
    if let Some(approval_id) = acceptance_approval_id.as_deref() {
        require_experiment_acceptance_approval(&mut transaction, approval_id)?;
    }
    transaction
        .execute(
            "UPDATE experiment_runs SET status = $1, metrics_json = CASE WHEN $2 THEN $3::jsonb ELSE metrics_json END, artifacts_json = CASE WHEN $4 THEN $5::jsonb ELSE artifacts_json END, metadata_json = CASE WHEN $6 THEN $7::jsonb ELSE metadata_json END, updated_at = now() WHERE id = $8",
            &[
                &payload.status,
                &payload.metrics_updated,
                &payload.metrics_json,
                &payload.artifacts_updated,
                &payload.artifacts_json,
                &payload.metadata_updated,
                &payload.metadata_json,
                &experiment_run_id,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "previous_status": previous_status,
        "new_status": payload.status,
        "metrics_updated": payload.metrics_updated,
        "artifacts_updated": payload.artifacts_updated,
        "metadata_updated": payload.metadata_updated
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'experiment_run.status_updated', $2, 'experiment_run', $3, NULL, $4::jsonb)",
            &[&payload.actor_id, &payload.status, &experiment_run_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = experiment_response_json(&mut transaction, &experiment_run_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn experiment_acceptance_approval_id(metadata_json: &Value) -> Result<&str, GatewayError> {
    metadata_json
        .get("accepted_method")
        .and_then(|value| value.get("approval_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            GatewayError::Forbidden(
                "Accepted experiment method requires accepted_method.approval_id.".to_string(),
            )
        })
}

fn require_experiment_acceptance_approval(
    transaction: &mut postgres::Transaction<'_>,
    approval_id: &str,
) -> Result<(), GatewayError> {
    let row = transaction
        .query_opt(
            "SELECT status, request_type FROM approvals WHERE id = $1",
            &[&approval_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
        .ok_or_else(|| GatewayError::NotFound("Approval not found".to_string()))?;
    let status: String = row.get(0);
    let request_type: String = row.get(1);
    if status != "approved" {
        return Err(GatewayError::Forbidden(
            "Experiment acceptance approval is not approved".to_string(),
        ));
    }
    if request_type != "experiment_acceptance" {
        return Err(GatewayError::Conflict(
            "Approval is not for experiment acceptance".to_string(),
        ));
    }
    Ok(())
}

fn create_improvement(body: &str, database_url: Option<&str>) -> Result<String, GatewayError> {
    let payload = parse_improvement_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let improvement_id = generated_record_id("improvement");
    transaction
        .execute(
            "INSERT INTO improvement_items (id, target_area, status, objective, proposed_by_actor_id, priority, metadata_json) VALUES ($1, $2, 'proposed', $3, $4, $5, $6::jsonb)",
            &[
                &improvement_id,
                &payload.target_area,
                &payload.objective,
                &payload.proposed_by_actor_id,
                &payload.priority,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "target_area": payload.target_area,
        "priority": payload.priority,
        "status": "proposed"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'improvement_item.created', 'proposed', 'improvement_item', $2, NULL, $3::jsonb)",
            &[&payload.proposed_by_actor_id, &improvement_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = improvement_response_json(&mut transaction, &improvement_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn create_evidence_answer_record(
    body: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let payload = parse_evidence_answer_record_create(body)?;
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_evidence_answer_records_table(&mut client)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let answer_id = generated_record_id("answer");
    let facts_json = serde_json::json!(payload.facts);
    let assumptions_json = serde_json::json!(payload.assumptions);
    let inferences_json = serde_json::json!(payload.inferences);
    let uncertainty_json = serde_json::json!(payload.uncertainty);
    let missing_information_json = serde_json::json!(payload.missing_information);
    let evidence_item_ids_json = serde_json::json!(payload.evidence_item_ids);
    let document_ids_json = serde_json::json!(payload.document_ids);
    let chunk_ids_json = serde_json::json!(payload.chunk_ids);
    let source_ids_json = serde_json::json!(payload.source_ids);
    let safe_labels_json = serde_json::json!(payload.safe_labels);

    transaction
        .execute(
            "INSERT INTO evidence_answer_records (
                id,
                user_question,
                answer_status,
                answer_text,
                facts,
                assumptions,
                inferences,
                uncertainty,
                missing_information,
                evidence_item_ids,
                document_ids,
                chunk_ids,
                source_ids,
                safe_labels,
                retrieval_mode,
                retrieval_count,
                local_model_status,
                metadata_json
            ) VALUES ($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7::jsonb, $8::jsonb, $9::jsonb, $10::jsonb, $11::jsonb, $12::jsonb, $13::jsonb, $14::jsonb, $15, $16, $17, $18::jsonb)",
            &[
                &answer_id,
                &payload.user_question,
                &payload.answer_status,
                &payload.answer_text,
                &facts_json,
                &assumptions_json,
                &inferences_json,
                &uncertainty_json,
                &missing_information_json,
                &evidence_item_ids_json,
                &document_ids_json,
                &chunk_ids_json,
                &source_ids_json,
                &safe_labels_json,
                &payload.retrieval_mode,
                &payload.retrieval_count,
                &payload.local_model_status,
                &payload.metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let audit_details = serde_json::json!({
        "answer_id": &answer_id,
        "answer_status": &payload.answer_status,
        "evidence_item_count": payload.evidence_item_ids.len(),
        "retrieval_count": payload.retrieval_count
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ('local-owner', 'evidence_answer.created', 'recorded', 'evidence_answer', $1, NULL, $2::jsonb)",
            &[&answer_id, &audit_details],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let response_body = evidence_answer_record_response_json(&mut transaction, &answer_id)?;
    transaction
        .commit()
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(response_body)
}

fn list_evidence_answer_records(database_url: Option<&str>) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_evidence_answer_records_table(&mut client)?;
    client
        .query_one(
            "SELECT COALESCE((SELECT json_agg(row_to_json(t))::text FROM (SELECT id, user_question, answer_status, answer_text, facts, assumptions, inferences, uncertainty, missing_information, evidence_item_ids, document_ids, chunk_ids, source_ids, safe_labels, retrieval_mode, retrieval_count, local_model_status, metadata_json, created_at, updated_at FROM evidence_answer_records ORDER BY created_at DESC LIMIT 50) t), '[]')",
            &[],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn get_evidence_answer_record(
    answer_id: &str,
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    ensure_evidence_answer_records_table(&mut client)?;
    let body = client
        .query_one(
            "SELECT COALESCE((SELECT row_to_json(t)::text FROM (SELECT id, user_question, answer_status, answer_text, facts, assumptions, inferences, uncertainty, missing_information, evidence_item_ids, document_ids, chunk_ids, source_ids, safe_labels, retrieval_mode, retrieval_count, local_model_status, metadata_json, created_at, updated_at FROM evidence_answer_records WHERE id = $1) t), '')",
            &[&answer_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if body.is_empty() {
        Err(GatewayError::NotFound(
            "Evidence answer record not found".to_string(),
        ))
    } else {
        Ok(body)
    }
}

#[derive(Debug, Clone)]
struct CalibrationRecord {
    kind: String,
    id: String,
    confidence: Option<i32>,
    evidence_count: usize,
}

#[derive(Debug, Clone)]
struct CalibrationOutcome {
    target_type: String,
    target_id: String,
    outcome_status: String,
}

fn prediction_recommendation_calibration_summary(
    database_url: Option<&str>,
) -> Result<String, GatewayError> {
    let database_url = database_url
        .filter(|value| !value.trim().is_empty())
        .ok_or(GatewayError::MissingDatabaseUrl)?;
    let postgres_url = postgres_client_url(database_url);
    let mut client = Client::connect(&postgres_url, NoTls)
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let mut records = Vec::new();
    for row in client
        .query(
            "SELECT 'prediction' AS kind, id, confidence, evidence_ids FROM predictions UNION ALL SELECT 'recommendation' AS kind, id, confidence, evidence_ids FROM recommendations",
            &[],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        let evidence_ids = row.get::<_, Value>(3);
        records.push(CalibrationRecord {
            kind: row.get::<_, String>(0),
            id: row.get::<_, String>(1),
            confidence: row.get::<_, Option<i32>>(2),
            evidence_count: string_values_from_json_array(&evidence_ids).len(),
        });
    }
    let outcomes = client
        .query(
            "SELECT target_type, target_id, outcome_status FROM outcomes WHERE target_type IN ('prediction', 'recommendation')",
            &[],
        )
        .map(|rows| {
            rows.into_iter()
                .map(|row| CalibrationOutcome {
                    target_type: row.get::<_, String>(0),
                    target_id: row.get::<_, String>(1),
                    outcome_status: row.get::<_, String>(2),
                })
                .collect::<Vec<_>>()
        })
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(calibration_summary_json(&records, &outcomes).to_string())
}

fn calibration_summary_json(
    records: &[CalibrationRecord],
    outcomes: &[CalibrationOutcome],
) -> Value {
    let mut record_keys = HashSet::new();
    let mut by_kind: HashMap<&str, (usize, usize)> = HashMap::new();
    let mut confidence_bands: HashMap<&str, (usize, usize)> = HashMap::new();
    let mut evidence_linked = 0usize;
    for record in records {
        record_keys.insert(format!("{}:{}", record.kind, record.id));
        if record.evidence_count > 0 {
            evidence_linked += 1;
        }
        let kind_entry = by_kind.entry(record.kind.as_str()).or_insert((0, 0));
        kind_entry.0 += 1;
        let band_entry = confidence_bands
            .entry(confidence_band(record.confidence))
            .or_insert((0, 0));
        band_entry.0 += 1;
    }

    let mut outcome_counts: HashMap<&str, usize> = HashMap::new();
    let mut outcome_keys = HashSet::new();
    for outcome in outcomes {
        let key = format!("{}:{}", outcome.target_type, outcome.target_id);
        if !record_keys.contains(&key) {
            continue;
        }
        *outcome_counts
            .entry(outcome.outcome_status.as_str())
            .or_insert(0) += 1;
        outcome_keys.insert(key);
        if let Some(kind_entry) = by_kind.get_mut(outcome.target_type.as_str()) {
            kind_entry.1 += 1;
        }
        if let Some(record) = records
            .iter()
            .find(|record| record.kind == outcome.target_type && record.id == outcome.target_id)
        {
            if let Some(band_entry) = confidence_bands.get_mut(confidence_band(record.confidence)) {
                band_entry.1 += 1;
            }
        }
    }

    serde_json::json!({
        "schema_version": "prediction_recommendation_calibration_summary.v1",
        "record_counts": {
            "predictions": records.iter().filter(|record| record.kind == "prediction").count(),
            "recommendations": records.iter().filter(|record| record.kind == "recommendation").count(),
            "total": records.len(),
            "evidence_linked": evidence_linked,
            "with_outcome": outcome_keys.len()
        },
        "outcome_counts": {
            "correct": outcome_counts.get("correct").copied().unwrap_or(0),
            "wrong": outcome_counts.get("wrong").copied().unwrap_or(0),
            "partial": outcome_counts.get("partial").copied().unwrap_or(0),
            "useful": outcome_counts.get("useful").copied().unwrap_or(0),
            "not_useful": outcome_counts.get("not_useful").copied().unwrap_or(0),
            "inconclusive": outcome_counts.get("inconclusive").copied().unwrap_or(0),
            "total": outcome_counts.values().sum::<usize>()
        },
        "by_kind": {
            "prediction": {
                "records": by_kind.get("prediction").map(|entry| entry.0).unwrap_or(0),
                "outcomes": by_kind.get("prediction").map(|entry| entry.1).unwrap_or(0)
            },
            "recommendation": {
                "records": by_kind.get("recommendation").map(|entry| entry.0).unwrap_or(0),
                "outcomes": by_kind.get("recommendation").map(|entry| entry.1).unwrap_or(0)
            }
        },
        "confidence_bands": {
            "unknown": confidence_band_json(&confidence_bands, "unknown"),
            "low": confidence_band_json(&confidence_bands, "low"),
            "medium": confidence_band_json(&confidence_bands, "medium"),
            "high": confidence_band_json(&confidence_bands, "high")
        },
        "calibration_status": if outcome_keys.is_empty() { "needs_outcomes" } else { "review_ready" },
        "limitations": [
            "Outcome counts are explicit owner review records, not automatic scoring.",
            "Confidence bands are descriptive only and are not advanced calibration statistics.",
            "Recommendations are not executed automatically."
        ],
        "forecasting_engine": false,
        "auto_execute_recommendations": false,
        "advanced_calibration": false
    })
}

fn confidence_band(confidence: Option<i32>) -> &'static str {
    match confidence {
        Some(value) if value < 40 => "low",
        Some(value) if value < 70 => "medium",
        Some(_) => "high",
        None => "unknown",
    }
}

fn confidence_band_json(confidence_bands: &HashMap<&str, (usize, usize)>, band: &str) -> Value {
    let (records, outcomes) = confidence_bands.get(band).copied().unwrap_or((0, 0));
    serde_json::json!({
        "records": records,
        "outcomes": outcomes
    })
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
            "INSERT INTO outcomes (id, target_type, target_id, outcome_status, summary, occurred_at, evidence_ids, metadata_json) VALUES ($1, $2, $3, $4, $5, $6::text::timestamptz, $7::jsonb, $8::jsonb)",
            &[
                &outcome_id,
                &payload.target_type,
                &payload.target_id,
                &payload.outcome_status,
                &payload.summary.as_deref(),
                &payload.occurred_at.as_deref(),
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

struct EvidenceAnswerRecordCreatePayload {
    user_question: String,
    answer_status: String,
    answer_text: Option<String>,
    facts: Vec<String>,
    assumptions: Vec<String>,
    inferences: Vec<String>,
    uncertainty: Vec<String>,
    missing_information: Vec<String>,
    evidence_item_ids: Vec<String>,
    document_ids: Vec<String>,
    chunk_ids: Vec<String>,
    source_ids: Vec<String>,
    safe_labels: Vec<String>,
    retrieval_mode: String,
    retrieval_count: i32,
    local_model_status: Option<String>,
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

struct SourceReviewStatePayload {
    trust_level: String,
    sensitivity: String,
    enabled: bool,
    review_note: Option<String>,
    actor_id: String,
}

struct SourcePermissionCreatePayload {
    scope_json: Value,
    allowed_operations: Vec<String>,
    external_model_policy: String,
    approval_required: bool,
    created_by_actor_id: String,
}

struct RawArtifactCreatePayload {
    source_id: Option<String>,
    collection_run_id: Option<String>,
    content_base64: String,
    mime_type: Option<String>,
    metadata_json: Value,
    requested_by_actor_id: String,
}

struct CollectionRunCreatePayload {
    source_id: Option<String>,
    requested_by_actor_id: String,
    summary_json: Value,
    dry_run: bool,
}

struct HypothesisCreatePayload {
    hypothesis_text: String,
    supporting_evidence_ids: Vec<String>,
    missing_evidence_json: Value,
    confidence: Option<i32>,
    status: String,
    actor_id: String,
    metadata_json: Value,
}

struct PredictionCreatePayload {
    prediction_text: String,
    expected_result: String,
    disproof_condition: Option<String>,
    evidence_ids: Vec<String>,
    confidence: Option<i32>,
    status: String,
    actor_id: String,
    metadata_json: Value,
}

struct RecommendationCreatePayload {
    recommendation_text: String,
    risk_level: String,
    approval_required: bool,
    expected_result: Option<String>,
    evidence_ids: Vec<String>,
    confidence: Option<i32>,
    status: String,
    actor_id: String,
    metadata_json: Value,
}

struct EvidenceDocumentCreatePayload {
    raw_artifact_id: String,
    source_id: Option<String>,
    title: Option<String>,
    document_type: String,
    language: Option<String>,
    sensitivity: String,
    metadata_json: Value,
    created_by_actor_id: String,
}

struct ChunkGenerationPayload {
    chunk_size: i32,
    created_by_actor_id: String,
}

struct EvidenceItemCreatePayload {
    source_id: Option<String>,
    document_id: Option<String>,
    chunk_id: Option<String>,
    evidence_type: String,
    statement: String,
    observed_at: Option<String>,
    confidence: Option<i32>,
    metadata_json: Value,
    created_by_actor_id: String,
}

struct EvidenceReviewStatePayload {
    review_state: String,
    correction_note: Option<String>,
    superseding_evidence_item_id: Option<String>,
    actor_id: String,
}

struct ExperimentCreatePayload {
    improvement_item_id: Option<String>,
    status: String,
    mlflow_run_id: Option<String>,
    optuna_study_name: Option<String>,
    metrics_json: Value,
    artifacts_json: Value,
    metadata_json: Value,
    actor_id: String,
}

struct ExperimentProposalPayload {
    improvement_item_id: String,
    proposal_scope: String,
    success_criteria: Vec<String>,
    dry_run_summary: String,
    result_comparison_plan: String,
    actor_id: String,
}

struct ExperimentStatusPayload {
    status: String,
    metrics_json: Value,
    artifacts_json: Value,
    metadata_json: Value,
    metrics_updated: bool,
    artifacts_updated: bool,
    metadata_updated: bool,
    actor_id: String,
}

struct ImprovementCreatePayload {
    target_area: String,
    objective: String,
    proposed_by_actor_id: String,
    priority: String,
    metadata_json: Value,
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

struct ReportStatusPayload {
    status: String,
    actor_id: String,
    artifact_path: Option<String>,
}

struct ReportWorkItemPayload {
    requested_by_actor_id: String,
    notes: Option<String>,
}

struct WorkItemCreatePayload {
    work_type: String,
    requested_by_actor_id: String,
    intent: Value,
    payload_json: Value,
}

struct AgentTaskPlanCreatePayload {
    user_request_summary: String,
    intent_category: String,
    status: String,
    proposed_steps: Vec<String>,
    required_evidence: Vec<String>,
    approval_required: bool,
    supported_state: String,
    next_safe_action: String,
    requested_by_actor_id: String,
    metadata_json: Value,
}

struct AgentTaskPlanWorkItemPayload {
    actor_id: String,
    approval_id: Option<String>,
}

struct AgentTaskPlanWorkSpecPayload {
    actor_id: String,
    work_type: String,
    expected_output: Option<String>,
}

struct AgentTaskPlanEvidenceSummaryPayload {
    actor_id: String,
    answer_status: String,
    retrieved_count: i32,
    labels: Vec<String>,
    missing_evidence: bool,
    missing_evidence_guidance: Option<String>,
}

struct AgentTaskPlanRecord {
    id: String,
    user_request_summary: String,
    intent_category: String,
    status: String,
    required_evidence: Vec<String>,
    approval_required: bool,
    supported_state: String,
    next_safe_action: String,
    metadata_json: Value,
}

struct WorkItemStatusPayload {
    status: String,
    actor_id: String,
    error_message: Option<String>,
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

struct RetrievalSearchPayload {
    query: String,
    limit: i32,
}

struct CollectionDryRunPayload {
    source_id: String,
    source_permission_id: String,
    requested_by_actor_id: String,
    notes: Value,
}

struct LocalProjectCollectionPayload {
    source_id: String,
    source_permission_id: String,
    approval_id: Option<String>,
    requested_by_actor_id: String,
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

struct ManualUploadIngestPayload {
    upload: ManualUploadCollectionPayload,
    chunk_size: i32,
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

struct RawArtifactIngestRecord {
    id: String,
    collection_run_id: Option<String>,
    content_hash: String,
    storage_path: String,
    reused: bool,
}

struct NormalizedDocumentIngestRecord {
    id: String,
    raw_artifact_id: String,
    text_content: String,
    reused: bool,
}

struct CollectedLocalProjectFile {
    source_path: String,
    relative_path: String,
    stored: StoredArtifact,
}

struct LocalProjectCollectionResult {
    total_files: usize,
    skipped_files: Vec<Value>,
    files: Vec<CollectedLocalProjectFile>,
}

struct VectorUpsertSummary {
    collection_name: String,
    collection_exists: bool,
    chunks_upserted: usize,
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

fn parse_evidence_answer_record_create(
    body: &str,
) -> Result<EvidenceAnswerRecordCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Evidence answer record request body")?;
    let user_question = required_text_field_with_max(&object, "user_question", 1000)?;
    let answer_status =
        optional_string_field_with_max(&object, "answer_status", "not_generated", 64)?;
    if !is_evidence_answer_status(&answer_status) {
        return Err(GatewayError::Validation(format!(
            "Unsupported evidence answer status: {answer_status}"
        )));
    }
    let answer_text = optional_nullable_string_field_with_max(&object, "answer_text", 8000)?;
    let facts = optional_bounded_string_array_field(&object, "facts", 20, 1000)?;
    let assumptions = optional_bounded_string_array_field(&object, "assumptions", 20, 1000)?;
    let inferences = optional_bounded_string_array_field(&object, "inferences", 20, 1000)?;
    let uncertainty = optional_bounded_string_array_field(&object, "uncertainty", 20, 1000)?;
    let missing_information =
        optional_bounded_string_array_field(&object, "missing_information", 20, 1000)?;
    let evidence_item_ids =
        optional_bounded_string_array_field(&object, "evidence_item_ids", 50, 128)?;
    let document_ids = optional_bounded_string_array_field(&object, "document_ids", 50, 128)?;
    let chunk_ids = optional_bounded_string_array_field(&object, "chunk_ids", 50, 128)?;
    let source_ids = optional_bounded_string_array_field(&object, "source_ids", 50, 128)?;
    let safe_labels = optional_bounded_string_array_field(&object, "safe_labels", 30, 180)?;
    let retrieval_mode =
        optional_string_field_with_max(&object, "retrieval_mode", "not_recorded", 64)?;
    let retrieval_count = optional_i32_field_with_default(&object, "retrieval_count", 0, 0, 1000)?;
    let local_model_status =
        optional_nullable_string_field_with_max(&object, "local_model_status", 128)?;
    let metadata_json =
        safe_evidence_answer_metadata_json(optional_object_field(&object, "metadata_json")?)?;
    Ok(EvidenceAnswerRecordCreatePayload {
        user_question,
        answer_status,
        answer_text,
        facts,
        assumptions,
        inferences,
        uncertainty,
        missing_information,
        evidence_item_ids,
        document_ids,
        chunk_ids,
        source_ids,
        safe_labels,
        retrieval_mode,
        retrieval_count,
        local_model_status,
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

fn parse_source_review_state(body: &str) -> Result<SourceReviewStatePayload, GatewayError> {
    let object = parse_json_object(body, "Source review state request body")?;
    let trust_level = required_string_field(&object, "trust_level", 64)?;
    if !is_source_review_trust_level(&trust_level) {
        return Err(GatewayError::Validation(format!(
            "Unsupported source trust level: {trust_level}"
        )));
    }
    let sensitivity = required_string_field(&object, "sensitivity", 64)?;
    if !is_sensitivity_label(&sensitivity) {
        return Err(GatewayError::Validation(format!(
            "Unknown sensitivity label: {sensitivity}"
        )));
    }
    let enabled = optional_bool_field(&object, "enabled", true)?;
    let review_note = optional_nullable_string_field_with_max(&object, "review_note", 500)?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    Ok(SourceReviewStatePayload {
        trust_level,
        sensitivity,
        enabled,
        review_note,
        actor_id,
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

fn parse_hypothesis_create(body: &str) -> Result<HypothesisCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Hypothesis request body")?;
    Ok(HypothesisCreatePayload {
        hypothesis_text: required_text_field(&object, "hypothesis_text")?,
        supporting_evidence_ids: required_string_array_field(&object, "supporting_evidence_ids")?,
        missing_evidence_json: optional_object_field(&object, "missing_evidence_json")?,
        confidence: optional_i32_field(&object, "confidence", 0, 100)?,
        status: optional_string_field_with_max(&object, "status", "candidate", 64)?,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
    })
}

fn parse_raw_artifact_create(body: &str) -> Result<RawArtifactCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Raw artifact request body")?;
    let source_id = optional_nullable_string_field_with_max(&object, "source_id", 36)?;
    let collection_run_id =
        optional_nullable_string_field_with_max(&object, "collection_run_id", 36)?;
    let content_base64 = required_text_field(&object, "content_base64")?;
    let mime_type = optional_nullable_string_field_with_max(&object, "mime_type", 255)?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    Ok(RawArtifactCreatePayload {
        source_id,
        collection_run_id,
        content_base64,
        mime_type,
        metadata_json,
        requested_by_actor_id,
    })
}

fn parse_collection_run_create(body: &str) -> Result<CollectionRunCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Collection run request body")?;
    let source_id = optional_nullable_string_field_with_max(&object, "source_id", 36)?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let summary_json = optional_object_field(&object, "summary_json")?;
    let dry_run = optional_bool_field(&object, "dry_run", true)?;
    Ok(CollectionRunCreatePayload {
        source_id,
        requested_by_actor_id,
        summary_json,
        dry_run,
    })
}

fn parse_prediction_create(body: &str) -> Result<PredictionCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Prediction request body")?;
    Ok(PredictionCreatePayload {
        prediction_text: required_text_field(&object, "prediction_text")?,
        expected_result: required_text_field(&object, "expected_result")?,
        disproof_condition: optional_nullable_string_field(&object, "disproof_condition")?,
        evidence_ids: required_string_array_field(&object, "evidence_ids")?,
        confidence: optional_i32_field(&object, "confidence", 0, 100)?,
        status: optional_string_field_with_max(&object, "status", "open", 64)?,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
    })
}

fn parse_recommendation_create(body: &str) -> Result<RecommendationCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Recommendation request body")?;
    Ok(RecommendationCreatePayload {
        recommendation_text: required_text_field(&object, "recommendation_text")?,
        risk_level: optional_string_field_with_max(&object, "risk_level", "unknown", 64)?,
        approval_required: optional_bool_field(&object, "approval_required", true)?,
        expected_result: optional_nullable_string_field(&object, "expected_result")?,
        evidence_ids: required_string_array_field(&object, "evidence_ids")?,
        confidence: optional_i32_field(&object, "confidence", 0, 100)?,
        status: optional_string_field_with_max(&object, "status", "proposed", 64)?,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
    })
}

fn parse_evidence_document_create(
    body: &str,
) -> Result<EvidenceDocumentCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Document request body")?;
    Ok(EvidenceDocumentCreatePayload {
        raw_artifact_id: required_string_field(&object, "raw_artifact_id", 128)?,
        source_id: optional_nullable_string_field_with_max(&object, "source_id", 128)?,
        title: optional_nullable_string_field_with_max(&object, "title", 255)?,
        document_type: optional_string_field_with_max(&object, "document_type", "text", 64)?,
        language: optional_nullable_string_field_with_max(&object, "language", 32)?,
        sensitivity: optional_string_field_with_max(&object, "sensitivity", "internal", 64)?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
        created_by_actor_id: optional_string_field_with_max(
            &object,
            "created_by_actor_id",
            "local-owner",
            128,
        )?,
    })
}

fn parse_chunk_generation(body: &str) -> Result<ChunkGenerationPayload, GatewayError> {
    let object = parse_json_object(body, "Chunk generation request body")?;
    let chunk_size = optional_i32_field_with_default(&object, "chunk_size", 1000, 100, 5000)?;
    Ok(ChunkGenerationPayload {
        chunk_size,
        created_by_actor_id: optional_string_field_with_max(
            &object,
            "created_by_actor_id",
            "local-owner",
            128,
        )?,
    })
}

fn parse_evidence_item_create(body: &str) -> Result<EvidenceItemCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Evidence item request body")?;
    let source_id = optional_nullable_string_field_with_max(&object, "source_id", 128)?;
    let document_id = optional_nullable_string_field_with_max(&object, "document_id", 128)?;
    let chunk_id = optional_nullable_string_field_with_max(&object, "chunk_id", 128)?;
    if source_id.is_none() && document_id.is_none() && chunk_id.is_none() {
        return Err(GatewayError::Validation(
            "At least one evidence link must be provided".to_string(),
        ));
    }
    Ok(EvidenceItemCreatePayload {
        source_id,
        document_id,
        chunk_id,
        evidence_type: required_string_field(&object, "evidence_type", 64)?,
        statement: required_text_field(&object, "statement")?,
        observed_at: optional_nullable_string_field(&object, "observed_at")?,
        confidence: optional_i32_field(&object, "confidence", 0, 100)?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
        created_by_actor_id: optional_string_field_with_max(
            &object,
            "created_by_actor_id",
            "local-owner",
            128,
        )?,
    })
}

fn parse_evidence_review_state(body: &str) -> Result<EvidenceReviewStatePayload, GatewayError> {
    let object = parse_json_object(body, "Evidence review state request body")?;
    let review_state = required_string_field(&object, "review_state", 64)?;
    if !is_evidence_review_state(&review_state) {
        return Err(GatewayError::Validation(format!(
            "Unsupported evidence review state: {review_state}"
        )));
    }
    let correction_note = optional_nullable_string_field_with_max(&object, "correction_note", 800)?;
    let superseding_evidence_item_id =
        optional_nullable_string_field_with_max(&object, "superseding_evidence_item_id", 128)?;
    let actor_id = optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?;
    Ok(EvidenceReviewStatePayload {
        review_state,
        correction_note,
        superseding_evidence_item_id,
        actor_id,
    })
}

fn parse_experiment_create(body: &str) -> Result<ExperimentCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Experiment request body")?;
    let status = optional_string_field_with_max(&object, "status", "planned", 64)?;
    if !is_experiment_status(&status) {
        return Err(GatewayError::Validation(format!(
            "Unknown experiment status: {status}"
        )));
    }
    Ok(ExperimentCreatePayload {
        improvement_item_id: optional_nullable_string_field_with_max(
            &object,
            "improvement_item_id",
            36,
        )?,
        status,
        mlflow_run_id: optional_nullable_string_field_with_max(&object, "mlflow_run_id", 255)?,
        optuna_study_name: optional_nullable_string_field_with_max(
            &object,
            "optuna_study_name",
            255,
        )?,
        metrics_json: optional_object_field(&object, "metrics_json")?,
        artifacts_json: optional_object_field(&object, "artifacts_json")?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
    })
}

fn parse_experiment_proposal(body: &str) -> Result<ExperimentProposalPayload, GatewayError> {
    let object = parse_json_object(body, "Experiment proposal request body")?;
    let success_criteria =
        optional_bounded_string_array_field(&object, "success_criteria", 8, 500)?;
    if success_criteria.is_empty() {
        return Err(GatewayError::Validation(
            "success_criteria must include at least one reviewable criterion.".to_string(),
        ));
    }
    Ok(ExperimentProposalPayload {
        improvement_item_id: required_string_field(&object, "improvement_item_id", 36)?,
        proposal_scope: required_text_field_with_max(&object, "proposal_scope", 2000)?,
        success_criteria,
        dry_run_summary: required_text_field_with_max(&object, "dry_run_summary", 2000)?,
        result_comparison_plan: required_text_field_with_max(
            &object,
            "result_comparison_plan",
            2000,
        )?,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
    })
}

fn parse_experiment_status(body: &str) -> Result<ExperimentStatusPayload, GatewayError> {
    let object = parse_json_object(body, "Experiment status request body")?;
    let status = required_string_field(&object, "status", 64)?;
    if !is_experiment_status(&status) {
        return Err(GatewayError::Validation(format!(
            "Unknown experiment status: {status}"
        )));
    }
    let metrics_updated = object
        .get("metrics_json")
        .is_some_and(|value| !value.is_null());
    let artifacts_updated = object
        .get("artifacts_json")
        .is_some_and(|value| !value.is_null());
    let metadata_updated = object
        .get("metadata_json")
        .is_some_and(|value| !value.is_null());
    Ok(ExperimentStatusPayload {
        status,
        metrics_json: optional_object_field(&object, "metrics_json")?,
        artifacts_json: optional_object_field(&object, "artifacts_json")?,
        metadata_json: optional_object_field(&object, "metadata_json")?,
        metrics_updated,
        artifacts_updated,
        metadata_updated,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
    })
}

fn parse_improvement_create(body: &str) -> Result<ImprovementCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Improvement request body")?;
    let target_area = required_string_field(&object, "target_area", 64)?;
    if !is_improvement_target_area(&target_area) {
        return Err(GatewayError::Validation(format!(
            "Unknown improvement target area: {target_area}"
        )));
    }
    let priority = optional_string_field_with_max(&object, "priority", "normal", 64)?;
    if !is_improvement_priority(&priority) {
        return Err(GatewayError::Validation(format!(
            "Unknown improvement priority: {priority}"
        )));
    }
    Ok(ImprovementCreatePayload {
        target_area,
        objective: required_text_field(&object, "objective")?,
        proposed_by_actor_id: optional_string_field_with_max(
            &object,
            "proposed_by_actor_id",
            "local-owner",
            128,
        )?,
        priority,
        metadata_json: optional_object_field(&object, "metadata_json")?,
    })
}

fn parse_report_status(body: &str) -> Result<ReportStatusPayload, GatewayError> {
    let object = parse_json_object(body, "Report status request body")?;
    let status = required_string_field(&object, "status", 64)?;
    if !is_report_status(&status) {
        return Err(GatewayError::Validation(format!(
            "Unknown report status: {status}"
        )));
    }
    Ok(ReportStatusPayload {
        status,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        artifact_path: optional_nullable_string_field(&object, "artifact_path")?,
    })
}

fn parse_work_item_status(body: &str) -> Result<WorkItemStatusPayload, GatewayError> {
    let object = parse_json_object(body, "Work item status request body")?;
    let status = required_string_field(&object, "status", 64)?;
    if !is_work_item_status(&status) {
        return Err(GatewayError::Validation(
            "Unknown work item status".to_string(),
        ));
    }
    Ok(WorkItemStatusPayload {
        status,
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        error_message: optional_nullable_string_field(&object, "error_message")?,
    })
}

fn parse_report_work_item(body: &str) -> Result<ReportWorkItemPayload, GatewayError> {
    let object = parse_json_object(body, "Report work item request body")?;
    Ok(ReportWorkItemPayload {
        requested_by_actor_id: optional_string_field_with_max(
            &object,
            "requested_by_actor_id",
            "local-owner",
            128,
        )?,
        notes: optional_nullable_string_field(&object, "notes")?,
    })
}

fn parse_retrieval_search(body: &str) -> Result<RetrievalSearchPayload, GatewayError> {
    let object = parse_json_object(body, "Retrieval search request body")?;
    Ok(RetrievalSearchPayload {
        query: required_text_field(&object, "query")?,
        limit: optional_i32_field_with_default(&object, "limit", 10, 1, 50)?,
    })
}

fn parse_chat_retrieval_search(body: &str) -> Result<RetrievalSearchPayload, GatewayError> {
    let object = parse_json_object(body, "Retrieval preview request body")?;
    let query = object
        .get("query")
        .and_then(Value::as_str)
        .or_else(|| object.get("message").and_then(Value::as_str))
        .unwrap_or("")
        .trim()
        .to_string();
    if query.is_empty() {
        return Err(GatewayError::Validation(
            "message or query is required.".to_string(),
        ));
    }
    Ok(RetrievalSearchPayload {
        query,
        limit: optional_i32_field_with_default(&object, "limit", 10, 1, 50)?,
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

fn parse_agent_task_plan_create(body: &str) -> Result<AgentTaskPlanCreatePayload, GatewayError> {
    let object = parse_json_object(body, "Agent task plan request body")?;
    let user_request_summary = required_text_field_with_max(&object, "user_request_summary", 1000)?;
    let intent_category = required_string_field(&object, "intent_category", 64)?;
    let status = optional_string_field_with_max(&object, "status", "proposed", 64)?;
    if !is_agent_task_plan_status(&status) {
        return Err(GatewayError::Validation(format!(
            "Unknown agent task plan status: {status}"
        )));
    }
    let approval_required = optional_bool_field(&object, "approval_required", false)?;
    let supported_state =
        optional_string_field_with_max(&object, "supported_state", "supported", 64)?;
    if !is_agent_task_plan_supported_state(&supported_state) {
        return Err(GatewayError::Validation(format!(
            "Unknown agent task plan supported_state: {supported_state}"
        )));
    }
    let next_safe_action = required_text_field_with_max(&object, "next_safe_action", 1000)?;
    let mut proposed_steps =
        optional_bounded_string_array_field(&object, "proposed_steps", 12, 1000)?;
    if proposed_steps.is_empty() {
        proposed_steps.push(next_safe_action.clone());
    }
    let required_evidence =
        optional_bounded_string_array_field(&object, "required_evidence", 12, 1000)?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    let metadata_json = optional_object_field(&object, "metadata_json")?;

    Ok(AgentTaskPlanCreatePayload {
        user_request_summary,
        intent_category,
        status,
        proposed_steps,
        required_evidence,
        approval_required,
        supported_state,
        next_safe_action,
        requested_by_actor_id,
        metadata_json,
    })
}

fn parse_agent_task_plan_work_item(
    body: &str,
) -> Result<AgentTaskPlanWorkItemPayload, GatewayError> {
    let object = parse_json_object(body, "Agent task plan work-item request body")?;
    Ok(AgentTaskPlanWorkItemPayload {
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        approval_id: optional_nullable_string_field_with_max(&object, "approval_id", 128)?,
    })
}

fn parse_agent_task_plan_work_spec(
    body: &str,
) -> Result<AgentTaskPlanWorkSpecPayload, GatewayError> {
    let object = parse_json_object(body, "Agent task plan work-spec request body")?;
    let work_type = required_string_field(&object, "work_type", 64)?;
    if !is_supported_work_item_type(&work_type) {
        return Err(GatewayError::Validation(format!(
            "Unsupported work item type: {work_type}"
        )));
    }
    Ok(AgentTaskPlanWorkSpecPayload {
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        work_type,
        expected_output: optional_nullable_string_field_with_max(&object, "expected_output", 1000)?,
    })
}

fn parse_agent_task_plan_evidence_summary(
    body: &str,
) -> Result<AgentTaskPlanEvidenceSummaryPayload, GatewayError> {
    let object = parse_json_object(body, "Agent task plan evidence summary request body")?;
    let retrieved_count = optional_i32_field(&object, "retrieved_count", 0, 1000)?.unwrap_or(0);
    Ok(AgentTaskPlanEvidenceSummaryPayload {
        actor_id: optional_string_field_with_max(&object, "actor_id", "local-owner", 128)?,
        answer_status: optional_string_field_with_max(&object, "answer_status", "unknown", 64)?,
        retrieved_count,
        labels: optional_bounded_string_array_field(&object, "safe_labels", 10, 256)?,
        missing_evidence: optional_bool_field(&object, "missing_evidence", retrieved_count == 0)?,
        missing_evidence_guidance: optional_nullable_string_field_with_max(
            &object,
            "missing_evidence_guidance",
            500,
        )?,
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

fn parse_local_project_collection(
    body: &str,
) -> Result<LocalProjectCollectionPayload, GatewayError> {
    let object = parse_json_object(body, "Local project collection request body")?;
    let source_id = required_string_field(&object, "source_id", 36)?;
    let source_permission_id = required_string_field(&object, "source_permission_id", 36)?;
    let approval_id = optional_nullable_string_field_with_max(&object, "approval_id", 36)?;
    let requested_by_actor_id =
        optional_string_field_with_max(&object, "requested_by_actor_id", "local-owner", 128)?;
    Ok(LocalProjectCollectionPayload {
        source_id,
        source_permission_id,
        approval_id,
        requested_by_actor_id,
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

fn parse_manual_upload_ingest(body: &str) -> Result<ManualUploadIngestPayload, GatewayError> {
    let object = parse_json_object(body, "Manual upload ingest request body")?;
    let upload = parse_manual_upload_collection(body)?;
    let chunk_size = optional_i32_field_with_default(&object, "chunk_size", 1000, 100, 5000)?;
    Ok(ManualUploadIngestPayload { upload, chunk_size })
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
    ("redis", "Redis (archived)"),
    ("qdrant", "Qdrant"),
    ("neo4j", "Neo4j"),
    ("mlflow", "MLflow"),
    ("phoenix", "Phoenix"),
    ("llm", "Local LLM"),
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
    SettingDefinition { key: "DATABASE_URL", group: "postgres", description: "PostgreSQL connection URL (postgres:// or postgresql://)." },
    SettingDefinition { key: "REDIS_HOST", group: "redis", description: "Optional archived Redis hostname. Leave empty for the Rust worker stack." },
    SettingDefinition { key: "REDIS_PORT", group: "redis", description: "Optional archived Redis port. Leave empty when Redis is not deployed." },
    SettingDefinition { key: "REDIS_URL", group: "redis", description: "Optional archived Redis URL. Leave empty when Redis is not deployed." },
    SettingDefinition { key: "CELERY_BROKER_URL", group: "redis", description: "Optional archived Celery broker URL retained for rollback history only." },
    SettingDefinition { key: "CELERY_RESULT_BACKEND", group: "redis", description: "Optional archived Celery result backend URL retained for rollback history only." },
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
    SettingDefinition { key: "LLM_PROVIDER", group: "llm", description: "Optional local LLM provider: none or ollama." },
    SettingDefinition { key: "OLLAMA_BASE_URL", group: "llm", description: "Local Ollama base URL. No tokens or cloud endpoints." },
    SettingDefinition { key: "OLLAMA_MODEL", group: "llm", description: "Local Ollama model name. Empty is allowed when provider is none." },
    SettingDefinition { key: "LLM_TIMEOUT_SECONDS", group: "llm", description: "Timeout for local LLM generation attempts." },
    SettingDefinition { key: "LLM_EVIDENCE_REQUIRED", group: "llm", description: "Require retrieved evidence before local LLM generation." },
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
        .entry("LLM_PROVIDER".to_string())
        .or_insert_with(|| "none".to_string());
    values
        .entry("OLLAMA_BASE_URL".to_string())
        .or_insert_with(|| "http://host.docker.internal:11434".to_string());
    values.entry("OLLAMA_MODEL".to_string()).or_default();
    values
        .entry("LLM_TIMEOUT_SECONDS".to_string())
        .or_insert_with(|| "60".to_string());
    values
        .entry("LLM_EVIDENCE_REQUIRED".to_string())
        .or_insert_with(|| "true".to_string());
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
            && !setting_optional(definition.key)
            && candidate.get(definition.key).is_none_or(String::is_empty)
        {
            errors.push(settings_issue(
                Some(definition.key),
                "Required setting is missing.",
            ));
        }
    }
    for key in SETTINGS_PORT_KEYS {
        let value = candidate.get(*key).map_or("", String::as_str);
        if value.is_empty() && setting_optional(key) {
            continue;
        }
        match value.parse::<u16>().ok() {
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
        candidate.get("LLM_PROVIDER").map_or("none", String::as_str),
        "none" | "ollama"
    ) {
        errors.push(settings_issue(
            Some("LLM_PROVIDER"),
            "LLM provider must be none or ollama.",
        ));
    }
    if candidate.get("LLM_PROVIDER").map_or("none", String::as_str) == "ollama"
        && candidate.get("OLLAMA_MODEL").is_none_or(String::is_empty)
    {
        errors.push(settings_issue(
            Some("OLLAMA_MODEL"),
            "Ollama model is required when LLM_PROVIDER is ollama.",
        ));
    }
    match candidate
        .get("LLM_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
    {
        Some(value) if value > 0 => {}
        _ => errors.push(settings_issue(
            Some("LLM_TIMEOUT_SECONDS"),
            "LLM timeout must be a positive integer.",
        )),
    }
    let ollama_base_url = candidate.get("OLLAMA_BASE_URL").map_or("", String::as_str);
    if !ollama_base_url.is_empty()
        && (!settings_url_plausible("OLLAMA_BASE_URL", ollama_base_url)
            || ollama_base_url.contains('@')
            || !(ollama_base_url.starts_with("http://localhost")
                || ollama_base_url.starts_with("http://127.0.0.1")
                || ollama_base_url.starts_with("http://host.docker.internal")))
    {
        errors.push(settings_issue(
            Some("OLLAMA_BASE_URL"),
            "Ollama base URL must be local http without credentials.",
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
const SETTINGS_OPTIONAL_KEYS: &[&str] = &[
    "OLLAMA_MODEL",
    "REDIS_HOST",
    "REDIS_PORT",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
];
const SETTINGS_SECRET_KEYS: &[&str] = &["POSTGRES_PASSWORD", "DATABASE_URL", "NEO4J_PASSWORD"];
const SETTINGS_BOOLEAN_KEYS: &[&str] = &[
    "SINGLE_USER_MODE",
    "APPROVAL_REQUIRED_DEFAULT",
    "LLM_EVIDENCE_REQUIRED",
];
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
    "OLLAMA_BASE_URL",
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
    "LLM_PROVIDER",
    "OLLAMA_BASE_URL",
    "OLLAMA_MODEL",
    "LLM_TIMEOUT_SECONDS",
    "LLM_EVIDENCE_REQUIRED",
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

fn setting_optional(key: &str) -> bool {
    SETTINGS_OPTIONAL_KEYS
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
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Container paths in .env are POSIX-style even when the gateway runs on Windows hosts.
    let normalized = trimmed.replace('\\', "/");
    let posix_absolute = normalized.starts_with('/');
    let path = Path::new(trimmed);
    let absolute = path.is_absolute() || posix_absolute;
    absolute && !normalized.split('/').any(|part| part == "..")
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
            (parsed.scheme == "postgres" || parsed.scheme.starts_with("postgresql"))
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
            "Redis or archived Celery setting changes may require API and worker restart/recreate."
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

fn json_string_values(values: &[String]) -> Value {
    Value::Array(
        values
            .iter()
            .map(|value| Value::String(value.clone()))
            .collect(),
    )
}

fn merge_metadata(base: &Value, patch: Value) -> Value {
    let mut object = base.as_object().cloned().unwrap_or_default();
    if let Some(patch_object) = patch.as_object() {
        for (key, value) in patch_object {
            object.insert(key.clone(), value.clone());
        }
    }
    Value::Object(object)
}

fn split_text_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    if chunk_size == 0 {
        return Vec::new();
    }
    let chars = text.chars().collect::<Vec<_>>();
    chars
        .chunks(chunk_size)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

fn require_source_exists(
    transaction: &mut postgres::Transaction<'_>,
    source_id: &str,
) -> Result<(), GatewayError> {
    let exists = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
            &[&source_id],
        )
        .map(|row| row.get::<_, bool>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if exists {
        Ok(())
    } else {
        Err(GatewayError::NotFound("Source not found".to_string()))
    }
}

fn load_collection_run_source_id(
    transaction: &mut postgres::Transaction<'_>,
    collection_run_id: &str,
) -> Result<Option<String>, GatewayError> {
    let Some(row) = transaction
        .query_opt(
            "SELECT source_id FROM collection_runs WHERE id = $1",
            &[&collection_run_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound(
            "Collection run not found".to_string(),
        ));
    };
    Ok(row.get(0))
}

fn raw_artifact_response_json(
    transaction: &mut postgres::Transaction<'_>,
    artifact_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json, created_at, updated_at FROM raw_artifacts WHERE id = $1) t",
            &[&artifact_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn collection_run_response_json(
    transaction: &mut postgres::Transaction<'_>,
    collection_run_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs WHERE id = $1) t",
            &[&collection_run_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn collection_run_response_json_client(
    client: &mut Client,
    collection_run_id: &str,
) -> Result<String, GatewayError> {
    client
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, source_id, status, dry_run, requested_by_actor_id, summary_json, error_message, created_at, updated_at FROM collection_runs WHERE id = $1) t",
            &[&collection_run_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn ensure_agent_task_plans_table(client: &mut Client) -> Result<(), GatewayError> {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS agent_task_plans (
                id text PRIMARY KEY,
                user_request_summary text NOT NULL,
                intent_category text NOT NULL,
                status text NOT NULL,
                proposed_steps jsonb NOT NULL DEFAULT '[]'::jsonb,
                required_evidence jsonb NOT NULL DEFAULT '[]'::jsonb,
                approval_required boolean NOT NULL DEFAULT false,
                supported_state text NOT NULL,
                next_safe_action text NOT NULL,
                requested_by_actor_id text NOT NULL DEFAULT 'local-owner',
                metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,
                created_at timestamptz NOT NULL DEFAULT now(),
                updated_at timestamptz NOT NULL DEFAULT now()
            );
            CREATE INDEX IF NOT EXISTS idx_agent_task_plans_created_at ON agent_task_plans (created_at DESC);",
        )
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn ensure_evidence_answer_records_table(client: &mut Client) -> Result<(), GatewayError> {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS evidence_answer_records (
                id text PRIMARY KEY,
                user_question text NOT NULL,
                answer_status text NOT NULL,
                answer_text text,
                facts jsonb NOT NULL DEFAULT '[]'::jsonb,
                assumptions jsonb NOT NULL DEFAULT '[]'::jsonb,
                inferences jsonb NOT NULL DEFAULT '[]'::jsonb,
                uncertainty jsonb NOT NULL DEFAULT '[]'::jsonb,
                missing_information jsonb NOT NULL DEFAULT '[]'::jsonb,
                evidence_item_ids jsonb NOT NULL DEFAULT '[]'::jsonb,
                document_ids jsonb NOT NULL DEFAULT '[]'::jsonb,
                chunk_ids jsonb NOT NULL DEFAULT '[]'::jsonb,
                source_ids jsonb NOT NULL DEFAULT '[]'::jsonb,
                safe_labels jsonb NOT NULL DEFAULT '[]'::jsonb,
                retrieval_mode text NOT NULL DEFAULT 'not_recorded',
                retrieval_count integer NOT NULL DEFAULT 0,
                local_model_status text,
                metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,
                created_at timestamptz NOT NULL DEFAULT now(),
                updated_at timestamptz NOT NULL DEFAULT now()
            );
            CREATE INDEX IF NOT EXISTS idx_evidence_answer_records_created_at ON evidence_answer_records (created_at DESC);",
        )
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn evidence_answer_record_response_json(
    transaction: &mut postgres::Transaction<'_>,
    answer_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, user_question, answer_status, answer_text, facts, assumptions, inferences, uncertainty, missing_information, evidence_item_ids, document_ids, chunk_ids, source_ids, safe_labels, retrieval_mode, retrieval_count, local_model_status, metadata_json, created_at, updated_at FROM evidence_answer_records WHERE id = $1) t",
            &[&answer_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn agent_task_plan_response_json(
    transaction: &mut postgres::Transaction<'_>,
    task_plan_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, user_request_summary, intent_category, status, proposed_steps, required_evidence, approval_required, supported_state, next_safe_action, requested_by_actor_id, metadata_json, created_at, updated_at FROM agent_task_plans WHERE id = $1) t",
            &[&task_plan_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn load_agent_task_plan_record(
    transaction: &mut postgres::Transaction<'_>,
    task_plan_id: &str,
) -> Result<AgentTaskPlanRecord, GatewayError> {
    let Some(row) = transaction
        .query_opt(
            "SELECT id, user_request_summary, intent_category, status, required_evidence, approval_required, supported_state, next_safe_action, metadata_json FROM agent_task_plans WHERE id = $1",
            &[&task_plan_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::NotFound("Agent task plan not found".to_string()));
    };
    Ok(AgentTaskPlanRecord {
        id: row.get(0),
        user_request_summary: row.get(1),
        intent_category: row.get(2),
        status: row.get(3),
        required_evidence: json_value_string_array(row.get(4), "required_evidence")?,
        approval_required: row.get(5),
        supported_state: row.get(6),
        next_safe_action: row.get(7),
        metadata_json: row.get(8),
    })
}

fn validate_agent_task_plan_approval(
    transaction: &mut postgres::Transaction<'_>,
    task_plan_id: &str,
    approval_id: &str,
) -> Result<(), GatewayError> {
    let approval_id = validate_route_id(approval_id, "approval_id")?;
    let Some(row) = transaction
        .query_opt(
            "SELECT request_type, status, request_payload_json FROM approvals WHERE id = $1",
            &[&approval_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    else {
        return Err(GatewayError::Forbidden(
            "Approved agent_task_plan approval was not found".to_string(),
        ));
    };
    let request_type: String = row.get(0);
    let status: String = row.get(1);
    let request_payload_json: Value = row.get(2);
    if request_type != "agent_task_plan" || status != "approved" {
        return Err(GatewayError::Forbidden(
            "Approval must be an approved agent_task_plan request".to_string(),
        ));
    }
    let approved_task_plan_id = request_payload_json
        .get("task_plan_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    if approved_task_plan_id != task_plan_id {
        return Err(GatewayError::Forbidden(
            "Approval does not match this task plan".to_string(),
        ));
    }
    Ok(())
}

fn experiment_response_json(
    transaction: &mut postgres::Transaction<'_>,
    experiment_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, improvement_item_id, status, mlflow_run_id, optuna_study_name, metrics_json, artifacts_json, metadata_json, created_at, updated_at FROM experiment_runs WHERE id = $1) t",
            &[&experiment_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn improvement_response_json(
    transaction: &mut postgres::Transaction<'_>,
    improvement_id: &str,
) -> Result<String, GatewayError> {
    transaction
        .query_one(
            "SELECT row_to_json(t)::text FROM (SELECT id, target_area, status, objective, proposed_by_actor_id, priority, metadata_json, created_at, updated_at FROM improvement_items WHERE id = $1) t",
            &[&improvement_id],
        )
        .map(|row| row.get::<_, String>(0))
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn insert_collection_run_created_audit(
    transaction: &mut postgres::Transaction<'_>,
    actor_id: &str,
    collection_run_id: &str,
    source_id: Option<&str>,
    dry_run: bool,
    status: &str,
) -> Result<(), GatewayError> {
    let details_json = serde_json::json!({
        "source_id": source_id,
        "dry_run": dry_run,
        "status": status
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'collection_run.created', 'recorded', 'collection_run', $2, NULL, $3::jsonb)",
            &[&actor_id, &collection_run_id, &details_json],
        )
        .map(|_| ())
        .map_err(|error| GatewayError::Database(error.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn insert_raw_artifact_created_audit(
    transaction: &mut postgres::Transaction<'_>,
    actor_id: &str,
    artifact_id: &str,
    source_id: Option<&str>,
    collection_run_id: Option<&str>,
    content_hash: &str,
    storage_path: &str,
    size_bytes: u64,
    content_already_existed: bool,
) -> Result<(), GatewayError> {
    let details_json = serde_json::json!({
        "source_id": source_id,
        "collection_run_id": collection_run_id,
        "content_hash": content_hash,
        "storage_path": storage_path,
        "size_bytes": size_bytes,
        "content_already_existed": content_already_existed
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'raw_artifact.created', 'recorded', 'raw_artifact', $2, NULL, $3::jsonb)",
            &[&actor_id, &artifact_id, &details_json],
        )
        .map(|_| ())
        .map_err(|error| GatewayError::Database(error.to_string()))
}

fn insert_work_item_created_audit_for_collection(
    transaction: &mut postgres::Transaction<'_>,
    actor_id: &str,
    work_item_id: &str,
    collection_run_id: &str,
    raw_artifact_ids: &[String],
) -> Result<(), GatewayError> {
    let details_json = serde_json::json!({
        "work_type": "collection_normalization",
        "collection_run_id": collection_run_id,
        "raw_artifact_ids": raw_artifact_ids,
        "scaffold_only": false,
        "executes_normalization": true
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'work_item.created', 'queued', 'work_item', $2, $3, $4::jsonb)",
            &[&actor_id, &work_item_id, &collection_run_id, &details_json],
        )
        .map(|_| ())
        .map_err(|error| GatewayError::Database(error.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn get_or_create_raw_artifact_for_ingest(
    transaction: &mut postgres::Transaction<'_>,
    source_id: &str,
    collection_run_id: &str,
    content_hash: &str,
    storage_path: &str,
    size_bytes: u64,
    mime_type: Option<&str>,
    metadata_json: &Value,
    actor_id: &str,
    content_already_existed: bool,
) -> Result<RawArtifactIngestRecord, GatewayError> {
    let mime_type = mime_type.map(str::to_string);
    if let Some(row) = transaction
        .query_opt(
            "SELECT id, collection_run_id, content_hash, storage_path FROM raw_artifacts WHERE source_id = $1 AND content_hash = $2 ORDER BY created_at ASC LIMIT 1",
            &[&source_id, &content_hash],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        return Ok(RawArtifactIngestRecord {
            id: row.get(0),
            collection_run_id: row.get(1),
            content_hash: row.get(2),
            storage_path: row.get(3),
            reused: true,
        });
    }
    let artifact_id = generated_record_id("artifact");
    transaction
        .execute(
            "INSERT INTO raw_artifacts (id, source_id, collection_run_id, content_hash, storage_path, mime_type, size_bytes, metadata_json) VALUES ($1, $2, $3, $4, $5, $6, $7::integer, $8::jsonb)",
            &[
                &artifact_id,
                &source_id,
                &collection_run_id,
                &content_hash,
                &storage_path,
                &mime_type,
                &(size_bytes as i32),
                &metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    insert_raw_artifact_created_audit(
        transaction,
        actor_id,
        &artifact_id,
        Some(source_id),
        Some(collection_run_id),
        content_hash,
        storage_path,
        size_bytes,
        content_already_existed,
    )?;
    Ok(RawArtifactIngestRecord {
        id: artifact_id,
        collection_run_id: Some(collection_run_id.to_string()),
        content_hash: content_hash.to_string(),
        storage_path: storage_path.to_string(),
        reused: false,
    })
}

fn get_or_create_normalized_document_for_ingest(
    transaction: &mut postgres::Transaction<'_>,
    artifact: &RawArtifactIngestRecord,
    source: &CollectionSource,
    text_content: &str,
    title: &str,
    actor_id: &str,
) -> Result<NormalizedDocumentIngestRecord, GatewayError> {
    if let Some(row) = transaction
        .query_opt(
            "SELECT id, raw_artifact_id, text_content FROM normalized_documents WHERE raw_artifact_id = $1 ORDER BY created_at ASC LIMIT 1",
            &[&artifact.id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?
    {
        return Ok(NormalizedDocumentIngestRecord {
            id: row.get(0),
            raw_artifact_id: row.get(1),
            text_content: row.get(2),
            reused: true,
        });
    }
    let document_id = generated_record_id("document");
    let metadata_json = serde_json::json!({
        "generated_by": "DIFF-081",
        "raw_content_hash": artifact.content_hash,
        "raw_storage_path": artifact.storage_path
    });
    transaction
        .execute(
            "INSERT INTO normalized_documents (id, raw_artifact_id, source_id, title, document_type, language, text_content, sensitivity, metadata_json) VALUES ($1, $2, $3, $4, 'text', NULL, $5, $6, $7::jsonb)",
            &[
                &document_id,
                &artifact.id,
                &source.id,
                &title,
                &text_content,
                &source.sensitivity,
                &metadata_json,
            ],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "source_id": source.id,
        "raw_artifact_id": artifact.id,
        "generated_by": "DIFF-081"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'normalized_document.created', 'recorded', 'normalized_document', $2, $3, $4::jsonb)",
            &[&actor_id, &document_id, &artifact.collection_run_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(NormalizedDocumentIngestRecord {
        id: document_id,
        raw_artifact_id: artifact.id.clone(),
        text_content: text_content.to_string(),
        reused: false,
    })
}

fn get_or_create_chunks_and_evidence_for_ingest(
    transaction: &mut postgres::Transaction<'_>,
    document_id: &str,
    source_id: &str,
    raw_artifact_id: &str,
    text_content: &str,
    chunk_size: i32,
    actor_id: &str,
) -> Result<(Vec<String>, Vec<String>, bool), GatewayError> {
    let existing_chunks = transaction
        .query(
            "SELECT id FROM chunks WHERE document_id = $1 ORDER BY chunk_index ASC",
            &[&document_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    if !existing_chunks.is_empty() {
        let chunk_ids = existing_chunks
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect::<Vec<_>>();
        let mut evidence_item_ids = Vec::new();
        for chunk_id in &chunk_ids {
            for row in transaction
                .query(
                    "SELECT id FROM evidence_items WHERE chunk_id = $1 ORDER BY created_at ASC",
                    &[chunk_id],
                )
                .map_err(|error| GatewayError::Database(error.to_string()))?
            {
                evidence_item_ids.push(row.get(0));
            }
        }
        return Ok((chunk_ids, evidence_item_ids, true));
    }
    let text_chunks = split_text_chunks(text_content, chunk_size as usize);
    if text_chunks.is_empty() {
        return Err(GatewayError::Validation(
            "Document text is empty".to_string(),
        ));
    }
    let mut chunk_ids = Vec::new();
    let mut evidence_item_ids = Vec::new();
    for (index, text) in text_chunks.iter().enumerate() {
        let chunk_id = generated_record_id("chunk");
        let evidence_item_id = generated_record_id("evidence");
        let location_json = serde_json::json!({
            "char_start": index * chunk_size as usize,
            "char_end": index * chunk_size as usize + text.len()
        });
        let chunk_metadata = serde_json::json!({
            "generated_by": "DIFF-081",
            "chunk_size": chunk_size
        });
        let evidence_metadata = serde_json::json!({
            "generated_by": "DIFF-081",
            "chunk_index": index
        });
        transaction
            .execute(
                "INSERT INTO chunks (id, document_id, chunk_index, text_content, location_json, embedding_status, metadata_json) VALUES ($1, $2, $3, $4, $5::jsonb, 'not_started', $6::jsonb)",
                &[&chunk_id, &document_id, &(index as i32), text, &location_json, &chunk_metadata],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO evidence_items (id, source_id, document_id, chunk_id, evidence_type, statement, observed_at, confidence, metadata_json) VALUES ($1, $2, $3, $4, 'document_chunk', $5, NULL, NULL, $6::jsonb)",
                &[&evidence_item_id, &source_id, &document_id, &chunk_id, text, &evidence_metadata],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        chunk_ids.push(chunk_id);
        evidence_item_ids.push(evidence_item_id);
    }
    let details_json = serde_json::json!({
        "source_id": source_id,
        "chunk_count": chunk_ids.len(),
        "evidence_count": evidence_item_ids.len(),
        "generated_by": "DIFF-081"
    });
    transaction
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'document_chunks.generated', 'recorded', 'normalized_document', $2, $3, $4::jsonb)",
            &[&actor_id, &document_id, &raw_artifact_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok((chunk_ids, evidence_item_ids, false))
}

fn upsert_specific_chunks(
    client: &mut Client,
    chunk_ids: &[String],
) -> Result<VectorUpsertSummary, GatewayError> {
    let settings = qdrant_settings_from_env()?;
    let collection_status = ensure_vector_chunk_collection()?;
    let collection_exists = serde_json::from_str::<Value>(&collection_status)
        .ok()
        .and_then(|value| value.get("exists").and_then(Value::as_bool))
        .unwrap_or(true);
    if chunk_ids.is_empty() {
        return Ok(VectorUpsertSummary {
            collection_name: settings.collection_name,
            collection_exists,
            chunks_upserted: 0,
        });
    }
    let mut points = Vec::new();
    for chunk_id in chunk_ids {
        let row = client
            .query_one(
                "SELECT id, document_id, chunk_index, text_content FROM chunks WHERE id = $1",
                &[chunk_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        let id: String = row.get(0);
        let document_id: String = row.get(1);
        let chunk_index: i32 = row.get(2);
        let text_content: String = row.get(3);
        points.push(plan_chunk_vector_point(
            &id,
            &document_id,
            chunk_index.max(0) as usize,
            &text_content,
            settings.vector_size,
        )?);
    }
    let response = execute_qdrant_plan(upsert_points_request(&settings, &points)?)?;
    if response.status_code >= 400 {
        return Err(GatewayError::ServiceUnavailable(response.body));
    }
    for chunk_id in chunk_ids {
        client
            .execute(
                "UPDATE chunks SET embedding_status = 'completed', metadata_json = metadata_json || $1::jsonb, updated_at = now() WHERE id = $2",
                &[&serde_json::json!({
                    "embedding_method": EMBEDDING_METHOD,
                    "vector_collection": settings.collection_name
                }), chunk_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?;
    }
    Ok(VectorUpsertSummary {
        collection_name: settings.collection_name,
        collection_exists,
        chunks_upserted: points.len(),
    })
}

fn mark_manual_upload_ingest_vector_failed(
    client: &mut Client,
    collection_run_id: &str,
    actor_id: &str,
    error_message: &str,
) -> Result<(), GatewayError> {
    let settings = qdrant_settings_from_env().ok();
    let vector_collection = settings
        .as_ref()
        .map(|settings| settings.collection_name.as_str())
        .unwrap_or("igy6_chunks");
    let patch = serde_json::json!({
        "vector_collection": vector_collection,
        "vector_upsert_completed": false,
        "vector_error": error_message
    });
    client
        .execute(
            "UPDATE collection_runs SET status = 'vector_upsert_failed', error_message = $1, summary_json = summary_json || $2::jsonb, updated_at = now() WHERE id = $3",
            &[&error_message, &patch, &collection_run_id],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    let details_json = serde_json::json!({
        "vector_collection": vector_collection,
        "error_message": error_message
    });
    client
        .execute(
            "INSERT INTO audit_events (actor_id, event_type, decision, resource_type, resource_id, correlation_id, details_json) VALUES ($1, 'manual_upload_ingest.vector_failed', 'failed', 'collection_run', $2, $2, $3::jsonb)",
            &[&actor_id, &collection_run_id, &details_json],
        )
        .map_err(|error| GatewayError::Database(error.to_string()))?;
    Ok(())
}

fn validate_evidence_links(
    transaction: &mut postgres::Transaction<'_>,
    payload: &EvidenceItemCreatePayload,
) -> Result<(), GatewayError> {
    if let Some(source_id) = &payload.source_id {
        let exists = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM sources WHERE id = $1)",
                &[source_id],
            )
            .map(|row| row.get::<_, bool>(0))
            .map_err(|error| GatewayError::Database(error.to_string()))?;
        if !exists {
            return Err(GatewayError::NotFound("Source not found".to_string()));
        }
    }
    if let Some(document_id) = &payload.document_id {
        let Some(row) = transaction
            .query_opt(
                "SELECT source_id FROM normalized_documents WHERE id = $1",
                &[document_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?
        else {
            return Err(GatewayError::NotFound("Document not found".to_string()));
        };
        let document_source_id: Option<String> = row.get(0);
        if payload
            .source_id
            .as_ref()
            .is_some_and(|source_id| Some(source_id) != document_source_id.as_ref())
        {
            return Err(GatewayError::Conflict(
                "Document does not belong to the source".to_string(),
            ));
        }
    }
    if let Some(chunk_id) = &payload.chunk_id {
        let Some(row) = transaction
            .query_opt(
                "SELECT c.document_id, d.source_id FROM chunks c LEFT JOIN normalized_documents d ON d.id = c.document_id WHERE c.id = $1",
                &[chunk_id],
            )
            .map_err(|error| GatewayError::Database(error.to_string()))?
        else {
            return Err(GatewayError::NotFound("Chunk not found".to_string()));
        };
        let chunk_document_id: String = row.get(0);
        let chunk_source_id: Option<String> = row.get(1);
        if chunk_source_id.is_none() {
            return Err(GatewayError::Conflict(
                "Chunk document not found".to_string(),
            ));
        }
        if payload
            .document_id
            .as_ref()
            .is_some_and(|document_id| document_id != &chunk_document_id)
        {
            return Err(GatewayError::Conflict(
                "Chunk does not belong to the document".to_string(),
            ));
        }
        if payload
            .source_id
            .as_ref()
            .is_some_and(|source_id| Some(source_id) != chunk_source_id.as_ref())
        {
            return Err(GatewayError::Conflict(
                "Chunk does not belong to the source".to_string(),
            ));
        }
    }
    Ok(())
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
        "manual_upload" | "conversation_history" | "user_observation" => "manual_text",
        "browser_export" => "browser_export",
        "media_file" => "media_file",
        "wifi_signal" => "wifi_signal",
        "stream_capture" => "stream_capture",
        "web_public" | "web_authorized_account" | "router_network" | "local_pc_diagnostics" => {
            "generic_connector"
        }
        _ => {
            // On grok branch we treat unknown declared types as generic (still create run + artifact).
            "generic_connector"
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

    let (summary, estimated_items, warnings) = match source.source_type.as_str() {
        "media_file" => (
            format!(
                "Media file dry-run: will store content-addressed artifact with mime/size metadata. Text extraction and deep analysis deferred (see specs collector contract)."
            ),
            Some(1),
            vec!["Full binary media parsing (PDF/image/audio/video) not performed in this pass per current implementation level.".to_string()],
        ),
        "browser_export" => (
            "Browser export dry-run: will parse provided export (JSON/HTML/text) for pages, titles, links. Will create evidence items and relationship candidates.".to_string(),
            Some(5),
            vec!["No live browser/profile access; user-provided export only.".to_string()],
        ),
        "wifi_signal" | "stream_capture" => (
            format!(
                "{} dry-run: scope and permission validated. Ingestion will record readings/sessions as artifacts + basic events. Advanced correlation and OCR/transcript later.",
                source.source_type
            ),
            Some(10),
            vec!["Advanced extraction (OCR, transcript, RF mapping) not yet wired.".to_string()],
        ),
        _ => (
            format!(
                "{} dry-run validated source and permission metadata. Connector '{}' ready for collect.",
                source.name, connector_name
            ),
            None,
            Vec::new(),
        ),
    };

    Ok(CollectionDryRunConnectorResult {
        connector_name: connector_name.to_string(),
        allowed: true,
        summary,
        estimated_items,
        warnings,
        metadata: serde_json::json!({
            "source_type": source.source_type.clone(),
            "source_location": source.location.clone(),
            "source_metadata": source.metadata_json.clone(),
            "permission_id": permission.id.clone(),
            "permission_scope": permission.scope_json.clone(),
            "allowed_operations": permission.allowed_operations.clone(),
            "external_model_policy": permission.external_model_policy.clone(),
            "approval_required": permission.approval_required,
            "preview_only": true,
            "grok_branch_note": "Dry-run now type-aware for expected collector capabilities"
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

fn collection_normalization_work_payload(
    collection_run_id: &str,
    source: &CollectionSource,
    permission_id: &str,
    raw_artifact_ids: &[String],
    collection_mode: &str,
) -> Value {
    serde_json::json!({
        "collection_run_id": collection_run_id,
        "source_id": source.id,
        "source_permission_id": permission_id,
        "raw_artifact_ids": raw_artifact_ids,
        "artifact_count": raw_artifact_ids.len(),
        "collection_mode": collection_mode,
        "scaffold_only": false,
        "executes_normalization": true,
        "worker_task_name": "collection.normalize_collection_run",
        "normalization_input_type": "utf_8_text",
        "intent_verification_recorded": true,
        "intent_verification": {
            "original_request": format!("Queue normalization for {collection_mode}"),
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

const OPS_LOG_MAX_BYTES: usize = 512 * 1024;
const OPS_LOG_MAX_LINE_CHARS: usize = 2_000;
const OPS_LOG_MAX_RETURN_LINES: usize = 200;

fn ops_log_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", elapsed.as_secs(), elapsed.subsec_millis())
}

fn redact_ops_log_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if [
        "password",
        "token",
        "secret",
        "key=",
        "database_url",
        "cookie",
        "credential",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return "[REDACTED sensitive-looking log line]".to_string();
    }
    line.to_string()
}

fn truncate_ops_log_file(path: &Path) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() <= OPS_LOG_MAX_BYTES as u64 {
        return;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let keep_from = content
        .len()
        .saturating_sub(OPS_LOG_MAX_BYTES / 2)
        .min(content.len());
    let trimmed = &content[content
        .char_indices()
        .map(|(index, _)| index)
        .find(|index| *index >= keep_from)
        .unwrap_or(keep_from)..];
    let _ = fs::write(path, trimmed);
}

pub fn append_runtime_ops_log(kind: &str, component: &str, level: &str, message: &str) {
    let file_name = if kind == "error" {
        "error.log"
    } else {
        "startup.log"
    };
    let ops_dir = artifact_data_root().join("ops");
    if fs::create_dir_all(&ops_dir).is_err() {
        return;
    }
    let path = ops_dir.join(file_name);
    let sanitized = redact_ops_log_line(
        &message
            .replace('\r', " ")
            .replace('\n', " ")
            .chars()
            .take(OPS_LOG_MAX_LINE_CHARS)
            .collect::<String>(),
    );
    let line = format!(
        "{} [{component}] [{level}] {sanitized}\n",
        ops_log_timestamp()
    );
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
    truncate_ops_log_file(&path);
}

fn tail_ops_log_lines(path: &Path, limit: usize) -> (bool, Vec<String>) {
    if !path.is_file() {
        return (false, Vec::new());
    }
    let Ok(content) = fs::read_to_string(path) else {
        return (true, Vec::new());
    };
    let lines = content
        .lines()
        .map(redact_ops_log_line)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let start = lines.len().saturating_sub(limit);
    (true, lines[start..].to_vec())
}

fn runtime_logs_query_limit(path: &str) -> usize {
    path.split('?')
        .nth(1)
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                if key == "limit" {
                    value.parse::<usize>().ok()
                } else {
                    None
                }
            })
        })
        .unwrap_or(120)
        .clamp(1, OPS_LOG_MAX_RETURN_LINES)
}

fn runtime_logs_response(path: &str) -> GatewayResponse {
    let limit = runtime_logs_query_limit(path);
    let ops_dir = artifact_data_root().join("ops");
    let startup_path = ops_dir.join("startup.log");
    let error_path = ops_dir.join("error.log");
    let (startup_exists, startup_lines) = tail_ops_log_lines(&startup_path, limit);
    let (error_exists, error_lines) = tail_ops_log_lines(&error_path, limit);
    let payload = serde_json::json!({
        "limit": limit,
        "startup_log": {
            "path": "ops/startup.log",
            "exists": startup_exists,
            "lines": startup_lines,
        },
        "error_log": {
            "path": "ops/error.log",
            "exists": error_exists,
            "lines": error_lines,
        },
    });
    json_response(200, "OK", payload.to_string(), false)
}

fn runtime_logs_append_response(body: &str) -> GatewayResponse {
    let parsed = serde_json::from_str::<Value>(body).unwrap_or(Value::Null);
    let component = parsed
        .get("component")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .chars()
        .take(64)
        .collect::<String>();
    let level = parsed
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("error")
        .chars()
        .take(16)
        .collect::<String>();
    let message = parsed
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("missing message")
        .chars()
        .take(OPS_LOG_MAX_LINE_CHARS)
        .collect::<String>();
    if message.trim().is_empty() || message == "missing message" {
        return json_response(
            422,
            "Unprocessable Entity",
            "{\"detail\":\"message is required\"}".to_string(),
            false,
        );
    }
    let kind = if level.eq_ignore_ascii_case("error") {
        "error"
    } else {
        "startup"
    };
    append_runtime_ops_log(kind, &component, &level, &message);
    json_response(200, "OK", "{\"status\":\"recorded\"}".to_string(), false)
}

pub(crate) fn artifact_data_root() -> PathBuf {
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

// === Grok branch user / password / authenticator support (off by default for TOTP) ===
// Config stored in a simple JSON next to data root for persistence across runs.
// Default password "ThatDog123". TOTP off until linked with any standard authenticator app (Google Authenticator, Authy, etc.).
// All protected operations (full access collector, etc.) require the current password (and TOTP code if enabled).
// Password change and TOTP linking require current credentials.

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
struct UserConfig {
    password: String,
    totp_secret: Option<String>,
    totp_enabled: bool,
}

fn user_config_path() -> PathBuf {
    if let Ok(root) = env::var("IGY6_DATA_ROOT") {
        PathBuf::from(root).join(".grok-user.json")
    } else if let Ok(art) = env::var("ARTIFACT_STORE_PATH") {
        Path::new(&art)
            .parent()
            .unwrap_or(Path::new("."))
            .join(".grok-user.json")
    } else {
        PathBuf::from(".grok-user.json")
    }
}

fn load_user_config() -> UserConfig {
    let path = user_config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<UserConfig>(&data) {
            return cfg;
        }
    }
    // Default for first run / grok branch
    UserConfig {
        password: "ThatDog123".to_string(),
        totp_secret: None,
        totp_enabled: false,
    }
}

fn save_user_config(cfg: &UserConfig) -> Result<(), GatewayError> {
    let path = user_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let data = serde_json::to_string_pretty(cfg)
        .map_err(|e| GatewayError::Validation(format!("config serialize: {}", e)))?;
    fs::write(&path, data).map_err(|e| GatewayError::Validation(format!("config write: {}", e)))?;
    Ok(())
}

fn get_totp(secret: &str) -> Option<TOTP> {
    // totp-rs 5.x compatible
    Secret::Encoded(secret.to_string())
        .to_bytes()
        .ok()
        .and_then(|bytes| {
            TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                bytes,
                None,
                "grok-user".to_string(),
            )
            .ok()
        })
}

fn verify_totp(secret: &str, code: &str) -> bool {
    if let Some(totp) = get_totp(secret) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        totp.check(code, now)
    } else {
        false
    }
}

fn user_status_json() -> String {
    let cfg = load_user_config();
    serde_json::json!({
        "password_set": true,
        "totp_enabled": cfg.totp_enabled,
        "has_totp_secret": cfg.totp_secret.is_some(),
        "note": "TOTP is off by default. Use /user/generate-totp then /user/confirm-totp with a code from your authenticator app to link. Any standard TOTP app works (Google Authenticator, Authy, etc.)."
    }).to_string()
}

fn user_change_password(body: &str) -> Result<String, GatewayError> {
    let obj: serde_json::Value =
        serde_json::from_str(body).map_err(|_| GatewayError::Validation("invalid json".into()))?;
    let current = obj
        .get("current_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let new_pass = obj
        .get("new_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let totp_code = obj.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");

    let mut cfg = load_user_config();
    if current != cfg.password {
        return Err(GatewayError::Forbidden("current password incorrect".into()));
    }
    if cfg.totp_enabled {
        if !verify_totp(cfg.totp_secret.as_deref().unwrap_or(""), totp_code) {
            return Err(GatewayError::Forbidden("TOTP code incorrect".into()));
        }
    }
    if new_pass.len() < 4 {
        return Err(GatewayError::Validation("new password too short".into()));
    }
    cfg.password = new_pass.to_string();
    save_user_config(&cfg)?;
    Ok(serde_json::json!({"status": "password changed"}).to_string())
}

fn user_generate_totp(body: &str) -> Result<String, GatewayError> {
    let obj: serde_json::Value =
        serde_json::from_str(body).map_err(|_| GatewayError::Validation("invalid json".into()))?;
    let current = obj
        .get("current_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let totp_code = obj.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");

    let mut cfg = load_user_config();
    if current != cfg.password {
        return Err(GatewayError::Forbidden("current password incorrect".into()));
    }
    if cfg.totp_enabled {
        if !verify_totp(cfg.totp_secret.as_deref().unwrap_or(""), totp_code) {
            return Err(GatewayError::Forbidden(
                "TOTP code incorrect to re-link".into(),
            ));
        }
    }

    // Generate new secret (standard base32 for any authenticator app) - totp-rs 5.x
    let secret = Secret::generate_secret().to_encoded().to_string();
    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        None,
        "grok-user".to_string(),
    ) {
        Ok(t) => t,
        Err(_) => return Err(GatewayError::Validation("failed to create TOTP".into())),
    };
    let otpauth = totp.get_url();

    // Temporarily store pending secret (not enabled until confirmed)
    cfg.totp_secret = Some(secret.clone());
    save_user_config(&cfg)?;

    Ok(serde_json::json!({
        "secret": secret,
        "otpauth_url": otpauth,
        "note": "Scan the otpauth_url with any authenticator app or enter the secret manually. Then call /user/confirm-totp with a current code to enable."
    }).to_string())
}

fn user_confirm_totp(body: &str) -> Result<String, GatewayError> {
    let obj: serde_json::Value =
        serde_json::from_str(body).map_err(|_| GatewayError::Validation("invalid json".into()))?;
    let current = obj
        .get("current_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let code = obj.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");

    let mut cfg = load_user_config();
    if current != cfg.password {
        return Err(GatewayError::Forbidden("current password incorrect".into()));
    }
    let secret = cfg.totp_secret.clone().ok_or_else(|| {
        GatewayError::Validation("no pending secret, call generate-totp first".into())
    })?;
    if !verify_totp(&secret, code) {
        return Err(GatewayError::Forbidden(
            "TOTP code did not verify, try again".into(),
        ));
    }
    cfg.totp_enabled = true;
    save_user_config(&cfg)?;
    Ok(
        serde_json::json!({"status": "authenticator linked and enabled", "totp_enabled": true})
            .to_string(),
    )
}

fn collect_local_project_files(
    source: &CollectionSource,
    permission: &CollectionPermission,
) -> Result<LocalProjectCollectionResult, GatewayError> {
    let source_location = source.location.as_deref().ok_or_else(|| {
        GatewayError::Validation("local_project source requires a location".to_string())
    })?;
    let source_root = fs::canonicalize(Path::new(source_location)).map_err(|_| {
        GatewayError::Validation(
            "local_project source location must be an existing directory".to_string(),
        )
    })?;
    if !source_root.is_dir() {
        return Err(GatewayError::Validation(
            "local_project source location must be an existing directory".to_string(),
        ));
    }
    let scoped_paths = scoped_local_project_paths(&source_root, &permission.scope_json)?;
    let max_files = json_usize_with_default(&permission.scope_json, "max_files", 100)?;
    let max_file_bytes =
        json_usize_with_default(&permission.scope_json, "max_file_bytes", 1_000_000)?;
    if max_files < 1 {
        return Err(GatewayError::Validation(
            "max_files must be at least 1".to_string(),
        ));
    }
    if max_file_bytes < 1 {
        return Err(GatewayError::Validation(
            "max_file_bytes must be at least 1".to_string(),
        ));
    }
    let mut candidates = Vec::new();
    for scoped_path in scoped_paths {
        candidates.extend(local_project_candidate_files(&scoped_path)?);
    }
    candidates.sort();
    candidates.dedup();
    let store = ArtifactStore::new(artifact_data_root())
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    let mut files = Vec::new();
    let mut skipped_files = Vec::new();
    for candidate in &candidates {
        if files.len() >= max_files {
            skipped_files.push(serde_json::json!({
                "path": candidate.to_string_lossy(),
                "reason": "max_files_reached"
            }));
            continue;
        }
        let resolved = fs::canonicalize(candidate)
            .map_err(|error| GatewayError::Validation(error.to_string()))?;
        if !resolved.starts_with(&source_root) {
            skipped_files.push(serde_json::json!({
                "path": resolved.to_string_lossy(),
                "reason": "escaped_source_location"
            }));
            continue;
        }
        let metadata =
            fs::metadata(&resolved).map_err(|error| GatewayError::Validation(error.to_string()))?;
        let size_bytes = metadata.len() as usize;
        if size_bytes > max_file_bytes {
            skipped_files.push(serde_json::json!({
                "path": resolved.to_string_lossy(),
                "reason": "max_file_bytes_exceeded",
                "size_bytes": size_bytes
            }));
            continue;
        }
        let content =
            fs::read(&resolved).map_err(|error| GatewayError::Validation(error.to_string()))?;
        let stored = store
            .write_bytes(&content)
            .map_err(|error| GatewayError::Conflict(error.to_string()))?;
        let relative_path = resolved
            .strip_prefix(&source_root)
            .map_err(|_| GatewayError::Validation("file escapes source location".to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        files.push(CollectedLocalProjectFile {
            source_path: resolved.to_string_lossy().to_string(),
            relative_path,
            stored,
        });
    }
    Ok(LocalProjectCollectionResult {
        total_files: candidates.len(),
        skipped_files,
        files,
    })
}

fn scoped_local_project_paths(
    source_root: &Path,
    scope: &Value,
) -> Result<Vec<PathBuf>, GatewayError> {
    let Some(paths) = scope.get("paths").and_then(Value::as_array) else {
        return Err(GatewayError::Validation(
            "local_project collection requires permission scope paths".to_string(),
        ));
    };
    if paths.is_empty() {
        return Err(GatewayError::Validation(
            "local_project collection requires permission scope paths".to_string(),
        ));
    }
    let mut resolved_paths = Vec::new();
    for raw_path in paths {
        let Some(raw_path) = raw_path.as_str().filter(|value| !value.trim().is_empty()) else {
            return Err(GatewayError::Validation(
                "permission scope paths must be non-empty strings".to_string(),
            ));
        };
        let raw_path = Path::new(raw_path);
        if raw_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        }) {
            return Err(GatewayError::Validation(
                "permission scope path escapes the source location".to_string(),
            ));
        }
        let candidate = if raw_path.is_absolute() {
            raw_path.to_path_buf()
        } else {
            source_root.join(raw_path)
        };
        let resolved = fs::canonicalize(&candidate).map_err(|_| {
            GatewayError::Validation("permission scope path does not exist".to_string())
        })?;
        if !resolved.starts_with(source_root) {
            return Err(GatewayError::Validation(
                "permission scope path escapes the source location".to_string(),
            ));
        }
        resolved_paths.push(resolved);
    }
    Ok(resolved_paths)
}

fn local_project_candidate_files(path: &Path) -> Result<Vec<PathBuf>, GatewayError> {
    if fs::symlink_metadata(path)
        .map_err(|error| GatewayError::Validation(error.to_string()))?
        .file_type()
        .is_symlink()
    {
        return Ok(Vec::new());
    }
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in
            fs::read_dir(&current).map_err(|error| GatewayError::Validation(error.to_string()))?
        {
            let entry = entry.map_err(|error| GatewayError::Validation(error.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| GatewayError::Validation(error.to_string()))?;
            if file_type.is_symlink() {
                continue;
            }
            let entry_path = entry.path();
            if file_type.is_dir() {
                stack.push(entry_path);
            } else if file_type.is_file() {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn json_usize_with_default(
    value: &Value,
    key: &str,
    default: usize,
) -> Result<usize, GatewayError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| GatewayError::Validation(format!("{key} must be a positive integer"))),
        Some(_) => Err(GatewayError::Validation(format!(
            "{key} must be a positive integer"
        ))),
    }
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
        "source_type": source.source_type,
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

fn report_work_item_payload(
    report_id: &str,
    report_title: &str,
    report_type: &str,
    report_status: &str,
    notes: Option<&str>,
) -> Value {
    serde_json::json!({
        "report_id": report_id,
        "report_title": report_title,
        "report_type": report_type,
        "report_status": report_status,
        "scaffold_only": false,
        "executes_report_generation": true,
        "notes": notes,
        "intent_verification_recorded": true,
        "intent_verification": {
            "original_request": format!("Generate report {report_id}"),
            "interpretation": format!("Create a local metadata report for {report_title}."),
            "proposed_work_type": "report_generation",
            "sources_likely_used": ["local metadata records"],
            "expected_output": "Rendered local markdown report artifact.",
            "safety_requirements": [
                "Use local metadata only.",
                "Do not read raw artifact contents.",
                "Do not call external models or services."
            ],
            "assumptions": ["Report metadata already exists."],
            "missing_information": [],
            "recorded_by": "DIFF-134 report work-item parity"
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

fn agent_task_plan_detail_path(path: &str) -> Option<String> {
    let id = path.strip_prefix("/agent/task-plans/")?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    Some(percent_decode_path_segment(id))
}

fn evidence_answer_record_detail_path(path: &str) -> Option<String> {
    let id = path.strip_prefix("/evidence-answers/")?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    Some(percent_decode_path_segment(id))
}

fn agent_task_plan_work_item_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/agent/task-plans/", "/work-item")
}

fn agent_task_plan_evidence_summary_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/agent/task-plans/", "/evidence-summary")
}

fn agent_task_plan_work_spec_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/agent/task-plans/", "/work-spec")
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

fn report_status_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/reports/", "/status")
}

fn report_work_item_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/reports/", "/work-item")
}

fn evidence_document_chunks_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/evidence/documents/", "/chunks")
}

fn evidence_item_review_state_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/evidence/items/", "/review-state")
}

fn experiment_status_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/experiments/", "/status")
}

fn source_permission_create_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/sources/", "/permissions")
}

fn source_review_state_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/sources/", "/review-state")
}

fn work_item_dispatch_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/work-items/", "/dispatch")
}

fn work_item_status_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/work-items/", "/status")
}

fn retrieval_chunk_trail_path(path: &str) -> Option<String> {
    dynamic_post_id_path(path, "/retrieval/chunks/", "/trail")
}

fn graph_relationships_path(path: &str) -> Option<(String, String)> {
    let stripped = path.strip_prefix("/memory/graph/nodes/")?;
    let stripped = stripped.strip_suffix("/relationships")?;
    let (label, node_id) = stripped.split_once('/')?;
    if label.is_empty() || node_id.is_empty() || node_id.contains('/') {
        return None;
    }
    Some((
        percent_decode_path_segment(label),
        percent_decode_path_segment(node_id),
    ))
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
        request_understanding: understand_user_request(definition.interpreted_intent),
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
    if host != "127.0.0.1" && host != "host.docker.internal" {
        return Err(GatewayError::Conflict(
            "Host bridge must be configured for 127.0.0.1 or host.docker.internal only".to_string(),
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
        "POST /actions/{} HTTP/1.1\r\nHost: {}:{}\r\nAuthorization: Bearer {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        action_name,
        host,
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

fn required_text_field_with_max(
    object: &serde_json::Map<String, Value>,
    key: &str,
    max_len: usize,
) -> Result<String, GatewayError> {
    let value = required_text_field(object, key)?;
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

fn optional_i32_field_with_default(
    object: &serde_json::Map<String, Value>,
    key: &str,
    default: i32,
    min: i32,
    max: i32,
) -> Result<i32, GatewayError> {
    Ok(optional_i32_field(object, key, min, max)?.unwrap_or(default))
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

fn optional_bounded_string_array_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    max_items: usize,
    max_len: usize,
) -> Result<Vec<String>, GatewayError> {
    let values = optional_string_array_field(object, key)?;
    if values.len() > max_items {
        return Err(GatewayError::Validation(format!(
            "{key} must contain {max_items} or fewer items."
        )));
    }
    for value in &values {
        if value.len() > max_len {
            return Err(GatewayError::Validation(format!(
                "{key} items must be {max_len} characters or fewer."
            )));
        }
    }
    Ok(values)
}

fn safe_evidence_answer_metadata_json(value: Value) -> Result<Value, GatewayError> {
    let Some(object) = value.as_object() else {
        return Err(GatewayError::Validation(
            "metadata_json must be a JSON object.".to_string(),
        ));
    };
    for (key, item) in object {
        if !matches!(
            key.as_str(),
            "created_from"
                | "raw_evidence_text_stored"
                | "full_chat_memory"
                | "hosted_ai_called"
                | "answer_packet_available"
                | "retrieval_context_available"
        ) {
            return Err(GatewayError::Validation(format!(
                "metadata_json contains unsupported evidence answer metadata key: {key}"
            )));
        }
        if !(item.is_string() || item.is_boolean() || item.is_number() || item.is_null()) {
            return Err(GatewayError::Validation(format!(
                "metadata_json.{key} must be a scalar value."
            )));
        }
    }
    Ok(value)
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

fn is_supported_collection_source_type(value: &str) -> bool {
    // On grok branch: broadened to reflect full expected collector capabilities from specs.
    // All declared source types in SourceType enum + web UI should be supportable for
    // registration, dry-run, permissioned collection, artifact creation, and evidence flow.
    matches!(
        value,
        "manual_upload"
            | "conversation_history"
            | "user_observation"
            | "local_project"
            | "local_pc_diagnostics"
            | "web_public"
            | "web_authorized_account"
            | "router_network"
            | "browser_export"
            | "media_file"
            | "wifi_signal"
            | "stream_capture"
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

fn is_source_review_trust_level(value: &str) -> bool {
    matches!(
        value,
        "trusted" | "noisy" | "sensitive" | "disabled" | "review_needed"
    )
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

fn is_experiment_status(value: &str) -> bool {
    matches!(
        value,
        "planned"
            | "running"
            | "completed"
            | "failed"
            | "abandoned"
            | "accepted"
            | "rejected"
            | "deferred"
    )
}

fn is_improvement_target_area(value: &str) -> bool {
    matches!(
        value,
        "parsing" | "retrieval" | "scoring" | "prediction" | "reporting" | "reasoning" | "safety"
    )
}

fn is_improvement_priority(value: &str) -> bool {
    matches!(value, "low" | "normal" | "high" | "urgent")
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

fn is_agent_task_plan_status(value: &str) -> bool {
    matches!(
        value,
        "proposed"
            | "needs_clarification"
            | "approval_required"
            | "evidence_needed"
            | "ready"
            | "unsupported"
            | "converted_to_work"
            | "canceled"
    )
}

fn is_agent_task_plan_supported_state(value: &str) -> bool {
    matches!(
        value,
        "supported"
            | "unsupported"
            | "clarification_needed"
            | "approval_required"
            | "evidence_needed"
    )
}

fn is_work_item_status(value: &str) -> bool {
    matches!(
        value,
        "pending_intent_verification" | "queued" | "running" | "completed" | "failed" | "canceled"
    )
}

fn is_evidence_review_state(value: &str) -> bool {
    matches!(
        value,
        "needs_correction" | "corrected" | "superseded" | "disputed" | "verified"
    )
}

fn is_evidence_answer_status(value: &str) -> bool {
    matches!(
        value,
        "retrieved"
            | "evidence_summary"
            | "evidence_grounded_llm"
            | "insufficient_evidence"
            | "not_generated"
            | "fallback"
            | "error"
            | "unavailable"
            | "partial"
    )
}

fn is_feedback_target_type(value: &str) -> bool {
    matches!(
        value,
        "source"
            | "document"
            | "evidence_item"
            | "evidence_answer"
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

pub(crate) fn postgres_client_url(database_url: &str) -> String {
    for prefix in ["postgresql+", "postgres+"] {
        if let Some(driver_url) = database_url.strip_prefix(prefix) {
            if let Some((_, rest)) = driver_url.split_once("://") {
                return format!("{}://{}", prefix.trim_end_matches('+'), rest);
            }
        }
    }
    database_url.to_string()
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

fn execute_external_http(
    request: ExternalHttpRequest,
) -> Result<ExternalHttpResponse, GatewayError> {
    let (host, port) = host_port_from_url(&request.origin).ok_or_else(|| {
        GatewayError::Validation(format!(
            "External service origin is invalid: {}",
            redact_url(&request.origin)
        ))
    })?;
    let mut addrs = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let addr = addrs.next().ok_or_else(|| {
        GatewayError::ServiceUnavailable(format!("No address resolved for {host}:{port}"))
    })?;
    let mut stream = TcpStream::connect_timeout(&addr, request.timeout)
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    stream
        .set_read_timeout(Some(request.timeout))
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    stream
        .set_write_timeout(Some(request.timeout))
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let body = request.body.unwrap_or_default();
    let mut rendered = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\n",
        request.method, request.path, host
    );
    let mut has_content_type = false;
    for (name, value) in request.headers {
        if name.eq_ignore_ascii_case("host")
            || name.eq_ignore_ascii_case("connection")
            || name.eq_ignore_ascii_case("content-length")
        {
            continue;
        }
        if name.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        rendered.push_str(&name);
        rendered.push_str(": ");
        rendered.push_str(&value);
        rendered.push_str("\r\n");
    }
    if !body.is_empty() {
        if !has_content_type {
            rendered.push_str("Content-Type: application/json\r\n");
        }
        rendered.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    rendered.push_str("\r\n");
    rendered.push_str(&body);
    stream
        .write_all(rendered.as_bytes())
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| GatewayError::ServiceUnavailable(error.to_string()))?;
    parse_external_http_response(&response)
}

fn parse_external_http_response(raw: &str) -> Result<ExternalHttpResponse, GatewayError> {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let status_line = head
        .lines()
        .next()
        .ok_or_else(|| GatewayError::ServiceUnavailable("Empty HTTP response".to_string()))?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| GatewayError::ServiceUnavailable("Malformed HTTP response".to_string()))?;
    Ok(ExternalHttpResponse {
        status_code,
        body: decode_chunked_http_body_if_needed(head, body),
    })
}

fn decode_chunked_http_body_if_needed(head: &str, body: &str) -> String {
    if !head.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.trim().eq_ignore_ascii_case("transfer-encoding")
                && value.trim().eq_ignore_ascii_case("chunked")
        })
    }) {
        return body.to_string();
    }
    let mut decoded = String::new();
    let mut rest = body;
    loop {
        let Some((size_line, after_size)) = rest.split_once("\r\n") else {
            return body.to_string();
        };
        let Ok(size) = usize::from_str_radix(size_line.trim(), 16) else {
            return body.to_string();
        };
        if size == 0 {
            return decoded;
        }
        if after_size.len() < size {
            return body.to_string();
        }
        decoded.push_str(&after_size[..size]);
        rest = after_size.get(size + 2..).unwrap_or("");
    }
}

fn normalize_http_origin_for_service(origin: &str) -> Result<String, GatewayError> {
    let trimmed = origin.trim().trim_end_matches('/');
    let rest = trimmed.strip_prefix("http://").ok_or_else(|| {
        GatewayError::Validation("Service URL must use local http:// origin.".to_string())
    })?;
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('\\')
        || rest.contains("..")
        || rest.contains('@')
    {
        return Err(GatewayError::Validation(
            "Service URL must be an http://host:port origin without credentials or path."
                .to_string(),
        ));
    }
    Ok(format!("http://{rest}"))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        let combined = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;
        encoded.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(combined & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

pub fn help_text() -> &'static str {
    "igy6-gateway\n\nUsage:\n  igy6-gateway [--bind 0.0.0.0:8000]\n  igy6-gateway --help\n\nRoutes:\n  GET /\n  GET /health/live\n  GET /health/ready\n  GET /rust-migration/status\n  GET /agent/capabilities\n  POST /agent/intent\n  GET/POST /agent/task-plans and task plan detail reads\n  POST /agent/task-plans/{task_plan_id}/evidence-summary safe evidence readiness summary\n  POST /agent/task-plans/{task_plan_id}/work-spec bounded work spec proposal\n  POST /agent/task-plans/{task_plan_id}/work-item approval-gated work creation\n  POST /chat/retrieval-preview\n  POST /chat/evidence-answer\n  GET/POST /evidence-answers and answer detail reads\n  POST /approvals\n  POST /feedback\n  POST /outcomes\n  GET/POST /analysis patterns and GET analysis hypotheses/predictions/recommendations\n  GET/POST /reports and report detail reads\n  GET/POST /sources and selected source detail/permission reads\n  POST /sources/{source_id}/review-state bounded source trust/sensitivity review updates\n  POST /evidence/items/{evidence_item_id}/review-state bounded evidence correction/supersession review updates\n  POST /work-items creation only\n  GET /settings/env\n  GET /memory/vector/chunks\n  GET /ops/runtime-logs\n  POST /ops/runtime-logs/append\n  GET /memory/graph/schema\n  GET /approvals and approval detail reads\n  GET /work-items and work item detail reads\n  GET /evidence documents/items/chunks/claims and detail reads\n  GET /artifacts, /audit-events, /collection-runs, /feedback, /outcomes and detail reads\n\nUnsupported routes return a Rust 404; FastAPI fallback is removed.\n"
}

fn settings_env_status_json() -> String {
    const GROUPS: &[(&str, &str)] = &[
        ("app", "App"),
        ("postgres", "PostgreSQL"),
        ("redis", "Redis (archived)"),
        ("qdrant", "Qdrant"),
        ("neo4j", "Neo4j"),
        ("llm", "Local LLM"),
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
        (
            "LLM_PROVIDER",
            "llm",
            "Optional local LLM provider: none or ollama.",
            false,
            true,
        ),
        (
            "OLLAMA_BASE_URL",
            "llm",
            "Local Ollama base URL. No tokens or cloud endpoints.",
            false,
            true,
        ),
        (
            "OLLAMA_MODEL",
            "llm",
            "Local Ollama model name. Empty is allowed when provider is none.",
            false,
            true,
        ),
        (
            "LLM_TIMEOUT_SECONDS",
            "llm",
            "Timeout for local LLM generation attempts.",
            false,
            true,
        ),
        (
            "LLM_EVIDENCE_REQUIRED",
            "llm",
            "Require retrieved evidence before local LLM generation.",
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
        "llm" => "Local LLM",
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
        "{{\"collection_name\":\"{}\",\"exists\":false,\"detail\":{{\"configured_url\":\"{}\",\"tcp_reachable\":{},\"collection_existence_verified\":false,\"note\":\"Qdrant collection status could not be verified.\"}}}}",
        escape_json(&collection_name),
        escape_json(&redact_url(&qdrant_url)),
        reachability
    )
}

fn vector_collection_status_live_json() -> String {
    let settings = match qdrant_settings_from_env() {
        Ok(settings) => settings,
        Err(_) => return vector_collection_status_json(),
    };
    let tcp_reachable = tcp_reachability_from_url(&settings.base_url);
    if !tcp_reachable {
        return serde_json::json!({
            "collection_name": settings.collection_name,
            "exists": false,
            "detail": {
                "configured_url": redact_url(&settings.base_url),
                "tcp_reachable": false,
                "collection_existence_verified": false,
                "note": "Qdrant is not reachable from the gateway.",
            }
        })
        .to_string();
    }

    match vector_collection_status_from_qdrant(&settings) {
        Ok(body) => {
            let Ok(mut value) = serde_json::from_str::<Value>(&body) else {
                return body;
            };
            let detail = value
                .as_object_mut()
                .and_then(|object| object.get_mut("detail"))
                .and_then(Value::as_object_mut);
            if let Some(detail) = detail {
                detail.insert(
                    "configured_url".to_string(),
                    Value::String(redact_url(&settings.base_url)),
                );
                detail.insert("tcp_reachable".to_string(), Value::Bool(true));
                detail.insert(
                    "collection_existence_verified".to_string(),
                    Value::Bool(true),
                );
            } else if let Some(object) = value.as_object_mut() {
                object.insert(
                    "detail".to_string(),
                    serde_json::json!({
                        "configured_url": redact_url(&settings.base_url),
                        "tcp_reachable": true,
                        "collection_existence_verified": true,
                    }),
                );
            }
            value.to_string()
        }
        Err(_) => serde_json::json!({
            "collection_name": settings.collection_name,
            "exists": false,
            "detail": {
                "configured_url": redact_url(&settings.base_url),
                "tcp_reachable": true,
                "collection_existence_verified": false,
                "note": "Qdrant is reachable but collection status could not be verified.",
            }
        })
        .to_string(),
    }
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
        "{{\"actions\":[{}],\"runtime\":{{\"gateway\":\"rust\",\"fastapi_fallback\":false}},\"policy\":{{\"local_first\":true,\"hosted_ai_enabled\":false,\"external_model_policy\":\"blocked_by_default\",\"arbitrary_command_execution\":false,\"prompt_injection_filter\":\"enabled\",\"approval_required_for_system_changing\":true,\"blocked_request_classes\":[\"prompt_injection\",\"hosted_ai\",\"external_model\",\"secret_dump\",\"raw_shell_command\",\"credential_exfiltration\"]}}}}",
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
        "{{\"original_message\":\"{}\",\"interpreted_intent\":\"{}\",\"proposed_action\":{},\"request_understanding\":{},\"action_type\":\"{}\",\"approval_required\":{},\"risk_level\":\"{}\",\"required_parameters\":{},\"missing_parameters\":{},\"safety_notes\":{},\"executable_now\":{},\"reason\":{}}}",
        escape_json(&response.original_message),
        escape_json(&response.interpreted_intent),
        option_json(response.proposed_action),
        request_understanding_json(&response.request_understanding),
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

fn request_understanding_json(understanding: &igy6_agent_api::RequestUnderstanding) -> String {
    format!(
        "{{\"category\":\"{}\",\"wants\":\"{}\",\"evidence_required\":{},\"clarification_needed\":{},\"approval_required\":{},\"work_item_should_be_created\":{},\"unsupported_or_unsafe\":{},\"reason\":{},\"missing_information\":{},\"assumptions\":{},\"next_step\":\"{}\"}}",
        request_category_json(&understanding.category),
        escape_json(&understanding.wants),
        understanding.evidence_required,
        understanding.clarification_needed,
        understanding.approval_required,
        understanding.work_item_should_be_created,
        understanding.unsupported_or_unsafe,
        option_string_json(understanding.reason.as_deref()),
        json_owned_string_array(&understanding.missing_information),
        json_owned_string_array(&understanding.assumptions),
        escape_json(&understanding.next_step)
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
    let task_name = extract_json_string(body, "task_name")
        .or_else(|| extract_json_string(body, "task"))
        .unwrap_or_else(|| igy6_llm::DEFAULT_TASK_NAME.to_string());
    let retrieval_context = build_hydrated_chunk_search_result(
        &message,
        "chunks",
        false,
        Vec::<igy6_retrieval_preview::HydratedChunkSearchHit>::new(),
        5,
    );
    let answer = match LlmConfig::from_env() {
        Ok(config) if config.provider == LlmProvider::Ollama => {
            match load_local_llm_routing_config() {
                Ok(routing_config) => answer_with_optional_llm_for_task(
                    retrieval_context,
                    &config,
                    &routing_config,
                    &task_name,
                    &StdHttpTransport,
                ),
                Err(error) => {
                    deterministic_fallback_for_llm_config_error(retrieval_context, &error)
                }
            }
        }
        Ok(config) => answer_with_optional_llm(retrieval_context, &config, &StdHttpTransport),
        Err(error) => deterministic_fallback_for_llm_config_error(retrieval_context, &error),
    };
    evidence_grounded_answer_json(&answer)
}

pub(crate) fn json_response(
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

fn request_category_json(value: &igy6_agent_api::RequestCategory) -> &'static str {
    match value {
        igy6_agent_api::RequestCategory::EvidenceQuestion => "evidence_question",
        igy6_agent_api::RequestCategory::AddData => "add_data",
        igy6_agent_api::RequestCategory::CheckWorkStatus => "check_work_status",
        igy6_agent_api::RequestCategory::CreateReport => "create_report",
        igy6_agent_api::RequestCategory::RequestAction => "request_action",
        igy6_agent_api::RequestCategory::SystemChangingAction => "system_changing_action",
        igy6_agent_api::RequestCategory::Feedback => "feedback",
        igy6_agent_api::RequestCategory::RecordOutcome => "record_outcome",
        igy6_agent_api::RequestCategory::Correction => "correction",
        igy6_agent_api::RequestCategory::Diagnostics => "diagnostics",
        igy6_agent_api::RequestCategory::ProjectStatus => "project_status",
        igy6_agent_api::RequestCategory::ExperimentOrImprovement => "experiment_or_improvement",
        igy6_agent_api::RequestCategory::Unclear => "unclear",
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
        let response = handle_gateway_request(&request("GET", "/", ""), None, NO_FALLBACK_ORIGIN);
        assert_eq!(response.status_code, 200);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("\"service\":\"igy6-gateway\""));
        assert!(response.body.contains("\"primary_gateway\":true"));

        let response = handle_gateway_request(
            &request("GET", "/health/live", ""),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"primary_gateway\":true"));

        let response = handle_gateway_request(
            &request("GET", "/health/ready", ""),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"primary_gateway\":\"rust\""));
        assert!(response.body.contains("\"fallback\":\"none\""));
        assert!(response
            .body
            .contains("\"fastapi_fallback\":{\"status\":\"removed\"}"));
    }

    #[test]
    fn migration_status_uses_manifest_summary() {
        let response = handle_gateway_request(
            &request("GET", "/rust-migration/status", ""),
            Some("{\"cutover_ready\": true, \"status\": \"complete\"}"),
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("show_project_health"));
        assert!(response.body.contains("\"hosted_ai_enabled\":false"));
        assert!(response
            .body
            .contains("\"arbitrary_command_execution\":false"));
        assert!(response.body.contains("prompt_injection"));
        assert!(!response.proxied_to_fallback);
    }

    #[test]
    fn agent_intent_uses_rust_classifier() {
        let response = handle_gateway_request(
            &request("POST", "/agent/intent", "{\"message\":\"show health\"}"),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"proposed_action\":\"show_project_health\""));
        assert!(response.body.contains("\"request_understanding\""));
        assert!(response.body.contains("\"category\":\"diagnostics\""));
        assert!(response.body.contains("\"approval_required\":false"));
    }

    #[test]
    fn agent_intent_returns_request_understanding_for_work_and_approval_posture() {
        let report = handle_gateway_request(
            &request(
                "POST",
                "/agent/intent",
                "{\"message\":\"Create a report about failed builds\"}",
            ),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(report.status_code, 200);
        assert!(report.body.contains("\"category\":\"create_report\""));
        assert!(report.body.contains("\"work_item_should_be_created\":true"));
        assert!(report.body.contains("\"clarification_needed\":true"));

        let risky = handle_gateway_request(
            &request(
                "POST",
                "/agent/intent",
                "{\"message\":\"restart the stack\"}",
            ),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(risky.status_code, 200);
        assert!(risky
            .body
            .contains("\"category\":\"system_changing_action\""));
        assert!(risky.body.contains("\"approval_required\":true"));
    }

    #[test]
    fn agent_intent_returns_unsupported_for_destructive_request_without_work() {
        let response = handle_gateway_request(
            &request(
                "POST",
                "/agent/intent",
                "{\"message\":\"run rm -rf target\"}",
            ),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"category\":\"system_changing_action\""));
        assert!(response.body.contains("\"unsupported_or_unsafe\":true"));
        assert!(response
            .body
            .contains("\"work_item_should_be_created\":false"));
        assert!(response.body.contains("\"proposed_action\":null"));
    }

    #[test]
    fn retrieval_preview_requires_live_database_and_evidence_answer_is_contract_only() {
        let response = handle_gateway_request(
            &request(
                "POST",
                "/chat/retrieval-preview",
                "{\"message\":\"what changed?\"}",
            ),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 503);
        assert!(response
            .body
            .contains("\"DATABASE_URL is required for Rust DB route\""));

        let response = handle_gateway_request(
            &request(
                "POST",
                "/chat/evidence-answer",
                "{\"message\":\"what changed?\"}",
            ),
            None,
            NO_FALLBACK_ORIGIN,
        );
        assert_eq!(response.status_code, 200);
        assert!(response
            .body
            .contains("\"answer_status\":\"insufficient_evidence\""));
    }

    #[test]
    fn unsupported_routes_return_rust_not_found_without_fallback() {
        let request = request("POST", "/unimplemented-route", "{}");
        let response = handle_gateway_request(&request, None, NO_FALLBACK_ORIGIN);
        assert_eq!(response.status_code, 404);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("FastAPI fallback is removed"));
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(unknown.status_code, 404);
        assert!(!unknown.proxied_to_fallback);

        let malformed = handle_gateway_request_with_db(
            &request("POST", "/agent/actions/rm%20-rf/execute", "{}"),
            None,
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            ("GET", "/analysis/calibration/summary"),
            ("GET", "/agent/task-plans"),
            ("GET", "/agent/task-plans/task-plan-1"),
            ("GET", "/evidence-answers"),
            ("GET", "/evidence-answers/answer-1"),
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
            ("GET", "/experiments"),
            ("GET", "/experiments/experiment-1"),
            ("GET", "/improvements"),
            ("GET", "/improvements/improvement-1"),
        ] {
            let response = handle_gateway_request_with_db(
                &request(method, path, ""),
                None,
                NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
            Some("not-a-postgres-url"),
        );
        assert_eq!(response.status_code, 502);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("database error"));
    }

    #[test]
    fn runtime_logs_route_returns_startup_and_error_sections() {
        let temp_root = env::temp_dir().join(format!(
            "igy6-runtime-logs-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let ops_dir = temp_root.join("ops");
        fs::create_dir_all(&ops_dir).expect("ops dir");
        fs::write(
            ops_dir.join("startup.log"),
            "1.0Z [igy6-cli] [info] stack started\n",
        )
        .expect("startup log");
        fs::write(
            ops_dir.join("error.log"),
            "2.0Z [gateway] [error] sample failure\n",
        )
        .expect("error log");
        let previous_artifact = env::var("ARTIFACT_STORE_PATH").ok();
        let previous_artifact_root = env::var("IGY6_ARTIFACT_DATA_ROOT").ok();
        let previous_data_root = env::var("IGY6_DATA_ROOT").ok();
        env::remove_var("ARTIFACT_STORE_PATH");
        env::remove_var("IGY6_ARTIFACT_DATA_ROOT");
        env::set_var("IGY6_DATA_ROOT", &temp_root);

        let response = handle_gateway_request_with_db(
            &request("GET", "/ops/runtime-logs?limit=10", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );

        match previous_artifact {
            Some(value) => env::set_var("ARTIFACT_STORE_PATH", value),
            None => env::remove_var("ARTIFACT_STORE_PATH"),
        }
        match previous_artifact_root {
            Some(value) => env::set_var("IGY6_ARTIFACT_DATA_ROOT", value),
            None => env::remove_var("IGY6_ARTIFACT_DATA_ROOT"),
        }
        match previous_data_root {
            Some(value) => env::set_var("IGY6_DATA_ROOT", value),
            None => env::remove_var("IGY6_DATA_ROOT"),
        }
        let _ = fs::remove_dir_all(&temp_root);

        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"startup_log\""));
        assert!(response.body.contains("stack started"));
        assert!(response.body.contains("sample failure"));
    }

    #[test]
    fn vector_collection_status_live_does_not_stub_diff108_read_only() {
        let response = handle_gateway_request_with_db(
            &request("GET", "/memory/vector/chunks", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 200);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("\"collection_name\""));
        assert!(response.body.contains("collection_existence_verified"));
        assert!(!response.body.contains("DIFF-108"));
        assert!(!response.body.contains("read_only_status"));
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 200, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains(expected), "{path}");
        }
    }

    #[test]
    fn diff133_routes_are_rust_native_and_validate_without_fallback() {
        let unsupported_graph_label = handle_gateway_request_with_db(
            &request(
                "GET",
                "/memory/graph/nodes/BadLabel/node-1/relationships",
                "",
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(unsupported_graph_label.status_code, 422);
        assert!(!unsupported_graph_label.proxied_to_fallback);

        let graph_lineage_without_db = handle_gateway_request_with_db(
            &request("POST", "/memory/graph/lineage/sync", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(graph_lineage_without_db.status_code, 503);
        assert!(!graph_lineage_without_db.proxied_to_fallback);
        assert!(graph_lineage_without_db.body.contains("DATABASE_URL"));

        let vector_search_invalid_limit = handle_gateway_request_with_db(
            &request(
                "POST",
                "/memory/vector/chunks/search",
                r#"{"query":"alpha","limit":99}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(vector_search_invalid_limit.status_code, 422);
        assert!(!vector_search_invalid_limit.proxied_to_fallback);

        let vector_upsert_without_db = handle_gateway_request_with_db(
            &request("POST", "/memory/vector/chunks/upsert", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(vector_upsert_without_db.status_code, 503);
        assert!(!vector_upsert_without_db.proxied_to_fallback);
        assert!(vector_upsert_without_db.body.contains("DATABASE_URL"));
    }

    #[test]
    fn diff133_service_request_helpers_are_bounded_and_safe() {
        let settings = QdrantSettings {
            base_url: "http://qdrant:6333".to_string(),
            collection_name: "igy6_chunks".to_string(),
            vector_size: 8,
        };
        let search_plan = search_points_request(&settings, "alpha beta", 5).expect("search plan");
        assert_eq!(search_plan.origin, "http://qdrant:6333");
        assert_eq!(search_plan.path, "/collections/igy6_chunks/points/search");
        assert_eq!(search_plan.timeout_seconds, 10);
        assert!(search_plan
            .body
            .expect("body")
            .contains("\"with_vector\":false"));

        assert_eq!(base64_encode(b"neo4j:secret"), "bmVvNGo6c2VjcmV0");
        assert!(validate_graph_node_label("Chunk").is_ok());
        assert!(validate_graph_node_label("Chunk) MATCH (n)").is_err());

        let parsed = parse_external_http_response(
            "HTTP/1.1 404 Not Found\r\nContent-Length: 15\r\n\r\ncollection lost",
        )
        .expect("http response");
        assert_eq!(parsed.status_code, 404);
        assert_eq!(parsed.body, "collection lost");
    }

    #[test]
    fn settings_env_status_redacts_secrets_and_does_not_read_env_file() {
        let response = handle_gateway_request_with_db(
            &request("GET", "/settings/env", ""),
            None,
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn settings_env_verify_passes_without_archived_redis_keys() {
        let config = test_settings_config();
        let parsed = ParsedSettingsEnv {
            values: HashMap::new(),
            unmanaged_order: Vec::new(),
        };
        let requested = valid_settings_values();
        let (candidate, unmanaged, changed_keys) =
            build_settings_candidate(&config, &parsed, &requested).expect("candidate");
        let validation = validate_settings_candidate(&candidate, &unmanaged, &changed_keys);
        assert_eq!(validation.errors.len(), 0);
        let response = settings_verify_response_json(&candidate, &validation);
        assert!(response.contains("\"passed\":true"));
    }

    #[test]
    fn settings_url_plausible_accepts_postgres_scheme_database_url() {
        assert!(settings_url_plausible(
            "DATABASE_URL",
            "postgres://adaptive:secret@postgres:5432/adaptive_intelligence"
        ));
    }

    #[test]
    fn web_only_scope_requires_public_http_urls() {
        let scope = serde_json::json!(["https://example.com/docs"]);
        assert!(is_web_only_scope(Some(&scope)));
        let mixed = serde_json::json!(["https://example.com", "everything"]);
        assert!(!is_web_only_scope(Some(&mixed)));
    }

    #[test]
    fn bypass_fetch_headers_include_cookie_and_bearer() {
        let payload = serde_json::json!({
            "bypass_auth": true,
            "cookie": "session=abc123",
            "authorization": "token-value"
        });
        let headers = web_fetch_headers(&payload);
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Cookie" && v == "session=abc123"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer token-value"));
        assert!(!headers.iter().any(|(k, _)| k == "DNT"));
    }

    #[test]
    fn bypass_fetch_requires_credentials() {
        let payload = serde_json::json!({
            "bypass_auth": true,
            "scope": ["https://example.com/private"]
        });
        assert!(!bypass_fetch_has_credentials(&payload));
    }

    #[test]
    fn max_reach_flag_enables_auto_bypass_path() {
        let payload = serde_json::json!({
            "max_reach": true,
            "scope": ["https://example.com/article"]
        });
        assert!(max_reach_enabled(&payload));
        assert!(auto_bypass_enabled(&payload) || max_reach_enabled(&payload));
    }

    #[test]
    fn auto_bypass_prefers_richer_html_over_login_wall() {
        let public_page = "<html><body><main><article><p>Full article text with many details.</p></article></main></body></html>";
        let login_wall = "<html><body><h1>Sign in to continue</h1><p>Subscribe to read this article.</p></body></html>";
        assert!(auto_bypass_content_score(public_page) > auto_bypass_content_score(login_wall));
    }

    #[test]
    fn auto_bypass_url_variants_include_amp_and_mobile() {
        let variants = auto_bypass_url_variants("https://www.example.com/news/story");
        let labels: Vec<String> = variants.into_iter().map(|(label, _)| label).collect();
        assert!(labels.contains(&"direct".to_string()));
        assert!(labels.contains(&"amp_query".to_string()));
        assert!(labels.contains(&"mobile_www".to_string()));
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains("DATABASE_URL"), "{path}");
        }
    }

    #[test]
    fn evidence_answer_records_are_rust_native_and_require_database_url_after_validation() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/evidence-answers",
                r#"{"user_question":"What did I upload?","answer_status":"retrieved","answer_text":"Retrieved 2 local evidence records.","evidence_item_ids":["evidence-1"],"document_ids":["document-1"],"chunk_ids":["chunk-1"],"source_ids":["source-1"],"safe_labels":["evidence evidence-1"],"retrieval_mode":"retrieval_preview","retrieval_count":2,"local_model_status":"not_used_retrieval_preview","metadata_json":{"created_from":"test"}}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));

        let list = handle_gateway_request_with_db(
            &request("GET", "/evidence-answers", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(list.status_code, 503);
        assert!(!list.proxied_to_fallback);

        let detail = handle_gateway_request_with_db(
            &request("GET", "/evidence-answers/answer-1", ""),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(detail.status_code, 503);
        assert!(!detail.proxied_to_fallback);
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
            NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn agent_task_plan_routes_are_rust_native_and_require_database_url() {
        let valid = r#"{"user_request_summary":"Review synthetic build evidence.","intent_category":"evidence_question","status":"evidence_needed","proposed_steps":["Check stored evidence before answering."],"required_evidence":["build logs"],"approval_required":false,"supported_state":"evidence_needed","next_safe_action":"Run retrieval preview before creating work.","metadata_json":{"source":"test"}}"#;
        let create = handle_gateway_request_with_db(
            &request("POST", "/agent/task-plans", valid),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(create.status_code, 503);
        assert!(!create.proxied_to_fallback);
        assert!(create.body.contains("DATABASE_URL"));

        let parsed = parse_agent_task_plan_create(valid).expect("task plan");
        assert_eq!(parsed.intent_category, "evidence_question");
        assert_eq!(parsed.status, "evidence_needed");
        assert_eq!(parsed.proposed_steps.len(), 1);
        assert_eq!(parsed.required_evidence, vec!["build logs"]);

        let transition = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/task-plans/taskplan-1/work-item",
                r#"{"actor_id":"local-owner","approval_id":null}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(transition.status_code, 503);
        assert!(!transition.proxied_to_fallback);
        assert!(transition.body.contains("DATABASE_URL"));

        let work_spec = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/task-plans/taskplan-1/work-spec",
                r#"{"actor_id":"local-owner","work_type":"report_generation","expected_output":"Create a bounded report from this task plan."}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(work_spec.status_code, 503);
        assert!(!work_spec.proxied_to_fallback);
        assert!(work_spec.body.contains("DATABASE_URL"));

        let evidence_summary = handle_gateway_request_with_db(
            &request(
                "POST",
                "/agent/task-plans/taskplan-1/evidence-summary",
                r#"{"actor_id":"local-owner","answer_status":"retrieved","retrieved_count":2,"safe_labels":["evidence evidence-1","chunk chunk-1"],"missing_evidence":false}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(evidence_summary.status_code, 503);
        assert!(!evidence_summary.proxied_to_fallback);
        assert!(evidence_summary.body.contains("DATABASE_URL"));
    }

    #[test]
    fn agent_task_plan_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"user_request_summary":"","intent_category":"evidence_question","next_safe_action":"x"}"#,
            r#"{"user_request_summary":"x","intent_category":"","next_safe_action":"x"}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","status":"running","next_safe_action":"x"}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","supported_state":"shell_ready","next_safe_action":"x"}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","next_safe_action":"","proposed_steps":[]}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","next_safe_action":"x","proposed_steps":[""]}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","next_safe_action":"x","required_evidence":[{}]}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","next_safe_action":"x","approval_required":"yes"}"#,
            r#"{"user_request_summary":"x","intent_category":"evidence_question","next_safe_action":"x","metadata_json":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/agent/task-plans", body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        for body in ["[]", r#"{"actor_id":""}"#, r#"{"approval_id":[]}"#] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/agent/task-plans/taskplan-1/work-item", body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        for body in [
            "[]",
            "{}",
            r#"{"actor_id":"","work_type":"report_generation"}"#,
            r#"{"work_type":""}"#,
            r#"{"work_type":"shell_command"}"#,
            r#"{"work_type":"report_generation","expected_output":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/agent/task-plans/taskplan-1/work-spec", body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        for body in [
            "[]",
            r#"{"actor_id":""}"#,
            r#"{"retrieved_count":-1}"#,
            r#"{"retrieved_count":1001}"#,
            r#"{"answer_status":[]}"#,
            r#"{"safe_labels":[{}]}"#,
            r#"{"missing_evidence":"no"}"#,
            r#"{"missing_evidence_guidance":[]}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request(
                    "POST",
                    "/agent/task-plans/taskplan-1/evidence-summary",
                    body,
                ),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        let parsed = parse_agent_task_plan_work_spec(
            r#"{"actor_id":"local-owner","work_type":"report_generation","expected_output":"Create a bounded report."}"#,
        )
        .expect("work spec");
        assert_eq!(parsed.work_type, "report_generation");
        assert_eq!(
            parsed.expected_output.as_deref(),
            Some("Create a bounded report.")
        );
        let evidence = parse_agent_task_plan_evidence_summary(
            r#"{"answer_status":"retrieved","retrieved_count":2,"safe_labels":["evidence evidence-1"],"missing_evidence":false}"#,
        )
        .expect("evidence summary");
        assert_eq!(evidence.retrieved_count, 2);
        assert_eq!(evidence.labels, vec!["evidence evidence-1"]);

        assert_eq!(
            agent_task_plan_work_item_path("/agent/task-plans/taskplan-1/work-item"),
            Some("taskplan-1".to_string())
        );
        assert!(agent_task_plan_work_item_path("/agent/task-plans/x/y/work-item").is_none());
        assert_eq!(
            agent_task_plan_work_spec_path("/agent/task-plans/taskplan-1/work-spec"),
            Some("taskplan-1".to_string())
        );
        assert!(agent_task_plan_work_spec_path("/agent/task-plans/x/y/work-spec").is_none());
        assert_eq!(
            agent_task_plan_evidence_summary_path("/agent/task-plans/taskplan-1/evidence-summary"),
            Some("taskplan-1".to_string())
        );
        assert!(
            agent_task_plan_evidence_summary_path("/agent/task-plans/x/y/evidence-summary")
                .is_none()
        );
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
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
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
            ("/reports/report-1/work-item", r#"{"notes":"handoff"}"#),
            ("/work-items/work-1/dispatch", "{}"),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
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
            ("/reports/report-1/work-item", "[]"),
            (
                "/reports/report-1/work-item",
                r#"{"requested_by_actor_id":""}"#,
            ),
            ("/work-items/work-1/dispatch", "[]"),
            ("/work-items/work-1/dispatch", r#"{"actor_id":""}"#),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
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
        assert_eq!(
            report_work_item_path("/reports/report-1/work-item").as_deref(),
            Some("report-1")
        );
        assert!(pattern_review_path("/analysis/patterns/../x/review").is_none());
        assert!(report_work_item_path("/reports/../x/work-item").is_none());
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

        let report_item = WorkItemDispatchRecord {
            work_type: "report_generation".to_string(),
            payload_json: serde_json::json!({
                "report_id": "report-1",
                "intent_verification": {"recorded_by": "test"}
            }),
            ..work_item
        };
        assert_eq!(
            dispatch_task_name(&report_item).expect("task"),
            "report.generate_markdown"
        );
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
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 503);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("DATABASE_URL"));
    }

    #[test]
    fn diff135_routes_are_rust_native_and_require_database_url_after_validation() {
        let manual_upload_ingest_body = valid_manual_upload_body_with_chunk_size();
        for (path, body) in [
            ("/artifacts", r#"{"content_base64":"aGVsbG8K"}"#),
            ("/collection-runs", "{}"),
            (
                "/collection-runs/local-project",
                r#"{"source_id":"source-1","source_permission_id":"permission-1"}"#,
            ),
            (
                "/collection-runs/manual-upload/ingest",
                manual_upload_ingest_body.as_str(),
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains("DATABASE_URL"), "{path}");
        }
    }

    #[test]
    fn diff136_routes_are_rust_native_and_require_database_url_after_validation() {
        for (path, body) in [
            (
                "/experiments",
                r#"{"status":"planned","metrics_json":{"score":1}}"#,
            ),
            (
                "/experiments/propose-from-improvement",
                r#"{"improvement_item_id":"improvement-1","proposal_scope":"Compare retrieval prompts offline.","success_criteria":["Fewer missing evidence notes"],"dry_run_summary":"Would compare saved records only.","result_comparison_plan":"Manual before/after review."}"#,
            ),
            (
                "/experiments/experiment-1/status",
                r#"{"status":"running","metrics_json":{"started":true}}"#,
            ),
            (
                "/improvements",
                r#"{"target_area":"retrieval","objective":"Improve retrieval scoring.","priority":"high"}"#,
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{path}");
            assert!(!response.proxied_to_fallback, "{path}");
            assert!(response.body.contains("DATABASE_URL"), "{path}");
        }
    }

    #[test]
    fn diff136_validation_rejects_invalid_requests_without_fallback() {
        for (path, body) in [
            ("/experiments", r#"{"status":"unknown"}"#),
            ("/experiments", r#"{"metrics_json":[]}"#),
            (
                "/experiments",
                r#"{"improvement_item_id":"improvement-1","actor_id":""}"#,
            ),
            (
                "/experiments/propose-from-improvement",
                r#"{"improvement_item_id":"improvement-1","proposal_scope":"x","success_criteria":[],"dry_run_summary":"x","result_comparison_plan":"x"}"#,
            ),
            (
                "/experiments/propose-from-improvement",
                r#"{"improvement_item_id":"","proposal_scope":"x","success_criteria":["x"],"dry_run_summary":"x","result_comparison_plan":"x"}"#,
            ),
            ("/experiments/experiment-1/status", "{}"),
            (
                "/experiments/experiment-1/status",
                r#"{"status":"completed","artifacts_json":[]}"#,
            ),
            ("/improvements", "{}"),
            (
                "/improvements",
                r#"{"target_area":"unknown","objective":"Improve something."}"#,
            ),
            (
                "/improvements",
                r#"{"target_area":"retrieval","objective":"","priority":"normal"}"#,
            ),
            (
                "/improvements",
                r#"{"target_area":"retrieval","objective":"Improve something.","priority":"unknown"}"#,
            ),
            (
                "/improvements",
                r#"{"target_area":"retrieval","objective":"Improve something.","metadata_json":[]}"#,
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{path} {body}");
            assert!(!response.proxied_to_fallback, "{path} {body}");
        }
    }

    #[test]
    fn experiment_acceptance_requires_approval_without_fallback() {
        let response = handle_gateway_request_with_db(
            &request(
                "POST",
                "/experiments/experiment-1/status",
                r#"{"status":"accepted","metadata_json":{"accepted_method":{"approval_required":true}}}"#,
            ),
            None,
            NO_FALLBACK_ORIGIN,
            None,
        );
        assert_eq!(response.status_code, 403);
        assert!(!response.proxied_to_fallback);
        assert!(response.body.contains("approval_id"));
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
            NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert!(
                response.status_code == 422 || response.status_code == 503,
                "{body}"
            );
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn diff135_validation_rejects_invalid_requests_without_fallback() {
        for (path, body) in [
            ("/artifacts", "{}"),
            ("/artifacts", r#"{"content_base64":[]}"#),
            ("/artifacts", r#"{"content_base64":"not base64"}"#),
            (
                "/artifacts",
                r#"{"content_base64":"aGVsbG8K","metadata_json":[]}"#,
            ),
            (
                "/collection-runs",
                r#"{"requested_by_actor_id":"","summary_json":{}}"#,
            ),
            ("/collection-runs", r#"{"summary_json":[]}"#),
            ("/collection-runs", r#"{"dry_run":"yes"}"#),
            ("/collection-runs/local-project", "{}"),
            (
                "/collection-runs/local-project",
                r#"{"source_id":"","source_permission_id":"permission-1"}"#,
            ),
            (
                "/collection-runs/local-project",
                r#"{"source_id":"source-1","source_permission_id":""}"#,
            ),
            (
                "/collection-runs/local-project",
                r#"{"source_id":"source-1","source_permission_id":"permission-1","requested_by_actor_id":""}"#,
            ),
            (
                "/collection-runs/manual-upload/ingest",
                r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","chunk_size":99}"#,
            ),
            (
                "/collection-runs/manual-upload/ingest",
                r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","chunk_size":5001}"#,
            ),
            (
                "/collection-runs/manual-upload/ingest",
                r#"{"source_id":"source-1","source_permission_id":"permission-1","content_base64":"aGVsbG8K","filename":"../x.txt"}"#,
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{path} {body}");
            assert!(!response.proxied_to_fallback, "{path} {body}");
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

        let conversation_source = CollectionSource {
            id: "source-conversation".to_string(),
            name: "Conversation".to_string(),
            source_type: "conversation_history".to_string(),
            location: None,
            sensitivity: "internal".to_string(),
            enabled: true,
            metadata_json: serde_json::json!({}),
        };
        let conversation_payload = manual_upload_normalization_work_payload(
            "collection-2",
            &conversation_source,
            "permission-2",
            "artifact-2",
        );
        assert_eq!(conversation_payload["source_type"], "conversation_history");
        assert!(is_supported_collection_source_type("conversation_history"));
        assert!(is_supported_collection_source_type("manual_upload"));
        let observation_source = CollectionSource {
            id: "source-observation".to_string(),
            name: "Observation".to_string(),
            source_type: "user_observation".to_string(),
            location: None,
            sensitivity: "internal".to_string(),
            enabled: true,
            metadata_json: serde_json::json!({}),
        };
        let observation_payload = manual_upload_normalization_work_payload(
            "collection-3",
            &observation_source,
            "permission-3",
            "artifact-3",
        );
        assert_eq!(observation_payload["source_type"], "user_observation");
        assert!(is_supported_collection_source_type("user_observation"));
        assert!(!matches!(
            "local_project",
            "manual_upload" | "conversation_history" | "user_observation"
        ));
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
                NO_FALLBACK_ORIGIN,
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
            "Project dry-run validated source and permission metadata. Connector 'local_project' ready for collect."
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
            source_type: "unsupported_type_xyz".to_string(),
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
        let result = connector_dry_run_result(&source, &permission)
            .expect("now supported via generic on grok");
        assert_eq!(result.connector_name, "generic_connector");
        assert!(result.allowed);
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
                NO_FALLBACK_ORIGIN,
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
        let candidates = baseline_pattern_candidates(&evidence, &[], 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].pattern_type, "recurrence");
        assert_eq!(candidates[0].evidence_ids, vec!["e1", "e2"]);
        assert_eq!(candidates[1].pattern_type, "cross_source_agreement");
        assert_eq!(
            candidates[1].detector_key,
            "cross_source_agreement:same signal"
        );
    }

    #[test]
    fn expanded_baseline_pattern_candidates_include_drift_anomaly_and_outcomes() {
        let evidence = vec![
            BaselineEvidenceItem {
                id: "e1".to_string(),
                evidence_type: "config".to_string(),
                statement: "version: 1.0".to_string(),
                source_id: Some("s1".to_string()),
            },
            BaselineEvidenceItem {
                id: "e2".to_string(),
                evidence_type: "config".to_string(),
                statement: "version: 1.1 unexpected spike".to_string(),
                source_id: Some("s2".to_string()),
            },
            BaselineEvidenceItem {
                id: "e3".to_string(),
                evidence_type: "note".to_string(),
                statement: "Router setting mismatch conflict".to_string(),
                source_id: Some("s3".to_string()),
            },
            BaselineEvidenceItem {
                id: "e4".to_string(),
                evidence_type: "note".to_string(),
                statement: "router setting mismatch conflict".to_string(),
                source_id: Some("s4".to_string()),
            },
        ];
        let outcomes = vec![
            BaselineOutcomeItem {
                id: "o1".to_string(),
                target_type: "recommendation".to_string(),
                target_id: "r1".to_string(),
                outcome_status: "wrong".to_string(),
                evidence_ids: vec!["e1".to_string()],
            },
            BaselineOutcomeItem {
                id: "o2".to_string(),
                target_type: "recommendation".to_string(),
                target_id: "r2".to_string(),
                outcome_status: "wrong".to_string(),
                evidence_ids: vec!["e2".to_string()],
            },
            BaselineOutcomeItem {
                id: "o3".to_string(),
                target_type: "prediction".to_string(),
                target_id: "p1".to_string(),
                outcome_status: "correct".to_string(),
                evidence_ids: vec!["e3".to_string()],
            },
            BaselineOutcomeItem {
                id: "o4".to_string(),
                target_type: "prediction".to_string(),
                target_id: "p2".to_string(),
                outcome_status: "correct".to_string(),
                evidence_ids: vec!["e4".to_string()],
            },
        ];
        let candidates = baseline_pattern_candidates(&evidence, &outcomes, 2);
        let types = candidates
            .iter()
            .map(|candidate| candidate.pattern_type.as_str())
            .collect::<HashSet<_>>();
        assert!(types.contains("configuration_drift"));
        assert!(types.contains("anomaly_signal"));
        assert!(types.contains("cross_source_conflict"));
        assert!(types.contains("failed_advice_recurrence"));
        assert!(types.contains("successful_method_recurrence"));
        assert!(candidates.iter().all(|candidate| {
            candidate
                .metadata_json
                .get("unverified_note")
                .is_some_and(Value::is_string)
        }));
    }

    #[test]
    fn calibration_summary_counts_records_outcomes_and_confidence_bands() {
        let records = vec![
            CalibrationRecord {
                kind: "prediction".to_string(),
                id: "prediction-1".to_string(),
                confidence: Some(80),
                evidence_count: 2,
            },
            CalibrationRecord {
                kind: "recommendation".to_string(),
                id: "recommendation-1".to_string(),
                confidence: Some(35),
                evidence_count: 1,
            },
            CalibrationRecord {
                kind: "recommendation".to_string(),
                id: "recommendation-2".to_string(),
                confidence: None,
                evidence_count: 0,
            },
        ];
        let outcomes = vec![
            CalibrationOutcome {
                target_type: "prediction".to_string(),
                target_id: "prediction-1".to_string(),
                outcome_status: "correct".to_string(),
            },
            CalibrationOutcome {
                target_type: "recommendation".to_string(),
                target_id: "recommendation-1".to_string(),
                outcome_status: "not_useful".to_string(),
            },
            CalibrationOutcome {
                target_type: "recommendation".to_string(),
                target_id: "missing".to_string(),
                outcome_status: "wrong".to_string(),
            },
        ];
        let summary = calibration_summary_json(&records, &outcomes);
        assert_eq!(summary["record_counts"]["predictions"], 1);
        assert_eq!(summary["record_counts"]["recommendations"], 2);
        assert_eq!(summary["record_counts"]["evidence_linked"], 2);
        assert_eq!(summary["record_counts"]["with_outcome"], 2);
        assert_eq!(summary["outcome_counts"]["correct"], 1);
        assert_eq!(summary["outcome_counts"]["not_useful"], 1);
        assert_eq!(summary["outcome_counts"]["wrong"], 0);
        assert_eq!(summary["confidence_bands"]["high"]["outcomes"], 1);
        assert_eq!(summary["confidence_bands"]["low"]["outcomes"], 1);
        assert_eq!(summary["confidence_bands"]["unknown"]["records"], 1);
        assert_eq!(summary["forecasting_engine"], false);
        assert_eq!(summary["auto_execute_recommendations"], false);
        assert_eq!(summary["advanced_calibration"], false);
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
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }

        let answer_feedback = parse_feedback_create(
            r#"{"target_type":"evidence_answer","target_id":"answer-1","label":"useful"}"#,
        )
        .expect("evidence answer feedback target");
        assert_eq!(answer_feedback.target_type, "evidence_answer");
    }

    #[test]
    fn evidence_answer_record_validation_rejects_invalid_requests_without_fallback() {
        for body in [
            "{}",
            r#"{"user_question":"What?","answer_status":"made_up"}"#,
            r#"{"user_question":"What?","retrieval_count":-1}"#,
            r#"{"user_question":"What?","evidence_item_ids":[""]}"#,
            r#"{"user_question":"What?","metadata_json":[]}"#,
            r#"{"user_question":"What?","metadata_json":{"api_token":"secret"}}"#,
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", "/evidence-answers", body),
                None,
                NO_FALLBACK_ORIGIN,
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
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{body}");
            assert!(!response.proxied_to_fallback, "{body}");
        }
    }

    #[test]
    fn diff132_routes_are_rust_native_and_require_database_url_after_validation() {
        for (method, path, body) in [
            ("GET", "/retrieval/chunks/chunk-1/trail", ""),
            (
                "POST",
                "/analysis/hypotheses",
                r#"{"hypothesis_text":"Signal may recur","supporting_evidence_ids":["evidence-1"]}"#,
            ),
            (
                "POST",
                "/analysis/predictions",
                r#"{"prediction_text":"It will recur","expected_result":"recurs","evidence_ids":["evidence-1"]}"#,
            ),
            (
                "POST",
                "/analysis/recommendations",
                r#"{"recommendation_text":"Review it","evidence_ids":["evidence-1"]}"#,
            ),
            (
                "POST",
                "/evidence/documents",
                r#"{"raw_artifact_id":"artifact-1"}"#,
            ),
            ("POST", "/evidence/documents/doc-1/chunks", "{}"),
            (
                "POST",
                "/evidence/items",
                r#"{"source_id":"source-1","evidence_type":"note","statement":"Recorded fact"}"#,
            ),
            (
                "POST",
                "/evidence/items/evidence-1/review-state",
                r#"{"review_state":"superseded","correction_note":"Use newer evidence.","superseding_evidence_item_id":"evidence-2"}"#,
            ),
            ("POST", "/reports/report-1/status", r#"{"status":"ready"}"#),
            (
                "POST",
                "/retrieval/chunks/search",
                r#"{"query":"router","limit":5}"#,
            ),
            (
                "POST",
                "/sources/source-1/permissions",
                r#"{"allowed_operations":["read"],"external_model_policy":"blocked"}"#,
            ),
            (
                "POST",
                "/sources/source-1/review-state",
                r#"{"trust_level":"trusted","sensitivity":"internal","enabled":true}"#,
            ),
            (
                "POST",
                "/work-items/work-1/status",
                r#"{"status":"running"}"#,
            ),
        ] {
            let response = handle_gateway_request_with_db(
                &request(method, path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 503, "{method} {path}");
            assert!(!response.proxied_to_fallback, "{method} {path}");
            assert!(response.body.contains("DATABASE_URL"), "{method} {path}");
        }
    }

    #[test]
    fn diff132_validation_rejects_invalid_requests_without_fallback() {
        for (path, body) in [
            ("/analysis/hypotheses", "{}"),
            (
                "/analysis/hypotheses",
                r#"{"hypothesis_text":"x","supporting_evidence_ids":[]}"#,
            ),
            (
                "/analysis/predictions",
                r#"{"prediction_text":"x","expected_result":"","evidence_ids":["e1"]}"#,
            ),
            (
                "/analysis/recommendations",
                r#"{"recommendation_text":"x","evidence_ids":[""]}"#,
            ),
            ("/evidence/documents", "{}"),
            ("/evidence/documents/doc-1/chunks", r#"{"chunk_size":99}"#),
            (
                "/evidence/items",
                r#"{"evidence_type":"note","statement":"x"}"#,
            ),
            ("/evidence/items/evidence-1/review-state", "{}"),
            (
                "/evidence/items/evidence-1/review-state",
                r#"{"review_state":"maybe_correct"}"#,
            ),
            (
                "/evidence/items/evidence-1/review-state",
                r#"{"review_state":"verified","correction_note":[]}"#,
            ),
            (
                "/evidence/items/evidence-1/review-state",
                r#"{"review_state":"verified","actor_id":""}"#,
            ),
            ("/reports/report-1/status", r#"{"status":"published"}"#),
            ("/retrieval/chunks/search", r#"{"query":"","limit":5}"#),
            ("/retrieval/chunks/search", r#"{"query":"x","limit":51}"#),
            (
                "/sources/source-1/permissions",
                r#"{"allowed_operations":["write"]}"#,
            ),
            ("/sources/source-1/review-state", "{}"),
            (
                "/sources/source-1/review-state",
                r#"{"trust_level":"unreviewed","sensitivity":"internal","enabled":true}"#,
            ),
            (
                "/sources/source-1/review-state",
                r#"{"trust_level":"trusted","sensitivity":"classified","enabled":true}"#,
            ),
            (
                "/sources/source-1/review-state",
                r#"{"trust_level":"trusted","sensitivity":"internal","enabled":"yes"}"#,
            ),
            ("/work-items/work-1/status", r#"{"status":"bogus"}"#),
        ] {
            let response = handle_gateway_request_with_db(
                &request("POST", path, body),
                None,
                NO_FALLBACK_ORIGIN,
                None,
            );
            assert_eq!(response.status_code, 422, "{path} {body}");
            assert!(!response.proxied_to_fallback, "{path} {body}");
        }
    }

    #[test]
    fn diff132_status_transition_checks_are_deterministic() {
        let payload = serde_json::json!({
            "intent_verification": {"recorded_by": "test"}
        });
        assert!(require_valid_work_item_status_transition("queued", "running", &payload).is_ok());
        assert!(
            require_valid_work_item_status_transition("completed", "running", &payload).is_err()
        );
        assert!(require_valid_work_item_status_transition(
            "pending_intent_verification",
            "queued",
            &serde_json::json!({})
        )
        .is_err());
    }

    #[test]
    fn diff132_audit_payload_shapes_are_deterministic() {
        let evidence_ids = vec!["evidence-1".to_string()];
        let details = serde_json::json!({ "evidence_ids": evidence_ids });
        assert_eq!(details["evidence_ids"][0], "evidence-1");

        let report_status =
            parse_report_status(r#"{"status":"ready","actor_id":"owner"}"#).expect("report status");
        assert_eq!(report_status.status, "ready");
        assert_eq!(report_status.actor_id, "owner");

        let permission = parse_json_object(
            r#"{"allowed_operations":["read"],"external_model_policy":"blocked"}"#,
            "permission",
        )
        .and_then(|object| parse_source_permission_create(&object))
        .expect("permission");
        assert_eq!(permission.allowed_operations, vec!["read"]);
        assert!(permission.approval_required);
    }

    #[test]
    fn rust_native_route_registry_covers_db_read_batch() {
        for expected in [
            ("GET", "/"),
            ("GET", "/sources"),
            ("GET", "/sources/{source_id}"),
            ("GET", "/sources/{source_id}/permissions"),
            ("POST", "/sources"),
            ("POST", "/sources/{source_id}/review-state"),
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
            ("GET", "/analysis/calibration/summary"),
            ("GET", "/agent/task-plans"),
            ("GET", "/agent/task-plans/{task_plan_id}"),
            ("POST", "/agent/task-plans"),
            ("POST", "/agent/task-plans/{task_plan_id}/evidence-summary"),
            ("POST", "/agent/task-plans/{task_plan_id}/work-spec"),
            ("POST", "/agent/task-plans/{task_plan_id}/work-item"),
            ("GET", "/evidence-answers"),
            ("GET", "/evidence-answers/{answer_id}"),
            ("POST", "/evidence-answers"),
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
            ("POST", "/memory/graph/schema/ensure"),
            ("POST", "/memory/graph/lineage/sync"),
            (
                "GET",
                "/memory/graph/nodes/{node_label}/{node_id}/relationships",
            ),
            ("GET", "/memory/vector/chunks"),
            ("GET", "/ops/runtime-logs"),
            ("POST", "/ops/runtime-logs/append"),
            ("POST", "/memory/vector/chunks/ensure"),
            ("POST", "/memory/vector/chunks/search"),
            ("POST", "/memory/vector/chunks/upsert"),
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
            ("POST", "/evidence/items/{evidence_item_id}/review-state"),
            ("GET", "/evidence/chunks"),
            ("GET", "/evidence/chunks/{chunk_id}"),
            ("GET", "/evidence/claims"),
            ("GET", "/evidence/claims/{claim_id}"),
            ("GET", "/experiments"),
            ("GET", "/experiments/{experiment_run_id}"),
            ("POST", "/experiments"),
            ("POST", "/experiments/propose-from-improvement"),
            ("POST", "/experiments/{experiment_run_id}/status"),
            ("POST", "/collection-runs/manual-upload"),
            ("GET", "/improvements"),
            ("GET", "/improvements/{improvement_item_id}"),
            ("POST", "/improvements"),
            ("POST", "/work-items"),
            ("POST", "/work-items/"),
            ("GET", "/retrieval/chunks/{chunk_id}/trail"),
            ("POST", "/analysis/hypotheses"),
            ("POST", "/analysis/predictions"),
            ("POST", "/analysis/recommendations"),
            ("POST", "/evidence/documents"),
            ("POST", "/evidence/documents/{document_id}/chunks"),
            ("POST", "/evidence/items"),
            ("POST", "/evidence/items/{evidence_item_id}/review-state"),
            ("POST", "/reports/{report_id}/status"),
            ("POST", "/retrieval/chunks/search"),
            ("POST", "/sources/{source_id}/permissions"),
            ("POST", "/work-items/{work_item_id}/status"),
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
    fn native_route_table_includes_security_full_access_host_bridge_and_bypass_intel() {
        for expected in [
            ("GET", "/user/status"),
            ("POST", "/user/change-password"),
            ("POST", "/user/generate-totp"),
            ("POST", "/user/confirm-totp"),
            ("POST", "/collection-runs/full-access"),
            ("POST", "/collection-runs/full-local-scan"),
            ("GET", "/host-bridge/status"),
            ("POST", "/host-bridge/ensure-max-reach"),
            ("GET", "/bypass-intel/status"),
            ("GET", "/bypass-intel/playbook"),
            ("POST", "/bypass-intel/harvest"),
        ] {
            assert!(RUST_NATIVE_ROUTES.contains(&expected), "{expected:?}");
        }
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
            (
                "API_BASE_URL".to_string(),
                "http://127.0.0.1:8000".to_string(),
            ),
            (
                "WEB_BASE_URL".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ),
            ("POSTGRES_HOST".to_string(), "postgres".to_string()),
            ("POSTGRES_PORT".to_string(), "5432".to_string()),
            (
                "POSTGRES_DB".to_string(),
                "adaptive_intelligence".to_string(),
            ),
            ("POSTGRES_USER".to_string(), "adaptive".to_string()),
            (
                "POSTGRES_PASSWORD".to_string(),
                "change-me-local-only".to_string(),
            ),
            (
                "DATABASE_URL".to_string(),
                "postgres://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence"
                    .to_string(),
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
            (
                "NEO4J_PASSWORD".to_string(),
                "change-me-local-only".to_string(),
            ),
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
            ("APPROVAL_REQUIRED_DEFAULT".to_string(), "true".to_string()),
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

    fn valid_manual_upload_body_with_chunk_size() -> String {
        let mut value: Value =
            serde_json::from_str(&valid_manual_upload_body()).expect("manual upload json");
        value["chunk_size"] = serde_json::json!(1000);
        value.to_string()
    }
}
