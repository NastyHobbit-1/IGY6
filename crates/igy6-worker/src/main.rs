use std::env;
use std::process;

use igy6_worker::{
    execute_worker_live_canary, parse_usize_setting, parse_worker_runtime_args,
    plan_worker_runtime, render_worker_live_canary_result, render_worker_runtime_status,
    worker_runtime_help, WorkerRuntimeConfig, DEFAULT_DATABASE_URL, DEFAULT_IGY6_DATA_ROOT,
    DEFAULT_QDRANT_CHUNK_COLLECTION, DEFAULT_QDRANT_CHUNK_VECTOR_SIZE, DEFAULT_QDRANT_URL,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("igy6-worker error: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = parse_worker_runtime_args(env::args().skip(1)).map_err(|error| error.to_string())?;
    if matches!(args.mode, igy6_worker::WorkerRuntimeMode::Help) {
        print!("{}", worker_runtime_help());
        return Ok(());
    }

    let config = config_from_env(&args)?;
    let plan =
        plan_worker_runtime(args.clone(), config.clone()).map_err(|error| error.to_string())?;
    if args.canary_live && plan.live_execution_enabled {
        let result =
            execute_worker_live_canary(&args, &config).map_err(|error| error.to_string())?;
        println!("{}", render_worker_live_canary_result(&result, &config));
    } else {
        println!("{}", render_worker_runtime_status(&plan, &config));
    }
    Ok(())
}

fn config_from_env(args: &igy6_worker::WorkerRuntimeArgs) -> Result<WorkerRuntimeConfig, String> {
    Ok(WorkerRuntimeConfig {
        database_url: env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string()),
        qdrant_url: env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string()),
        igy6_data_root: env::var("IGY6_DATA_ROOT")
            .unwrap_or_else(|_| DEFAULT_IGY6_DATA_ROOT.to_string()),
        qdrant_chunk_collection: env::var("QDRANT_CHUNK_COLLECTION")
            .unwrap_or_else(|_| DEFAULT_QDRANT_CHUNK_COLLECTION.to_string()),
        qdrant_chunk_vector_size: parse_usize_setting(
            env::var("QDRANT_CHUNK_VECTOR_SIZE").ok().as_deref(),
            DEFAULT_QDRANT_CHUNK_VECTOR_SIZE,
        )?,
        claim_limit: args.claim_limit,
        poll_interval_ms: args.poll_interval_ms,
        max_jobs: args.max_jobs,
        max_idle_polls: args.max_idle_polls,
        live_execution_enabled: env::var("IGY6_WORKER_LIVE_CANARY")
            .map(|value| value == "DIFF-148")
            .unwrap_or(false),
        process_canary_enabled: env::var("IGY6_WORKER_PROCESS_CANARY")
            .map(|value| value == "DIFF-159")
            .unwrap_or(false),
    })
}
