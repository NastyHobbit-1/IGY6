use std::fmt;

pub const LOCAL_OWNER: &str = "local-owner";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteApiError {
    EmptyValue { field: &'static str },
    InvalidSourceType(String),
    InvalidAllowedOperation(String),
    InvalidSensitivity(String),
    InvalidExternalModelPolicy(String),
    InvalidApprovalDecision(String),
    InvalidApprovalTransition { from: String, attempted: String },
    ApprovalRequired { action_type: String },
    ApprovalNotApproved { approval_id: String, status: String },
    InvalidFeedbackTargetType(String),
    InvalidFeedbackLabel(String),
    InvalidOutcomeTargetType(String),
    InvalidOutcomeStatus(String),
}

impl fmt::Display for WriteApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidSourceType(value) => write!(formatter, "unknown source type: {value}"),
            Self::InvalidAllowedOperation(value) => {
                write!(formatter, "unknown allowed operation: {value}")
            }
            Self::InvalidSensitivity(value) => {
                write!(formatter, "unknown sensitivity label: {value}")
            }
            Self::InvalidExternalModelPolicy(value) => {
                write!(formatter, "unknown external model policy: {value}")
            }
            Self::InvalidApprovalDecision(value) => {
                write!(
                    formatter,
                    "approval decision must be approved or denied: {value}"
                )
            }
            Self::InvalidApprovalTransition { from, attempted } => {
                write!(
                    formatter,
                    "cannot transition approval from {from} to {attempted}"
                )
            }
            Self::ApprovalRequired { action_type } => {
                write!(formatter, "{action_type} requires explicit approval")
            }
            Self::ApprovalNotApproved {
                approval_id,
                status,
            } => write!(
                formatter,
                "approval {approval_id} is {status}, not approved"
            ),
            Self::InvalidFeedbackTargetType(value) => {
                write!(formatter, "unknown feedback target type: {value}")
            }
            Self::InvalidFeedbackLabel(value) => {
                write!(formatter, "unknown feedback label: {value}")
            }
            Self::InvalidOutcomeTargetType(value) => {
                write!(formatter, "unknown outcome target type: {value}")
            }
            Self::InvalidOutcomeStatus(value) => {
                write!(formatter, "unknown outcome status: {value}")
            }
        }
    }
}

impl std::error::Error for WriteApiError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventDraft {
    pub actor_id: String,
    pub event_type: String,
    pub decision: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub correlation_id: Option<String>,
    pub details_json: Vec<(String, String)>,
}

impl AuditEventDraft {
    pub fn new(
        actor_id: impl Into<String>,
        event_type: impl Into<String>,
    ) -> Result<Self, WriteApiError> {
        let actor_id = non_empty_string("actor_id", actor_id.into())?;
        let event_type = non_empty_string("event_type", event_type.into())?;
        Ok(Self {
            actor_id,
            event_type,
            decision: None,
            resource_type: None,
            resource_id: None,
            correlation_id: None,
            details_json: Vec::new(),
        })
    }

    pub fn decision(mut self, decision: impl Into<String>) -> Result<Self, WriteApiError> {
        self.decision = Some(non_empty_string("decision", decision.into())?);
        Ok(self)
    }

    pub fn resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Result<Self, WriteApiError> {
        self.resource_type = Some(non_empty_string("resource_type", resource_type.into())?);
        self.resource_id = Some(non_empty_string("resource_id", resource_id.into())?);
        Ok(self)
    }

    pub fn correlation_id(
        mut self,
        correlation_id: impl Into<String>,
    ) -> Result<Self, WriteApiError> {
        self.correlation_id = Some(non_empty_string("correlation_id", correlation_id.into())?);
        Ok(self)
    }

    pub fn details(mut self, details_json: Vec<(String, String)>) -> Result<Self, WriteApiError> {
        for (key, _) in &details_json {
            validate_non_empty("details_json key", key)?;
        }
        self.details_json = details_json;
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    ManualUpload,
    LocalProject,
    LocalPcDiagnostics,
    WebPublic,
    WebAuthorizedAccount,
    RouterNetwork,
    UserObservation,
    ConversationHistory,
    // DIFF-246 Grok6 foundations: extended per specs collector contract + post-245 plan
    // to enable real backend registration for browser exports, media, wifi, streams.
    BrowserExport,
    MediaFile,
    WifiSignal,
    StreamCapture,
}

impl SourceType {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "manual_upload" => Ok(Self::ManualUpload),
            "local_project" => Ok(Self::LocalProject),
            "local_pc_diagnostics" => Ok(Self::LocalPcDiagnostics),
            "web_public" => Ok(Self::WebPublic),
            "web_authorized_account" => Ok(Self::WebAuthorizedAccount),
            "router_network" => Ok(Self::RouterNetwork),
            "user_observation" => Ok(Self::UserObservation),
            "conversation_history" => Ok(Self::ConversationHistory),
            // DIFF-246 additions (collector foundations)
            "browser_export" => Ok(Self::BrowserExport),
            "media_file" => Ok(Self::MediaFile),
            "wifi_signal" => Ok(Self::WifiSignal),
            "stream_capture" => Ok(Self::StreamCapture),
            _ => Err(WriteApiError::InvalidSourceType(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManualUpload => "manual_upload",
            Self::LocalProject => "local_project",
            Self::LocalPcDiagnostics => "local_pc_diagnostics",
            Self::WebPublic => "web_public",
            Self::WebAuthorizedAccount => "web_authorized_account",
            Self::RouterNetwork => "router_network",
            Self::UserObservation => "user_observation",
            Self::ConversationHistory => "conversation_history",
            // DIFF-246
            Self::BrowserExport => "browser_export",
            Self::MediaFile => "media_file",
            Self::WifiSignal => "wifi_signal",
            Self::StreamCapture => "stream_capture",
        }
    }

    /// DIFF-246: supports_dry_run_preview returns true for collector types
    /// that have (or will have) explicit preview + permission flows per the
    /// Finished Product Capability Specification collector contract.
    pub fn supports_dry_run_preview(&self) -> bool {
        matches!(
            self,
            Self::BrowserExport
                | Self::MediaFile
                | Self::WifiSignal
                | Self::StreamCapture
                | Self::LocalProject
                | Self::LocalPcDiagnostics
        )
    }

    /// DIFF-246: requires_explicit_approval for higher-sensitivity or external
    /// source types. Used to drive permission records and approval gates.
    pub fn requires_explicit_approval(&self) -> bool {
        matches!(
            self,
            Self::MediaFile | Self::WifiSignal | Self::StreamCapture | Self::WebAuthorizedAccount
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowedOperation {
    DryRun,
    Read,
    Collect,
    Normalize,
    ClassifySensitivity,
    ExtractMetadata,
}

impl AllowedOperation {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "dry_run" => Ok(Self::DryRun),
            "read" => Ok(Self::Read),
            "collect" => Ok(Self::Collect),
            "normalize" => Ok(Self::Normalize),
            "classify_sensitivity" => Ok(Self::ClassifySensitivity),
            "extract_metadata" => Ok(Self::ExtractMetadata),
            _ => Err(WriteApiError::InvalidAllowedOperation(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DryRun => "dry_run",
            Self::Read => "read",
            Self::Collect => "collect",
            Self::Normalize => "normalize",
            Self::ClassifySensitivity => "classify_sensitivity",
            Self::ExtractMetadata => "extract_metadata",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sensitivity {
    Public,
    Internal,
    Sensitive,
    Secret,
}

impl Sensitivity {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "public" => Ok(Self::Public),
            "internal" => Ok(Self::Internal),
            "sensitive" => Ok(Self::Sensitive),
            "secret" => Ok(Self::Secret),
            _ => Err(WriteApiError::InvalidSensitivity(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Sensitive => "sensitive",
            Self::Secret => "secret",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalModelPolicy {
    Blocked,
    MetadataOnly,
    AllowedWithApproval,
}

impl ExternalModelPolicy {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "blocked" => Ok(Self::Blocked),
            "metadata_only" => Ok(Self::MetadataOnly),
            "allowed_with_approval" => Ok(Self::AllowedWithApproval),
            _ => Err(WriteApiError::InvalidExternalModelPolicy(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::MetadataOnly => "metadata_only",
            Self::AllowedWithApproval => "allowed_with_approval",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePermissionCreateRequest {
    pub scope_json: Vec<(String, String)>,
    pub allowed_operations: Vec<String>,
    pub external_model_policy: String,
    pub approval_required: bool,
    pub created_by_actor_id: String,
}

impl Default for SourcePermissionCreateRequest {
    fn default() -> Self {
        Self {
            scope_json: Vec::new(),
            allowed_operations: Vec::new(),
            external_model_policy: "blocked".to_string(),
            approval_required: true,
            created_by_actor_id: LOCAL_OWNER.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePermissionDraft {
    pub id: String,
    pub source_id: String,
    pub scope_json: Vec<(String, String)>,
    pub allowed_operations: Vec<AllowedOperation>,
    pub external_model_policy: ExternalModelPolicy,
    pub approval_required: bool,
    pub created_by_actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCreateRequest {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub location: Option<String>,
    pub owner_actor_id: String,
    pub sensitivity: String,
    pub trust_level: String,
    pub enabled: bool,
    pub metadata_json: Vec<(String, String)>,
    pub permission: Option<SourcePermissionCreateRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCreatePlan {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub location: Option<String>,
    pub owner_actor_id: String,
    pub sensitivity: Sensitivity,
    pub trust_level: String,
    pub enabled: bool,
    pub metadata_json: Vec<(String, String)>,
    pub permission: Option<SourcePermissionDraft>,
    pub audit_event: AuditEventDraft,
}

pub fn plan_source_create(payload: SourceCreateRequest) -> Result<SourceCreatePlan, WriteApiError> {
    let id = non_empty_string("source.id", payload.id)?;
    let name = non_empty_string("source.name", payload.name)?;
    let source_type = SourceType::parse(&payload.source_type)?;
    let owner_actor_id = non_empty_string("source.owner_actor_id", payload.owner_actor_id)?;
    let sensitivity = Sensitivity::parse(&payload.sensitivity)?;
    let trust_level = non_empty_string("source.trust_level", payload.trust_level)?;
    validate_metadata(&payload.metadata_json)?;

    let permission = payload
        .permission
        .map(|permission| plan_source_permission(&id, format!("{id}:permission"), permission))
        .transpose()?;

    let audit_event = source_audit_event(
        &owner_actor_id,
        "source.created",
        "source",
        &id,
        vec![
            ("source_type".to_string(), source_type.as_str().to_string()),
            ("sensitivity".to_string(), sensitivity.as_str().to_string()),
            (
                "permission_included".to_string(),
                permission.is_some().to_string(),
            ),
        ],
    )?;

    Ok(SourceCreatePlan {
        id,
        name,
        source_type,
        location: payload.location,
        owner_actor_id,
        sensitivity,
        trust_level,
        enabled: payload.enabled,
        metadata_json: payload.metadata_json,
        permission,
        audit_event,
    })
}

pub fn plan_source_permission_create(
    source_id: impl Into<String>,
    permission_id: impl Into<String>,
    payload: SourcePermissionCreateRequest,
) -> Result<(SourcePermissionDraft, AuditEventDraft), WriteApiError> {
    let source_id = non_empty_string("source_id", source_id.into())?;
    let permission_id = non_empty_string("permission_id", permission_id.into())?;
    let permission = plan_source_permission(&source_id, permission_id, payload)?;
    let audit_event = source_audit_event(
        &permission.created_by_actor_id,
        "source_permission.created",
        "source",
        &source_id,
        vec![
            ("permission_id".to_string(), permission.id.clone()),
            (
                "allowed_operations".to_string(),
                join_operations(&permission.allowed_operations),
            ),
            (
                "approval_required".to_string(),
                permission.approval_required.to_string(),
            ),
            (
                "external_model_policy".to_string(),
                permission.external_model_policy.as_str().to_string(),
            ),
        ],
    )?;
    Ok((permission, audit_event))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCreateRequest {
    pub id: String,
    pub request_type: String,
    pub requested_by_actor_id: String,
    pub request_payload_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalDraft {
    pub id: String,
    pub request_type: String,
    pub status: String,
    pub requested_by_actor_id: String,
    pub decided_by_actor_id: Option<String>,
    pub decision_reason: Option<String>,
    pub request_payload_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalDecisionRequest {
    pub status: String,
    pub decided_by_actor_id: String,
    pub decision_reason: Option<String>,
}

pub fn plan_approval_create(
    payload: ApprovalCreateRequest,
) -> Result<(ApprovalDraft, AuditEventDraft), WriteApiError> {
    let id = non_empty_string("approval.id", payload.id)?;
    let request_type = non_empty_string("approval.request_type", payload.request_type)?;
    let requested_by_actor_id = non_empty_string(
        "approval.requested_by_actor_id",
        payload.requested_by_actor_id,
    )?;
    validate_metadata(&payload.request_payload_json)?;
    let approval = ApprovalDraft {
        id: id.clone(),
        request_type,
        status: "pending".to_string(),
        requested_by_actor_id: requested_by_actor_id.clone(),
        decided_by_actor_id: None,
        decision_reason: None,
        request_payload_json: payload.request_payload_json,
    };
    let audit_event = approval_audit_event(
        &requested_by_actor_id,
        "approval.requested",
        "pending",
        &approval,
    )?;
    Ok((approval, audit_event))
}

pub fn plan_approval_decision(
    approval: &ApprovalDraft,
    payload: ApprovalDecisionRequest,
) -> Result<(ApprovalDraft, AuditEventDraft), WriteApiError> {
    validate_non_empty("approval.id", &approval.id)?;
    validate_non_empty("approval.status", &approval.status)?;
    if approval.status != "pending" {
        return Err(WriteApiError::InvalidApprovalTransition {
            from: approval.status.clone(),
            attempted: payload.status,
        });
    }
    validate_approval_decision(&payload.status)?;
    let decided_by_actor_id =
        non_empty_string("approval.decided_by_actor_id", payload.decided_by_actor_id)?;
    let mut decided = approval.clone();
    decided.status = payload.status;
    decided.decided_by_actor_id = Some(decided_by_actor_id.clone());
    decided.decision_reason = payload.decision_reason;
    let audit_event = approval_audit_event(
        &decided_by_actor_id,
        "approval.decided",
        &decided.status,
        &decided,
    )?;
    Ok((decided, audit_event))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalGateRequest {
    pub action_type: String,
    pub system_changing: bool,
    pub approval_id: Option<String>,
    pub approval_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalGateDecision {
    pub allowed: bool,
    pub approval_required: bool,
    pub reason: String,
    pub approval_id: Option<String>,
}

pub fn enforce_approval_gate(
    payload: ApprovalGateRequest,
) -> Result<ApprovalGateDecision, WriteApiError> {
    let action_type = non_empty_string("action_type", payload.action_type)?;
    if !payload.system_changing {
        return Ok(ApprovalGateDecision {
            allowed: true,
            approval_required: false,
            reason: "read-only local action".to_string(),
            approval_id: payload.approval_id,
        });
    }

    let Some(approval_id) = payload.approval_id else {
        return Err(WriteApiError::ApprovalRequired { action_type });
    };
    let approval_id = non_empty_string("approval_id", approval_id)?;
    let approval_status = non_empty_string(
        "approval_status",
        payload.approval_status.unwrap_or_default(),
    )?;
    if approval_status != "approved" {
        return Err(WriteApiError::ApprovalNotApproved {
            approval_id,
            status: approval_status,
        });
    }

    Ok(ApprovalGateDecision {
        allowed: true,
        approval_required: true,
        reason: "approved system-changing local action".to_string(),
        approval_id: Some(approval_id),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackTargetType {
    Source,
    Document,
    EvidenceItem,
    Claim,
    Pattern,
    Hypothesis,
    Prediction,
    Recommendation,
    Report,
    WorkItem,
}

impl FeedbackTargetType {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "source" => Ok(Self::Source),
            "document" => Ok(Self::Document),
            "evidence_item" => Ok(Self::EvidenceItem),
            "claim" => Ok(Self::Claim),
            "pattern" => Ok(Self::Pattern),
            "hypothesis" => Ok(Self::Hypothesis),
            "prediction" => Ok(Self::Prediction),
            "recommendation" => Ok(Self::Recommendation),
            "report" => Ok(Self::Report),
            "work_item" => Ok(Self::WorkItem),
            _ => Err(WriteApiError::InvalidFeedbackTargetType(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Document => "document",
            Self::EvidenceItem => "evidence_item",
            Self::Claim => "claim",
            Self::Pattern => "pattern",
            Self::Hypothesis => "hypothesis",
            Self::Prediction => "prediction",
            Self::Recommendation => "recommendation",
            Self::Report => "report",
            Self::WorkItem => "work_item",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackLabel {
    Useful,
    NotUseful,
    Wrong,
    Verified,
    Incomplete,
    Noisy,
    Trusted,
    Rejected,
}

impl FeedbackLabel {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "useful" => Ok(Self::Useful),
            "not_useful" => Ok(Self::NotUseful),
            "wrong" => Ok(Self::Wrong),
            "verified" => Ok(Self::Verified),
            "incomplete" => Ok(Self::Incomplete),
            "noisy" => Ok(Self::Noisy),
            "trusted" => Ok(Self::Trusted),
            "rejected" => Ok(Self::Rejected),
            _ => Err(WriteApiError::InvalidFeedbackLabel(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Useful => "useful",
            Self::NotUseful => "not_useful",
            Self::Wrong => "wrong",
            Self::Verified => "verified",
            Self::Incomplete => "incomplete",
            Self::Noisy => "noisy",
            Self::Trusted => "trusted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackCreateRequest {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub label: String,
    pub actor_id: String,
    pub note: Option<String>,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTrustUpdate {
    pub trust_level: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImprovementItemDraft {
    pub id: String,
    pub target_area: String,
    pub status: String,
    pub objective: String,
    pub proposed_by_actor_id: String,
    pub priority: String,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackCreatePlan {
    pub id: String,
    pub target_type: FeedbackTargetType,
    pub target_id: String,
    pub label: FeedbackLabel,
    pub actor_id: String,
    pub note: Option<String>,
    pub metadata_json: Vec<(String, String)>,
    pub audit_event: AuditEventDraft,
    pub source_trust_update: Option<SourceTrustUpdate>,
    pub source_trust_audit_event: Option<AuditEventDraft>,
    pub improvement_item: Option<ImprovementItemDraft>,
    pub improvement_audit_event: Option<AuditEventDraft>,
}

pub fn plan_feedback_create(
    payload: FeedbackCreateRequest,
) -> Result<FeedbackCreatePlan, WriteApiError> {
    let id = non_empty_string("feedback.id", payload.id)?;
    let target_type = FeedbackTargetType::parse(&payload.target_type)?;
    let target_id = non_empty_string("feedback.target_id", payload.target_id)?;
    let label = FeedbackLabel::parse(&payload.label)?;
    let actor_id = non_empty_string("feedback.actor_id", payload.actor_id)?;
    validate_metadata(&payload.metadata_json)?;
    let audit_event = AuditEventDraft::new(&actor_id, "feedback.created")?
        .decision("recorded")?
        .resource(target_type.as_str(), &target_id)?
        .details(vec![
            ("feedback_id".to_string(), id.clone()),
            ("label".to_string(), label.as_str().to_string()),
        ])?;

    let source_trust_update = source_trust_update(&target_type, &label);
    let source_trust_audit_event = if let Some(update) = &source_trust_update {
        Some(
            AuditEventDraft::new(&actor_id, "source.trust_feedback_applied")?
                .decision(label.as_str())?
                .resource("source", &target_id)?
                .details(vec![
                    ("feedback_id".to_string(), id.clone()),
                    ("new_trust_level".to_string(), update.trust_level.clone()),
                    ("new_enabled".to_string(), update.enabled.to_string()),
                ])?,
        )
    } else {
        None
    };

    let improvement_item = improvement_item_for_feedback(
        &id,
        &target_type,
        &target_id,
        &label,
        &actor_id,
        payload.note.as_deref(),
    );
    let improvement_audit_event = if let Some(item) = &improvement_item {
        Some(
            AuditEventDraft::new(&actor_id, "improvement_item.created")?
                .decision("proposed")?
                .resource("improvement_item", &item.id)?
                .correlation_id(&id)?
                .details(vec![
                    ("target_area".to_string(), item.target_area.clone()),
                    ("priority".to_string(), item.priority.clone()),
                    ("source_feedback_id".to_string(), id.clone()),
                ])?,
        )
    } else {
        None
    };

    Ok(FeedbackCreatePlan {
        id,
        target_type,
        target_id,
        label,
        actor_id,
        note: payload.note,
        metadata_json: payload.metadata_json,
        audit_event,
        source_trust_update,
        source_trust_audit_event,
        improvement_item,
        improvement_audit_event,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutcomeTargetType {
    Prediction,
    Recommendation,
    WorkItem,
    Hypothesis,
    Pattern,
    Report,
}

impl OutcomeTargetType {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "prediction" => Ok(Self::Prediction),
            "recommendation" => Ok(Self::Recommendation),
            "work_item" => Ok(Self::WorkItem),
            "hypothesis" => Ok(Self::Hypothesis),
            "pattern" => Ok(Self::Pattern),
            "report" => Ok(Self::Report),
            _ => Err(WriteApiError::InvalidOutcomeTargetType(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prediction => "prediction",
            Self::Recommendation => "recommendation",
            Self::WorkItem => "work_item",
            Self::Hypothesis => "hypothesis",
            Self::Pattern => "pattern",
            Self::Report => "report",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutcomeStatus {
    Correct,
    Wrong,
    Useful,
    NotUseful,
    Partial,
    Inconclusive,
    Confirmed,
    Disconfirmed,
}

impl OutcomeStatus {
    pub fn parse(value: &str) -> Result<Self, WriteApiError> {
        match value {
            "correct" => Ok(Self::Correct),
            "wrong" => Ok(Self::Wrong),
            "useful" => Ok(Self::Useful),
            "not_useful" => Ok(Self::NotUseful),
            "partial" => Ok(Self::Partial),
            "inconclusive" => Ok(Self::Inconclusive),
            "confirmed" => Ok(Self::Confirmed),
            "disconfirmed" => Ok(Self::Disconfirmed),
            _ => Err(WriteApiError::InvalidOutcomeStatus(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Wrong => "wrong",
            Self::Useful => "useful",
            Self::NotUseful => "not_useful",
            Self::Partial => "partial",
            Self::Inconclusive => "inconclusive",
            Self::Confirmed => "confirmed",
            Self::Disconfirmed => "disconfirmed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeCreateRequest {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub outcome_status: String,
    pub summary: Option<String>,
    pub occurred_at: Option<String>,
    pub evidence_ids: Vec<String>,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetOutcomeUpdate {
    pub target_status: String,
    pub metadata_json: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeCreatePlan {
    pub id: String,
    pub target_type: OutcomeTargetType,
    pub target_id: String,
    pub outcome_status: OutcomeStatus,
    pub summary: Option<String>,
    pub occurred_at: Option<String>,
    pub evidence_ids: Vec<String>,
    pub metadata_json: Vec<(String, String)>,
    pub audit_event: AuditEventDraft,
    pub target_update: TargetOutcomeUpdate,
    pub target_audit_event: AuditEventDraft,
}

pub fn plan_outcome_create(
    payload: OutcomeCreateRequest,
) -> Result<OutcomeCreatePlan, WriteApiError> {
    let id = non_empty_string("outcome.id", payload.id)?;
    let target_type = OutcomeTargetType::parse(&payload.target_type)?;
    let target_id = non_empty_string("outcome.target_id", payload.target_id)?;
    let outcome_status = OutcomeStatus::parse(&payload.outcome_status)?;
    let evidence_ids = dedupe_non_empty("outcome.evidence_ids", payload.evidence_ids)?;
    validate_metadata(&payload.metadata_json)?;
    let target_status = outcome_target_status(&outcome_status).to_string();
    let audit_event = AuditEventDraft::new(LOCAL_OWNER, "outcome.created")?
        .decision("recorded")?
        .resource(target_type.as_str(), &target_id)?
        .details(vec![
            ("outcome_id".to_string(), id.clone()),
            (
                "outcome_status".to_string(),
                outcome_status.as_str().to_string(),
            ),
        ])?;
    let target_update = TargetOutcomeUpdate {
        target_status: target_status.clone(),
        metadata_json: vec![
            ("latest_outcome_id".to_string(), id.clone()),
            (
                "latest_outcome_status".to_string(),
                outcome_status.as_str().to_string(),
            ),
        ],
    };
    let target_audit_event = AuditEventDraft::new(LOCAL_OWNER, "outcome.target_updated")?
        .decision(&target_status)?
        .resource(target_type.as_str(), &target_id)?
        .correlation_id(&id)?
        .details(vec![
            ("new_status".to_string(), target_status),
            (
                "outcome_status".to_string(),
                outcome_status.as_str().to_string(),
            ),
        ])?;

    Ok(OutcomeCreatePlan {
        id,
        target_type,
        target_id,
        outcome_status,
        summary: payload.summary,
        occurred_at: payload.occurred_at,
        evidence_ids,
        metadata_json: payload.metadata_json,
        audit_event,
        target_update,
        target_audit_event,
    })
}

pub fn outcome_target_status(outcome_status: &OutcomeStatus) -> &'static str {
    match outcome_status {
        OutcomeStatus::Correct => "correct",
        OutcomeStatus::Useful => "useful",
        OutcomeStatus::Confirmed => "confirmed",
        OutcomeStatus::Wrong => "wrong",
        OutcomeStatus::NotUseful => "not_useful",
        OutcomeStatus::Disconfirmed => "disconfirmed",
        OutcomeStatus::Partial => "partial",
        OutcomeStatus::Inconclusive => "inconclusive",
    }
}

fn plan_source_permission(
    source_id: &str,
    permission_id: String,
    payload: SourcePermissionCreateRequest,
) -> Result<SourcePermissionDraft, WriteApiError> {
    validate_metadata(&payload.scope_json)?;
    let allowed_operations = payload
        .allowed_operations
        .iter()
        .map(|operation| AllowedOperation::parse(operation))
        .collect::<Result<Vec<_>, _>>()?;
    let external_model_policy = ExternalModelPolicy::parse(&payload.external_model_policy)?;
    let created_by_actor_id = non_empty_string(
        "source_permission.created_by_actor_id",
        payload.created_by_actor_id,
    )?;

    Ok(SourcePermissionDraft {
        id: permission_id,
        source_id: source_id.to_string(),
        scope_json: payload.scope_json,
        allowed_operations,
        external_model_policy,
        approval_required: payload.approval_required,
        created_by_actor_id,
    })
}

fn source_audit_event(
    actor_id: &str,
    event_type: &str,
    resource_type: &str,
    resource_id: &str,
    details: Vec<(String, String)>,
) -> Result<AuditEventDraft, WriteApiError> {
    AuditEventDraft::new(actor_id, event_type)?
        .decision("recorded")?
        .resource(resource_type, resource_id)?
        .details(details)
}

fn approval_audit_event(
    actor_id: &str,
    event_type: &str,
    decision: &str,
    approval: &ApprovalDraft,
) -> Result<AuditEventDraft, WriteApiError> {
    AuditEventDraft::new(actor_id, event_type)?
        .decision(decision)?
        .resource("approval", &approval.id)?
        .details(vec![
            ("request_type".to_string(), approval.request_type.clone()),
            ("status".to_string(), approval.status.clone()),
        ])
}

fn source_trust_update(
    target_type: &FeedbackTargetType,
    label: &FeedbackLabel,
) -> Option<SourceTrustUpdate> {
    if target_type != &FeedbackTargetType::Source {
        return None;
    }
    match label {
        FeedbackLabel::Trusted => Some(SourceTrustUpdate {
            trust_level: "trusted".to_string(),
            enabled: true,
        }),
        FeedbackLabel::Noisy => Some(SourceTrustUpdate {
            trust_level: "noisy".to_string(),
            enabled: true,
        }),
        FeedbackLabel::Rejected => Some(SourceTrustUpdate {
            trust_level: "rejected".to_string(),
            enabled: false,
        }),
        _ => None,
    }
}

fn improvement_item_for_feedback(
    feedback_id: &str,
    target_type: &FeedbackTargetType,
    target_id: &str,
    label: &FeedbackLabel,
    actor_id: &str,
    note: Option<&str>,
) -> Option<ImprovementItemDraft> {
    let weak = matches!(
        label,
        FeedbackLabel::NotUseful
            | FeedbackLabel::Wrong
            | FeedbackLabel::Incomplete
            | FeedbackLabel::Rejected
    );
    if !weak || target_type == &FeedbackTargetType::Source {
        return None;
    }
    let target_type_value = target_type.as_str();
    let label_value = label.as_str();
    Some(ImprovementItemDraft {
        id: format!("improvement:{feedback_id}"),
        target_area: improvement_target_area(target_type).to_string(),
        status: "proposed".to_string(),
        objective: format!(
            "Investigate {label_value} feedback for {target_type_value} {target_id}."
        ),
        proposed_by_actor_id: actor_id.to_string(),
        priority: "normal".to_string(),
        metadata_json: vec![
            ("generated_by".to_string(), "DIFF-100".to_string()),
            ("feedback_id".to_string(), feedback_id.to_string()),
            ("feedback_label".to_string(), label_value.to_string()),
            ("target_type".to_string(), target_type_value.to_string()),
            ("target_id".to_string(), target_id.to_string()),
            ("note".to_string(), note.unwrap_or("").to_string()),
        ],
    })
}

fn improvement_target_area(target_type: &FeedbackTargetType) -> &'static str {
    match target_type {
        FeedbackTargetType::Document => "parsing",
        FeedbackTargetType::EvidenceItem => "retrieval",
        FeedbackTargetType::Claim
        | FeedbackTargetType::Pattern
        | FeedbackTargetType::Hypothesis
        | FeedbackTargetType::Recommendation => "reasoning",
        FeedbackTargetType::Prediction => "prediction",
        FeedbackTargetType::Report => "reporting",
        FeedbackTargetType::WorkItem => "safety",
        FeedbackTargetType::Source => "reasoning",
    }
}

fn join_operations(operations: &[AllowedOperation]) -> String {
    operations
        .iter()
        .map(AllowedOperation::as_str)
        .collect::<Vec<_>>()
        .join(",")
}

fn dedupe_non_empty(
    field: &'static str,
    values: Vec<String>,
) -> Result<Vec<String>, WriteApiError> {
    let mut unique = Vec::new();
    for value in values {
        let value = non_empty_string(field, value)?;
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    Ok(unique)
}

fn validate_metadata(values: &[(String, String)]) -> Result<(), WriteApiError> {
    for (key, _) in values {
        validate_non_empty("metadata_json key", key)?;
    }
    Ok(())
}

fn validate_approval_decision(value: &str) -> Result<(), WriteApiError> {
    match value {
        "approved" | "denied" => Ok(()),
        _ => Err(WriteApiError::InvalidApprovalDecision(value.to_string())),
    }
}

fn non_empty_string(field: &'static str, value: String) -> Result<String, WriteApiError> {
    validate_non_empty(field, &value)?;
    Ok(value)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), WriteApiError> {
    if value.trim().is_empty() {
        Err(WriteApiError::EmptyValue { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_create_preserves_python_shape_and_audit_semantics() {
        let plan = plan_source_create(SourceCreateRequest {
            id: "source-1".to_string(),
            name: "Manual notes".to_string(),
            source_type: "manual_upload".to_string(),
            location: None,
            owner_actor_id: LOCAL_OWNER.to_string(),
            sensitivity: "internal".to_string(),
            trust_level: "unreviewed".to_string(),
            enabled: true,
            metadata_json: vec![("kind".to_string(), "notes".to_string())],
            permission: Some(SourcePermissionCreateRequest {
                allowed_operations: vec!["dry_run".to_string(), "read".to_string()],
                ..SourcePermissionCreateRequest::default()
            }),
        })
        .expect("valid source plan");

        assert_eq!(plan.source_type, SourceType::ManualUpload);
        assert_eq!(plan.sensitivity, Sensitivity::Internal);
        assert!(plan.permission.is_some());
        assert_eq!(plan.audit_event.event_type, "source.created");
        assert_eq!(plan.audit_event.decision.as_deref(), Some("recorded"));
        assert_eq!(plan.audit_event.resource_type.as_deref(), Some("source"));
        assert_eq!(plan.audit_event.resource_id.as_deref(), Some("source-1"));
    }

    #[test]
    fn source_validation_rejects_empty_ids_and_unknown_values() {
        let mut payload = base_source_request();
        payload.id = " ".to_string();
        assert!(matches!(
            plan_source_create(payload),
            Err(WriteApiError::EmptyValue { field: "source.id" })
        ));

        let mut payload = base_source_request();
        payload.source_type = "generic_chatbot".to_string();
        assert!(matches!(
            plan_source_create(payload),
            Err(WriteApiError::InvalidSourceType(_))
        ));

        let error = plan_source_permission_create(
            "source-1",
            "permission-1",
            SourcePermissionCreateRequest {
                allowed_operations: vec!["write".to_string()],
                ..SourcePermissionCreateRequest::default()
            },
        )
        .expect_err("write is not allowed");
        assert!(matches!(error, WriteApiError::InvalidAllowedOperation(_)));
    }

    #[test]
    fn diff_246_source_type_extensions_and_contract_helpers() {
        // DIFF-246 Grok6: new collector source types + dry-run / approval helpers
        assert!(SourceType::parse("browser_export").is_ok());
        assert!(SourceType::parse("media_file").is_ok());
        assert!(SourceType::parse("wifi_signal").is_ok());
        assert!(SourceType::parse("stream_capture").is_ok());
        assert_eq!(SourceType::BrowserExport.as_str(), "browser_export");
        assert_eq!(SourceType::MediaFile.as_str(), "media_file");

        assert!(SourceType::BrowserExport.supports_dry_run_preview());
        assert!(SourceType::MediaFile.supports_dry_run_preview());
        assert!(SourceType::WifiSignal.supports_dry_run_preview());
        assert!(SourceType::LocalProject.supports_dry_run_preview());

        assert!(SourceType::MediaFile.requires_explicit_approval());
        assert!(SourceType::WifiSignal.requires_explicit_approval());
        assert!(SourceType::StreamCapture.requires_explicit_approval());
        assert!(!SourceType::ManualUpload.requires_explicit_approval());
    }

    #[test]
    fn approval_create_and_decision_are_deterministic() {
        let (approval, requested_audit) = plan_approval_create(ApprovalCreateRequest {
            id: "approval-1".to_string(),
            request_type: "start_stack".to_string(),
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            request_payload_json: vec![("action_type".to_string(), "start_stack".to_string())],
        })
        .expect("approval request");

        assert_eq!(approval.status, "pending");
        assert_eq!(requested_audit.event_type, "approval.requested");
        assert_eq!(requested_audit.decision.as_deref(), Some("pending"));

        let (decided, decided_audit) = plan_approval_decision(
            &approval,
            ApprovalDecisionRequest {
                status: "approved".to_string(),
                decided_by_actor_id: LOCAL_OWNER.to_string(),
                decision_reason: Some("operator approved".to_string()),
            },
        )
        .expect("decision");

        assert_eq!(decided.status, "approved");
        assert_eq!(decided_audit.event_type, "approval.decided");
        assert_eq!(decided_audit.decision.as_deref(), Some("approved"));
    }

    #[test]
    fn approval_decision_rejects_invalid_transitions_and_statuses() {
        let mut approval = base_approval();
        let error = plan_approval_decision(
            &approval,
            ApprovalDecisionRequest {
                status: "maybe".to_string(),
                decided_by_actor_id: LOCAL_OWNER.to_string(),
                decision_reason: None,
            },
        )
        .expect_err("invalid status");
        assert!(matches!(error, WriteApiError::InvalidApprovalDecision(_)));

        approval.status = "approved".to_string();
        let error = plan_approval_decision(
            &approval,
            ApprovalDecisionRequest {
                status: "denied".to_string(),
                decided_by_actor_id: LOCAL_OWNER.to_string(),
                decision_reason: None,
            },
        )
        .expect_err("already decided");
        assert!(matches!(
            error,
            WriteApiError::InvalidApprovalTransition { .. }
        ));
    }

    #[test]
    fn approval_gate_blocks_system_changing_actions_without_approval() {
        let error = enforce_approval_gate(ApprovalGateRequest {
            action_type: "collect".to_string(),
            system_changing: true,
            approval_id: None,
            approval_status: None,
        })
        .expect_err("approval required");
        assert!(matches!(error, WriteApiError::ApprovalRequired { .. }));

        let decision = enforce_approval_gate(ApprovalGateRequest {
            action_type: "collect".to_string(),
            system_changing: true,
            approval_id: Some("approval-1".to_string()),
            approval_status: Some("approved".to_string()),
        })
        .expect("approved action");
        assert!(decision.allowed);
        assert!(decision.approval_required);
    }

    #[test]
    fn read_only_actions_do_not_require_approval() {
        let decision = enforce_approval_gate(ApprovalGateRequest {
            action_type: "dry_run".to_string(),
            system_changing: false,
            approval_id: None,
            approval_status: None,
        })
        .expect("read-only allowed");
        assert!(decision.allowed);
        assert!(!decision.approval_required);
    }

    #[test]
    fn feedback_create_plans_audit_and_source_trust_side_effect() {
        let plan = plan_feedback_create(FeedbackCreateRequest {
            id: "feedback-1".to_string(),
            target_type: "source".to_string(),
            target_id: "source-1".to_string(),
            label: "rejected".to_string(),
            actor_id: LOCAL_OWNER.to_string(),
            note: None,
            metadata_json: Vec::new(),
        })
        .expect("feedback plan");

        assert_eq!(plan.audit_event.event_type, "feedback.created");
        assert_eq!(
            plan.source_trust_update,
            Some(SourceTrustUpdate {
                trust_level: "rejected".to_string(),
                enabled: false
            })
        );
        assert!(plan.source_trust_audit_event.is_some());
        assert!(plan.improvement_item.is_none());
    }

    #[test]
    fn weak_non_source_feedback_plans_improvement_item() {
        let plan = plan_feedback_create(FeedbackCreateRequest {
            id: "feedback-2".to_string(),
            target_type: "prediction".to_string(),
            target_id: "prediction-1".to_string(),
            label: "wrong".to_string(),
            actor_id: LOCAL_OWNER.to_string(),
            note: Some("missed evidence".to_string()),
            metadata_json: Vec::new(),
        })
        .expect("feedback plan");

        let item = plan.improvement_item.expect("weak feedback creates item");
        assert_eq!(item.target_area, "prediction");
        assert_eq!(item.id, "improvement:feedback-2");
        assert!(plan.improvement_audit_event.is_some());
    }

    #[test]
    fn feedback_validation_rejects_invalid_target_and_label() {
        let mut payload = base_feedback_request();
        payload.target_type = "router".to_string();
        assert!(matches!(
            plan_feedback_create(payload),
            Err(WriteApiError::InvalidFeedbackTargetType(_))
        ));

        let mut payload = base_feedback_request();
        payload.label = "excellent".to_string();
        assert!(matches!(
            plan_feedback_create(payload),
            Err(WriteApiError::InvalidFeedbackLabel(_))
        ));
    }

    #[test]
    fn outcome_create_dedupes_evidence_and_plans_target_update() {
        let plan = plan_outcome_create(OutcomeCreateRequest {
            id: "outcome-1".to_string(),
            target_type: "prediction".to_string(),
            target_id: "prediction-1".to_string(),
            outcome_status: "wrong".to_string(),
            summary: Some("Prediction failed".to_string()),
            occurred_at: None,
            evidence_ids: vec![
                "evidence-1".to_string(),
                "evidence-1".to_string(),
                "evidence-2".to_string(),
            ],
            metadata_json: Vec::new(),
        })
        .expect("outcome plan");

        assert_eq!(plan.evidence_ids, vec!["evidence-1", "evidence-2"]);
        assert_eq!(plan.audit_event.event_type, "outcome.created");
        assert_eq!(plan.target_update.target_status, "wrong");
        assert_eq!(plan.target_audit_event.event_type, "outcome.target_updated");
        assert_eq!(
            plan.target_audit_event.correlation_id.as_deref(),
            Some("outcome-1")
        );
    }

    #[test]
    fn outcome_validation_rejects_invalid_status_target_and_empty_evidence_id() {
        let mut payload = base_outcome_request();
        payload.outcome_status = "helpfulish".to_string();
        assert!(matches!(
            plan_outcome_create(payload),
            Err(WriteApiError::InvalidOutcomeStatus(_))
        ));

        let mut payload = base_outcome_request();
        payload.target_type = "source".to_string();
        assert!(matches!(
            plan_outcome_create(payload),
            Err(WriteApiError::InvalidOutcomeTargetType(_))
        ));

        let mut payload = base_outcome_request();
        payload.evidence_ids = vec![" ".to_string()];
        assert!(matches!(
            plan_outcome_create(payload),
            Err(WriteApiError::EmptyValue {
                field: "outcome.evidence_ids"
            })
        ));
    }

    #[test]
    fn audit_event_constructor_rejects_empty_required_fields() {
        assert!(matches!(
            AuditEventDraft::new(" ", "source.created"),
            Err(WriteApiError::EmptyValue { field: "actor_id" })
        ));
        assert!(matches!(
            AuditEventDraft::new(LOCAL_OWNER, " "),
            Err(WriteApiError::EmptyValue {
                field: "event_type"
            })
        ));
    }

    fn base_source_request() -> SourceCreateRequest {
        SourceCreateRequest {
            id: "source-1".to_string(),
            name: "Local project".to_string(),
            source_type: "local_project".to_string(),
            location: Some("/repo".to_string()),
            owner_actor_id: LOCAL_OWNER.to_string(),
            sensitivity: "internal".to_string(),
            trust_level: "unreviewed".to_string(),
            enabled: true,
            metadata_json: Vec::new(),
            permission: None,
        }
    }

    fn base_approval() -> ApprovalDraft {
        ApprovalDraft {
            id: "approval-1".to_string(),
            request_type: "collect".to_string(),
            status: "pending".to_string(),
            requested_by_actor_id: LOCAL_OWNER.to_string(),
            decided_by_actor_id: None,
            decision_reason: None,
            request_payload_json: Vec::new(),
        }
    }

    fn base_feedback_request() -> FeedbackCreateRequest {
        FeedbackCreateRequest {
            id: "feedback-1".to_string(),
            target_type: "source".to_string(),
            target_id: "source-1".to_string(),
            label: "trusted".to_string(),
            actor_id: LOCAL_OWNER.to_string(),
            note: None,
            metadata_json: Vec::new(),
        }
    }

    fn base_outcome_request() -> OutcomeCreateRequest {
        OutcomeCreateRequest {
            id: "outcome-1".to_string(),
            target_type: "prediction".to_string(),
            target_id: "prediction-1".to_string(),
            outcome_status: "correct".to_string(),
            summary: None,
            occurred_at: None,
            evidence_ids: Vec::new(),
            metadata_json: Vec::new(),
        }
    }
}
