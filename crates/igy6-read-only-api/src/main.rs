use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use igy6_read_only_api::{
    handle_request, help_text, parse_http_request, render_http_response, HttpResponse,
    DEFAULT_BIND_ADDR,
};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{}", help_text());
        return;
    }

    let bind_addr = parse_bind_addr(&args).unwrap_or(DEFAULT_BIND_ADDR.to_string());
    if let Err(error) = serve(&bind_addr) {
        eprintln!("igy6-read-only-api failed: {error}");
        std::process::exit(1);
    }
}

fn parse_bind_addr(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == "--bind")
        .map(|pair| pair[1].clone())
}

fn serve(bind_addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    println!("igy6-read-only-api listening on http://{bind_addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_stream(&mut stream) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_stream(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0_u8; 4096];
    let bytes_read = stream.read(&mut buffer)?;
    let raw = String::from_utf8_lossy(&buffer[..bytes_read]);
    let manifest = fs::read_to_string("configs/rust-cutover-manifest.json").ok();
    let response = match parse_http_request(&raw) {
        Ok(request) => handle_request(&request, manifest.as_deref()),
        Err(error) => HttpResponse {
            status_code: 400,
            reason: "Bad Request".to_string(),
            content_type: "application/json".to_string(),
            body: format!("{{\"error\":\"bad_request\",\"message\":\"{error}\"}}"),
        },
    };
    stream.write_all(render_http_response(&response).as_bytes())
}
