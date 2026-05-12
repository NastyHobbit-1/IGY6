use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use igy6_policy::{ActionRisk, ApprovalRequirement};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSummary {
    pub cutover_ready: bool,
    pub phases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingArgument(&'static str),
    UnsupportedCommand(String),
    MissingManifest(String),
    InvalidManifest(String),
    UnknownPhase(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingArgument(name) => write!(formatter, "missing required argument: {name}"),
            Self::UnsupportedCommand(command) => {
                write!(formatter, "unsupported command: {command}")
            }
            Self::MissingManifest(path) => {
                write!(formatter, "manifest file does not exist: {path}")
            }
            Self::InvalidManifest(reason) => write!(formatter, "invalid manifest: {reason}"),
            Self::UnknownPhase(phase) => write!(formatter, "unknown phase: {phase}"),
        }
    }
}

impl std::error::Error for CliError {}

pub fn run_cli<I>(args: I) -> Result<String, CliError>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(help_text());
    };

    match command {
        "--help" | "-h" | "help" => Ok(help_text()),
        "version" => Ok(format!("igy6-cli {VERSION}\n")),
        "phases" => {
            let manifest_path = manifest_arg(&args[1..])?;
            let manifest = load_manifest(&manifest_path)?;
            Ok(render_phases(&manifest))
        }
        "phase-status" => {
            let phase = args
                .get(1)
                .ok_or(CliError::MissingArgument("phase"))?
                .to_string();
            let manifest_path = manifest_arg(&args[2..])?;
            let manifest = load_manifest(&manifest_path)?;
            render_phase_status(&manifest, &phase)
        }
        "validate-manifest" => {
            let path = args
                .get(1)
                .ok_or(CliError::MissingArgument("manifest path"))?
                .to_string();
            let manifest = load_manifest(&path)?;
            Ok(format!(
                "manifest valid\ncutover_ready: {}\nphase_count: {}\n",
                manifest.cutover_ready,
                manifest.phases.len()
            ))
        }
        other => Err(CliError::UnsupportedCommand(other.to_string())),
    }
}

pub fn help_text() -> String {
    let read_only = ApprovalRequirement::for_action(ActionRisk::ReadOnly);
    format!(
        "IGY6 local Rust CLI foundation\n\n\
Usage:\n  \
igy6-cli --help\n  \
igy6-cli version\n  \
igy6-cli phases --manifest <path>\n  \
igy6-cli phase-status <phase> --manifest <path>\n  \
igy6-cli validate-manifest <path>\n\n\
Safety:\n  \
local-only: true\n  \
action_type: read-only\n  \
approval_required: {}\n  \
external_model_calls: false\n",
        read_only.required
    )
}

pub fn load_manifest(path: &str) -> Result<ManifestSummary, CliError> {
    let path_ref = Path::new(path);
    if !path_ref.is_file() {
        return Err(CliError::MissingManifest(path.to_string()));
    }
    let content = fs::read_to_string(path_ref)
        .map_err(|error| CliError::InvalidManifest(error.to_string()))?;
    parse_manifest(&content)
}

pub fn parse_manifest(content: &str) -> Result<ManifestSummary, CliError> {
    let cutover_ready = parse_cutover_ready(content)?;
    let required_phases = parse_required_phases(content)?;
    let phases_object = object_after_key(content, "\"phases\"")
        .ok_or_else(|| CliError::InvalidManifest("missing phases object".to_string()))?;
    let mut phases = BTreeMap::new();

    for phase in required_phases {
        let phase_object = object_after_key(phases_object, &format!("\"{phase}\""))
            .ok_or_else(|| CliError::InvalidManifest(format!("missing phase entry {phase}")))?;
        let status = string_value_after_key(phase_object, "\"status\"").ok_or_else(|| {
            CliError::InvalidManifest(format!("missing status for phase {phase}"))
        })?;
        phases.insert(phase, status);
    }

    if phases.is_empty() {
        return Err(CliError::InvalidManifest(
            "required_phases must not be empty".to_string(),
        ));
    }

    Ok(ManifestSummary {
        cutover_ready,
        phases,
    })
}

pub fn render_phases(manifest: &ManifestSummary) -> String {
    let mut output = format!("cutover_ready: {}\nphases:\n", manifest.cutover_ready);
    for (phase, status) in &manifest.phases {
        output.push_str(&format!("  {phase}: {status}\n"));
    }
    output
}

pub fn render_phase_status(manifest: &ManifestSummary, phase: &str) -> Result<String, CliError> {
    let status = manifest
        .phases
        .get(phase)
        .ok_or_else(|| CliError::UnknownPhase(phase.to_string()))?;
    Ok(format!("{phase}: {status}\n"))
}

fn manifest_arg(args: &[String]) -> Result<String, CliError> {
    let Some(flag_index) = args.iter().position(|arg| arg == "--manifest") else {
        return Err(CliError::MissingArgument("--manifest <path>"));
    };
    args.get(flag_index + 1)
        .cloned()
        .ok_or(CliError::MissingArgument("manifest path"))
}

fn parse_cutover_ready(content: &str) -> Result<bool, CliError> {
    let marker = "\"cutover_ready\"";
    let Some(after_key) = content.split_once(marker).map(|(_, rest)| rest) else {
        return Err(CliError::InvalidManifest(
            "missing cutover_ready".to_string(),
        ));
    };
    let Some(after_colon) = after_key.split_once(':').map(|(_, rest)| rest.trim_start()) else {
        return Err(CliError::InvalidManifest(
            "invalid cutover_ready".to_string(),
        ));
    };
    if after_colon.starts_with("true") {
        Ok(true)
    } else if after_colon.starts_with("false") {
        Ok(false)
    } else {
        Err(CliError::InvalidManifest(
            "cutover_ready must be boolean".to_string(),
        ))
    }
}

fn parse_required_phases(content: &str) -> Result<Vec<String>, CliError> {
    let array = array_after_key(content, "\"required_phases\"")
        .ok_or_else(|| CliError::InvalidManifest("missing required_phases".to_string()))?;
    let phases = parse_string_array(array);
    if phases.is_empty() {
        Err(CliError::InvalidManifest(
            "required_phases must contain phases".to_string(),
        ))
    } else {
        Ok(phases)
    }
}

fn array_after_key<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let after_key = content.split_once(key)?.1;
    let start = after_key.find('[')?;
    let after_start = &after_key[start + 1..];
    let end = find_matching(after_start, '[', ']')?;
    Some(&after_start[..end])
}

fn object_after_key<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let after_key = content.split_once(key)?.1;
    let start = after_key.find('{')?;
    let after_start = &after_key[start + 1..];
    let end = find_matching(after_start, '{', '}')?;
    Some(&after_start[..end])
}

fn string_value_after_key(content: &str, key: &str) -> Option<String> {
    let after_key = content.split_once(key)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let value_end = after_quote.find('"')?;
    Some(after_quote[..value_end].to_string())
}

fn parse_string_array(content: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut remaining = content;
    while let Some(start) = remaining.find('"') {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        values.push(after_start[..end].to_string());
        remaining = &after_start[end + 1..];
    }
    values
}

fn find_matching(content_after_open: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in content_after_open.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
        } else if character == open {
            depth += 1;
        } else if character == close {
            if depth == 0 {
                return Some(index);
            }
            depth -= 1;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_MANIFEST: &str = r#"{
      "schema_version": 1,
      "cutover_ready": false,
      "required_phases": ["host_bridge", "workspace", "cli"],
      "phases": {
        "host_bridge": {"status": "complete"},
        "workspace": {"status": "complete"},
        "cli": {"status": "pending"}
      },
      "archive_plan": {"move": [], "keep": [], "rewrite": [], "create_if_missing": []}
    }"#;

    #[test]
    fn help_and_version_are_available() {
        assert!(run_cli(["--help".to_string()])
            .expect("help")
            .contains("IGY6 local Rust CLI foundation"));
        assert!(run_cli(["version".to_string()])
            .expect("version")
            .starts_with("igy6-cli "));
    }

    #[test]
    fn manifest_validation_succeeds() {
        let manifest = parse_manifest(TEST_MANIFEST).expect("manifest");
        assert!(!manifest.cutover_ready);
        assert_eq!(manifest.phases.get("cli").expect("cli"), "pending");
    }

    #[test]
    fn known_phase_lookup_works() {
        let manifest = parse_manifest(TEST_MANIFEST).expect("manifest");
        assert_eq!(
            render_phase_status(&manifest, "cli").expect("phase"),
            "cli: pending\n"
        );
    }

    #[test]
    fn unknown_phase_is_rejected() {
        let manifest = parse_manifest(TEST_MANIFEST).expect("manifest");
        assert!(matches!(
            render_phase_status(&manifest, "missing"),
            Err(CliError::UnknownPhase(phase)) if phase == "missing"
        ));
    }

    #[test]
    fn missing_manifest_path_is_rejected() {
        assert!(matches!(
            run_cli(["phases".to_string()]),
            Err(CliError::MissingArgument("--manifest <path>"))
        ));
    }

    #[test]
    fn unsupported_command_is_rejected() {
        assert!(matches!(
            run_cli(["shell".to_string(), "rm -rf /".to_string()]),
            Err(CliError::UnsupportedCommand(command)) if command == "shell"
        ));
    }

    #[test]
    fn validates_manifest_from_path() {
        let mut path = env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        path.push(format!("igy6-cli-test-{unique}.json"));
        fs::write(&path, TEST_MANIFEST).expect("write manifest");

        let output = run_cli([
            "validate-manifest".to_string(),
            path.to_string_lossy().to_string(),
        ])
        .expect("validate");
        fs::remove_file(&path).expect("remove manifest");
        assert!(output.contains("manifest valid"));
    }
}
