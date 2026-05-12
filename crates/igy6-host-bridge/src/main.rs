use std::env;
use std::path::PathBuf;

use igy6_host_bridge::{
    load_token, serve, validate_bind_host, BridgeConfig, DEFAULT_HOST, DEFAULT_PORT,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut host = env::var("IGY6_HOST_BRIDGE_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let mut port = env::var("IGY6_HOST_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);
    let mut repo_root = env::var("IGY6_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut token = env::var("IGY6_HOST_BRIDGE_TOKEN").ok();
    let mut token_file = env::var("IGY6_HOST_BRIDGE_TOKEN_FILE")
        .ok()
        .map(PathBuf::from);

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                index += 1;
                host = args
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--host requires a value".to_string())?;
            }
            "--port" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--port requires a value".to_string())?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| "--port must be an integer from 1 to 65535".to_string())?;
            }
            "--repo-root" => {
                index += 1;
                repo_root = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--repo-root requires a value".to_string())?,
                );
            }
            "--token-file" => {
                index += 1;
                token_file =
                    Some(PathBuf::from(args.get(index).ok_or_else(|| {
                        "--token-file requires a value".to_string()
                    })?));
            }
            "--token" => {
                index += 1;
                token = Some(
                    args.get(index)
                        .ok_or_else(|| "--token requires a value".to_string())?
                        .to_string(),
                );
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }

    validate_bind_host(&host)?;
    let token = load_token(token, token_file)?;
    let config = BridgeConfig {
        host,
        port,
        repo_root,
        token,
    };
    serve(config)
}

fn print_help() {
    println!(
        "Usage: igy6-host-bridge [--host 127.0.0.1] [--port 8765] [--repo-root PATH] [--token-file PATH]\n\
\n\
Environment:\n\
  IGY6_HOST_BRIDGE_HOST       Defaults to 127.0.0.1. Other hosts are rejected.\n\
  IGY6_HOST_BRIDGE_PORT       Defaults to 8765.\n\
  IGY6_HOST_BRIDGE_TOKEN      Token value. Do not put this in git.\n\
  IGY6_HOST_BRIDGE_TOKEN_FILE Token file path. Preferred for local use.\n\
  IGY6_REPO_ROOT              Repository root containing scripts/run.sh.\n\
\n\
Endpoints require Authorization: Bearer <token>.\n\
Allowed actions only: start_stack, stop_stack, run_last_healthy_stack."
    );
}
