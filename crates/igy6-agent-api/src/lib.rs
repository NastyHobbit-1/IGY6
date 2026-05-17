#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    ReadOnly,
    SystemChanging,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentActionDefinition {
    pub name: &'static str,
    pub interpreted_intent: &'static str,
    pub action_type: ActionType,
    pub approval_required: bool,
    pub risk_level: RiskLevel,
    pub required_parameters: &'static [&'static str],
    pub safety_notes: &'static [&'static str],
    pub script_argv: Option<&'static [&'static str]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIntentRequest {
    pub message: String,
    pub parameters: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIntentResponse {
    pub original_message: String,
    pub interpreted_intent: String,
    pub proposed_action: Option<&'static str>,
    pub action_type: ActionType,
    pub approval_required: bool,
    pub risk_level: RiskLevel,
    pub required_parameters: Vec<&'static str>,
    pub missing_parameters: Vec<&'static str>,
    pub safety_notes: Vec<&'static str>,
    pub executable_now: bool,
    pub reason: Option<String>,
}

const READ_ONLY_LOCAL_HEALTH: &[&str] = &["Read-only local health check."];
const READ_ONLY_GIT: &[&str] =
    &["Read-only git metadata only; no diff content or secrets are returned."];
const READ_ONLY_DIFF: &[&str] = &["Read-only DIFF metadata lookup."];
const READ_ONLY_WORK_ITEMS: &[&str] = &["Read-only PostgreSQL work-item metadata."];
const READ_ONLY_RETRIEVAL: &[&str] =
    &["Uses existing retrieval preview; no LLM or external model is called."];
const START_STACK_NOTES: &[&str] = &[
    "Requires approved approval record.",
    "Uses scripts/run.sh --detached only.",
];
const STOP_STACK_NOTES: &[&str] = &[
    "Requires approved approval record.",
    "Uses scripts/stop.sh and preserves volumes/data.",
];
const LAST_HEALTHY_NOTES: &[&str] = &[
    "Requires approved approval record.",
    "Uses scripts/run-last-healthy-config.sh only.",
];
const RETRIEVAL_PARAMS: &[&str] = &["message"];
const RUN_SCRIPT: &[&str] = &["scripts/run.sh", "--detached"];
const STOP_SCRIPT: &[&str] = &["scripts/stop.sh"];
const LAST_HEALTHY_SCRIPT: &[&str] = &["scripts/run-last-healthy-config.sh"];

pub const ACTION_REGISTRY: &[AgentActionDefinition] = &[
    AgentActionDefinition {
        name: "show_project_health",
        interpreted_intent: "Show local IGY6 API readiness and dependency health.",
        action_type: ActionType::ReadOnly,
        approval_required: false,
        risk_level: RiskLevel::Low,
        required_parameters: &[],
        safety_notes: READ_ONLY_LOCAL_HEALTH,
        script_argv: None,
    },
    AgentActionDefinition {
        name: "show_git_status",
        interpreted_intent: "Show the current repository branch, commit, and dirty/clean state.",
        action_type: ActionType::ReadOnly,
        approval_required: false,
        risk_level: RiskLevel::Low,
        required_parameters: &[],
        safety_notes: READ_ONLY_GIT,
        script_argv: None,
    },
    AgentActionDefinition {
        name: "show_latest_diff",
        interpreted_intent: "Show the newest DIFF document and status.",
        action_type: ActionType::ReadOnly,
        approval_required: false,
        risk_level: RiskLevel::Low,
        required_parameters: &[],
        safety_notes: READ_ONLY_DIFF,
        script_argv: None,
    },
    AgentActionDefinition {
        name: "show_work_items",
        interpreted_intent: "Show recent local work items.",
        action_type: ActionType::ReadOnly,
        approval_required: false,
        risk_level: RiskLevel::Low,
        required_parameters: &[],
        safety_notes: READ_ONLY_WORK_ITEMS,
        script_argv: None,
    },
    AgentActionDefinition {
        name: "run_retrieval_preview",
        interpreted_intent: "Run deterministic local retrieval preview.",
        action_type: ActionType::ReadOnly,
        approval_required: false,
        risk_level: RiskLevel::Low,
        required_parameters: RETRIEVAL_PARAMS,
        safety_notes: READ_ONLY_RETRIEVAL,
        script_argv: None,
    },
    AgentActionDefinition {
        name: "start_stack",
        interpreted_intent: "Start the local IGY6 Docker Compose stack detached.",
        action_type: ActionType::SystemChanging,
        approval_required: true,
        risk_level: RiskLevel::High,
        required_parameters: &[],
        safety_notes: START_STACK_NOTES,
        script_argv: Some(RUN_SCRIPT),
    },
    AgentActionDefinition {
        name: "stop_stack",
        interpreted_intent: "Stop the local IGY6 Docker Compose stack without deleting data.",
        action_type: ActionType::SystemChanging,
        approval_required: true,
        risk_level: RiskLevel::High,
        required_parameters: &[],
        safety_notes: STOP_STACK_NOTES,
        script_argv: Some(STOP_SCRIPT),
    },
    AgentActionDefinition {
        name: "run_last_healthy_stack",
        interpreted_intent: "Start from the last healthy local stack snapshot.",
        action_type: ActionType::SystemChanging,
        approval_required: true,
        risk_level: RiskLevel::High,
        required_parameters: &[],
        safety_notes: LAST_HEALTHY_NOTES,
        script_argv: Some(LAST_HEALTHY_SCRIPT),
    },
];

const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf",
    "docker system prune",
    "docker volume rm",
    "git reset",
    "git checkout",
    "git stash",
    "bash -c",
    "sh -c",
    "powershell",
    "cmd.exe",
    "format ",
];

pub fn classify_agent_intent(payload: &AgentIntentRequest) -> AgentIntentResponse {
    let message = payload.message.trim().to_string();
    let normalized = message
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if DANGEROUS_PATTERNS
        .iter()
        .any(|pattern| normalized.contains(pattern))
    {
        return unknown_intent(
            message,
            "Arbitrary shell or destructive command requests are not allowed by the typed action registry.",
        );
    }

    let action_name = if normalized.contains("start") && normalized.contains("stack") {
        Some("start_stack")
    } else if normalized.contains("stop") && normalized.contains("stack") {
        Some("stop_stack")
    } else if normalized.contains("last healthy") || normalized.contains("last-known healthy") {
        Some("run_last_healthy_stack")
    } else if normalized.contains("git")
        && (normalized.contains("status") || normalized.contains("state"))
    {
        Some("show_git_status")
    } else if normalized.contains("latest diff")
        || normalized.contains("newest diff")
        || normalized.contains("current diff")
    {
        Some("show_latest_diff")
    } else if normalized.contains("work item") || normalized.contains("work queue") {
        Some("show_work_items")
    } else if normalized.contains("retrieval preview")
        || normalized.contains("preview retrieval")
        || normalized.contains("preview context")
    {
        Some("run_retrieval_preview")
    } else if normalized.contains("health")
        || normalized.contains("ready")
        || normalized.contains("readiness")
    {
        Some("show_project_health")
    } else {
        None
    };

    let Some(action_name) = action_name else {
        return unknown_intent(
            message,
            "No known local project action matched the message.",
        );
    };
    let definition = action_definition(action_name).expect("registry action exists");
    let missing_parameters = definition
        .required_parameters
        .iter()
        .copied()
        .filter(|parameter| !has_nonempty_parameter(&payload.parameters, parameter))
        .collect::<Vec<_>>();

    AgentIntentResponse {
        original_message: message,
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
    }
}

pub fn action_definition(name: &str) -> Option<&'static AgentActionDefinition> {
    ACTION_REGISTRY
        .iter()
        .find(|definition| definition.name == name)
}

fn unknown_intent(message: String, reason: &str) -> AgentIntentResponse {
    AgentIntentResponse {
        original_message: message,
        interpreted_intent: "No allowed typed action was selected.".to_string(),
        proposed_action: None,
        action_type: ActionType::Unknown,
        approval_required: false,
        risk_level: RiskLevel::High,
        required_parameters: Vec::new(),
        missing_parameters: Vec::new(),
        safety_notes: vec![
            "The agent command plane only accepts fixed local project actions.",
            "Arbitrary shell execution is not available.",
        ],
        executable_now: false,
        reason: Some(reason.to_string()),
    }
}

fn has_nonempty_parameter(parameters: &[(String, String)], name: &str) -> bool {
    parameters
        .iter()
        .any(|(key, value)| key == name && !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(message: &str) -> AgentIntentRequest {
        AgentIntentRequest {
            message: message.to_string(),
            parameters: Vec::new(),
        }
    }

    #[test]
    fn registry_covers_current_action_names() {
        let names = ACTION_REGISTRY
            .iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "show_project_health",
                "show_git_status",
                "show_latest_diff",
                "show_work_items",
                "run_retrieval_preview",
                "start_stack",
                "stop_stack",
                "run_last_healthy_stack",
            ]
        );
    }

    #[test]
    fn classifies_read_only_health() {
        let response = classify_agent_intent(&request("show project health"));
        assert_eq!(response.proposed_action, Some("show_project_health"));
        assert_eq!(response.action_type, ActionType::ReadOnly);
        assert!(!response.approval_required);
        assert!(response.executable_now);
    }

    #[test]
    fn system_changing_stack_action_requires_approval() {
        let response = classify_agent_intent(&request("start the stack"));
        assert_eq!(response.proposed_action, Some("start_stack"));
        assert_eq!(response.action_type, ActionType::SystemChanging);
        assert!(response.approval_required);
        assert!(!response.executable_now);
    }

    #[test]
    fn retrieval_preview_requires_message_parameter() {
        let mut payload = request("run retrieval preview");
        let response = classify_agent_intent(&payload);
        assert_eq!(response.proposed_action, Some("run_retrieval_preview"));
        assert_eq!(response.missing_parameters, vec!["message"]);
        assert!(!response.executable_now);

        payload
            .parameters
            .push(("message".to_string(), "what changed?".to_string()));
        let response = classify_agent_intent(&payload);
        assert!(response.missing_parameters.is_empty());
        assert!(response.executable_now);
    }

    #[test]
    fn dangerous_patterns_are_rejected() {
        let response = classify_agent_intent(&request("please run rm -rf target"));
        assert_eq!(response.proposed_action, None);
        assert_eq!(response.action_type, ActionType::Unknown);
        assert_eq!(response.risk_level, RiskLevel::High);
        assert!(!response.executable_now);
    }

    #[test]
    fn unknown_intent_is_not_executable() {
        let response = classify_agent_intent(&request("make coffee"));
        assert_eq!(response.proposed_action, None);
        assert_eq!(response.action_type, ActionType::Unknown);
        assert!(!response.executable_now);
    }
}
