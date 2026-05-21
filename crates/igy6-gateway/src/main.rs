use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use igy6_gateway::{
    handle_gateway_request_with_db, help_text, parse_gateway_request, render_http_response,
    GatewayResponse, DEFAULT_BIND_ADDR,
};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{}", help_text());
        return;
    }

    let bind_addr = parse_arg_value(&args, "--bind")
        .or_else(|| env::var("GATEWAY_BIND_ADDR").ok())
        .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());
    let database_url = env::var("DATABASE_URL").ok();

    if let Err(error) = serve(&bind_addr, database_url.as_deref()) {
        eprintln!("igy6-gateway failed: {error}");
        std::process::exit(1);
    }
}

fn parse_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn serve(bind_addr: &str, database_url: Option<&str>) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    println!("igy6-gateway listening on http://{bind_addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_stream(&mut stream, database_url) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_stream(stream: &mut TcpStream, database_url: Option<&str>) -> std::io::Result<()> {
    let mut buffer = [0_u8; 64 * 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let raw = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    let manifest = fs::read_to_string("configs/rust-cutover-manifest.json").ok();
    let response = match parse_gateway_request(&raw) {
        Ok(request) => {
            let response =
                handle_gateway_request_with_db(&request, manifest.as_deref(), "", database_url);
            response
        }
        Err(error) => GatewayResponse {
            status_code: 400,
            reason: "Bad Request".to_string(),
            content_type: "application/json".to_string(),
            body: format!("{{\"error\":\"bad_request\",\"message\":\"{error}\"}}"),
            proxied_to_fallback: false,
        },
    };
    stream.write_all(render_http_response(&response).as_bytes())
}
