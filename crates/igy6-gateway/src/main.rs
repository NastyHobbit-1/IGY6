use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use igy6_gateway::{
    build_fallback_proxy_plan, handle_gateway_request, help_text, parse_gateway_request,
    render_fallback_http_request, render_http_response, GatewayResponse, DEFAULT_BIND_ADDR,
    DEFAULT_FALLBACK_ORIGIN,
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
    let fallback_origin = parse_arg_value(&args, "--fallback")
        .or_else(|| env::var("FALLBACK_API_BASE_URL").ok())
        .unwrap_or_else(|| DEFAULT_FALLBACK_ORIGIN.to_string());

    if let Err(error) = serve(&bind_addr, &fallback_origin) {
        eprintln!("igy6-gateway failed: {error}");
        std::process::exit(1);
    }
}

fn parse_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn serve(bind_addr: &str, fallback_origin: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    println!("igy6-gateway listening on http://{bind_addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_stream(&mut stream, fallback_origin) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_stream(stream: &mut TcpStream, fallback_origin: &str) -> std::io::Result<()> {
    let mut buffer = [0_u8; 64 * 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let raw = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    let manifest = fs::read_to_string("configs/rust-cutover-manifest.json").ok();
    let response = match parse_gateway_request(&raw) {
        Ok(request) => {
            let response = handle_gateway_request(&request, manifest.as_deref(), fallback_origin);
            if response.proxied_to_fallback {
                match proxy_to_fallback(&request, fallback_origin) {
                    Ok(proxied) => proxied,
                    Err(error) => GatewayResponse {
                        status_code: 502,
                        reason: "Bad Gateway".to_string(),
                        content_type: "application/json".to_string(),
                        body: format!(
                            "{{\"detail\":\"FastAPI fallback proxy failed\",\"error\":\"{}\"}}",
                            json_escape(&error.to_string())
                        ),
                        proxied_to_fallback: true,
                    },
                }
            } else {
                response
            }
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

fn proxy_to_fallback(
    request: &igy6_gateway::GatewayRequest,
    fallback_origin: &str,
) -> std::io::Result<GatewayResponse> {
    let plan = build_fallback_proxy_plan(request, fallback_origin)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut stream = TcpStream::connect((plan.host.as_str(), plan.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    stream.set_write_timeout(Some(Duration::from_secs(15)))?;
    stream.write_all(render_fallback_http_request(&plan, request).as_bytes())?;

    let mut raw_response = String::new();
    stream.read_to_string(&mut raw_response)?;
    Ok(parse_fallback_response(&raw_response))
}

fn parse_fallback_response(raw: &str) -> GatewayResponse {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let status_line = head.lines().next().unwrap_or("HTTP/1.1 502 Bad Gateway");
    let mut parts = status_line.splitn(3, ' ');
    let _version = parts.next();
    let status_code = parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(502);
    let reason = parts.next().unwrap_or("Bad Gateway").to_string();
    let content_type = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-type") {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "application/json".to_string());
    GatewayResponse {
        status_code,
        reason,
        content_type,
        body: body.to_string(),
        proxied_to_fallback: true,
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
