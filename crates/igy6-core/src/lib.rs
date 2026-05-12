use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorId(String);

impl ActorId {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_non_empty("actor_id", &value)?;
        Ok(Self(value))
    }

    pub fn local_owner() -> Self {
        Self("local-owner".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_non_empty("resource_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    EmptyValue { field: &'static str },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue { field } => write!(formatter, "{field} must not be empty"),
        }
    }
}

impl std::error::Error for CoreError {}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        Err(CoreError::EmptyValue { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_owner_is_stable() {
        assert_eq!(ActorId::local_owner().as_str(), "local-owner");
    }

    #[test]
    fn rejects_empty_actor_id() {
        assert!(ActorId::new("  ").is_err());
    }

    #[test]
    fn resource_id_preserves_value() {
        let id = ResourceId::new("source-1").expect("valid id");
        assert_eq!(id.as_str(), "source-1");
    }
}
