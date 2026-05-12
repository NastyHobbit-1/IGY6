use std::fmt;

use igy6_core::ActorId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalModelPolicy {
    Blocked,
    Allowed,
}

impl ExternalModelPolicy {
    pub fn parse(value: &str) -> Result<Self, PolicyError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "blocked" => Ok(Self::Blocked),
            "allowed" => Ok(Self::Allowed),
            _ => Err(PolicyError::InvalidExternalModelPolicy(value.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Allowed => "allowed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionRisk {
    ReadOnly,
    SystemChanging,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequirement {
    pub required: bool,
    pub reason: String,
}

impl ApprovalRequirement {
    pub fn for_action(risk: ActionRisk) -> Self {
        match risk {
            ActionRisk::ReadOnly => Self {
                required: false,
                reason: "read-only local action".to_string(),
            },
            ActionRisk::SystemChanging => Self {
                required: true,
                reason: "system-changing local action requires explicit approval".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalDecision {
    pub approved: bool,
    pub actor_id: ActorId,
}

impl ApprovalDecision {
    pub fn approved_by(actor_id: ActorId) -> Self {
        Self {
            approved: true,
            actor_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    InvalidExternalModelPolicy(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExternalModelPolicy(value) => {
                write!(formatter, "invalid external model policy {value:?}")
            }
        }
    }
}

impl std::error::Error for PolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_model_policy_is_deterministic() {
        assert_eq!(
            ExternalModelPolicy::parse("blocked").expect("policy"),
            ExternalModelPolicy::Blocked
        );
        assert!(ExternalModelPolicy::parse("online").is_err());
    }

    #[test]
    fn system_changing_action_requires_approval() {
        let requirement = ApprovalRequirement::for_action(ActionRisk::SystemChanging);
        assert!(requirement.required);
        assert!(requirement.reason.contains("approval"));
    }

    #[test]
    fn read_only_action_does_not_require_approval() {
        let requirement = ApprovalRequirement::for_action(ActionRisk::ReadOnly);
        assert!(!requirement.required);
    }
}
