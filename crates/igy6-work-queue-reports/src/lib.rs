use std::fmt;

use igy6_write_api::{AuditEventDraft, WriteApiError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkQueueReportError {
    WriteApi(WriteApiError),
    EmptyValue { field: &'static str },
    InvalidWorkItemStatus(String),
    InvalidWorkItemTransition { from: String, attempted: String },
    MissingIntentVerification { action: String },
    UnsupportedDispatchType(String),
    InvalidDispatchPayload { work_type: String, reason: String },
    InvalidReportType(String),
    InvalidReportStatus(String),
}

impl fmt::Display for WorkQueueReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteApi(error) => write!(formatter, "{error}"),
            Self::EmptyValue { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidWorkItemStatus(value) => {
                write!(formatter, "unknown work item status: {value}")
            }
            Self::InvalidWorkItemTransition { from, attempted } => {
                write!(
                    formatter,
                    "invalid work item status transition from {from} to {attempted}"
                )
            }
            Self::MissingIntentVerification { action } => {
                write!(
                    formatter,
                    "work item requires recorded intent verification before {action}"
                )
            }
            Self::UnsupportedDispatchType(value) => {
                write!(formatter, "unsupported work item dispatch type: {value}")
            }
            Self::InvalidDispatchPayload { work_type, reason } => {
                write!(formatter, "invalid {work_type} dispatch payload: {reason}")
            }
            Self::InvalidReportType(value) => write!(formatter, "unknown report type: {value}"),
            Self::InvalidReportStatus(value) => write!(formatter, "unknown report status: {value}"),
        }
    }
}

impl std::error::Error for WorkQueueReportError {}

impl From<WriteApiError> for WorkQueueReportError {
    fn from(value: WriteApiError) -> Self {
        Self::WriteApi(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkItemStatus {
    PendingIntentVerification,
    Queued,
    Running,
    Completed,
    Failed,
    Canceled,
}

impl WorkItemStatus {
    pub fn parse(value: &str) -> Result<Self, WorkQueueReportError> {
        match value {
            "pending_intent_verification" => Ok(Self::PendingIntentVerification),
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "canceled" => Ok(Self::Canceled),
            _ => Err(WorkQueueReportError::InvalidWorkItemStatus(
                value.to_string(),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingIntentVerification => "pending_intent_verification",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        if self == next {
            return true;
        }
        matches!(
            (self, next),
            (Self::PendingIntentVerification, Self::Queued)
                | (Self::PendingIntentVerification, Self::Canceled)
                | (Self::Queued, Self::Running)
                | (Self::Queued, Self::Canceled)
                | (Self::Running, Self::Completed)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::Canceled)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentVerificationContext {
    pub original_request: String,
    pub interpretation: String,
    pub proposed_work_type: String,
    pub expected_output: String,
    pub safety_requirements: Vec<String>,
    pub assumptions: Vec<String>,
    pub missing_information: Vec<String>,
    pub sources_likely_used: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemCreateRequest {
    pub id: String,
    pub work_type: String,
    pub requested_by_actor_id: String,
    pub intent: IntentVerificationContext,
    pub payload_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemDraft {
    pub id: String,
    pub work_type: String,
    pub status: WorkItemStatus,
    pub requested_by_actor_id: String,
    pub payload_json: Vec<(String, String)>,
    pub intent_verification: IntentVerificationContext,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemCreatePlan {
    pub work_item: WorkItemDraft,
    pub audit_event: AuditEventDraft,
}

pub fn plan_work_item_create(
    payload: WorkItemCreateRequest,
) -> Result<WorkItemCreatePlan, WorkQueueReportError> {
    let id = non_empty_string("work_item.id", payload.id)?;
    let work_type = non_empty_string("work_item.work_type", payload.work_type)?;
    let requested_by_actor_id = non_empty_string(
        "work_item.requested_by_actor_id",
        payload.requested_by_actor_id,
    )?;
    validate_intent(&payload.intent)?;
    validate_pairs("work_item.payload_json key", &payload.payload_json)?;

    let work_item = WorkItemDraft {
        id: id.clone(),
        work_type: work_type.clone(),
        status: WorkItemStatus::PendingIntentVerification,
        requested_by_actor_id: requested_by_actor_id.clone(),
        payload_json: payload.payload_json,
        intent_verification: payload.intent,
        error_message: None,
    };
    let audit_event = AuditEventDraft::new(&requested_by_actor_id, "work_item.created")?
        .decision("intent_verification_required")?
        .resource("work_item", &id)?
        .details(vec![
            ("work_type".to_string(), work_type),
            (
                "status".to_string(),
                WorkItemStatus::PendingIntentVerification
                    .as_str()
                    .to_string(),
            ),
        ])?;

    Ok(WorkItemCreatePlan {
        work_item,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemStatusUpdateRequest {
    pub status: String,
    pub actor_id: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemStatusUpdatePlan {
    pub status: WorkItemStatus,
    pub error_message: Option<String>,
    pub audit_event: AuditEventDraft,
}

pub fn plan_work_item_status_update(
    work_item: &WorkItemDraft,
    payload: WorkItemStatusUpdateRequest,
) -> Result<WorkItemStatusUpdatePlan, WorkQueueReportError> {
    validate_non_empty("work_item.id", &work_item.id)?;
    let next_status = WorkItemStatus::parse(&payload.status)?;
    let actor_id = non_empty_string("work_item.actor_id", payload.actor_id)?;
    if !work_item.status.can_transition_to(&next_status) {
        return Err(WorkQueueReportError::InvalidWorkItemTransition {
            from: work_item.status.as_str().to_string(),
            attempted: next_status.as_str().to_string(),
        });
    }
    if next_status == WorkItemStatus::Queued && !has_intent_verification(work_item) {
        return Err(WorkQueueReportError::MissingIntentVerification {
            action: "queueing".to_string(),
        });
    }
    let audit_event = AuditEventDraft::new(&actor_id, "work_item.status_updated")?
        .decision(next_status.as_str())?
        .resource("work_item", &work_item.id)?
        .details(vec![
            (
                "previous_status".to_string(),
                work_item.status.as_str().to_string(),
            ),
            ("new_status".to_string(), next_status.as_str().to_string()),
            (
                "error_message".to_string(),
                payload.error_message.clone().unwrap_or_default(),
            ),
        ])?;

    Ok(WorkItemStatusUpdatePlan {
        status: next_status,
        error_message: payload.error_message,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchPayload {
    pub collection_run_id: Option<String>,
    pub raw_artifact_ids: Vec<String>,
    pub document_ids: Vec<String>,
    pub document_id: Option<String>,
    pub chunk_size: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchTaskPlan {
    pub task_name: String,
    pub args: Vec<String>,
    pub kwargs: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemDispatchRequest {
    pub actor_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemDispatchResult {
    pub work_item_id: String,
    pub work_type: String,
    pub task_name: String,
    pub task_id: String,
    pub status: WorkItemStatus,
    pub dispatch_metadata: Vec<(String, String)>,
    pub audit_event: AuditEventDraft,
}

pub fn build_dispatch_plan(
    work_item_id: &str,
    work_type: &str,
    payload: &DispatchPayload,
) -> Result<DispatchTaskPlan, WorkQueueReportError> {
    validate_non_empty("work_item.id", work_item_id)?;
    validate_non_empty("work_item.work_type", work_type)?;
    match work_type {
        "collection_normalization" => {
            let collection_run_id = payload.collection_run_id.as_deref().ok_or_else(|| {
                WorkQueueReportError::InvalidDispatchPayload {
                    work_type: work_type.to_string(),
                    reason: "collection_run_id is required".to_string(),
                }
            })?;
            validate_non_empty("collection_run_id", collection_run_id)?;
            let raw_artifact_ids =
                dedupe_non_empty("raw_artifact_ids", payload.raw_artifact_ids.clone())?;
            if raw_artifact_ids.is_empty() {
                return Err(WorkQueueReportError::InvalidDispatchPayload {
                    work_type: work_type.to_string(),
                    reason: "raw_artifact_ids must not be empty".to_string(),
                });
            }
            Ok(DispatchTaskPlan {
                task_name: "collection.normalize_collection_run".to_string(),
                args: vec![
                    work_item_id.to_string(),
                    collection_run_id.to_string(),
                    raw_artifact_ids.join(","),
                ],
                kwargs: Vec::new(),
            })
        }
        "document_chunking" => {
            let mut document_ids = payload.document_ids.clone();
            if document_ids.is_empty() {
                if let Some(document_id) = &payload.document_id {
                    document_ids.push(document_id.clone());
                }
            }
            let document_ids = dedupe_non_empty("document_ids", document_ids)?;
            if document_ids.is_empty() {
                return Err(WorkQueueReportError::InvalidDispatchPayload {
                    work_type: work_type.to_string(),
                    reason: "document_ids or document_id is required".to_string(),
                });
            }
            Ok(DispatchTaskPlan {
                task_name: "evidence.generate_document_chunks".to_string(),
                args: vec![document_ids.join(",")],
                kwargs: vec![
                    (
                        "chunk_size".to_string(),
                        payload.chunk_size.unwrap_or(1000).to_string(),
                    ),
                    ("work_item_id".to_string(), work_item_id.to_string()),
                ],
            })
        }
        "chunk_vector_upsert" => Ok(DispatchTaskPlan {
            task_name: "memory.vector.upsert_chunks".to_string(),
            args: Vec::new(),
            kwargs: vec![
                (
                    "limit".to_string(),
                    payload.limit.unwrap_or(100).to_string(),
                ),
                ("work_item_id".to_string(), work_item_id.to_string()),
            ],
        }),
        _ => Err(WorkQueueReportError::UnsupportedDispatchType(
            work_type.to_string(),
        )),
    }
}

pub fn plan_work_item_dispatch(
    work_item: &WorkItemDraft,
    payload: &DispatchPayload,
    request: WorkItemDispatchRequest,
) -> Result<WorkItemDispatchResult, WorkQueueReportError> {
    if work_item.status != WorkItemStatus::Queued {
        return Err(WorkQueueReportError::InvalidWorkItemTransition {
            from: work_item.status.as_str().to_string(),
            attempted: "dispatch".to_string(),
        });
    }
    if !has_intent_verification(work_item) {
        return Err(WorkQueueReportError::MissingIntentVerification {
            action: "dispatch".to_string(),
        });
    }
    let actor_id = non_empty_string("dispatch.actor_id", request.actor_id)?;
    let task_id = non_empty_string("dispatch.task_id", request.task_id)?;
    let task = build_dispatch_plan(&work_item.id, &work_item.work_type, payload)?;
    let dispatch_metadata = vec![
        ("task_name".to_string(), task.task_name.clone()),
        ("task_id".to_string(), task_id.clone()),
        ("dispatched_by_actor_id".to_string(), actor_id.clone()),
    ];
    let audit_event = AuditEventDraft::new(&actor_id, "work_item.dispatched")?
        .decision("dispatched")?
        .resource("work_item", &work_item.id)?
        .correlation_id(&task_id)?
        .details(vec![
            ("work_type".to_string(), work_item.work_type.clone()),
            ("task_name".to_string(), task.task_name.clone()),
            ("task_id".to_string(), task_id.clone()),
        ])?;

    Ok(WorkItemDispatchResult {
        work_item_id: work_item.id.clone(),
        work_type: work_item.work_type.clone(),
        task_name: task.task_name,
        task_id,
        status: work_item.status.clone(),
        dispatch_metadata,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportType {
    Summary,
    EvidenceReview,
    DecisionNote,
    Handoff,
    ExperimentSummary,
}

impl ReportType {
    pub fn parse(value: &str) -> Result<Self, WorkQueueReportError> {
        match value {
            "summary" => Ok(Self::Summary),
            "evidence_review" => Ok(Self::EvidenceReview),
            "decision_note" => Ok(Self::DecisionNote),
            "handoff" => Ok(Self::Handoff),
            "experiment_summary" => Ok(Self::ExperimentSummary),
            _ => Err(WorkQueueReportError::InvalidReportType(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::EvidenceReview => "evidence_review",
            Self::DecisionNote => "decision_note",
            Self::Handoff => "handoff",
            Self::ExperimentSummary => "experiment_summary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportStatus {
    Placeholder,
    Requested,
    Draft,
    Ready,
    Archived,
}

impl ReportStatus {
    pub fn parse(value: &str) -> Result<Self, WorkQueueReportError> {
        match value {
            "placeholder" => Ok(Self::Placeholder),
            "requested" => Ok(Self::Requested),
            "draft" => Ok(Self::Draft),
            "ready" => Ok(Self::Ready),
            "archived" => Ok(Self::Archived),
            _ => Err(WorkQueueReportError::InvalidReportStatus(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Placeholder => "placeholder",
            Self::Requested => "requested",
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCreateRequest {
    pub id: String,
    pub title: String,
    pub report_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub artifact_path: Option<String>,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDraft {
    pub id: String,
    pub title: String,
    pub report_type: ReportType,
    pub status: ReportStatus,
    pub requested_by_actor_id: String,
    pub artifact_path: Option<String>,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCreatePlan {
    pub report: ReportDraft,
    pub audit_event: AuditEventDraft,
}

pub fn plan_report_create(
    payload: ReportCreateRequest,
) -> Result<ReportCreatePlan, WorkQueueReportError> {
    let id = non_empty_string("report.id", payload.id)?;
    let title = non_empty_string("report.title", payload.title)?;
    let report_type = ReportType::parse(&payload.report_type)?;
    let status = ReportStatus::parse(&payload.status)?;
    let requested_by_actor_id = non_empty_string(
        "report.requested_by_actor_id",
        payload.requested_by_actor_id,
    )?;
    validate_pairs("report.metadata_json key", &payload.metadata_json)?;
    let report = ReportDraft {
        id: id.clone(),
        title,
        report_type,
        status,
        requested_by_actor_id: requested_by_actor_id.clone(),
        artifact_path: payload.artifact_path,
        metadata_json: payload.metadata_json,
    };
    let audit_event = AuditEventDraft::new(&requested_by_actor_id, "report.created")?
        .decision("recorded")?
        .resource("report", &id)?
        .details(vec![
            (
                "report_type".to_string(),
                report.report_type.as_str().to_string(),
            ),
            ("status".to_string(), report.status.as_str().to_string()),
        ])?;

    Ok(ReportCreatePlan {
        report,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportStatusUpdateRequest {
    pub status: String,
    pub actor_id: String,
    pub artifact_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportStatusUpdatePlan {
    pub status: ReportStatus,
    pub artifact_path: Option<String>,
    pub audit_event: AuditEventDraft,
}

pub fn plan_report_status_update(
    report: &ReportDraft,
    payload: ReportStatusUpdateRequest,
) -> Result<ReportStatusUpdatePlan, WorkQueueReportError> {
    let status = ReportStatus::parse(&payload.status)?;
    let actor_id = non_empty_string("report.actor_id", payload.actor_id)?;
    let next_artifact_path = payload
        .artifact_path
        .clone()
        .or_else(|| report.artifact_path.clone());
    let audit_event = AuditEventDraft::new(&actor_id, "report.status_updated")?
        .decision(status.as_str())?
        .resource("report", &report.id)?
        .details(vec![
            (
                "previous_status".to_string(),
                report.status.as_str().to_string(),
            ),
            ("new_status".to_string(), status.as_str().to_string()),
            (
                "previous_artifact_path".to_string(),
                report.artifact_path.clone().unwrap_or_default(),
            ),
            (
                "new_artifact_path".to_string(),
                next_artifact_path.clone().unwrap_or_default(),
            ),
        ])?;

    Ok(ReportStatusUpdatePlan {
        status,
        artifact_path: next_artifact_path,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportWorkItemCreateRequest {
    pub work_item_id: String,
    pub requested_by_actor_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportWorkItemPlan {
    pub work_item: WorkItemDraft,
    pub audit_event: AuditEventDraft,
}

pub fn plan_report_work_item_create(
    report: &ReportDraft,
    payload: ReportWorkItemCreateRequest,
) -> Result<ReportWorkItemPlan, WorkQueueReportError> {
    let work_item_id = non_empty_string("work_item.id", payload.work_item_id)?;
    let requested_by_actor_id = non_empty_string(
        "work_item.requested_by_actor_id",
        payload.requested_by_actor_id,
    )?;
    let intent = IntentVerificationContext {
        original_request: format!("Generate report {}", report.id),
        interpretation: format!("Create a local metadata report for {}", report.title),
        proposed_work_type: "report_generation".to_string(),
        expected_output: "Rendered local markdown report artifact.".to_string(),
        safety_requirements: vec![
            "Use local metadata only.".to_string(),
            "Do not read raw artifact contents.".to_string(),
            "Do not call external models or services.".to_string(),
        ],
        assumptions: vec!["Report metadata already exists.".to_string()],
        missing_information: Vec::new(),
        sources_likely_used: vec!["local metadata records".to_string()],
    };
    let work_item = WorkItemDraft {
        id: work_item_id.clone(),
        work_type: "report_generation".to_string(),
        status: WorkItemStatus::Queued,
        requested_by_actor_id: requested_by_actor_id.clone(),
        payload_json: vec![
            ("report_id".to_string(), report.id.clone()),
            (
                "report_type".to_string(),
                report.report_type.as_str().to_string(),
            ),
            (
                "report_status".to_string(),
                report.status.as_str().to_string(),
            ),
            ("scaffold_only".to_string(), "true".to_string()),
            (
                "executes_report_generation".to_string(),
                "false".to_string(),
            ),
            ("notes".to_string(), payload.notes.unwrap_or_default()),
        ],
        intent_verification: intent,
        error_message: None,
    };
    let audit_event = AuditEventDraft::new(&requested_by_actor_id, "work_item.created")?
        .decision("queued")?
        .resource("work_item", &work_item_id)?
        .correlation_id(&report.id)?
        .details(vec![
            ("work_type".to_string(), work_item.work_type.clone()),
            (
                "status".to_string(),
                WorkItemStatus::Queued.as_str().to_string(),
            ),
            ("report_id".to_string(), report.id.clone()),
            (
                "report_type".to_string(),
                report.report_type.as_str().to_string(),
            ),
            ("scaffold_only".to_string(), "true".to_string()),
        ])?;

    Ok(ReportWorkItemPlan {
        work_item,
        audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportInventory {
    pub counts: Vec<(String, usize)>,
    pub recent_records: Vec<RecentRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentRecord {
    pub label: String,
    pub value: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRenderRequest {
    pub actor_id: String,
    pub notes: Option<String>,
    pub artifact_id: String,
    pub artifact_path: String,
    pub content_hash: String,
    pub content_already_existed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRenderPlan {
    pub markdown: String,
    pub report_status: ReportStatus,
    pub artifact_path: String,
    pub report_metadata: Vec<(String, String)>,
    pub artifact_metadata: Vec<(String, String)>,
    pub audit_event: AuditEventDraft,
}

pub fn render_report_markdown(
    report: &ReportDraft,
    inventory: &ReportInventory,
    notes: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        format!("- Report ID: `{}`", report.id),
        format!("- Report type: `{}`", report.report_type.as_str()),
        format!("- Requested by: `{}`", report.requested_by_actor_id),
        format!("- Status before render: `{}`", report.status.as_str()),
        String::new(),
        "## Inventory Counts".to_string(),
        String::new(),
    ];

    let mut counts = inventory.counts.clone();
    counts.sort_by(|left, right| left.0.cmp(&right.0));
    for (key, value) in counts {
        lines.push(format!("- {key}: {value}"));
    }

    lines.extend([
        String::new(),
        "## Recent Records".to_string(),
        String::new(),
    ]);
    if inventory.recent_records.is_empty() {
        lines.push("- No recent records available.".to_string());
    } else {
        for record in inventory.recent_records.iter().take(20) {
            let suffix = record
                .status
                .as_ref()
                .map(|status| format!(" [{status}]"))
                .unwrap_or_default();
            lines.push(format!("- {}: {}{}", record.label, record.value, suffix));
        }
    }

    lines.extend([
        String::new(),
        "## Boundaries".to_string(),
        String::new(),
        "- This report is generated from local metadata records only.".to_string(),
        "- It does not read raw artifact contents.".to_string(),
        "- It does not call external models or execute actions.".to_string(),
    ]);
    if let Some(notes) = notes {
        if !notes.trim().is_empty() {
            lines.extend([
                String::new(),
                "## Notes".to_string(),
                String::new(),
                notes.to_string(),
            ]);
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn plan_report_render(
    report: &ReportDraft,
    inventory: &ReportInventory,
    payload: ReportRenderRequest,
) -> Result<ReportRenderPlan, WorkQueueReportError> {
    let actor_id = non_empty_string("report.actor_id", payload.actor_id)?;
    let artifact_id = non_empty_string("artifact.id", payload.artifact_id)?;
    let artifact_path = non_empty_string("artifact.path", payload.artifact_path)?;
    let content_hash = non_empty_string("artifact.content_hash", payload.content_hash)?;
    let markdown = render_report_markdown(report, inventory, payload.notes.as_deref());
    let report_metadata = vec![
        ("rendered_artifact_id".to_string(), artifact_id.clone()),
        (
            "rendered_mime_type".to_string(),
            "text/markdown".to_string(),
        ),
    ];
    let artifact_metadata = vec![
        ("generated_by".to_string(), "DIFF-101".to_string()),
        ("artifact_kind".to_string(), "report".to_string()),
        ("report_id".to_string(), report.id.clone()),
        (
            "report_type".to_string(),
            report.report_type.as_str().to_string(),
        ),
        ("filename".to_string(), format!("{}.md", report.id)),
    ];
    let audit_event = AuditEventDraft::new(&actor_id, "report.rendered")?
        .decision("ready")?
        .resource("report", &report.id)?
        .correlation_id(&artifact_id)?
        .details(vec![
            ("artifact_id".to_string(), artifact_id),
            ("artifact_path".to_string(), artifact_path.clone()),
            ("content_hash".to_string(), content_hash),
            (
                "content_already_existed".to_string(),
                payload.content_already_existed.to_string(),
            ),
        ])?;

    Ok(ReportRenderPlan {
        markdown,
        report_status: ReportStatus::Ready,
        artifact_path,
        report_metadata,
        artifact_metadata,
        audit_event,
    })
}

fn has_intent_verification(work_item: &WorkItemDraft) -> bool {
    !work_item
        .intent_verification
        .original_request
        .trim()
        .is_empty()
        && !work_item
            .intent_verification
            .interpretation
            .trim()
            .is_empty()
        && !work_item
            .intent_verification
            .proposed_work_type
            .trim()
            .is_empty()
        && !work_item
            .intent_verification
            .expected_output
            .trim()
            .is_empty()
}

fn validate_intent(intent: &IntentVerificationContext) -> Result<(), WorkQueueReportError> {
    validate_non_empty("intent.original_request", &intent.original_request)?;
    validate_non_empty("intent.interpretation", &intent.interpretation)?;
    validate_non_empty("intent.proposed_work_type", &intent.proposed_work_type)?;
    validate_non_empty("intent.expected_output", &intent.expected_output)?;
    validate_string_list("intent.safety_requirements", &intent.safety_requirements)?;
    validate_string_list("intent.assumptions", &intent.assumptions)?;
    validate_string_list("intent.missing_information", &intent.missing_information)?;
    validate_string_list("intent.sources_likely_used", &intent.sources_likely_used)
}

fn validate_string_list(
    field: &'static str,
    values: &[String],
) -> Result<(), WorkQueueReportError> {
    for value in values {
        validate_non_empty(field, value)?;
    }
    Ok(())
}

fn validate_pairs(
    field: &'static str,
    pairs: &[(String, String)],
) -> Result<(), WorkQueueReportError> {
    for (key, _) in pairs {
        validate_non_empty(field, key)?;
    }
    Ok(())
}

fn dedupe_non_empty(
    field: &'static str,
    values: Vec<String>,
) -> Result<Vec<String>, WorkQueueReportError> {
    let mut unique = Vec::new();
    for value in values {
        let value = non_empty_string(field, value)?;
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    Ok(unique)
}

fn non_empty_string(field: &'static str, value: String) -> Result<String, WorkQueueReportError> {
    validate_non_empty(field, &value)?;
    Ok(value)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), WorkQueueReportError> {
    if value.trim().is_empty() {
        Err(WorkQueueReportError::EmptyValue { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use igy6_write_api::LOCAL_OWNER;

    #[test]
    fn work_item_create_requires_and_records_intent_verification() {
        let plan = plan_work_item_create(WorkItemCreateRequest {
            id: "work-1".to_string(),
            work_type: "document_chunking".to_string(),
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            intent: base_intent(),
            payload_json: vec![("document_id".to_string(), "doc-1".to_string())],
        })
        .expect("work item plan");

        assert_eq!(
            plan.work_item.status,
            WorkItemStatus::PendingIntentVerification
        );
        assert_eq!(plan.audit_event.event_type, "work_item.created");
        assert_eq!(
            plan.audit_event.decision.as_deref(),
            Some("intent_verification_required")
        );
    }

    #[test]
    fn work_item_create_rejects_missing_intent_fields() {
        let mut intent = base_intent();
        intent.original_request = " ".to_string();
        let error = plan_work_item_create(WorkItemCreateRequest {
            id: "work-1".to_string(),
            work_type: "document_chunking".to_string(),
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            intent,
            payload_json: Vec::new(),
        })
        .expect_err("missing intent is rejected");
        assert!(matches!(
            error,
            WorkQueueReportError::EmptyValue {
                field: "intent.original_request"
            }
        ));
    }

    #[test]
    fn work_item_status_transitions_are_validated() {
        let work_item = base_work_item(
            "document_chunking",
            WorkItemStatus::PendingIntentVerification,
        );
        let queued = plan_work_item_status_update(
            &work_item,
            WorkItemStatusUpdateRequest {
                status: "queued".to_string(),
                actor_id: LOCAL_OWNER.to_string(),
                error_message: None,
            },
        )
        .expect("queue transition");
        assert_eq!(queued.status, WorkItemStatus::Queued);

        let completed_from_pending = plan_work_item_status_update(
            &work_item,
            WorkItemStatusUpdateRequest {
                status: "completed".to_string(),
                actor_id: LOCAL_OWNER.to_string(),
                error_message: None,
            },
        )
        .expect_err("invalid transition");
        assert!(matches!(
            completed_from_pending,
            WorkQueueReportError::InvalidWorkItemTransition { .. }
        ));
    }

    #[test]
    fn work_item_queueing_requires_intent_verification() {
        let mut work_item = base_work_item(
            "document_chunking",
            WorkItemStatus::PendingIntentVerification,
        );
        work_item.intent_verification.original_request.clear();
        let error = plan_work_item_status_update(
            &work_item,
            WorkItemStatusUpdateRequest {
                status: "queued".to_string(),
                actor_id: LOCAL_OWNER.to_string(),
                error_message: None,
            },
        )
        .expect_err("intent verification required");
        assert!(matches!(
            error,
            WorkQueueReportError::MissingIntentVerification { .. }
        ));
    }

    #[test]
    fn dispatch_plan_matches_python_task_shapes() {
        let normalization = build_dispatch_plan(
            "work-1",
            "collection_normalization",
            &DispatchPayload {
                collection_run_id: Some("run-1".to_string()),
                raw_artifact_ids: vec!["raw-1".to_string(), "raw-1".to_string()],
                document_ids: Vec::new(),
                document_id: None,
                chunk_size: None,
                limit: None,
            },
        )
        .expect("normalization dispatch");
        assert_eq!(
            normalization.task_name,
            "collection.normalize_collection_run"
        );
        assert_eq!(normalization.args[2], "raw-1");

        let chunking = build_dispatch_plan(
            "work-2",
            "document_chunking",
            &DispatchPayload {
                collection_run_id: None,
                raw_artifact_ids: Vec::new(),
                document_ids: Vec::new(),
                document_id: Some("doc-1".to_string()),
                chunk_size: Some(500),
                limit: None,
            },
        )
        .expect("chunking dispatch");
        assert_eq!(chunking.task_name, "evidence.generate_document_chunks");
        assert_eq!(
            chunking.kwargs[0],
            ("chunk_size".to_string(), "500".to_string())
        );

        let vector = build_dispatch_plan(
            "work-3",
            "chunk_vector_upsert",
            &DispatchPayload {
                collection_run_id: None,
                raw_artifact_ids: Vec::new(),
                document_ids: Vec::new(),
                document_id: None,
                chunk_size: None,
                limit: Some(12),
            },
        )
        .expect("vector dispatch");
        assert_eq!(vector.task_name, "memory.vector.upsert_chunks");
        assert_eq!(vector.kwargs[0], ("limit".to_string(), "12".to_string()));
    }

    #[test]
    fn dispatch_requires_queued_status_and_valid_payload() {
        let running = base_work_item("document_chunking", WorkItemStatus::Running);
        let error = plan_work_item_dispatch(
            &running,
            &empty_dispatch_payload(),
            WorkItemDispatchRequest {
                actor_id: LOCAL_OWNER.to_string(),
                task_id: "task-1".to_string(),
            },
        )
        .expect_err("only queued dispatches");
        assert!(matches!(
            error,
            WorkQueueReportError::InvalidWorkItemTransition { .. }
        ));

        let queued = base_work_item("document_chunking", WorkItemStatus::Queued);
        let error = plan_work_item_dispatch(
            &queued,
            &empty_dispatch_payload(),
            WorkItemDispatchRequest {
                actor_id: LOCAL_OWNER.to_string(),
                task_id: "task-1".to_string(),
            },
        )
        .expect_err("payload is invalid");
        assert!(matches!(
            error,
            WorkQueueReportError::InvalidDispatchPayload { .. }
        ));
    }

    #[test]
    fn dispatch_result_records_audit_event() {
        let queued = base_work_item("chunk_vector_upsert", WorkItemStatus::Queued);
        let result = plan_work_item_dispatch(
            &queued,
            &DispatchPayload {
                limit: Some(10),
                ..empty_dispatch_payload()
            },
            WorkItemDispatchRequest {
                actor_id: LOCAL_OWNER.to_string(),
                task_id: "task-1".to_string(),
            },
        )
        .expect("dispatch result");
        assert_eq!(result.task_name, "memory.vector.upsert_chunks");
        assert_eq!(result.audit_event.event_type, "work_item.dispatched");
        assert_eq!(result.audit_event.correlation_id.as_deref(), Some("task-1"));
    }

    #[test]
    fn report_create_validates_type_and_status() {
        let plan = plan_report_create(base_report_create()).expect("report plan");
        assert_eq!(plan.report.report_type, ReportType::Summary);
        assert_eq!(plan.report.status, ReportStatus::Requested);
        assert_eq!(plan.audit_event.event_type, "report.created");

        let mut invalid = base_report_create();
        invalid.report_type = "dashboard".to_string();
        assert!(matches!(
            plan_report_create(invalid),
            Err(WorkQueueReportError::InvalidReportType(_))
        ));

        let mut invalid = base_report_create();
        invalid.status = "published".to_string();
        assert!(matches!(
            plan_report_create(invalid),
            Err(WorkQueueReportError::InvalidReportStatus(_))
        ));
    }

    #[test]
    fn report_status_update_preserves_artifact_path_when_not_supplied() {
        let mut report = plan_report_create(base_report_create())
            .expect("report plan")
            .report;
        report.artifact_path = Some("sha256/aa/bb/hash.md".to_string());
        let update = plan_report_status_update(
            &report,
            ReportStatusUpdateRequest {
                status: "ready".to_string(),
                actor_id: LOCAL_OWNER.to_string(),
                artifact_path: None,
            },
        )
        .expect("status update");
        assert_eq!(update.status, ReportStatus::Ready);
        assert_eq!(
            update.artifact_path.as_deref(),
            Some("sha256/aa/bb/hash.md")
        );
        assert_eq!(update.audit_event.event_type, "report.status_updated");
    }

    #[test]
    fn report_work_item_is_queued_scaffold_with_audit() {
        let report = plan_report_create(base_report_create())
            .expect("report plan")
            .report;
        let plan = plan_report_work_item_create(
            &report,
            ReportWorkItemCreateRequest {
                work_item_id: "work-report-1".to_string(),
                requested_by_actor_id: LOCAL_OWNER.to_string(),
                notes: Some("weekly handoff".to_string()),
            },
        )
        .expect("report work item");
        assert_eq!(plan.work_item.work_type, "report_generation");
        assert_eq!(plan.work_item.status, WorkItemStatus::Queued);
        assert_eq!(plan.audit_event.decision.as_deref(), Some("queued"));
        assert_eq!(plan.audit_event.correlation_id.as_deref(), Some("report-1"));
    }

    #[test]
    fn report_markdown_is_deterministic_and_local_only() {
        let report = plan_report_create(base_report_create())
            .expect("report plan")
            .report;
        let markdown = render_report_markdown(&report, &base_inventory(), Some("Operator note."));
        assert!(markdown.contains("# Weekly Summary"));
        assert!(markdown.contains("- approvals: 2"));
        assert!(markdown.contains("- source: Router export [trusted]"));
        assert!(markdown.contains("It does not read raw artifact contents."));
        assert!(markdown.contains("Operator note."));
    }

    #[test]
    fn report_render_plans_artifact_metadata_and_audit() {
        let report = plan_report_create(base_report_create())
            .expect("report plan")
            .report;
        let plan = plan_report_render(
            &report,
            &base_inventory(),
            ReportRenderRequest {
                actor_id: LOCAL_OWNER.to_string(),
                notes: None,
                artifact_id: "artifact-1".to_string(),
                artifact_path: "sha256/aa/bb/report.md".to_string(),
                content_hash: "hash-1".to_string(),
                content_already_existed: false,
            },
        )
        .expect("render plan");
        assert_eq!(plan.report_status, ReportStatus::Ready);
        assert_eq!(plan.artifact_path, "sha256/aa/bb/report.md");
        assert_eq!(plan.audit_event.event_type, "report.rendered");
        assert_eq!(
            plan.audit_event.correlation_id.as_deref(),
            Some("artifact-1")
        );
        assert!(plan
            .artifact_metadata
            .contains(&("generated_by".to_string(), "DIFF-101".to_string())));
    }

    fn base_intent() -> IntentVerificationContext {
        IntentVerificationContext {
            original_request: "Chunk these documents".to_string(),
            interpretation: "Generate deterministic chunks".to_string(),
            proposed_work_type: "document_chunking".to_string(),
            expected_output: "Chunk rows and evidence plans".to_string(),
            safety_requirements: vec!["Use local metadata only.".to_string()],
            assumptions: vec!["Documents exist.".to_string()],
            missing_information: Vec::new(),
            sources_likely_used: vec!["normalized_documents".to_string()],
        }
    }

    fn base_work_item(work_type: &str, status: WorkItemStatus) -> WorkItemDraft {
        WorkItemDraft {
            id: "work-1".to_string(),
            work_type: work_type.to_string(),
            status,
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            payload_json: Vec::new(),
            intent_verification: base_intent(),
            error_message: None,
        }
    }

    fn empty_dispatch_payload() -> DispatchPayload {
        DispatchPayload {
            collection_run_id: None,
            raw_artifact_ids: Vec::new(),
            document_ids: Vec::new(),
            document_id: None,
            chunk_size: None,
            limit: None,
        }
    }

    fn base_report_create() -> ReportCreateRequest {
        ReportCreateRequest {
            id: "report-1".to_string(),
            title: "Weekly Summary".to_string(),
            report_type: "summary".to_string(),
            status: "requested".to_string(),
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            artifact_path: None,
            metadata_json: Vec::new(),
        }
    }

    fn base_inventory() -> ReportInventory {
        ReportInventory {
            counts: vec![
                ("work_items".to_string(), 3),
                ("approvals".to_string(), 2),
                ("sources".to_string(), 1),
            ],
            recent_records: vec![RecentRecord {
                label: "source".to_string(),
                value: "Router export".to_string(),
                status: Some("trusted".to_string()),
            }],
        }
    }
}
