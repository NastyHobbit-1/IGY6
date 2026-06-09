use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use ureq::Agent;

use crate::artifact_data_root;
use crate::encode_url_query_component;
use crate::is_forbidden;
use crate::jitter_delay;
use crate::postgres_client_url;
use crate::GatewayError;

const PLAYBOOK_VERSION: u64 = 1;
const HARVEST_INTERVAL_SECS: u64 = 6 * 60 * 60;
const DEFAULT_SEED_DOMAINS: &[&str] = &[
    "patreon.com",
    "patreonusercontent.com",
    "medium.com",
    "substack.com",
    "onlyfans.com",
    "fansly.com",
    "nytimes.com",
    "wsj.com",
    "bloomberg.com",
    "reddit.com",
    "twitter.com",
    "x.com",
];

static HARVEST_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

pub fn bypass_intel_targets_path() -> PathBuf {
    artifact_data_root().join("ops/bypass-intel-targets.json")
}

pub fn bypass_intel_playbook_path() -> PathBuf {
    artifact_data_root().join("ops/bypass-intel-playbook.json")
}

pub fn bypass_intel_last_run_path() -> PathBuf {
    artifact_data_root().join("ops/bypass-intel-last-run.json")
}

pub fn is_paid_content_platform_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    [
        "patreon.com",
        "patreonusercontent.com",
        "onlyfans.com",
        "fansly.com",
    ]
    .iter()
    .any(|host| lower.contains(host))
}

pub fn scope_requires_paid_content_escalation(scope: Option<&Value>) -> bool {
    let Some(items) = scope.and_then(|value| value.as_array()) else {
        return false;
    };
    items
        .iter()
        .any(|item| item.as_str().is_some_and(is_paid_content_platform_url))
}

pub fn patreon_session_path() -> PathBuf {
    artifact_data_root().join("ops/patreon-session.json")
}

pub fn load_patreon_session_credentials() -> Option<(String, Option<String>)> {
    let content = fs::read_to_string(patreon_session_path()).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    let cookie = value
        .get("cookie")
        .and_then(|entry| entry.as_str())
        .filter(|entry| !entry.trim().is_empty())?
        .to_string();
    let authorization = value
        .get("authorization")
        .and_then(|entry| entry.as_str())
        .filter(|entry| !entry.trim().is_empty())
        .map(str::to_string);
    Some((cookie, authorization))
}

pub fn domain_from_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    let host = host.split(':').next()?.trim_start_matches("www.");
    if host.is_empty() || !host.contains('.') {
        return None;
    }
    Some(host.to_string())
}

fn ensure_ops_dir() -> Result<(), GatewayError> {
    let ops = artifact_data_root().join("ops");
    fs::create_dir_all(&ops).map_err(|error| GatewayError::Conflict(error.to_string()))
}

fn read_json_file(path: &PathBuf) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| json!({}))
}

fn write_json_file(path: &PathBuf, value: &Value) -> Result<(), GatewayError> {
    ensure_ops_dir()?;
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|error| GatewayError::Conflict(error.to_string()))?;
    fs::write(path, serialized).map_err(|error| GatewayError::Conflict(error.to_string()))
}

fn now_rfc3339() -> String {
    format!("{:?}", SystemTime::now())
}

pub fn load_target_domains() -> BTreeSet<String> {
    let mut domains = BTreeSet::new();
    let payload = read_json_file(&bypass_intel_targets_path());
    if let Some(items) = payload.get("domains").and_then(|value| value.as_array()) {
        for item in items {
            if let Some(domain) = item.as_str() {
                domains.insert(domain.to_string());
            }
        }
    }
    for seed in DEFAULT_SEED_DOMAINS {
        domains.insert((*seed).to_string());
    }
    domains
}

pub fn record_bypass_intel_domains(urls: &[String]) {
    if urls.is_empty() {
        return;
    }
    let _ = ensure_ops_dir();
    let path = bypass_intel_targets_path();
    let mut payload = read_json_file(&path);
    let mut domains = load_target_domains();
    let mut added = false;
    for url in urls {
        if let Some(domain) = domain_from_url(url) {
            if domains.insert(domain) {
                added = true;
            }
        }
    }
    if !added {
        return;
    }
    let domain_list: Vec<Value> = domains.into_iter().map(Value::from).collect();
    payload["version"] = json!(1);
    payload["updated_at"] = json!(now_rfc3339());
    payload["domains"] = json!(domain_list);
    let _ = write_json_file(&path, &payload);
}

pub fn discover_domains_from_database(database_url: &str) -> Vec<String> {
    let postgres_url = postgres_client_url(database_url);
    let Ok(mut client) = postgres::Client::connect(&postgres_url, postgres::NoTls) else {
        return vec![];
    };
    let mut domains = BTreeSet::new();
    let queries = [
        "SELECT metadata_json->>'scraped_url' AS url FROM raw_artifacts WHERE metadata_json->>'scraped_url' IS NOT NULL ORDER BY created_at DESC LIMIT 250",
        "SELECT metadata_json->>'requested_url' AS url FROM raw_artifacts WHERE metadata_json->>'requested_url' IS NOT NULL ORDER BY created_at DESC LIMIT 250",
        "SELECT metadata_json->>'original_url' AS url FROM raw_artifacts WHERE metadata_json->>'original_url' IS NOT NULL ORDER BY created_at DESC LIMIT 250",
    ];
    for query in queries {
        let Ok(rows) = client.query(query, &[]) else {
            continue;
        };
        for row in rows {
            let Ok(url) = row.try_get::<_, Option<String>>("url") else {
                continue;
            };
            if let Some(url) = url {
                if let Some(domain) = domain_from_url(&url) {
                    domains.insert(domain);
                }
            }
        }
    }
    domains.into_iter().collect()
}

fn merge_domains_for_harvest(database_url: Option<&str>) -> BTreeSet<String> {
    let mut domains = load_target_domains();
    if let Some(database_url) = database_url.filter(|value| !value.trim().is_empty()) {
        for domain in discover_domains_from_database(database_url) {
            domains.insert(domain);
        }
    }
    domains
}

fn search_duckduckgo_html(agent: &Agent, query: &str) -> Option<String> {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        encode_url_query_component(query)
    );
    let response = agent
        .get(&url)
        .set(
            "User-Agent",
            "Mozilla/5.0 (compatible; IGY6BypassIntel/1.0; +local)",
        )
        .call()
        .ok()?;
    response.into_string().ok()
}

fn extract_snippets(html: &str) -> Vec<String> {
    let mut snippets = Vec::new();
    for fragment in html.split("result__snippet") {
        let cleaned = fragment
            .split('<')
            .next()
            .unwrap_or(fragment)
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .trim()
            .to_string();
        if cleaned.len() >= 40 && cleaned.len() <= 1200 {
            snippets.push(cleaned);
        }
        if snippets.len() >= 8 {
            break;
        }
    }
    snippets
}

fn extract_urls_from_text(text: &str) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for token in text.split_whitespace() {
        let trimmed = token.trim_matches(|ch: char| {
            !ch.is_ascii()
                && ch != '.'
                && ch != '/'
                && ch != ':'
                && ch != '?'
                && ch != '&'
                && ch != '='
                && ch != '%'
        });
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            urls.insert(trimmed.to_string());
        }
    }
    urls.into_iter().collect()
}

fn technique_templates_for_domain(_domain: &str) -> Vec<Value> {
    vec![
        json!({
            "id": "archive_wayback",
            "kind": "url_template",
            "label": "archive_org_wayback",
            "url_template": "https://web.archive.org/web/{url}",
            "confidence": 72,
            "source": "baseline"
        }),
        json!({
            "id": "amp_query",
            "kind": "url_template",
            "label": "amp_query_suffix",
            "url_template": "https://{domain}{path}?amp=1",
            "confidence": 58,
            "source": "baseline"
        }),
        json!({
            "id": "mobile_prefix",
            "kind": "url_template",
            "label": "mobile_m_prefix",
            "url_template": "https://m.{domain}{path}",
            "confidence": 52,
            "source": "baseline"
        }),
        json!({
            "id": "google_referer",
            "kind": "referer",
            "label": "google_referer",
            "referer": "https://www.google.com/",
            "confidence": 48,
            "source": "baseline"
        }),
        json!({
            "id": "twitter_referer",
            "kind": "referer",
            "label": "twitter_referer",
            "referer": "https://t.co/",
            "confidence": 44,
            "source": "baseline"
        }),
        json!({
            "id": "googlebot_ua",
            "kind": "user_agent",
            "label": "googlebot",
            "user_agent": "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            "confidence": 46,
            "source": "baseline"
        }),
    ]
}

fn paid_platform_techniques(domain: &str) -> Vec<Value> {
    if domain.contains("patreon") {
        return vec![
            json!({
                "id": "patreon_api_posts",
                "kind": "url_template",
                "label": "patreon_api_post",
                "url_template": "https://www.patreon.com/api/posts/{path}?include=attachments,audio,user,images,media&json-api-version=1.0",
                "confidence": 88,
                "source": "patreon_paid"
            }),
            json!({
                "id": "patreon_mobile",
                "kind": "url_template",
                "label": "patreon_mobile",
                "url_template": "https://www.patreon.com{path}",
                "confidence": 70,
                "source": "patreon_paid"
            }),
            json!({
                "id": "patreon_session_referer",
                "kind": "referer",
                "label": "patreon_home_referer",
                "referer": "https://www.patreon.com/home",
                "confidence": 75,
                "source": "patreon_paid"
            }),
        ];
    }
    vec![]
}

fn harvest_domain(agent: &Agent, domain: &str) -> Value {
    let queries = if domain.contains("patreon") {
        vec![
            format!("{domain} patreon api download paid post media"),
            format!("site:reddit.com {domain} download patron content"),
            format!("{domain} patreonusercontent token media url"),
            format!("{domain} logged in session cookie patron post"),
        ]
    } else {
        vec![
            format!("{domain} paywall bypass"),
            format!("{domain} archive wayback reader"),
            format!("site:reddit.com {domain} bypass paywall"),
            format!("{domain} amp mobile reader mode"),
        ]
    };
    let mut snippets = Vec::new();
    let mut discovered_urls = BTreeSet::new();
    let mut techniques = technique_templates_for_domain(domain);
    techniques.extend(paid_platform_techniques(domain));
    for query in queries {
        jitter_delay();
        if let Some(html) = search_duckduckgo_html(agent, &query) {
            snippets.extend(extract_snippets(&html));
            for url in extract_urls_from_text(&html) {
                discovered_urls.insert(url);
            }
            let lower = html.to_lowercase();
            if lower.contains("12ft.io") {
                techniques.push(json!({
                    "id": "mention_12ft",
                    "kind": "url_template",
                    "label": "12ft_proxy",
                    "url_template": "https://12ft.io/{url}",
                    "confidence": 61,
                    "source": "web_search"
                }));
            }
            if lower.contains("archive.is") || lower.contains("archive.today") {
                techniques.push(json!({
                    "id": "mention_archive_is",
                    "kind": "url_template",
                    "label": "archive_is_proxy",
                    "url_template": "https://archive.is/latest/{url}",
                    "confidence": 63,
                    "source": "web_search"
                }));
            }
            if lower.contains("outline.com") {
                techniques.push(json!({
                    "id": "mention_outline",
                    "kind": "url_template",
                    "label": "outline_reader",
                    "url_template": "https://outline.com/{url}",
                    "confidence": 57,
                    "source": "web_search"
                }));
            }
            if lower.contains("removepaywall") {
                techniques.push(json!({
                    "id": "mention_removepaywall",
                    "kind": "url_template",
                    "label": "removepaywall_proxy",
                    "url_template": "https://www.removepaywall.com/search?url={url}",
                    "confidence": 60,
                    "source": "web_search"
                }));
            }
        }
    }
    let mut deduped_techniques = Vec::new();
    let mut seen = BTreeSet::new();
    for technique in techniques {
        let key = technique
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        if seen.insert(key) {
            deduped_techniques.push(technique);
        }
    }
    json!({
        "domain": domain,
        "last_harvested": now_rfc3339(),
        "search_snippets": snippets.into_iter().take(12).collect::<Vec<_>>(),
        "discovered_urls": discovered_urls.into_iter().take(20).collect::<Vec<_>>(),
        "techniques": deduped_techniques
    })
}

pub fn run_bypass_intel_harvest(database_url: Option<&str>) -> Result<Value, GatewayError> {
    ensure_ops_dir()?;
    let domains = merge_domains_for_harvest(database_url);
    let agent = Agent::new();
    let mut domain_entries = BTreeMap::new();
    for domain in domains {
        if is_forbidden(&format!("https://{domain}/")) {
            continue;
        }
        domain_entries.insert(domain.clone(), harvest_domain(&agent, &domain));
    }
    let playbook = json!({
        "version": PLAYBOOK_VERSION,
        "updated_at": now_rfc3339(),
        "domains": domain_entries
    });
    write_json_file(&bypass_intel_playbook_path(), &playbook)?;
    let targets: Vec<Value> = playbook
        .get("domains")
        .and_then(|value| value.as_object())
        .map(|entries| entries.keys().map(|domain| json!(domain)).collect())
        .unwrap_or_default();
    write_json_file(
        &bypass_intel_targets_path(),
        &json!({
            "version": 1,
            "updated_at": now_rfc3339(),
            "domains": targets
        }),
    )?;
    let finished_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let summary = json!({
        "ok": true,
        "domains_harvested": playbook.get("domains").and_then(|v| v.as_object()).map(|m| m.len()).unwrap_or(0),
        "techniques_total": playbook
            .get("domains")
            .and_then(|v| v.as_object())
            .map(|entries| {
                entries
                    .values()
                    .filter_map(|entry| entry.get("techniques").and_then(|v| v.as_array()))
                    .map(|items| items.len())
                    .sum::<usize>()
            })
            .unwrap_or(0),
        "finished_at": now_rfc3339(),
        "finished_at_unix": finished_at_unix
    });
    write_json_file(&bypass_intel_last_run_path(), &summary)?;
    Ok(summary)
}

fn harvest_is_due() -> bool {
    let last = read_json_file(&bypass_intel_last_run_path());
    let Some(finished_at_unix) = last
        .get("finished_at_unix")
        .and_then(|value| value.as_u64())
    else {
        return true;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    now.saturating_sub(finished_at_unix) >= HARVEST_INTERVAL_SECS
}

pub fn maybe_background_bypass_intel_harvest(database_url: Option<String>) {
    if !harvest_is_due() {
        return;
    }
    if HARVEST_IN_FLIGHT
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    thread::spawn(move || {
        let _ = run_bypass_intel_harvest(database_url.as_deref());
        HARVEST_IN_FLIGHT.store(false, Ordering::SeqCst);
    });
}

pub fn bypass_intel_status_json() -> String {
    let targets = load_target_domains();
    let playbook = read_json_file(&bypass_intel_playbook_path());
    let last_run = read_json_file(&bypass_intel_last_run_path());
    let technique_count = playbook
        .get("domains")
        .and_then(|value| value.as_object())
        .map(|entries| {
            entries
                .values()
                .filter_map(|entry| entry.get("techniques").and_then(|v| v.as_array()))
                .map(|items| items.len())
                .sum::<usize>()
        })
        .unwrap_or(0);
    json!({
        "ok": true,
        "targets_count": targets.len(),
        "techniques_total": technique_count,
        "last_run": last_run,
        "playbook_updated_at": playbook.get("updated_at"),
        "sample_domains": targets.into_iter().take(12).collect::<Vec<_>>(),
        "harvest_interval_hours": HARVEST_INTERVAL_SECS / 3600
    })
    .to_string()
}

fn matching_playbook_domain<'a>(playbook: &'a Value, url: &str) -> Option<&'a Value> {
    let domain = domain_from_url(url)?;
    let entries = playbook.get("domains")?.as_object()?;
    if let Some(entry) = entries.get(&domain) {
        return Some(entry);
    }
    for (key, entry) in entries {
        if domain.ends_with(key) || key.ends_with(&domain) {
            return Some(entry);
        }
    }
    None
}

fn apply_url_template(template: &str, original_url: &str) -> Option<String> {
    let lower = original_url.to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let mut parts = without_scheme.splitn(2, '/');
    let host = parts.next()?;
    let domain = host.trim_start_matches("www.");
    let path = parts
        .next()
        .map(|value| format!("/{value}"))
        .unwrap_or_else(|| "/".to_string());
    let candidate = template
        .replace("{url}", &encode_url_query_component(original_url))
        .replace("{domain}", domain)
        .replace("{path}", &path);
    if candidate.starts_with("http://") || candidate.starts_with("https://") {
        Some(candidate)
    } else {
        None
    }
}

pub fn playbook_url_variants_for_url(original_url: &str) -> Vec<(String, String)> {
    let playbook = read_json_file(&bypass_intel_playbook_path());
    let Some(entry) = matching_playbook_domain(&playbook, original_url) else {
        return vec![];
    };
    let Some(techniques) = entry.get("techniques").and_then(|value| value.as_array()) else {
        return vec![];
    };
    let mut variants = Vec::new();
    let mut seen = BTreeSet::new();
    for technique in techniques {
        if technique.get("kind").and_then(|value| value.as_str()) != Some("url_template") {
            continue;
        }
        let label = technique
            .get("label")
            .and_then(|value| value.as_str())
            .unwrap_or("playbook");
        let template = technique
            .get("url_template")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let Some(candidate) = apply_url_template(template, original_url) else {
            continue;
        };
        if is_forbidden(&candidate) || !seen.insert(candidate.clone()) {
            continue;
        }
        variants.push((format!("playbook:{label}"), candidate));
    }
    variants
}

pub fn playbook_header_strategies_for_url(
    original_url: &str,
) -> Vec<(String, Vec<(String, String)>)> {
    let playbook = read_json_file(&bypass_intel_playbook_path());
    let Some(entry) = matching_playbook_domain(&playbook, original_url) else {
        return vec![];
    };
    let Some(techniques) = entry.get("techniques").and_then(|value| value.as_array()) else {
        return vec![];
    };
    let mut strategies = Vec::new();
    for technique in techniques {
        let kind = technique
            .get("kind")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        match kind {
            "referer" => {
                let label = technique
                    .get("label")
                    .and_then(|value| value.as_str())
                    .unwrap_or("playbook_referer");
                let referer = technique
                    .get("referer")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                if referer.is_empty() {
                    continue;
                }
                let mut headers = crate::anon_headers();
                headers.push(("Referer".to_string(), referer.to_string()));
                strategies.push((format!("playbook:{label}"), headers));
            }
            "user_agent" => {
                let label = technique
                    .get("label")
                    .and_then(|value| value.as_str())
                    .unwrap_or("playbook_ua");
                let user_agent = technique
                    .get("user_agent")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                if user_agent.is_empty() {
                    continue;
                }
                let headers = vec![
                    ("User-Agent".to_string(), user_agent.to_string()),
                    (
                        "Accept".to_string(),
                        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                            .to_string(),
                    ),
                    ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
                ];
                strategies.push((format!("playbook:{label}"), headers));
            }
            _ => {}
        }
    }
    strategies
}

pub fn bypass_intel_harvest_response(
    body: &str,
    database_url: Option<&str>,
) -> crate::GatewayResponse {
    let object: Value = serde_json::from_str(body).unwrap_or(json!({}));
    let force = object
        .get("force")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    if !force && !harvest_is_due() {
        return crate::json_response(
            200,
            "OK",
            json!({
                "ok": true,
                "skipped": true,
                "reason": "recent_harvest",
                "status": serde_json::from_str::<Value>(&bypass_intel_status_json()).unwrap_or(json!({}))
            })
            .to_string(),
            false,
        );
    }
    crate::write_route_response(run_bypass_intel_harvest(database_url).map(|summary| {
        json!({
            "ok": true,
            "summary": summary,
            "status": serde_json::from_str::<Value>(&bypass_intel_status_json()).unwrap_or(json!({}))
        })
        .to_string()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paid_platform_detection_includes_patreon() {
        assert!(is_paid_content_platform_url(
            "https://www.patreon.com/posts/example-123456"
        ));
        assert!(is_paid_content_platform_url(
            "https://c10.patreonusercontent.com/4/patreon-media/file.jpg"
        ));
        assert!(!is_paid_content_platform_url("https://example.com/page"));
    }

    #[test]
    fn domain_from_url_strips_www_and_scheme() {
        assert_eq!(
            domain_from_url("https://www.patreon.com/posts/example-123"),
            Some("patreon.com".to_string())
        );
    }

    #[test]
    fn apply_url_template_replaces_tokens() {
        let candidate = apply_url_template(
            "https://web.archive.org/web/{url}",
            "https://www.example.com/news/story",
        )
        .expect("template");
        assert!(candidate.starts_with("https://web.archive.org/web/"));
        assert!(candidate.contains("example.com"));
    }
}
