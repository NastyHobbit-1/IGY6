use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use igy6_config::{render_cli_report, validate_repo_config};
use igy6_policy::{ActionRisk, ApprovalRequirement};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_SCRIPT_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const RUN_SCRIPT: &[&str] = &["scripts/run.sh"];
const STOP_SCRIPT: &[&str] = &["scripts/stop.sh"];
const RUN_LAST_HEALTHY_SCRIPT: &[&str] = &["scripts/run-last-healthy-config.sh"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSummary {
    pub cutover_ready: bool,
    pub phases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutcome {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CliOutcome {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            exit_code: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingArgument(&'static str),
    UnsupportedCommand(String),
    MissingManifest(String),
    InvalidManifest(String),
    UnknownPhase(String),
    InvalidCommandShape(&'static str),
    RepoRootNotFound,
    RepoCheckFailed(Vec<String>),
    ConfigCheckFailed(Vec<String>),
    ProcessLaunch(String),
    ProcessTimeout(String),
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
            Self::InvalidCommandShape(shape) => write!(formatter, "invalid command shape: {shape}"),
            Self::RepoRootNotFound => write!(formatter, "could not find IGY6 repository root"),
            Self::RepoCheckFailed(items) => {
                write!(formatter, "repo health checks failed: {}", items.join(", "))
            }
            Self::ConfigCheckFailed(items) => {
                write!(formatter, "config checks failed: {}", items.join(", "))
            }
            Self::ProcessLaunch(reason) => write!(formatter, "process launch failed: {reason}"),
            Self::ProcessTimeout(command) => write!(formatter, "process timed out: {command}"),
        }
    }
}

impl std::error::Error for CliError {}

enum CommandAction {
    Render(String),
    RunFixedArgv(&'static [&'static str]),
    StartWithBrowser,
}

pub fn execute_cli<I>(args: I) -> Result<CliOutcome, CliError>
where
    I: IntoIterator<Item = String>,
{
    let repo_root = find_repo_root(&env::current_dir().map_err(|_| CliError::RepoRootNotFound)?)?;
    let action = plan_cli(args, &repo_root)?;
    match action {
        CommandAction::Render(output) => Ok(CliOutcome::success(output)),
        CommandAction::RunFixedArgv(argv) => {
            run_fixed_argv(argv, &repo_root, DEFAULT_SCRIPT_TIMEOUT)
        }
        CommandAction::StartWithBrowser => {
            start_stack_and_open_browser(&repo_root)
        }
    }
}

pub fn run_cli<I>(args: I) -> Result<String, CliError>
where
    I: IntoIterator<Item = String>,
{
    let outcome = execute_cli(args)?;
    if outcome.exit_code == 0 {
        Ok(format!("{}{}", outcome.stdout, outcome.stderr))
    } else {
        Err(CliError::ProcessLaunch(format!(
            "fixed-argv command exited with code {}",
            outcome.exit_code
        )))
    }
}

fn plan_cli<I>(args: I, repo_root: &Path) -> Result<CommandAction, CliError>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let Some(command) = args.first().map(String::as_str) else {
        // Bare `igy6` launches the full program (starts stack + opens browser)
        return Ok(CommandAction::StartWithBrowser);
    };

    match command {
        "--help" | "-h" | "help" => Ok(CommandAction::Render(help_text())),
        "version" => Ok(CommandAction::Render(format!("igy6 {VERSION}\n"))),
        "health" => {
            require_exact_len(&args, 1, "igy6 health")?;
            Ok(CommandAction::Render(render_health(repo_root)?))
        }
        "run" => {
            require_exact_len(&args, 1, "igy6 run")?;
            Ok(CommandAction::RunFixedArgv(RUN_SCRIPT))
        }
        "start" => {
            require_exact_len(&args, 1, "igy6 start")?;
            // Special handling below in execute
            Ok(CommandAction::StartWithBrowser)
        }
        "stop" => {
            require_exact_len(&args, 1, "igy6 stop")?;
            Ok(CommandAction::RunFixedArgv(STOP_SCRIPT))
        }
        "run-last-healthy" => {
            require_exact_len(&args, 1, "igy6 run-last-healthy")?;
            Ok(CommandAction::RunFixedArgv(RUN_LAST_HEALTHY_SCRIPT))
        }
        "config" => {
            let subcommand = args
                .get(1)
                .ok_or(CliError::MissingArgument("config subcommand"))?;
            match subcommand.as_str() {
                "check" => {
                    require_exact_len(&args, 2, "igy6 config check")?;
                    Ok(CommandAction::Render(render_config_check(repo_root)?))
                }
                other => Err(CliError::UnsupportedCommand(format!("config {other}"))),
            }
        }
        "snapshot" => {
            let subcommand = args
                .get(1)
                .ok_or(CliError::MissingArgument("snapshot subcommand"))?;
            match subcommand.as_str() {
                "show" => {
                    require_exact_len(&args, 2, "igy6 snapshot show")?;
                    Ok(CommandAction::Render(snapshot_show_placeholder()))
                }
                other => Err(CliError::UnsupportedCommand(format!("snapshot {other}"))),
            }
        }
        "phases" => {
            let manifest_path = manifest_arg(&args[1..])?;
            let manifest = load_manifest(&manifest_path)?;
            Ok(CommandAction::Render(render_phases(&manifest)))
        }
        "phase-status" => {
            let phase = args
                .get(1)
                .ok_or(CliError::MissingArgument("phase"))?
                .to_string();
            let manifest_path = manifest_arg(&args[2..])?;
            let manifest = load_manifest(&manifest_path)?;
            Ok(CommandAction::Render(render_phase_status(
                &manifest, &phase,
            )?))
        }
        "validate-manifest" => {
            let path = args
                .get(1)
                .ok_or(CliError::MissingArgument("manifest path"))?
                .to_string();
            require_exact_len(&args, 2, "igy6 validate-manifest <path>")?;
            let manifest = load_manifest(&path)?;
            Ok(CommandAction::Render(format!(
                "manifest valid\ncutover_ready: {}\nphase_count: {}\n",
                manifest.cutover_ready,
                manifest.phases.len()
            )))
        }
        other => Err(CliError::UnsupportedCommand(other.to_string())),
    }
}

pub fn help_text() -> String {
    let read_only = ApprovalRequirement::for_action(ActionRisk::ReadOnly);
    format!(
        "IGY6 local Rust CLI\n\n\
Usage:\n  \
igy6                 # Start the full stack (detached) + open browser to UI\n  \
igy6 start           # Same as above\n  \
igy6 --help\n  \
igy6 health\n  \
igy6 run             # Foreground logs (old scripts/run.sh behavior)\n  \
igy6 stop\n  \
igy6 run-last-healthy\n  \
igy6 config check\n  \
igy6 snapshot show\n  \
igy6 version\n\n\
Admin manifest commands:\n  \
igy6 phases --manifest <path>\n  \
igy6 phase-status <phase> --manifest <path>\n  \
igy6 validate-manifest <path>\n\n\
Safety:\n  \
local-only: true\n  \
health/config/snapshot: read-only\n  \
script wrappers: fixed argv only\n  \
approval_required_for_read_only: {}\n  \
external_model_calls: false\n",
        read_only.required
    )
}

pub fn render_health(repo_root: &Path) -> Result<String, CliError> {
    let required_files = [
        "AGENTS.md",
        "Cargo.toml",
        "infra/docker-compose.yml",
        "scripts/run.sh",
        "scripts/stop.sh",
        "scripts/run-last-healthy-config.sh",
        "scripts/lib/igy6-ops.sh",
        "configs/rust-cutover-manifest.json",
    ];
    let mut failures = Vec::new();
    let mut output = String::from("IGY6 local health\n");

    for relative in required_files {
        let exists = repo_root.join(relative).is_file();
        output.push_str(&format!(
            "{}: {}\n",
            relative,
            if exists { "ok" } else { "missing" }
        ));
        if !exists {
            failures.push(relative.to_string());
        }
    }

    for command in ["cargo", "git"] {
        let available = command_available(command);
        output.push_str(&format!(
            "tool {command}: {}\n",
            if available { "ok" } else { "missing" }
        ));
        if !available {
            failures.push(format!("tool {command}"));
        }
    }

    let docker_available = command_available("docker");
    output.push_str(&format!(
        "tool docker: {}\n",
        if docker_available {
            "available"
        } else {
            "not found; script wrappers will report this if used"
        }
    ));

    if failures.is_empty() {
        output.push_str("status: ok\n");
        Ok(output)
    } else {
        Err(CliError::RepoCheckFailed(failures))
    }
}

pub fn render_config_check(repo_root: &Path) -> Result<String, CliError> {
    let manifest_path = repo_root.join("configs/rust-cutover-manifest.json");
    let compose_path = repo_root.join("infra/docker-compose.yml");
    let report = validate_repo_config(repo_root)
        .map_err(|error| CliError::ConfigCheckFailed(vec![error.to_string()]))?;
    let mut failures = report.error_messages();

    if !manifest_path.is_file() {
        failures.push("configs/rust-cutover-manifest.json missing".to_string());
    }
    if !compose_path.is_file() {
        failures.push("infra/docker-compose.yml missing".to_string());
    }

    if manifest_path.is_file() {
        let manifest = load_manifest(&manifest_path.to_string_lossy())?;
        if !manifest.phases.contains_key("cli") {
            failures.push("manifest missing cli phase".to_string());
        }
    }

    if failures.is_empty() {
        let mut output = render_cli_report(&report);
        output.push_str("manifest: ok\ncompose file: ok\n");
        Ok(output)
    } else {
        Err(CliError::ConfigCheckFailed(failures))
    }
}

pub fn snapshot_show_placeholder() -> String {
    "IGY6 snapshot show\nstatus: not implemented in Rust yet\nbehavior: non-destructive placeholder\nruntime_data_read: false\nnote: existing Bash snapshot behavior remains in scripts/run-last-healthy-config.sh; Rust snapshot reading is deferred to a later DIFF that can safely define IGY6_DATA_ROOT access rules.\n".to_string()
}

pub fn run_fixed_argv(
    argv: &'static [&'static str],
    repo_root: &Path,
    timeout: Duration,
) -> Result<CliOutcome, CliError> {
    if argv.is_empty() {
        return Err(CliError::InvalidCommandShape(
            "fixed argv must not be empty",
        ));
    }

    let mut child = Command::new(argv[0])
        .args(&argv[1..])
        .current_dir(repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| CliError::ProcessLaunch(error.to_string()))?;

    let deadline = Instant::now() + timeout;
    loop {
        if child
            .try_wait()
            .map_err(|error| CliError::ProcessLaunch(error.to_string()))?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| CliError::ProcessLaunch(error.to_string()))?;
            return Ok(CliOutcome {
                stdout: redact_sensitive_output(&String::from_utf8_lossy(&output.stdout)),
                stderr: redact_sensitive_output(&String::from_utf8_lossy(&output.stderr)),
                exit_code: output.status.code().unwrap_or(1),
            });
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CliError::ProcessTimeout(argv.join(" ")));
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/// Starts the IGY6 stack in detached mode (triggering bootstrap if needed),
/// waits for the web UI, then opens the browser. This makes the compiled
/// `igy6` binary act like a normal executable program.
/// Works on Linux, macOS, and Windows (requires Docker Desktop on Windows).
pub fn start_stack_and_open_browser(repo_root: &Path) -> Result<CliOutcome, CliError> {
    // Cross-platform bootstrap: create .env from example if missing (grok branch defaults)
    let env_file = repo_root.join(".env");
    if !env_file.exists() {
        let example = repo_root.join(".env.example");
        if example.exists() {
            fs::copy(&example, &env_file)
                .map_err(|e| CliError::ProcessLaunch(format!("Failed to copy .env.example: {}", e)))?;
            
            // Set grok-friendly defaults
            let data_dir = dirs::home_dir()
                .map(|h| h.join("IGY6_Data").to_string_lossy().to_string())
                .unwrap_or_else(|| "./IGY6_Data".to_string());
            
            // Simple in-place edits for key vars
            let mut content = fs::read_to_string(&env_file)
                .map_err(|e| CliError::ProcessLaunch(format!("Failed to read .env: {}", e)))?;
            
            content = content.replace("IGY6_DATA_ROOT=../IGY6_Data", &format!("IGY6_DATA_ROOT={}", data_dir));
            if !content.contains("SINGLE_USER_MODE=") {
                content.push_str("\nSINGLE_USER_MODE=true\n");
            } else {
                content = content.replace("SINGLE_USER_MODE=false", "SINGLE_USER_MODE=true");
            }
            if !content.contains("GROK BRANCH NOTE") {
                content.push_str(&format!(
                    "\n# GROK BRANCH NOTE (auto-added by igy6 installer)\n\
                     # Default program password is ThatDog123 (change in UI User & Security after first unlock).\n\
                     # All deep/safe collection, Media Library, TOTP linking, etc. is done from the web UI.\n\
                     # Data lives under {} (including full-res images/videos from safe sources only).\n\
                     # Telemetry disabled.\n",
                    data_dir
                ));
            }
            
            fs::write(&env_file, content)
                .map_err(|e| CliError::ProcessLaunch(format!("Failed to write .env: {}", e)))?;
            
            println!("Created .env with grok defaults (password: ThatDog123, data dir: {})", data_dir);
        }
    }

    // Start the stack detached (builds if needed). Uses "docker compose" (works on Docker Desktop Windows too)
    println!("Starting IGY6 stack (detached mode)...");
    let up_status = Command::new("docker")
        .args([
            "compose",
            "-f",
            "infra/docker-compose.yml",
            "--env-file",
            ".env",
            "up",
            "-d",
            "--build",
        ])
        .current_dir(repo_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| CliError::ProcessLaunch(format!("docker compose failed: {}", e)))?;

    if !up_status.success() {
        return Err(CliError::ProcessLaunch("docker compose up -d failed (is Docker running?)".to_string()));
    }

    // Wait for the web UI (port 3000) to be ready - cross platform TCP check
    wait_for_port("127.0.0.1", 3000, Duration::from_secs(180))?;

    let url = "http://127.0.0.1:3000";
    println!("Web UI is ready at {}", url);

    // Open the browser (webbrowser crate handles Windows, Linux, macOS)
    match webbrowser::open(url) {
        Ok(_) => println!("Opened browser to the IGY6 UI."),
        Err(e) => eprintln!("Could not auto-open browser ({}). Please visit {} manually.", e, url),
    }

    println!("\nIGY6 is running as a compiled executable.");
    println!("- To view logs: docker compose -f infra/docker-compose.yml --env-file .env logs -f web");
    println!("- To stop: igy6 stop");
    println!("- Password for UI: ThatDog123 (change in User & Security section)");
    println!("- Telemetry is disabled.");

    Ok(CliOutcome::success(format!("Started and opened {}\n", url)))
}

fn wait_for_port(host: &str, port: u16, timeout: Duration) -> Result<(), CliError> {
    println!("Waiting for UI on {}:{} (up to {}s)...", host, port, timeout.as_secs());
    let start = Instant::now();
    let addr = format!("{}:{}", host, port);

    loop {
        if start.elapsed() > timeout {
            return Err(CliError::ProcessTimeout(format!("UI on {}:{}", host, port)));
        }

        if let Ok(_) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_secs(2),
        ) {
            // Port is open — give it a couple more seconds for the server to be fully ready
            std::thread::sleep(Duration::from_secs(3));
            return Ok(());
        }

        std::thread::sleep(Duration::from_secs(2));
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
}

pub fn redact_sensitive_output(output: &str) -> String {
    output
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if ["password", "token", "secret", "key=", "database_url"]
                .iter()
                .any(|needle| lower.contains(needle))
            {
                "[REDACTED sensitive-looking output]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if output.ends_with('\n') { "\n" } else { "" }
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

fn require_exact_len(
    args: &[String],
    expected: usize,
    shape: &'static str,
) -> Result<(), CliError> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(CliError::InvalidCommandShape(shape))
    }
}

fn find_repo_root(start: &Path) -> Result<PathBuf, CliError> {
    // Allow overriding via env var for installed binaries (useful on Windows and for global installs)
    if let Ok(env_root) = env::var("IGY6_REPO") {
        let p = PathBuf::from(env_root);
        if p.join("Cargo.toml").is_file()
            && p.join("configs/rust-cutover-manifest.json").is_file()
            && p.join("infra/docker-compose.yml").is_file()
        {
            return Ok(p);
        }
    }

    for candidate in start.ancestors() {
        if candidate.join("Cargo.toml").is_file()
            && candidate
                .join("configs/rust-cutover-manifest.json")
                .is_file()
            && candidate.join("infra/docker-compose.yml").is_file()
        {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(CliError::RepoRootNotFound)
}

fn command_available(command: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path_var).any(|directory| directory.join(command).is_file())
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
            .contains("igy6 health"));
        assert!(run_cli(["version".to_string()])
            .expect("version")
            .starts_with("igy6 "));
    }

    #[test]
    fn required_commands_are_planned() {
        let repo_root = find_repo_root(&env::current_dir().expect("cwd")).expect("repo");
        assert!(matches!(
            plan_cli(["run".to_string()], &repo_root).expect("run"),
            CommandAction::RunFixedArgv(argv) if argv == RUN_SCRIPT
        ));
        assert!(matches!(
            plan_cli(["stop".to_string()], &repo_root).expect("stop"),
            CommandAction::RunFixedArgv(argv) if argv == STOP_SCRIPT
        ));
        assert!(matches!(
            plan_cli(["run-last-healthy".to_string()], &repo_root).expect("last healthy"),
            CommandAction::RunFixedArgv(argv) if argv == RUN_LAST_HEALTHY_SCRIPT
        ));
    }

    #[test]
    fn config_and_snapshot_commands_render_without_private_runtime_reads() {
        let repo_root = find_repo_root(&env::current_dir().expect("cwd")).expect("repo");
        let config =
            plan_cli(["config".to_string(), "check".to_string()], &repo_root).expect("config");
        assert!(
            matches!(config, CommandAction::Render(output) if output.contains("values: redacted") && output.contains("runtime_data_read: false"))
        );
        let snapshot =
            plan_cli(["snapshot".to_string(), "show".to_string()], &repo_root).expect("snapshot");
        assert!(
            matches!(snapshot, CommandAction::Render(output) if output.contains("runtime_data_read: false"))
        );
    }

    #[test]
    fn command_shapes_are_strict() {
        let repo_root = find_repo_root(&env::current_dir().expect("cwd")).expect("repo");
        assert!(matches!(
            plan_cli(["run".to_string(), "--detached".to_string()], &repo_root),
            Err(CliError::InvalidCommandShape("igy6 run"))
        ));
        assert!(matches!(
            plan_cli(["config".to_string()], &repo_root),
            Err(CliError::MissingArgument("config subcommand"))
        ));
    }

    #[test]
    fn redacts_sensitive_looking_output() {
        let redacted = redact_sensitive_output("DATABASE_URL=postgres://x\nnormal line\n");
        assert!(redacted.contains("[REDACTED sensitive-looking output]"));
        assert!(redacted.contains("normal line"));
        assert!(!redacted.contains("postgres://x"));
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

    #[test]
    fn fixed_argv_rejects_empty_input() {
        assert!(matches!(
            run_fixed_argv(&[], Path::new("."), Duration::from_millis(1)),
            Err(CliError::InvalidCommandShape(
                "fixed argv must not be empty"
            ))
        ));
    }
}
