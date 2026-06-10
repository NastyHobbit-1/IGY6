use std::fmt;

pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8766";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnlyApiError {
    EmptyRequest,
    MalformedRequestLine,
}

impl fmt::Display for ReadOnlyApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(formatter, "request is empty"),
            Self::MalformedRequestLine => write!(formatter, "request line is malformed"),
        }
    }
}

impl std::error::Error for ReadOnlyApiError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub reason: String,
    pub content_type: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSummary {
    pub cutover_ready: bool,
    pub complete_phases: usize,
    pub pending_phases: usize,
}

pub fn parse_http_request(raw: &str) -> Result<HttpRequest, ReadOnlyApiError> {
    let request_line = raw.lines().next().ok_or(ReadOnlyApiError::EmptyRequest)?;
    if request_line.trim().is_empty() {
        return Err(ReadOnlyApiError::EmptyRequest);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or(ReadOnlyApiError::MalformedRequestLine)?;
    let path = parts.next().ok_or(ReadOnlyApiError::MalformedRequestLine)?;
    let version = parts.next().ok_or(ReadOnlyApiError::MalformedRequestLine)?;
    if !version.starts_with("HTTP/") || parts.next().is_some() {
        return Err(ReadOnlyApiError::MalformedRequestLine);
    }
    Ok(HttpRequest {
        method: method.to_string(),
        path: path.to_string(),
    })
}

pub fn handle_request(request: &HttpRequest, manifest_content: Option<&str>) -> HttpResponse {
    if request.method != "GET" {
        return json_response(
            405,
            "Method Not Allowed",
            "{\"error\":\"method_not_allowed\",\"allowed\":\"GET\"}".to_string(),
        );
    }

    match request.path.as_str() {
        "/health/live" => json_response(
            200,
            "OK",
            "{\"status\":\"ok\",\"service\":\"igy6-read-only-api\"}".to_string(),
        ),
        "/health/ready" => json_response(
            200,
            "OK",
            "{\"status\":\"ok\",\"checks\":{\"rust_sidecar\":{\"status\":\"ok\"}},\"primary_gateway\":\"rust\"}".to_string(),
        ),
        "/rust-migration/status" => {
            let summary = summarize_manifest(manifest_content.unwrap_or_default());
            json_response(
                200,
                "OK",
                format!(
                    "{{\"status\":\"ok\",\"cutover_ready\":{},\"complete_phases\":{},\"pending_phases\":{},\"primary_gateway\":\"rust\"}}",
                    summary.cutover_ready, summary.complete_phases, summary.pending_phases
                ),
            )
        }
        _ => json_response(
            404,
            "Not Found",
            "{\"error\":\"not_found\",\"message\":\"read-only route is not available\"}"
                .to_string(),
        ),
    }
}

pub fn summarize_manifest(content: &str) -> ManifestSummary {
    ManifestSummary {
        cutover_ready: content.contains("\"cutover_ready\": true"),
        complete_phases: content.matches("\"status\": \"complete\"").count(),
        pending_phases: content.matches("\"status\": \"pending\"").count(),
    }
}

pub fn render_http_response(response: &HttpResponse) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_code,
        response.reason,
        response.content_type,
        response.body.len(),
        response.body
    )
}

pub fn help_text() -> &'static str {
    "igy6-read-only-api\n\nUsage:\n  igy6-read-only-api [--bind 127.0.0.1:8766]\n  igy6-read-only-api --help\n\nRoutes:\n  GET /health/live\n  GET /health/ready\n  GET /rust-migration/status\n\nRead-only migration sidecar library. The Rust gateway is the primary API.\n"
}

fn json_response(status_code: u16, reason: &str, body: String) -> HttpResponse {
    HttpResponse {
        status_code,
        reason: reason.to_string(),
        content_type: "application/json".to_string(),
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_request_line() {
        let request = parse_http_request("GET /health/live HTTP/1.1\r\nHost: localhost\r\n")
            .expect("request");
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/health/live");
    }

    #[test]
    fn rejects_malformed_request_line() {
        assert_eq!(
            parse_http_request("GET /health/live\r\n").expect_err("error"),
            ReadOnlyApiError::MalformedRequestLine
        );
        assert_eq!(
            parse_http_request("").expect_err("error"),
            ReadOnlyApiError::EmptyRequest
        );
    }

    #[test]
    fn live_route_is_safe_and_read_only() {
        let response = handle_request(
            &HttpRequest {
                method: "GET".to_string(),
                path: "/health/live".to_string(),
            },
            None,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"service\":\"igy6-read-only-api\""));
    }

    #[test]
    fn ready_route_reports_rust_primary() {
        let response = handle_request(
            &HttpRequest {
                method: "GET".to_string(),
                path: "/health/ready".to_string(),
            },
            None,
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"primary_gateway\":\"rust\""));
    }

    #[test]
    fn migration_status_summarizes_manifest_safely() {
        let manifest = "{\"cutover_ready\": false, \"phases\": {\"a\": {\"status\": \"complete\"}, \"b\": {\"status\": \"pending\"}}}";
        let response = handle_request(
            &HttpRequest {
                method: "GET".to_string(),
                path: "/rust-migration/status".to_string(),
            },
            Some(manifest),
        );
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("\"cutover_ready\":false"));
        assert!(response.body.contains("\"complete_phases\":1"));
        assert!(response.body.contains("\"pending_phases\":1"));
    }

    #[test]
    fn unsupported_method_and_path_fail_predictably() {
        let response = handle_request(
            &HttpRequest {
                method: "POST".to_string(),
                path: "/health/live".to_string(),
            },
            None,
        );
        assert_eq!(response.status_code, 405);

        let response = handle_request(
            &HttpRequest {
                method: "GET".to_string(),
                path: "/private".to_string(),
            },
            None,
        );
        assert_eq!(response.status_code, 404);
    }

    #[test]
    fn renders_http_response_with_content_length() {
        let response = handle_request(
            &HttpRequest {
                method: "GET".to_string(),
                path: "/health/live".to_string(),
            },
            None,
        );
        let rendered = render_http_response(&response);
        assert!(rendered.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(rendered.contains("Content-Length: "));
        assert!(rendered.ends_with(&response.body));
    }
}
