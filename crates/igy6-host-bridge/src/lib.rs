use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const BRIDGE_VERSION: &str = "0.1.0";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8765;

const MAX_OUTPUT_CHARS: usize = 4000;
const ACTION_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_REACH_ACTION_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSpec {
    pub name: &'static str,
    pub runner: Option<&'static str>,
    pub script: &'static str,
    pub args: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeConfig {
    pub host: String,
    pub port: u16,
    pub repo_root: PathBuf,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub action_name: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub started_at: String,
    pub finished_at: String,
    pub bridge_version: String,
}

pub fn action_specs() -> &'static [ActionSpec] {
    &[
        ActionSpec {
            name: "start_stack",
            runner: None,
            script: "scripts/run.sh",
            args: &["--detached"],
        },
        ActionSpec {
            name: "stop_stack",
            runner: None,
            script: "scripts/stop.sh",
            args: &[],
        },
        ActionSpec {
            name: "run_last_healthy_stack",
            runner: None,
            script: "scripts/run-last-healthy-config.sh",
            args: &[],
        },
        ActionSpec {
            name: "deep_bypass_collect",
            runner: Some("node"),
            script: "apps/web/scripts/deep-bypass-collect.mjs",
            args: &[],
        },
        ActionSpec {
            name: "max_reach_collect",
            runner: Some("node"),
            script: "apps/web/scripts/max-reach-collect.mjs",
            args: &[],
        },
    ]
}

fn action_timeout(action_name: &str) -> Duration {
    if action_name == "max_reach_collect" {
        MAX_REACH_ACTION_TIMEOUT
    } else {
        ACTION_TIMEOUT
    }
}

pub fn allowed_action(action_name: &str) -> Option<&'static ActionSpec> {
    action_specs().iter().find(|spec| spec.name == action_name)
}

pub fn fixed_argv(repo_root: &Path, action_name: &str) -> Option<Vec<String>> {
    let spec = allowed_action(action_name)?;
    let script = repo_root.join(spec.script);
    // Shell scripts always use POSIX paths, even when the bridge runs on Windows hosts.
    let script_path = script.to_string_lossy().replace('\\', "/");
    if let Some(runner) = spec.runner {
        Some(vec![runner.to_string(), script_path])
    } else {
        let mut argv = vec![script_path];
        argv.extend(spec.args.iter().map(|arg| (*arg).to_string()));
        Some(argv)
    }
}

pub fn validate_bind_host(host: &str) -> Result<(), String> {
    if host == DEFAULT_HOST {
        Ok(())
    } else {
        Err("host bridge must bind only to 127.0.0.1".to_string())
    }
}

pub fn load_token(token: Option<String>, token_file: Option<PathBuf>) -> Result<String, String> {
    if let Some(value) = token {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            return Err("token is empty".to_string());
        }
        return Ok(trimmed);
    }

    let path = token_file.ok_or_else(|| "token or token file is required".to_string())?;
    let value = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read token file {}: {error}", path.display()))?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err("token file is empty".to_string());
    }
    Ok(trimmed)
}

pub fn token_authorized(headers: &HashMap<String, String>, expected_token: &str) -> bool {
    let Some(value) = headers.get("authorization") else {
        return false;
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return false;
    };
    constant_time_eq(token.as_bytes(), expected_token.as_bytes())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}

pub fn redact_output(value: &str) -> String {
    let mut redacted = Vec::new();
    for line in value.lines() {
        let lowered = line.to_ascii_lowercase();
        if [
            "password",
            "token",
            "secret",
            "database_url",
            "neo4j_password",
            "api_key",
        ]
        .iter()
        .any(|needle| lowered.contains(needle))
        {
            redacted.push("[redacted sensitive output line]".to_string());
        } else {
            redacted.push(line.to_string());
        }
    }
    let joined = redacted.join("\n");
    if joined.chars().count() <= MAX_OUTPUT_CHARS {
        joined
    } else {
        let mut truncated: String = joined.chars().take(MAX_OUTPUT_CHARS - 20).collect();
        truncated.push_str("\n[output truncated]");
        truncated
    }
}

pub fn execute_allowed_action(repo_root: &Path, action_name: &str) -> Result<ActionResult, String> {
    execute_allowed_action_with_env(repo_root, action_name, &HashMap::new())
}

pub fn execute_allowed_action_with_env(
    repo_root: &Path,
    action_name: &str,
    extra_env: &HashMap<String, String>,
) -> Result<ActionResult, String> {
    let spec = allowed_action(action_name).ok_or_else(|| "unknown action".to_string())?;
    let argv = fixed_argv(repo_root, action_name).ok_or_else(|| "unknown action".to_string())?;
    let script_path = if spec.runner.is_some() {
        repo_root.join(spec.script)
    } else {
        PathBuf::from(&argv[0])
    };
    if !script_path.is_file() {
        return Err(format!(
            "operator script not found: {}",
            script_path.display()
        ));
    }

    let started_at = now_utc_string();
    let mut child = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(repo_root)
        .envs(extra_env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start action {action_name}: {error}"))?;

    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to wait for action {action_name}: {error}"))?
        {
            let output = child.wait_with_output().map_err(|error| {
                format!("failed to capture action {action_name} output: {error}")
            })?;
            let exit_code = status.code();
            let action_status = if status.success() {
                "completed"
            } else {
                "failed"
            };
            return Ok(ActionResult {
                action_name: action_name.to_string(),
                status: action_status.to_string(),
                exit_code,
                stdout_summary: redact_output(&String::from_utf8_lossy(&output.stdout)),
                stderr_summary: redact_output(&String::from_utf8_lossy(&output.stderr)),
                started_at,
                finished_at: now_utc_string(),
                bridge_version: BRIDGE_VERSION.to_string(),
            });
        }

        if started.elapsed() > action_timeout(action_name) {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .map_err(|error| format!("failed to capture timed-out action output: {error}"))?;
            return Ok(ActionResult {
                action_name: action_name.to_string(),
                status: "timed_out".to_string(),
                exit_code: None,
                stdout_summary: redact_output(&String::from_utf8_lossy(&output.stdout)),
                stderr_summary: redact_output(&String::from_utf8_lossy(&output.stderr)),
                started_at,
                finished_at: now_utc_string(),
                bridge_version: BRIDGE_VERSION.to_string(),
            });
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn serve(config: BridgeConfig) -> Result<(), String> {
    validate_bind_host(&config.host)?;
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .map_err(|error| format!("failed to bind host bridge: {error}"))?;
    println!(
        "IGY6 host control bridge listening on {}:{}",
        config.host, config.port
    );

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let config = config.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_stream(&mut stream, &config) {
                        let _ =
                            write_response(&mut stream, 500, &json_error("internal_error", &error));
                    }
                });
            }
            Err(error) => eprintln!("host bridge connection failed: {error}"),
        }
    }
    Ok(())
}

fn parse_json_url_from_body(body: &str) -> Option<String> {
    let trimmed = body.trim();
    let key = "\"url\"";
    let start = trimmed.find(key)? + key.len();
    let after_key = trimmed.get(start..)?.trim_start();
    let after_key = after_key.strip_prefix(':')?.trim_start();
    if after_key.starts_with('"') {
        let inner = after_key.get(1..)?;
        let end = inner.find('"')?;
        let url = inner.get(..end)?.trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url.to_string());
        }
    }
    None
}

fn parse_json_depth_from_body(body: &str) -> Option<u32> {
    let trimmed = body.trim();
    let key = "\"max_depth\"";
    let start = trimmed.find(key)? + key.len();
    let after_key = trimmed.get(start..)?.trim_start();
    let after_key = after_key.strip_prefix(':')?.trim_start();
    let digits: String = after_key.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

pub fn handle_request_with_body(
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
    body: &str,
    config: &BridgeConfig,
) -> (u16, String) {
    handle_request_internal(method, path, headers, Some(body), config)
}

pub fn handle_request(
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
    config: &BridgeConfig,
) -> (u16, String) {
    handle_request_internal(method, path, headers, None, config)
}

fn handle_request_internal(
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
    body: Option<&str>,
    config: &BridgeConfig,
) -> (u16, String) {
    if !token_authorized(headers, &config.token) {
        return (401, json_error("unauthorized", "missing or invalid token"));
    }

    match (method, path) {
        ("GET", "/health") => (200, health_json()),
        ("GET", "/capabilities") => (200, capabilities_json(&config.repo_root)),
        ("POST", action_path) if action_path.starts_with("/actions/") => {
            let action_name = action_path.trim_start_matches("/actions/");
            if allowed_action(action_name).is_none() {
                return (
                    404,
                    json_error("unknown_action", "action is not allowlisted"),
                );
            }
            let mut extra_env = HashMap::new();
            if action_name == "deep_bypass_collect" || action_name == "max_reach_collect" {
                let Some(body_text) = body.filter(|value| !value.trim().is_empty()) else {
                    return (
                        400,
                        json_error(
                            "missing_body",
                            &format!("{action_name} requires JSON body with url"),
                        ),
                    );
                };
                let Some(url) = parse_json_url_from_body(body_text) else {
                    return (
                        400,
                        json_error(
                            "missing_url",
                            &format!("{action_name} requires JSON body with url"),
                        ),
                    );
                };
                if action_name == "deep_bypass_collect" {
                    extra_env.insert("DEEP_BYPASS_URL".to_string(), url);
                } else {
                    extra_env.insert("MAX_REACH_URL".to_string(), url);
                    if let Some(depth) = parse_json_depth_from_body(body_text) {
                        extra_env.insert("MAX_REACH_DEPTH".to_string(), depth.to_string());
                    }
                    if let Ok(headed) = std::env::var("MAX_REACH_HEADED") {
                        if !headed.trim().is_empty() {
                            extra_env.insert("MAX_REACH_HEADED".to_string(), headed);
                        }
                    }
                    if let Ok(proxy) = std::env::var("MAX_REACH_PROXY") {
                        if !proxy.trim().is_empty() {
                            extra_env.insert("MAX_REACH_PROXY".to_string(), proxy);
                        }
                    }
                    if let Ok(cdp) = std::env::var("MAX_REACH_CDP_PORT") {
                        if !cdp.trim().is_empty() {
                            extra_env.insert("MAX_REACH_CDP_PORT".to_string(), cdp);
                        }
                    }
                }
                if let Ok(data_root) = std::env::var("IGY6_DATA_ROOT") {
                    if !data_root.trim().is_empty() {
                        extra_env.insert("IGY6_DATA_ROOT".to_string(), data_root);
                    }
                }
            }
            match execute_allowed_action_with_env(&config.repo_root, action_name, &extra_env) {
                Ok(result) => (200, action_result_json(&result)),
                Err(error) => (500, json_error("action_failed", &error)),
            }
        }
        _ => (404, json_error("not_found", "route not found")),
    }
}

fn handle_stream(stream: &mut TcpStream, config: &BridgeConfig) -> Result<(), String> {
    let mut buffer = [0u8; 8192];
    let read_count = stream
        .read(&mut buffer)
        .map_err(|error| format!("failed to read request: {error}"))?;
    let request = String::from_utf8_lossy(&buffer[..read_count]);
    let (method, path, headers, request_body) = parse_request(&request)?;
    let (status, body) = if request_body.is_empty() {
        handle_request(&method, &path, &headers, config)
    } else {
        handle_request_with_body(&method, &path, &headers, &request_body, config)
    };
    write_response(stream, status, &body)
}

fn parse_request(request: &str) -> Result<(String, String, HashMap<String, String>, String), String> {
    let mut header_lines = request.lines();
    let request_line = header_lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "missing method".to_string())?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| "missing path".to_string())?
        .to_string();
    let mut headers = HashMap::new();
    for line in header_lines.by_ref() {
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let body = header_lines.collect::<Vec<_>>().join("\n");
    Ok((method, path, headers, body))
}

fn write_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("failed to write response: {error}"))
}

fn health_json() -> String {
    format!(
        "{{\"status\":\"ok\",\"bind_host\":\"{}\",\"bridge_version\":\"{}\"}}",
        DEFAULT_HOST, BRIDGE_VERSION
    )
}

fn capabilities_json(repo_root: &Path) -> String {
    let mut actions = String::new();
    for (index, spec) in action_specs().iter().enumerate() {
        if index > 0 {
            actions.push(',');
        }
        let argv = fixed_argv(repo_root, spec.name).unwrap_or_default();
        let script_exists = argv.first().is_some_and(|path| Path::new(path).is_file());
        let argv_json = argv
            .iter()
            .map(|item| format!("\"{}\"", json_escape(item)))
            .collect::<Vec<_>>()
            .join(",");
        let _ = write!(
            actions,
            "{{\"name\":\"{}\",\"script\":\"{}\",\"script_exists\":{},\"fixed_argv\":[{}]}}",
            spec.name,
            json_escape(spec.script),
            script_exists,
            argv_json
        );
    }
    format!(
        "{{\"bridge_version\":\"{}\",\"bind_host\":\"{}\",\"actions\":[{}]}}",
        BRIDGE_VERSION, DEFAULT_HOST, actions
    )
}

fn action_result_json(result: &ActionResult) -> String {
    format!(
        "{{\"action_name\":\"{}\",\"status\":\"{}\",\"exit_code\":{},\"stdout_summary\":\"{}\",\"stderr_summary\":\"{}\",\"started_at\":\"{}\",\"finished_at\":\"{}\",\"bridge_version\":\"{}\"}}",
        json_escape(&result.action_name),
        json_escape(&result.status),
        result
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "null".to_string()),
        json_escape(&result.stdout_summary),
        json_escape(&result.stderr_summary),
        json_escape(&result.started_at),
        json_escape(&result.finished_at),
        json_escape(&result.bridge_version)
    )
}

fn json_error(code: &str, detail: &str) -> String {
    format!(
        "{{\"error\":\"{}\",\"detail\":\"{}\",\"bridge_version\":\"{}\"}}",
        json_escape(code),
        json_escape(detail),
        BRIDGE_VERSION
    )
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn now_utc_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    format!("{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BridgeConfig {
        BridgeConfig {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            repo_root: PathBuf::from("/tmp/igy6-test"),
            token: "test-token".to_string(),
        }
    }

    fn auth_headers(token: &str) -> HashMap<String, String> {
        HashMap::from([("authorization".to_string(), format!("Bearer {token}"))])
    }

    #[test]
    fn token_required() {
        let config = test_config();
        let headers = HashMap::new();
        let (status, body) = handle_request("GET", "/capabilities", &headers, &config);
        assert_eq!(status, 401);
        assert!(body.contains("unauthorized"));
    }

    #[test]
    fn invalid_token_rejected() {
        let config = test_config();
        let (status, _) = handle_request("GET", "/capabilities", &auth_headers("wrong"), &config);
        assert_eq!(status, 401);
    }

    #[test]
    fn unknown_action_rejected() {
        let config = test_config();
        let (status, body) = handle_request(
            "POST",
            "/actions/rm%20-rf%20/",
            &auth_headers("test-token"),
            &config,
        );
        assert_eq!(status, 404);
        assert!(body.contains("unknown_action"));
    }

    #[test]
    fn allowed_action_maps_to_fixed_argv() {
        let root = PathBuf::from("/repo");
        let argv = fixed_argv(&root, "start_stack").expect("start_stack is allowlisted");
        assert_eq!(argv, vec!["/repo/scripts/run.sh", "--detached"]);

        let stop_argv = fixed_argv(&root, "stop_stack").expect("stop_stack is allowlisted");
        assert_eq!(stop_argv, vec!["/repo/scripts/stop.sh"]);

        let deep_argv =
            fixed_argv(&root, "deep_bypass_collect").expect("deep_bypass_collect is allowlisted");
        assert_eq!(
            deep_argv,
            vec!["node", "/repo/apps/web/scripts/deep-bypass-collect.mjs"]
        );

        let max_argv =
            fixed_argv(&root, "max_reach_collect").expect("max_reach_collect is allowlisted");
        assert_eq!(
            max_argv,
            vec!["node", "/repo/apps/web/scripts/max-reach-collect.mjs"]
        );
    }

    #[test]
    fn deep_bypass_collect_requires_url_body() {
        let config = test_config();
        let (status, body) = handle_request_with_body(
            "POST",
            "/actions/deep_bypass_collect",
            &auth_headers("test-token"),
            "{}",
            &config,
        );
        assert_eq!(status, 400);
        assert!(body.contains("missing_url"));
    }

    #[test]
    fn max_reach_collect_requires_url_body() {
        let config = test_config();
        let (status, body) = handle_request_with_body(
            "POST",
            "/actions/max_reach_collect",
            &auth_headers("test-token"),
            "{}",
            &config,
        );
        assert_eq!(status, 400);
        assert!(body.contains("missing_url"));
    }

    #[test]
    fn dangerous_command_string_is_not_action() {
        assert!(allowed_action("rm -rf /").is_none());
        assert!(allowed_action("bash -c docker compose down").is_none());
    }

    #[test]
    fn output_redaction_removes_sensitive_lines() {
        let redacted = redact_output("ok\nPOSTGRES_PASSWORD=secret\napi_token=value\nsafe");
        assert!(redacted.contains("ok"));
        assert!(redacted.contains("safe"));
        assert!(!redacted.contains("POSTGRES_PASSWORD"));
        assert!(!redacted.contains("api_token"));
        assert_eq!(
            redacted.matches("[redacted sensitive output line]").count(),
            2
        );
    }

    #[test]
    fn non_local_bind_rejected() {
        assert!(validate_bind_host("127.0.0.1").is_ok());
        assert!(validate_bind_host("0.0.0.0").is_err());
        assert!(validate_bind_host("localhost").is_err());
    }
}
