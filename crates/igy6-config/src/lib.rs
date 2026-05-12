use std::fmt;

use igy6_core::validate_non_empty;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyKey,
    MissingSeparator,
    EmptyRequiredValue { key: String },
    InvalidBoolean { key: String, value: String },
    InvalidPort { key: String, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => write!(formatter, "environment key must not be empty"),
            Self::MissingSeparator => write!(formatter, "environment line must contain '='"),
            Self::EmptyRequiredValue { key } => write!(formatter, "{key} must not be empty"),
            Self::InvalidBoolean { key, value } => {
                write!(formatter, "{key} has invalid boolean value {value:?}")
            }
            Self::InvalidPort { key, value } => {
                write!(formatter, "{key} has invalid port {value:?}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn parse_env_line(line: &str) -> Result<Option<EnvPair>, ConfigError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }
    let Some((key, value)) = trimmed.split_once('=') else {
        return Err(ConfigError::MissingSeparator);
    };
    let key = key.trim();
    if key.is_empty() {
        return Err(ConfigError::EmptyKey);
    }
    Ok(Some(EnvPair {
        key: key.to_string(),
        value: value.trim().to_string(),
    }))
}

pub fn require_value(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_non_empty("value", value).map_err(|_| ConfigError::EmptyRequiredValue {
        key: key.to_string(),
    })
}

pub fn parse_bool(key: &str, value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(ConfigError::InvalidBoolean {
            key: key.to_string(),
            value: value.to_string(),
        }),
    }
}

pub fn parse_port(key: &str, value: &str) -> Result<u16, ConfigError> {
    let parsed = value
        .trim()
        .parse::<u16>()
        .map_err(|_| ConfigError::InvalidPort {
            key: key.to_string(),
            value: value.to_string(),
        })?;
    if parsed == 0 {
        Err(ConfigError::InvalidPort {
            key: key.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_env_line() {
        let pair = parse_env_line("APP_PORT=8000")
            .expect("line parses")
            .expect("not skipped");
        assert_eq!(pair.key, "APP_PORT");
        assert_eq!(pair.value, "8000");
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        assert_eq!(parse_env_line("# comment").expect("comment ok"), None);
        assert_eq!(parse_env_line("  ").expect("blank ok"), None);
    }

    #[test]
    fn validates_boolean_and_port() {
        assert!(parse_bool("SINGLE_USER_MODE", "true").expect("bool"));
        assert_eq!(parse_port("APP_PORT", "8000").expect("port"), 8000);
        assert!(parse_port("APP_PORT", "0").is_err());
    }
}
